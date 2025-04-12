# Continuous Rename

A command-line tool to batch rename files in a folder/directory using a numbered pattern. The tool is designed to be simple and efficient, allowing users to quickly rename multiple files without the need for complex scripts or manual renaming.

## Usage

Use `{}` as a placeholder for the file number in the pattern. The tool will replace `{}` with the file number, starting from 1.

```sh
continuous-rename path/to/folder pattern-{}
```

This will then query for confirmation before renaming the files. If you want to skip the confirmation step, use the `-y` or `--yes` flag.

```sh
continuous-rename path/to/folder pattern-{} -y
```

## Installation

Install using `cargo`:

```sh
cargo install continuous-rename
```

Install from source:

```sh
git clone https://github.com/DestinEcarma/continuous-rename.git
cd continuous-rename
cargo build --release
mv target/release/continuous-rename/usr/local/bin
```
