mod parser;

use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, Read, Seek, Write},
};

use crate::VERSION;

fn prologue() {
    eprintln!(
        "TapeHead v{VERSION}

Author: Emmanuel Amoah (https://emamoah.com/)

Enter \"help\" for more information.\n\n\n"
    );
}

pub fn run(path: &String, mut file: File, readable: bool, writable: bool) -> io::Result<()> {
    use parser::Op::*;

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

        let Some(command) = parser::parse_input(&buffer) else {
            error("Invalid command. Enter \"help\" for usage.");
            continue;
        };

        match command.op {
            NOP => continue,
            QUIT => break,
            HELP => help(),
            SEEK => {
                if let Err(e) = file.seek(command.seek) {
                    error(e);
                }
            }
            READ => {
                if let Err(e) = handle_read(&command, &mut file, &mut buffer, &mut read_count) {
                    error(e);
                    continue;
                }

                // Print contents.
                io::stdout().write_all(&buffer).unwrap_or_else(error);
                io::stdout().flush()?;
                if read_count > 0 {
                    // Prompt on new line.
                    eprintln!();
                }
            }
            WRITE => {
                let Some(write_buf_start) = command.arg else {
                    // error("Invalid state: write argument is `None`.");
                    error("If you're seeing this in your output, I should be arrested.");
                    continue;
                };

                let write_buf = &buffer[write_buf_start..];
                if write_buf.len() == 0 {
                    continue;
                }

                match file
                    .seek(command.seek)
                    .and_then(|_| file.write_all(write_buf))
                {
                    Err(e) => error(e),
                    Ok(_) => write_count = write_buf.len(),
                }
            }
        }
    }

    file.flush()?;

    Ok(())
}

fn handle_read(
    command: &parser::Command,
    file: &mut File,
    buffer: &mut Vec<u8>,
    read_count: &mut usize,
) -> Result<(), Box<dyn Error>> {
    buffer.clear();

    file.seek(command.seek)?;

    if let Some(mut count) = command.arg {
        // Count arg is present.

        if count > buffer.capacity() {
            buffer.try_reserve(count)?
        }

        buffer.resize(count, 0);

        let old_pos = file.stream_position()?;
        if let Err(e) = file.read_exact(buffer) {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                // Couldn't read up to given count. Infer read count from pos difference.
                let inferred_count = (file.stream_position()? - old_pos) as usize;

                buffer.truncate(inferred_count);

                count = inferred_count;
            } else {
                return Err(e.into());
            }
        }

        *read_count = count;
    } else {
        // No count arg. Read until end.

        let their_count = file.read_to_end(buffer)?;
        *read_count = their_count;
    }

    Ok(())
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
