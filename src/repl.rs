mod parser;

use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, Read, Seek, Write},
};

use crate::{VERSION, repl::parser::Command};

fn prologue() {
    eprintln!(
        "TapeHead v{VERSION}

Author: Emmanuel Amoah (https://emamoah.com/)

Enter \"help\" for more information.\n\n\n"
    );
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
                error(format!("{e} Enter \"help\" for usage."));
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

    let old_pos = file.stream_position()?;
    let Err(error) = file.read_exact(buffer) else {
        // Read successful.
        return Ok(count);
    };
    // Couldn't read exact.

    if error.kind() != io::ErrorKind::UnexpectedEof {
        return Err(error.into());
    }
    // Read less than `count`. Infer actual from pos difference.

    let inferred_count = (file.stream_position()? - old_pos) as usize;

    buffer.truncate(inferred_count);

    return Ok(inferred_count);
}

fn error(e: impl Into<Box<dyn Error>>) {
    eprintln!("error: {}", e.into());
}

fn help() {
    eprintln!(
        "TapeHead v{}

Visit https://github.com/emamoah/tapehead for official documentation.

{}",
        VERSION,
        include_str!("repl/help.txt")
    );
}
