# Depends Command Module Closeout

## Mission

Continue the command-module split by extracting the reverse dependency lookup
command, `mdloom depends`.

## Changes

- Added `src/cmd_depends.rs`.
- Moved reverse dependency lookup, mdloom-root discovery from the current
  directory, text rendering, and JSON rendering out of `main.rs`.
- Preserved existing root handling: `--root` when supplied, nearest ancestor
  `mdloom.toml` otherwise, and current directory as the fallback.
- Added a CLI regression that scans a `.source.md` mdloom fence and verifies JSON
  output reports the reference.

## Validation

- `cargo test binary_depends_prints_json_references`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`mdloom tree`, `mdloom spec-generate`, `mdloom compile`, and `mdloom layout` remain
in `main.rs`; `tree` is the next contained command-family extraction candidate.
