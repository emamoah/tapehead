use std::{env::args, error::Error, fs::File, process};
use tapehead::{
    self, PROGNAME,
    repl::{self, FileMode},
    strings::VERSION,
};

pub fn usage() {
    eprintln!("TapeHead v{}\n\nUsage: {} <file>", VERSION, *PROGNAME);
}

fn exit_with_error<T>(e: impl Error) -> T {
    let prefix = if PROGNAME.is_empty() {
        String::new()
    } else {
        format!("{}: ", *PROGNAME)
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
    let (file, file_mode) = try_open(&file_path).unwrap_or_else(exit_with_error);

    repl::run(&file_path, file, file_mode).unwrap_or_else(exit_with_error);
}

fn try_open(file_path: &String) -> std::io::Result<(File, FileMode)> {
    let mut file_mode = FileMode::RW;
    let mut file = File::options().read(true).write(true).open(file_path);
    if file.as_ref().is_err() {
        file = File::options().write(true).open(file_path);
        if file.is_ok() {
            file_mode = FileMode::WO;
        }
    }
    if file.as_ref().is_err() {
        file = File::options().read(true).open(file_path);
        if file.is_ok() {
            file_mode = FileMode::RO;
        }
    }

    file.map(|file| (file, file_mode))
}
