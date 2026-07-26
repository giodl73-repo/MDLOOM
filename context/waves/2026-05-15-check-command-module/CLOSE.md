# Check Command Module Closeout

## Mission

Continue the command-module split by extracting the default lint command,
`mdloom check`.

## Changes

- Added `src/cmd_check.rs`.
- Moved check orchestration out of `main.rs`, including DaVinci validation,
  unused-figure scanning, diagnostic filtering/sorting, output rendering,
  deduplication, summary printing, and `.mdloom/last-check.json` writing.
- Kept clap/default-path handling in `main.rs` and passed command/global flags
  through a focused `cmd_check::Options` struct.
- Preserved the existing check regression that verifies selected file counts
  honor include/exclude config.

## Validation

- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`mdloom fix`, `mdloom resolve`, and `mdloom depends` remain in `main.rs`; `fix` is
the next natural extraction target because its plan application is already
encapsulated in mdloom_lib and has an existing CLI regression.
