# lac

`lac` is a small command-line utility that removes LaTeX auxiliary files from a project directory.

## Features

- Cleans common LaTeX build artifacts by extension and filename suffix.
- Supports optional recursive cleanup.
- Supports dry-run mode to preview deletions.
- Removes `_minted*` directories (for the `minted` package cache).

## Installation

### Build from source

```bash
cargo build --release
```

Binary path:

```text
target/release/lac
```

### Install into Cargo bin directory

```bash
cargo install --path .
```

## Usage

```bash
lac [OPTIONS] [TARGET_DIR]
```

- `TARGET_DIR`: directory to clean. If omitted, current working directory is used.

### Options

- `--recursive`: process subdirectories recursively.
- `--dry-run`: print what would be removed without deleting anything.
- `-v`, `--verbose`: enable debug-level logging.
- `-h`, `--help`: print help.
- `-V`, `--version`: print version.

### Examples

```bash
# Clean current directory (non-recursive)
lac

# Clean a specific directory (non-recursive)
lac ./paper

# Recursively clean a directory
lac --recursive ./paper

# Preview recursive cleanup without deleting files
lac --dry-run --recursive ./paper
```

## What Gets Removed

`lac` removes files that match either of the following:

### Filename suffixes

- `.run.xml`
- `-SAVE-ERROR`

### File extensions

- `aux`
- `bbl`
- `bcf`
- `blg`
- `fdb_latexmk`
- `fls`
- `lof`
- `log`
- `lot`
- `nav`
- `out`
- `snm`
- `synctex(busy)`
- `toc`
- `vrb`

Additionally, directories whose names start with `_minted` are removed with recursive directory deletion.

## Behavior and Safety Notes

- **Safety First:** Auxiliary files (like `.log`, `.aux`, etc.) are removed only if a `.tex` file with the exact same name exists in the same directory. This prevents accidental deletion of unrelated log files or build artifacts from other tools.
- Default behavior is non-recursive.
- Recursive cleanup respects standard ignore files such as `.gitignore`, `.ignore`, and `.git/info/exclude`.
- Recursive cleanup uses parallel directory traversal, so removal logs may appear in a different order between runs.
- Logs are emitted through the standard Rust logging pipeline. By default, info-level messages and errors are shown; `--verbose` enables debug output.
- Symbolic links are not followed during traversal.
- In dry-run mode, paths are printed as `Would remove file: ...` or `Would remove directory: ...` and no deletion happens.
- Permission or deletion errors are reported to stderr, and processing continues.
- The command prints a summary: `Scanned X entries and removed Y entries.`
- If one or more deletions fail, the process exits with an error.

## Development

### Run locally

```bash
cargo run -- --help
cargo run -- --dry-run .
```

### Check and test

```bash
cargo fmt
cargo clippy
cargo check
cargo test
```

Note: there is currently no formal test suite in this repository yet, so `cargo test` may run zero tests.

## License

No license file is currently included in this repository.
