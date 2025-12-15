use std::sync::LazyLock;

pub const VERSION: &str = "0.1.0";

pub static PROLOGUE: LazyLock<String> = LazyLock::new(|| {
    format!(
        "TapeHead v{VERSION}\n\nAuthor: Emmanuel Amoah (https://emamoah.com/)\n\nEnter \"help\" for more information.\n\n\n"
    )
});
pub static HELP: LazyLock<String> = LazyLock::new(|| {
    format!(
        "TapeHead v{}\n\nVisit https://github.com/emamoah/tapehead for official documentation.\n\n{}",
        VERSION,
        include_str!("repl/help.txt")
    )
});

pub const NOT_SEEKABLE_USE_DOT: &str = "File not seekable. Use `.` in seek argument.";
pub const WEIRD_COMMAND_NOT_FOUND: &str = "Weird... Command not found.";
pub const WEIRD_SEEK_ARG_NOT_FOUND: &str = "Weird... Seek argument not found.";
pub const ENTER_HELP_FOR_USAGE: &str = "Enter \"help\" for usage.";
pub const UNRECOGNIZED_COMMAND: &str = "Unrecognized command.";
pub const MISSING_SEEK_ARG: &str = "Missing seek argument.";
pub const INVALID_DIGIT_IN_COUNT_ARG: &str = "Invalid digit in count argument.";
pub const INVALID_BYTE_ARG: &str = "Invalid byte argument.";
pub const INVALID_DIGIT_IN_SEEK_ARG: &str = "Invalid digit in seek argument.";
pub const INVALID_SEEK_ARG: &str = "Invalid seek argument.";
pub const INVALID_STATE_READ_RETURNED_WRONG_TYPE: &str =
    "INVALID STATE: `read` parser returned a wrong type.";
