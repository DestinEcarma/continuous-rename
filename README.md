# continuous-rename

A small CLI for batch-renaming files in a directory using continuous numbers.

It renames files into a predictable sequence like `1.png`, `2.png`, or custom names such as `photo-001.jpg`, `photo-002.jpg`, and so on.

> [!NOTE]
>
> - This tool renames files in a single directory only
> - It does not recurse into subdirectories
> - Non-matching files are assigned the next available number
> - Use `--dry-run` first when working on important files

## Features

- Rename files in one directory using continuous numbering
- Supports `{}` placeholders for plain numbering
- Supports `{n:WIDTH}` for zero-padded numbering
- Skips files that already match the target pattern
- Preserves file extensions
- Supports confirmation prompts
- Supports `--dry-run` preview mode

## Installation

Install from crates.io:

```sh
cargo install continuous-rename
```

Build from source:

```sh
git clone https://github.com/DestinEcarma/continuous-rename.git
cd continuous-rename
cargo build --release
install -Dm755 target/release/continuous-rename ~/.local/bin/continuous-rename
```

## Usage

```sh
continuous-rename <target> [pattern] [OPTIONS]
```

### Arguments

- `target` — directory containing the files to rename
- `pattern` — optional output name pattern

### Options

- `-y, --yes` — skip confirmation prompts
- `--dry-run` — preview changes without renaming files
- `-h, --help` — show help
- `-V, --version` — show version

## Patterns

Use `{}` to insert the current number:

```sh
continuous-rename ./images "file-{}"
```

This produces names like:

```text
file-1.png
file-2.jpg
file-3.webp
```

Use `{n:WIDTH}` to zero-pad the number:

```sh
continuous-rename ./images "photo-{n:03}"
```

This produces names like:

```text
photo-001.png
photo-002.jpg
photo-003.webp
```

If no pattern is given, the number itself is used:

```sh
continuous-rename ./images
```

This produces names like:

```text
1.png
2.jpg
3.webp
```

If the pattern has no placeholder, the number is appended to the end:

```sh
continuous-rename ./images "scan"
```

This produces names like:

```text
scan1.png
scan2.jpg
scan3.webp
```

## Examples

Rename files using a basic pattern:

```sh
continuous-rename ./photos "image-{}"
```

Rename files with zero-padded numbers:

```sh
continuous-rename ./photos "vacation-{n:04}"
```

Preview changes without touching files:

```sh
continuous-rename ./photos "image-{}" --dry-run
```

Skip all confirmation prompts:

```sh
continuous-rename ./photos "image-{}" --yes
```

## Behavior

Files are processed in sorted order.

Existing files that already match the target numbering pattern are skipped, and their numbers are treated as already used.

The tool preserves each file's extension while changing only the base filename.

## Example workflow

Suppose a directory contains:

```text
a.png
b.jpg
photo-001.webp
```

Running:

```sh
continuous-rename ./dir "photo-{n:03}"
```

may result in:

```text
photo-001.webp
photo-002.png
photo-003.jpg
```

