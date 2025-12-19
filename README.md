# TapeHead

A command-line utility for random access of file streams.

## Preview

```text
File: "test.txt" (67 bytes) [RW]

[pos:0]> seek 10
[pos:10]> read . 5
ello!
[in:5, pos:15]> read 9 5
hello
[in:5, pos:14]> quit
```

<details>
  <summary>Screen recording demo</summary>
  <img alt="TapeHead demo" src="https://raw.githubusercontent.com/emamoah/tapehead-media/main/demo.webp" />
</details>

The tool runs a REPL session in which you can run commands to read, write, and seek to any position within the file stream. It's quite useful for tasks like debugging a driver, or anything that has to do with file streams.

## Installation

```shell
cargo install --git https://github.com/emamoah/tapehead.git
```

## Motivation

I wrote a shabby version of this when I was trying to debug my [scull driver](https://github.com/emamoah/scull-rs), which controls a simple virtual character device. I was amazed I couldn't find a tool that allowed me to *statefully* seek, read and write to a file, so I just improvised some code. After that, I thought it a good idea to rewrite it properly and publish it, in case someone else one day might need it too.

The initial name I thought to give it was "seeker" (since seeking was the most important aspect) but there was already a crate with that name, so I came up with "TapeHead" which also characterises the tool's behaviour quite well.

## Running

```shell
$ tapehead test.txt
```

## Interface

```text
TapeHead v0.1.0

Author: Emmanuel Amoah (https://emamoah.com/)

Enter "help" for more information.



File: "test.txt" (67 bytes) [RW]

[pos:0]>
```

After the prologue is a line with the following details about the opened file:

```text
File: "test.txt" (67 bytes) [RW]
          ^          ^       ^
      filepath   filesize   permissions
```

The permissions can be one of these three, detected automatically when opening the file:

- `[RW]` - Readable & Writable
- `[RO]` - Read-only
- `[WO]` - Write-only

### Prompt

The prompt contains a combination of the following segments:

- `pos:<number>` - Current position of the file pointer, always shown. If the stream is not seekable (e.g., a Unix FIFO), it displays a `*` instead of a number.
- `in:<number>` - Number of bytes read from the file after executing the previous command. Not shown if nothing was read.
- `out:<number>` - Number of bytes written to the file after executing the previous command. Not shown if nothing was written.

### Usage

#### Commands

The following are the supported commands in the REPL, also accessible through the `help` command.

- `r[ead] <seek> [count]`
  - Read `count` number of bytes from the position specified by `seek`. If `count` is omitted, read to the end of the file.

- `r[ead]b <seek> [count]`
  - Same as `read`, but prints the contents as a hex dump. Useful for examining raw bytes.

- `w[rite] <seek> <contents>`
  - Write the given text in `contents` to the file from the position specified by `seek`. `contents` can contain whitespace only after the first non-whitespace character.

- `w[rite]b <seek> <hex bytes>`
  - Write the given raw bytes to the file. Bytes are written as space-separated hex values and are case-insensitive. e.g., `6C 6f 6C`.

- `s[eek] <seek>`
  - Move the file pointer to the position specified by `seek`.

- `h[elp]`
  - View this help menu.

- `q[uit]`
  - Quit the program.

#### Seek

The following syntaxes are allowed for commands with a `seek` argument.

- `.`
  - Keep file pointer at its current position.

- `number` (e.g. `9`)
  - Move to the `number`th position from the beginning of the file.

- `+number` (e.g. `+10`)
  - Move forward `number` bytes from the current position. It is possible to seek beyond the end of a file.

- `-number` (e.g. `-2`)
  - Move backwards `number` bytes from the current position. Seeking to a position before byte 0 is an error.

- `[number]<` (e.g. `40<` , `<`)
  - Move to the `number`'th byte from the end of the file. If `number` is omitted, move to the end of the file.

## Example commands

- `read .` - Read the rest of the file from the current position.
- `write < hello` - Seek to the end of the file and write the text "hello".
- `read 0 10` - Read the first 10 bytes of the file.
- `seek 5<` - Seek to the 5th-to-last byte of the file.
- `write -5 hello` - Move backwards 5 bytes and write "hello".
- `writeb 0 74 61 70 65 68 65 61 64 0a` - Write "tapehead" followed by a newline at the beginning of the file.
