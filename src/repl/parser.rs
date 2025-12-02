use std::io::SeekFrom;

#[derive(Debug, PartialEq)]
pub enum Op {
    READ,
    WRITE,
    SEEK,
    HELP,
    QUIT,
    NOP,
}

#[derive(Debug)]
pub struct Command {
    pub op: Op,
    pub seek: SeekFrom,
    /// Argument used for read/write operations. For write, it's the
    /// position in the command input buffer where the data begins.
    /// For read, it's the number of bytes to read. If value is `None`
    /// in that case, read the rest of the file.
    pub arg: Option<usize>,
}

impl Command {
    /// Constructs a `Command` with the provided `op` and a default
    /// for all other fields.
    fn op(op: Op) -> Self {
        Command {
            op,
            seek: SeekFrom::Current(0),
            arg: None,
        }
    }

    /// Constructs a `SEEK` `Command` with the provided `seek` and a
    /// default for all other fields.
    fn seek(seek: SeekFrom) -> Self {
        Command {
            op: Op::SEEK,
            seek,
            arg: None,
        }
    }

    /// Constructs a `READ` `Command` with the given `seek` and the
    /// number of bytes to read. If count is `None`, read the rest
    /// of the file.
    fn read(seek: SeekFrom, count: Option<usize>) -> Self {
        Command {
            op: Op::READ,
            seek,
            arg: count,
        }
    }

    /// Constructs a `WRITE` `Command` with the given `seek` and the
    /// offset from the command line where the data to write begins.
    fn write(seek: SeekFrom, buf_start: usize) -> Self {
        Command {
            op: Op::WRITE,
            seek,
            arg: Some(buf_start),
        }
    }
}

pub fn parse_input(input: &[u8]) -> Option<Command> {
    use Op::*;

    if input.len() == 0 {
        return Some(Command::op(NOP));
    }

    // Is there a better way? i.e. <&str>::split_whitespace, but for &[u8] ?
    let mut input_words = input
        .split(u8::is_ascii_whitespace)
        .filter(|chunk| !chunk.is_empty());

    let op_str = String::from_utf8_lossy(input_words.next()?);
    let op = match op_str.to_lowercase().as_str() {
        "read" => READ,
        "write" => WRITE,
        "seek" => SEEK,
        "help" => return Some(Command::op(HELP)),
        "quit" => return Some(Command::op(QUIT)),
        _ => return None,
    };

    let seek_str = String::from_utf8_lossy(input_words.next()?);
    if seek_str.is_empty() {
        return None;
    };
    let seek = match seek_str.chars().next()? {
        '.' if seek_str.len() == 1 => SeekFrom::Current(0),
        '<' if seek_str.len() == 1 => SeekFrom::End(0),
        '+' | '-' => SeekFrom::Current(seek_str.parse().ok()?),
        '0'..='9' if seek_str.ends_with('<') => {
            let num: i64 = (&seek_str[..seek_str.len() - 1]).parse().ok()?;
            SeekFrom::End(0 - num)
        }
        '0'..='9' => SeekFrom::Start(seek_str.parse().ok()?),
        _ => return None,
    };

    if let SEEK = op {
        return Some(Command::seek(seek));
    }

    if let READ = op {
        let read_count_str = input_words.next().map(String::from_utf8_lossy);
        let read_count = match read_count_str {
            None => None,
            Some(c) => Some(c.parse::<usize>().ok()?),
        };
        return Some(Command::read(seek, read_count));
    }
    if let WRITE = op {
        let write_buf = input.trim_ascii_start()[op_str.len()..].trim_ascii_start()
            [seek_str.len()..]
            .trim_ascii_start();
        let write_buf_start = input.len() - write_buf.len();
        return Some(Command::write(seek, write_buf_start));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use Op::*;

    #[test]
    fn empty_input_returns_nop_cmd() {
        let cmd = parse_input(b"").unwrap();

        assert_eq!(cmd.op, NOP);
    }

    #[test]
    fn invalid_input_returns_none() {
        let inputs: &[&[u8]] = &[b"\n", b" ", b"read", b"write", b"seek"];

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
    fn write_without_contents_returns_some_arg() {
        let cmd = parse_input(b"write .").unwrap();

        assert!(cmd.arg.is_some());
    }

    #[test]
    fn write_returns_correct_byte_position() {
        let input = b" \twrite \r. \x0c  x \n  ";

        let cmd = parse_input(input).unwrap();

        assert_eq!(cmd.arg, Some(14));
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

        assert_eq!(dot.seek, SeekFrom::Current(0));
        assert_eq!(forwards.seek, SeekFrom::Current(0));
        assert_eq!(backwards.seek, SeekFrom::Current(0));
        assert_eq!(from_end.seek, SeekFrom::End(0));
        assert_eq!(from_end_0.seek, SeekFrom::End(0));
        assert_eq!(from_end_1.seek, SeekFrom::End(-1));
        assert_eq!(from_start_0.seek, SeekFrom::Start(0));
        assert_eq!(from_start_1.seek, SeekFrom::Start(1));
    }
}
