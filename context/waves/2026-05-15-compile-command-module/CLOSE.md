# Compile Command Module Closeout

## Mission

Complete the command-module split for the largest remaining command,
`mdloom compile`.

## Changes

- Added `src/cmd_compile.rs`.
- Moved normal compile orchestration out of `main.rs`, including output-dir
  routing, compile target discovery, violation rendering, stale-output deletion,
  progress output, and exit behavior.
- Moved compile watch mode and its helper functions out of `main.rs`, including
  watch target discovery, initial compile pass, mdpath dependency indexing, and
  per-source recompilation.
- Updated `main.rs` so compile dispatch only normalizes default paths and calls
  `cmd_compile::run` or `cmd_compile::run_watch`.
- Preserved the existing CLI regression that verifies `--output-dir` routing.

## Validation

- `cargo test cli_compile_output_dir_flag`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The top-level CLI dispatch is now module-oriented for command implementations.
Future cleanup can focus on extracting shared command option structs or moving
remaining default-path normalization out of `main.rs`.
