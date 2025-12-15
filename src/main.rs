use std::{env::args, error::Error, fs::File, process};
use tapehead::{self, PROGNAME, repl, strings::VERSION};

pub fn usage() {
    eprintln!("TapeHead v{}\n\nUsage: {} <file>", VERSION, *PROGNAME);
}

fn exit_with_error<T>(e: impl Error) -> T {
    let prefix = if !PROGNAME.is_empty() {
        format!("{}: ", *PROGNAME)
    } else {
        String::new()
    };
    eprintln!("{prefix}error: {e}");
    process::exit(1);
}

fn exit_with_usage<T>() -> T {
    usage();
    process::exit(1);
}

fn main() {
    let file_path = args().nth(1).unwrap_or_else(exit_with_usage);
    let (file, readable, writable) = try_open(&file_path).unwrap_or_else(exit_with_error);

    repl::run(&file_path, file, readable, writable).unwrap_or_else(exit_with_error);
}

fn try_open(file_path: &String) -> std::io::Result<(File, bool, bool)> {
    let (mut readable, mut writable) = (true, true);
    let mut file = File::options().read(true).write(true).open(file_path);
    if file.as_ref().is_err() {
        file = File::options().write(true).open(file_path).and_then(|f| {
            readable = false;
            Ok(f)
        });
    }
    if file.as_ref().is_err() {
        file = File::options().read(true).open(file_path).and_then(|f| {
            writable = false;
            Ok(f)
        });
    }
    if file.as_ref().is_err() {
        (readable, writable) = (false, false);
    }

    file.map(|file| (file, readable, writable))
}
