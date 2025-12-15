use std::{error::Error, io::SeekFrom};

use crate::strings;

type ParseResult = Result<Command, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
pub enum Command {
    Read {
        seek: SeekFrom,
        count: Option<usize>,
    },
    Readb {
        seek: SeekFrom,
        count: Option<usize>,
    },
    Write {
        seek: SeekFrom,
        index: usize,
    },
    Writeb {
        seek: SeekFrom,
        bytes: Vec<u8>,
    },
    Seek(SeekFrom),
    Help,
    Quit,
    Nop,
}

impl Command {
    const OP_READ: &[u8] = b"read";
    const OP_READB: &[u8] = b"readb";
    const OP_WRITE: &[u8] = b"write";
    const OP_WRITEB: &[u8] = b"writeb";
    const OP_SEEK: &[u8] = b"seek";
    const OP_HELP: &[u8] = b"help";
    const OP_QUIT: &[u8] = b"quit";
}

pub fn parse_input(input: &[u8]) -> ParseResult {
    if input.len() == 0 {
        return Ok(Command::Nop);
    }

    // Is there a better way? i.e. <&str>::split_whitespace, but for &[u8] ?
    let mut input_words = input
        .split(u8::is_ascii_whitespace)
        .filter(|chunk| !chunk.is_empty());

    let op = input_words.next().ok_or(strings::WEIRD_COMMAND_NOT_FOUND)?;

    match op.to_ascii_lowercase().as_slice() {
        Command::OP_READ => parse_read_command(input_words),
        Command::OP_READB => parse_readb_command(input_words),
        Command::OP_WRITE => parse_write_command(input_words, input),
        Command::OP_WRITEB => parse_writeb_command(input_words),
        Command::OP_SEEK => parse_seek_command(input_words),
        Command::OP_HELP => Ok(Command::Help),
        Command::OP_QUIT => Ok(Command::Quit),
        _ => Err(strings::UNRECOGNIZED_COMMAND)?,
    }
}

fn parse_read_command<'a>(mut args: impl Iterator<Item = &'a [u8]>) -> ParseResult {
    let seek_arg = args.next().ok_or(strings::MISSING_SEEK_ARG)?;
    let seek = parse_seek_arg(seek_arg)?;

    let count_arg = args.next().map(String::from_utf8_lossy);
    let count = match count_arg {
        None => None,
        Some(c) => {
            let num = c
                .parse::<usize>()
                .map_err(|_| strings::INVALID_DIGIT_IN_COUNT_ARG)?;
            Some(num)
        }
    };
    Ok(Command::Read { seek, count })
}

fn parse_readb_command<'a>(args: impl Iterator<Item = &'a [u8]>) -> ParseResult {
    let Command::Read { seek, count } = parse_read_command(args)? else {
        panic!("{}", strings::INVALID_STATE_READ_RETURNED_WRONG_TYPE);
    };
    Ok(Command::Readb { seek, count })
}

fn parse_write_command<'a>(
    mut args: impl Iterator<Item = &'a [u8]>,
    command_line: &[u8],
) -> ParseResult {
    let seek_arg = args.next().ok_or(strings::MISSING_SEEK_ARG)?;
    let seek = parse_seek_arg(seek_arg)?;

    // Enumerate space-separated "words". Each whitespace character has two
    // "words" on either side, which could be 0 length.
    // E.g., "  write " => (0, b""), (1, b""), (2, b"write"), (3, b"")
    //     After filter => (2, b"write")
    let mut cmd_words = command_line
        .split(u8::is_ascii_whitespace)
        .enumerate()
        .filter(|(_, chunk)| !chunk.is_empty());

    // len(op + seek)
    let op_n_seek_len = cmd_words
        .by_ref()
        .take(2)
        .fold(0, |acc, (_, chunk)| acc + chunk.len());

    // Char index of first valid character in write contents.
    let write_buf_start = match cmd_words.next() {
        Some((i, _)) => op_n_seek_len + i,
        None => command_line.len(),
    };

    Ok(Command::Write {
        seek,
        index: write_buf_start,
    })
}

fn parse_writeb_command<'a>(mut args: impl Iterator<Item = &'a [u8]>) -> ParseResult {
    let seek_arg = args.next().ok_or(strings::MISSING_SEEK_ARG)?;
    let seek = parse_seek_arg(seek_arg)?;

    let mut bytes: Vec<u8> = Vec::with_capacity(1024);

    let byte_args = args.map(String::from_utf8_lossy);

    for byte_arg in byte_args {
        // TODO: use u8::from_ascii_radix once stable
        let byte = u8::from_str_radix(&byte_arg, 16).map_err(|_| strings::INVALID_BYTE_ARG)?;
        bytes.push(byte);
    }

    Ok(Command::Writeb { seek, bytes })
}

fn parse_seek_command<'a>(mut args: impl Iterator<Item = &'a [u8]>) -> ParseResult {
    let seek_arg = args.next().ok_or(strings::MISSING_SEEK_ARG)?;
    let seek = parse_seek_arg(seek_arg)?;
    Ok(Command::Seek(seek))
}

