use std::{env::args, error::Error, fs::File, io, process};
use tapehead::{self, PROGNAME, VERSION, repl};

pub fn usage() {
    let progname = &*PROGNAME;
    eprintln!(
        "TapeHead v{VERSION}

Usage: {progname} <file>"
    );
}

fn exit_with_error<T>(e: impl Error) -> T {
    let prefix = if !PROGNAME.is_empty() {
        format!("{}: ", &*PROGNAME)
    } else {
        format!("")
    };
    eprintln!("{prefix}error: {e}");
    process::exit(1);
}

fn exit_with_usage<T>() -> T {
    usage();
    process::exit(1);
}

fn main() {
    let file_path = args().skip(1).next().unwrap_or_else(exit_with_usage);
    let (file, readable, writable) = try_open(&file_path).unwrap_or_else(exit_with_error);

    repl::run(&file_path, file, readable, writable).unwrap_or_else(exit_with_error);
}

fn try_open(file_path: &String) -> std::io::Result<(File, bool, bool)> {
    let (mut readable, mut writable) = (true, true);
    let mut file = File::options().read(true).write(true).open(&file_path);
    if file
        .as_ref()
        .is_err_and(|e| e.kind() == io::ErrorKind::PermissionDenied)
    {
        file = File::options().write(true).open(&file_path).and_then(|f| {
            readable = false;
            Ok(f)
        });
    }
    if file
        .as_ref()
        .is_err_and(|e| e.kind() == io::ErrorKind::PermissionDenied)
    {
        file = File::options().read(true).open(&file_path).and_then(|f| {
            writable = false;
            Ok(f)
        });
    }
    if file.as_ref().is_err() {
        (readable, writable) = (false, false);
    }

    file.map(|file| (file, readable, writable))
}
