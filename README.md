# rtail

A Rust implementation of the Unix `tail` command, which outputs the last part of files to standard output. It supports options for following file changes, specifying the number of lines or bytes to display, and more.

## Features

- Display the last N lines or bytes of a file.
- Follow file changes in real-time (like `tail -f`).
- Support for multiple files.
- Handles both text and binary files.
- Pipe support for reading from standard input.
- Handles NUL-terminated lines.
- Graceful handling of file rotations when following files.

## Usage

```sh
rtail [OPTIONS] [FILENAME]...
```

## Examples

```sh
# Basic usage: print the last 10 lines of file.txt
rtail file.txt

# Print the last 20 lines of file.txt
rtail -n 20 file.txt

# Print the last 100 bytes of file.txt
rtail -c 100 file.txt

# Follow the file logfile.log for new lines
rtail -f logfile.log

# Follow logfile.log and terminate when process with PID 1234 ends
rtail -f --pid 1234 logfile.log

# Follow file logfile.log by name (useful for log rotation)
rtail --follow-name logfile.log

# Print all lines starting from line 50
rtail -n +50 file.txt

# Print all bytes starting from byte 200
rtail -c +200 file.txt

# Print the last 10 lines of multiple files
rtail -n 10 file1.txt file2.txt

# Read from standard input and print the last 15 lines
cat file.txt | rtail -n 15

# Print the last 10 lines of a file with NUL-terminated lines
rtail -z file_with_nul_lines.txt
```

## Options

- `-n, --lines <NUM>`: Output the last NUM lines, or use `+NUM` to start from line NUM.
- `-c, --bytes <NUM>`: Output the last NUM bytes, or use
  `+NUM` to start from byte NUM.
- `-f, --follow`: Output appended data as the file grows.
- `--pid <PID>`: With `-f`, terminate after process ID PID dies.
- `--follow-name`: Follow the file by name, useful for log rotation.
- `-q, --quiet`: Never output headers giving file names.
- `-v, --verbose`: Always output headers giving file names.
- `-z, --zero-terminated`: Line delimiter is NUL, not newline.
- `--verbose`: Always output headers giving file names.
- `-q, --quiet`, `--silent`: Never output headers giving file names.