fn parse_seek_arg(word: &[u8]) -> Result<SeekFrom, Box<dyn Error>> {
    let seek_arg = String::from_utf8_lossy(word);
    if seek_arg.is_empty() {
        Err(strings::MISSING_SEEK_ARG)?;
    };

    let first_char = seek_arg
        .chars()
        .next()
        .ok_or(strings::WEIRD_SEEK_ARG_NOT_FOUND)?;
    match first_char {
        '.' if seek_arg.len() == 1 => Ok(SeekFrom::Current(0)),
        '<' if seek_arg.len() == 1 => Ok(SeekFrom::End(0)),
        '+' | '-' => {
            let num = seek_arg
                .parse()
                .map_err(|_| strings::INVALID_DIGIT_IN_SEEK_ARG)?;
            Ok(SeekFrom::Current(num))
        }
        '0'..='9' if seek_arg.ends_with('<') => {
            let num: i64 = (&seek_arg[..seek_arg.len() - 1])
                .parse()
                .map_err(|_| strings::INVALID_DIGIT_IN_SEEK_ARG)?;
            Ok(SeekFrom::End(0 - num))
        }
        '0'..='9' => {
            let num = seek_arg
                .parse()
                .map_err(|_| strings::INVALID_DIGIT_IN_SEEK_ARG)?;
            Ok(SeekFrom::Start(num))
        }
        _ => Err(strings::INVALID_SEEK_ARG)?,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Command::*;

    #[test]
    fn empty_input_returns_nop_cmd() {
        let cmd = parse_input(b"").unwrap();

        assert_eq!(cmd, Nop);
    }

    #[test]
    fn invalid_input_returns_err() {
        let inputs: &[&[u8]] = &[
            b"\n",
            b" ",
            Command::OP_READ,
            Command::OP_WRITE,
            Command::OP_SEEK,
        ];

        for input in inputs {
            assert!(parse_input(input).is_err());
        }
    }

    #[test]
    fn superfluous_whitespace_is_ignored() {
        let inputs: &[&[u8]] = &[
            b" \t\r\x0c\n  help  \t\r\x0c\n",
            b"  \t\x0c\n  read  \t\r . \x0c\n ",
        ];

        for input in inputs {
            assert!(parse_input(input).is_ok());
        }
    }

    #[test]
    fn write_returns_correct_byte_position() {
        let inputs: &[(&[u8], usize)] = &[
            (b" \twrite \r. \x0c  x \n  ", 14),
            (b"write . \t  ", 11),
            (b"write .\n", 8),
            (b"write .", 7),
        ];

        for input in inputs {
            let cmd = parse_input(input.0).unwrap();
            assert_eq!(
                cmd,
                Write {
                    seek: SeekFrom::Current(0),
                    index: input.1
                }
            );
        }
    }

    #[test]
    fn invalid_number_returns_err() {
        let inputs: &[&[u8]] = &[
            b"seek x",
            b"seek -1<",
            b"seek +1<",
            b"seek -+3",
            b"seek +-6",
            b"seek --2",
            b"seek ++9",
            b"read . -1",
            b"read . x",
        ];

        for input in inputs {
            assert!(parse_input(input).is_err());
        }
    }

    #[test]
    fn seek_arg_returns_correct_seekfrom_value() {
        let dot = parse_input(b"seek .").unwrap();
        let forwards = parse_input(b"seek +0").unwrap();
        let backwards = parse_input(b"seek -0").unwrap();
        let from_end = parse_input(b"seek <").unwrap();
        let from_end_0 = parse_input(b"seek 0<").unwrap();
        let from_end_1 = parse_input(b"seek 1<").unwrap();
        let from_start_0 = parse_input(b"seek 0").unwrap();
        let from_start_1 = parse_input(b"seek 1").unwrap();

        assert_eq!(dot, Seek(SeekFrom::Current(0)));
        assert_eq!(forwards, Seek(SeekFrom::Current(0)));
        assert_eq!(backwards, Seek(SeekFrom::Current(0)));
        assert_eq!(from_end, Seek(SeekFrom::End(0)));
        assert_eq!(from_end_0, Seek(SeekFrom::End(0)));
        assert_eq!(from_end_1, Seek(SeekFrom::End(-1)));
        assert_eq!(from_start_0, Seek(SeekFrom::Start(0)));
        assert_eq!(from_start_1, Seek(SeekFrom::Start(1)));
    }

    #[test]
    fn writeb_returns_correct_byte_vector() {
        let input = b"writeb . 0  fF\t 00040";

        let cmd = parse_input(input).unwrap();

        assert_eq!(
            cmd,
            Command::Writeb {
                seek: SeekFrom::Current(0),
                bytes: vec![0, 0xff, 0x40]
            }
        )
    }

    #[test]
    fn writeb_returns_err_for_invalid_bytes() {
        let inputs: &[&[u8]] = &[b"writeb . g", b"writeb . 100", b"writeb . 40 41 100"];

        for input in inputs {
            assert!(parse_input(input).is_err());
        }
    }
}
