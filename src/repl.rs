mod parser;

use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, Read, Seek, Write},
};

use crate::{repl::parser::Command, strings};

fn prologue() {
    eprintln!("{}", *strings::PROLOGUE);
}

pub fn run(path: &String, mut file: File, readable: bool, writable: bool) -> io::Result<()> {
    use Command::*;

    let size = file.metadata()?.len();
    let unit = if size == 1 { "byte" } else { "bytes" };
    let mode = match (readable, writable) {
        (true, true) => "RW",
        (true, false) => "RO",
        (false, true) => "WO",
        (false, false) => "??",
    };

    prologue();

    eprintln!("File: \"{path}\" ({size} {unit}) [{mode}]\n");

    let mut buffer = Vec::<u8>::with_capacity(8192);
    let mut read_count = 0usize;
    let mut write_count = 0usize;

    loop {
        let pos = file.stream_position()?;
        let pos_str = format!("pos:{pos}");
        let in_str = if read_count > 0 {
            format!("in:{read_count}, ")
        } else {
            format!("")
        };
        let out_str = if write_count > 0 {
            format!("out:{write_count}, ")
        } else {
            format!("")
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
        if buffer.len() == 0 {
            eprintln!();
            break;
        }
        if buffer[buffer.len() - 1] == b'\n' {
            buffer.pop();
        } else {
            eprintln!()
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
            Seek(seek_from) => {
                if let Err(e) = file.seek(seek_from) {
                    error(e);
                }
            }
            Read { seek, count } => {
                match file
                    .seek(seek)
                    .and_then(|_| read_to_buffer(&mut file, &mut buffer, count))
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
            Readb { seek, count } => {
                let mut start_pos = 0; // Valid *after* seeking.

                match file.seek(seek).and_then(|new_pos| {
                    start_pos = new_pos;
                    read_to_buffer(&mut file, &mut buffer, count)
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
            Write { seek, index } => {
                let write_buf = &buffer[index..];
                if write_buf.len() == 0 {
                    continue;
                }

                match file.seek(seek).and_then(|_| file.write_all(write_buf)) {
                    Err(e) => error(e),
                    Ok(_) => write_count = write_buf.len(),
                }
            }
            Writeb { seek, bytes } => match file.seek(seek).and_then(|_| file.write_all(&bytes)) {
                Err(e) => error(e),
                Ok(_) => write_count = bytes.len(),
            },
        }
    }

    file.flush()?;

    Ok(())
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
        buffer.try_reserve(count)?
    }

    buffer.resize(count, 0);

    let actual_count = file.read(buffer)?;

    buffer.truncate(actual_count);

    Ok(actual_count)
}

fn print_hexdump(from_pos: u64, buffer: &Vec<u8>) -> io::Result<()> {
    if buffer.len() == 0 {
        return Ok(());
    }
    const COLUMNS: usize = 16; // Must be a multiple of 2.

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

    if last_row.len() > 0 {
        print_offset(rows.len(), &mut output);

        let (pairs, single) = last_row.as_chunks::<2>();
        print_pairs(pairs, &mut output);

        if single.len() > 0 {
            let mut single_hex = format!(" {:02x}", single[0]).into_bytes();
            output.append(&mut single_hex);
            output.push(b' '); // Fill space of missing half.
            output.push(b' ');
        }

        let num_missing_pairs = (COLUMNS - last_row.len()) / 2;
        for _ in 0..num_missing_pairs * 5 {
            output.push(b' ');
        }

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
