# lac — LaTeX auxiliary cleaner

`lac` is a small command-line utility to list or remove LaTeX auxiliary artifacts (e.g., `.aux`, `.log`, `.out`) from a target directory. It supports a safe dry-run by default, optional recursion, and removes `_minted*` directories produced by minted.

## Installation

- Prerequisite: Rust toolchain (edition 2024 compatible).
- From this repository:
  - `cargo install --path .` to install into Cargo's bin dir, **or**
  - `cargo build --release` and then copy `target/release/lac` wherever you like.

## Usage

```shell
# show help
lac --help

# dry-run (default) on current directory, recursive
lac

# dry-run on a specific directory, recursive
lac -t path/to/project

# actually delete matches, recursive
lac --execute

# actually delete matches on a specific directory, recursive
lac -t path/to/project --execute
```

### Flags

- `-t, --target <DIR>`: directory to inspect (default: `.`).
- `-r, --recursive`: recursively traverse subdirectories (default: on); set to `false` to scan only the target directory.
- `-e, --execute`: perform deletions; when omitted, actions are printed as "Would remove...".

## What gets removed

- Files with extensions: `aux`, `bbl`, `log`, `out`, `toc`, `lof`, `lot`, `fls`, `fdb_latexmk`, `blg`, `bcf`.
- Files whose names end with: `.run.xml`.
- Files whose names contain: `.synctex`.
- Directories whose names start with `_minted`.

## Behavior and safety

- **Dry-run by default**: without `--execute`, no files are deleted; intended to review actions first.
- Recursion is controlled by `--recursive` (default: true); set `false` to limit scanning to the target directory.
- Symlinks are skipped to avoid deleting outside targets.
- Deletion is pattern-based; review dry-run output carefully before rerunning with `--execute`.
