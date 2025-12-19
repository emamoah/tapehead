mod parser;

use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, Read, Seek, SeekFrom, Write},
};

use crate::{repl::parser::Command, strings};

#[derive(Debug)]
pub enum FileMode {
    RW,
    RO,
    WO,
}

fn prologue() {
    eprintln!("{}", *strings::PROLOGUE);
}

pub fn run(path: &String, mut file: File, file_mode: FileMode) -> io::Result<()> {
    use Command::*;

    let size = file.metadata()?.len();
    let unit = if size == 1 { "byte" } else { "bytes" };

    prologue();

    eprintln!("File: \"{path}\" ({size} {unit}) [{file_mode:?}]\n");

    let mut buffer = Vec::<u8>::with_capacity(8192);
    let mut read_count = 0usize;
    let mut write_count = 0usize;

    loop {
        let pos = try_get_pos(&file);
        let pos_str = format!("pos:{}", pos.map_or("*".into(), |p| p.to_string()));
        let in_str = if read_count > 0 {
            format!("in:{read_count}, ")
        } else {
            String::new()
        };
        let out_str = if write_count > 0 {
            format!("out:{write_count}, ")
        } else {
            String::new()
        };

        eprint!("[{in_str}{out_str}{pos_str}]> ");
        io::stderr().flush()?;

        buffer.clear();
        read_count = 0;
        write_count = 0;

        // Read command line.
        if let Err(e) = io::stdin().lock().read_until(b'\n', &mut buffer) {
            error(e);
            continue;
        }
        if buffer.is_empty() {
            eprintln!();
            break;
        }
        if buffer[buffer.len() - 1] == b'\n' {
            buffer.pop();
        } else {
            eprintln!();
        }

        let command = match parser::parse_input(&buffer) {
            Ok(command) => command,
            Err(e) => {
                error(format!("{} {}", e, strings::ENTER_HELP_FOR_USAGE));
                continue;
            }
        };

        match command {
            Nop => continue,
            Quit => break,
            Help => help(),
            Seek(cmd) => {
                if let Err(e) = try_seek(&file, cmd.0) {
                    error(e);
                }
            }
            Read(cmd) => {
                match try_seek(&file, cmd.seek)
                    .and_then(|_| read_to_buffer(&mut file, &mut buffer, cmd.count))
                {
                    Err(e) => {
                        error(e);
                        continue;
                    }
                    Ok(count) => read_count = count,
                }

                // Print contents.
                io::stdout().write_all(&buffer).unwrap_or_else(error);
                io::stdout().flush()?;
                if read_count > 0 {
                    // Prompt on new line.
                    eprintln!();
                }
            }
            Readb(cmd) => {
                let mut start_pos: Option<u64> = None;

                match try_seek(&file, cmd.seek).and_then(|new_pos| {
                    start_pos = new_pos;
                    read_to_buffer(&mut file, &mut buffer, cmd.count)
                }) {
                    Err(e) => {
                        error(e);
                        continue;
                    }
                    Ok(count) => read_count = count,
                }

                // Print hexdump
                print_hexdump(start_pos, &buffer).unwrap_or_else(error);
            }
            Write(cmd) => {
                let write_buf = &buffer[cmd.index..];
                if write_buf.is_empty() {
                    continue;
                }

                match try_seek(&file, cmd.seek).and_then(|_| file.write_all(write_buf)) {
                    Err(e) => error(e),
                    Ok(()) => write_count = write_buf.len(),
                }
            }
            Writeb(cmd) => match try_seek(&file, cmd.seek).and_then(|_| file.write_all(&cmd.bytes))
            {
                Err(e) => error(e),
                Ok(()) => write_count = cmd.bytes.len(),
            },
        }
    }

    file.flush()?;

    Ok(())
}

fn try_get_pos(mut file: &File) -> Option<u64> {
    file.stream_position().ok()
}

fn try_seek(mut file: &File, seek: SeekFrom) -> io::Result<Option<u64>> {
    if seek != SeekFrom::Current(0) {
        return match file.seek(seek) {
            Ok(new_pos) => Ok(Some(new_pos)),
            Err(_) => Err(io::Error::other(strings::NOT_SEEKABLE_USE_DOT)),
        };
    }
    // `seek` is `SeekFrom::Current(0)`.
    match file.seek(seek) {
        Ok(new_pos) => Ok(Some(new_pos)),
        Err(_) => Ok(None),
    }
}

fn read_to_buffer(
    file: &mut File,
    buffer: &mut Vec<u8>,
    count: Option<usize>,
) -> io::Result<usize> {
    buffer.clear();

    let Some(count) = count else {
        // No count arg. Read until end.
        let their_count = file.read_to_end(buffer)?;
        return Ok(their_count);
    };
    // Count arg is present.

    if count > buffer.capacity() {
        buffer.try_reserve(count)?;
    }

    buffer.resize(count, 0);

    let actual_count = file.read(buffer)?;

    buffer.truncate(actual_count);

    Ok(actual_count)
}

fn print_hexdump(from_pos: Option<u64>, buffer: &[u8]) -> io::Result<()> {
    const COLUMNS: usize = 16; // Must be a multiple of 2.

    if buffer.is_empty() {
        return Ok(());
    }

    let from_pos = from_pos.unwrap_or(0);

    let mut output = Vec::<u8>::with_capacity(4096);

    let (rows, last_row) = buffer.as_chunks::<COLUMNS>();

    let last_row_offset = from_pos + (COLUMNS * (buffer.len().div_ceil(COLUMNS) - 1)) as u64;
    let offset_width = 4.max(last_row_offset.to_string().len());

    let print_offset = |index: usize, output: &mut Vec<u8>| {
        let mut offset =
            format!("{:>offset_width$}:", from_pos + (COLUMNS * index) as u64).into_bytes();
        output.append(&mut offset);
    };

    let print_row_ascii = |row: &[u8], output: &mut Vec<u8>| {
        output.push(b' ');
        output.push(b' ');

        for byte in row {
            let rendered_char = if (32..=126).contains(byte) {
                byte
            } else {
                &b'.'
            };
            output.push(*rendered_char);
        }
        output.push(b'\n');
    };

    let print_pairs = |pairs: &[[u8; 2]], output: &mut Vec<u8>| {
        for pair in pairs {
            let mut pair_hex = format!(" {:02x}{:02x}", pair[0], pair[1]).into_bytes();
            output.append(&mut pair_hex);
        }
    };

    for (index, row) in rows.iter().enumerate() {
        print_offset(index, &mut output);

        let (pairs, _) = row.as_chunks::<2>();
        print_pairs(pairs, &mut output);

        print_row_ascii(row, &mut output);
    }

    if !last_row.is_empty() {
        print_offset(rows.len(), &mut output);

        let (pairs, single) = last_row.as_chunks::<2>();
        print_pairs(pairs, &mut output);

        if !single.is_empty() {
            let mut single_hex = format!(" {:02x}", single[0]).into_bytes();
            output.append(&mut single_hex);
            output.push(b' '); // Fill space of missing half.
            output.push(b' ');
        }

        let num_missing_pairs = (COLUMNS - last_row.len()) / 2;
        output.extend(std::iter::repeat_n(b' ', num_missing_pairs * 5));

        print_row_ascii(last_row, &mut output);
    }

    io::stdout().write_all(&output)?;
    io::stdout().flush()?;

    Ok(())
}

fn error(e: impl Into<Box<dyn Error>>) {
    eprintln!("error: {}", e.into());
}

fn help() {
    eprintln!("{}", *strings::HELP);
}
