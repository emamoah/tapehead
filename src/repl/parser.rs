use std::io::SeekFrom;

#[derive(Debug, PartialEq)]
pub enum Command {
    Read {
        seek: SeekFrom,
        count: Option<usize>,
    },
    Write {
        seek: SeekFrom,
        index: usize,
    },
    Seek(SeekFrom),
    Help,
    Quit,
    Nop,
}

impl Command {
    const OP_READ: &[u8] = b"read";
    const OP_WRITE: &[u8] = b"write";
    const OP_SEEK: &[u8] = b"seek";
    const OP_HELP: &[u8] = b"help";
    const OP_QUIT: &[u8] = b"quit";
}

pub fn parse_input(input: &[u8]) -> Option<Command> {
    if input.len() == 0 {
        return Some(Command::Nop);
    }

    // Is there a better way? i.e. <&str>::split_whitespace, but for &[u8] ?
    let mut input_words = input
        .split(u8::is_ascii_whitespace)
        .filter(|chunk| !chunk.is_empty());

    let op = input_words.next()?;

    match op.to_ascii_lowercase().as_slice() {
        Command::OP_READ => parse_read_command(input_words),
        Command::OP_WRITE => parse_write_command(input_words, input),
        Command::OP_SEEK => parse_seek_command(input_words),
        Command::OP_HELP => Some(Command::Help),
        Command::OP_QUIT => Some(Command::Quit),
        _ => None,
    }
}

fn parse_read_command<'a>(mut args: impl Iterator<Item = &'a [u8]>) -> Option<Command> {
    let seek = parse_seek_arg(args.next()?)?;

    let read_count_str = args.next().map(String::from_utf8_lossy);
    let read_count = match read_count_str {
        None => None,
        Some(c) => Some(c.parse::<usize>().ok()?),
    };
    Some(Command::Read {
        seek,
        count: read_count,
    })
}

fn parse_write_command<'a>(
    mut args: impl Iterator<Item = &'a [u8]>,
    command_line: &[u8],
) -> Option<Command> {
    let seek_arg = args.next()?;
    let seek = parse_seek_arg(seek_arg)?;

    let write_buf = command_line.trim_ascii_start()[Command::OP_WRITE.len()..].trim_ascii_start()
        [seek_arg.len()..]
        .trim_ascii_start();
    let write_buf_start = command_line.len() - write_buf.len();
    return Some(Command::Write {
        seek,
        index: write_buf_start,
    });
}

fn parse_seek_command<'a>(mut args: impl Iterator<Item = &'a [u8]>) -> Option<Command> {
    let seek = parse_seek_arg(args.next()?)?;
    Some(Command::Seek(seek))
}

fn parse_seek_arg(word: &[u8]) -> Option<SeekFrom> {
    let seek_str = String::from_utf8_lossy(word);
    if seek_str.is_empty() {
        return None;
    };

    match seek_str.chars().next()? {
        '.' if seek_str.len() == 1 => Some(SeekFrom::Current(0)),
        '<' if seek_str.len() == 1 => Some(SeekFrom::End(0)),
        '+' | '-' => Some(SeekFrom::Current(seek_str.parse().ok()?)),
        '0'..='9' if seek_str.ends_with('<') => {
            let num: i64 = (&seek_str[..seek_str.len() - 1]).parse().ok()?;
            Some(SeekFrom::End(0 - num))
        }
        '0'..='9' => Some(SeekFrom::Start(seek_str.parse().ok()?)),
        _ => None,
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
    fn invalid_input_returns_none() {
        let inputs: &[&[u8]] = &[
            b"\n",
            b" ",
            Command::OP_READ,
            Command::OP_WRITE,
            Command::OP_SEEK,
        ];

        for input in inputs {
            assert!(parse_input(input).is_none());
        }
    }

    #[test]
    fn superfluous_whitespace_is_ignored() {
        let inputs: &[&[u8]] = &[
            b" \t\r\x0c\n  help  \t\r\x0c\n",
            b"  \t\x0c\n  read  \t\r . \x0c\n ",
        ];

        for input in inputs {
            assert!(parse_input(input).is_some());
        }
    }

    #[test]
    fn write_returns_correct_byte_position() {
        let input = b" \twrite \r. \x0c  x \n  ";

        let cmd = parse_input(input).unwrap();

        assert_eq!(
            cmd,
            Write {
                seek: SeekFrom::Current(0),
                index: 14
            }
        );
    }

    #[test]
    fn invalid_numbers_returns_none() {
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
            assert!(parse_input(input).is_none());
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
}
