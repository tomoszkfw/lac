# lac — LaTeX auxiliary cleaner

`lac` is a small command-line tool that cleans LaTeX auxiliary artifacts from a directory.

It can:

- remove known LaTeX aux files
- remove directories named `_minted`
- run non-recursively (default) or recursively (`-r`)
- preview actions without deleting anything (`--dry-run`)

## Installation

Prerequisite: Rust toolchain.

From the repository root:

```sh
cargo install --path .
```

Or build a release binary:

```sh
cargo build --release
```

Binary path:

`target/release/lac`

## Usage

```sh
lac [TARGET_DIR] [OPTIONS]
```

### Arguments

- `TARGET_DIR` (optional): directory to clean  
  - default: current working directory

### Options

- `-r, --recursive`  
  Traverse subdirectories recursively.
- `--dry-run`  
  Show what would be removed, without deleting.
- `-h, --help`  
  Show help.
- `-V, --version`  
  Show version.

## Examples

Clean current directory (non-recursive):

```sh
lac
```

Preview current directory cleanup (non-recursive):

```sh
lac --dry-run
```


Recursively preview cleanup in a specific directory:

```sh
lac path/to/project -r --dry-run
```

Recursively clean a specific directory:

```sh
lac path/to/project -r
```

## What gets removed

### Files

A file is removed if either condition matches:

1. Its filename ends with one of:
   - `.run.xml`
   - `-SAVE-ERROR`

2. Its extension is one of:
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

### Directories

- Any directory whose name is exactly `_minted` is removed.
- Once matched, that directory is deleted as a whole and not traversed further.

## Runtime behavior

- If `TARGET_DIR` does not exist or is not a directory, the command exits with an error.
- Symbolic links are not followed during traversal.
- In `--dry-run` mode, output uses:
  - `Would remove: ...`
  - `Would remove directory: ...`
- In normal mode, output uses:
  - `Removed: ...`
  - `Removed directory: ...`
- A summary is always printed:

`Summary: Scanned <N> files, and found <M> files matched. Removed <R> files.`

- If some removals fail, failures are listed at the end and the process exits non-zero.
