# Depends Command Module Closeout

## Mission

Continue the command-module split by extracting the reverse dependency lookup
command, `proof depends`.

## Changes

- Added `src/cmd_depends.rs`.
- Moved reverse dependency lookup, proof-root discovery from the current
  directory, text rendering, and JSON rendering out of `main.rs`.
- Preserved existing root handling: `--root` when supplied, nearest ancestor
  `proof.toml` otherwise, and current directory as the fallback.
- Added a CLI regression that scans a `.source.md` proof fence and verifies JSON
  output reports the reference.

## Validation

- `cargo test binary_depends_prints_json_references`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`proof tree`, `proof spec-generate`, `proof compile`, and `proof layout` remain
in `main.rs`; `tree` is the next contained command-family extraction candidate.
