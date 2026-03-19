# lac — LaTeX auxiliary cleaner

`lac` is a small command-line utility to remove LaTeX auxiliary artifacts (e.g., `.aux`, `.log`, `.out`) from a target directory. It supports optional dry-run mode, optional recursion, and removes `_minted*` directories produced by minted.

## Installation

- Prerequisite: Rust toolchain (edition 2024 compatible).
- From this repository:
  - `cargo install --path .` to install into Cargo's bin dir, **or**
  - `cargo build --release` and then copy `target/release/lac` wherever you like.

## Usage

```shell
# show help
lac --help

# delete matches in current directory (non-recursive by default)
lac

# preview matches in current directory (non-recursive)
lac --dry-run

# preview matches in a specific directory, recursively
lac path/to/project -r --dry-run

# delete matches in a specific directory, recursively
lac path/to/project -r
```

### Arguments and flags

- `<TARGET_DIR>`: optional target directory (default: current directory).
- `-r, --recursive`: recursively traverse subdirectories (default: off).
- `--dry-run`: print actions without deleting files/directories.

## What gets removed

- Files with extensions: `aux`, `bbl`, `log`, `out`, `toc`, `lof`, `lot`, `fls`, `fdb_latexmk`, `blg`, `bcf`.
- Files whose names end with: `.run.xml` or `-SAVE-ERROR`.

- Directories whose names start with `_minted`.

## Behavior and safety

- Deletes by default; use `--dry-run` to preview actions first.
- Recursion is off by default; use `-r`/`--recursive` to include subdirectories.
- Symlinks are skipped to avoid deleting outside targets.
- In dry-run mode, actions are printed as "Would remove..." and "Would remove directory...".
- In execution mode, actions are printed as "Removed..." and "Removed directory...".
