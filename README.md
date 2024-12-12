# conv-trackuint-log

A command-line tool for converting and cleaning CSV log data, particularly focusing on converting time formats and ensuring chronological order.

## Features

- Reads CSV input from a file or standard input.
- Attempts to parse and convert timestamps in the first column from a `MM/DD/YY, HH:MM:SS AM/PM GMT+X` format to a 24-hour `YYYY/MM/DD HH:MM:SS` format.
- Ensures that records are chronologically ordered. If a record's time is older than a previous record, that record is skipped.
- Maintains the original CSV encoding (Shift-JIS) and passes through non-time fields without modification.
- Writes cleaned and converted CSV output to a file or standard output.

## Usage

```bash
Usage: conv-trackuint-log [OPTIONS]

Options:
  -i, --input <INPUT>    Input CSV file. If not specified, read from stdin
  -o, --output <OUTPUT>  Output CSV file. If not specified, write to stdout
  -h, --help             Print help
  -V, --version          Print version
```

### Example

```bash
# Reading from input.csv and writing to output.csv
conv-trackuint-log -i input.csv -o output.csv

# Reading from stdin and writing to stdout
cat input.csv | conv-trackuint-log
```

## Build & Installation

1. Ensure you have a Rust environment set up.
2. Clone this repository.
3. Run `cargo build --release` to build the binary.
4. The compiled binary will be located in the `target/release` directory.

## Notes

- The application assumes the input CSV is encoded in Shift-JIS.
- Records that cannot be parsed or are out-of-order will be skipped, and a warning will be logged.
- Logging can be controlled via the `RUST_LOG` environment variable.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

