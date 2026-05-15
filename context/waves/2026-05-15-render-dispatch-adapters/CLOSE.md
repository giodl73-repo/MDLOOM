# Render Dispatch Adapters Closeout

## Mission

Finish the module-owned dispatch adapter pattern for render-oriented commands.

## Changes

- Updated `cmd_compile::run` to accept `cmd_compile::Args` directly alongside
  the dispatch-normalized paths.
- Moved compile watch-vs-one-shot routing into `cmd_compile::run`.
- Kept the one-shot compile implementation as an internal `run_once` helper and
  watch mode as an internal helper.
- Updated `cmd_layout::run` to accept `cmd_layout::Args` directly.
- Simplified `main.rs` dispatch for `Command::Compile` and `Command::Layout`.
- Kept compile path defaulting in `main.rs`, where shared CLI path semantics are
  already centralized.
- Kept compile and layout behavior unchanged.

## Validation

- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_layout_composes_file_sources`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

With command modules now consuming their own argument structs, future CLI cleanup
can focus on the remaining default-check routing and root path helpers.

