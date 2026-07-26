# CLI Global Context Closeout

## Mission

Keep parser-owned data decomposition in the CLI module so `main.rs` receives a
small dispatch-ready shape.

## Changes

- Moved `GlobalOptions` from `main.rs` into `src/cli.rs`.
- Added `Cli::into_parts()` to decompose the parsed CLI into command, top-level
  default-check paths, and global options.
- Updated `main.rs` to use `cli.into_parts()` instead of destructuring every
  root parser field itself.
- Kept command dispatch behavior and public CLI flags unchanged.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue keeping parser ownership in `cli.rs` and behavior ownership in command
modules; `main.rs` should stay focused on dispatch and shared path defaults.

