# CLI Parser Module Closeout

## Mission

Continue slimming `main.rs` by moving the root clap parser and command enum into
a dedicated CLI module.

## Changes

- Added `src/cli.rs` to own the root `Cli` parser and `Command` enum.
- Updated `main.rs` to import `cli::{Cli, Command}` and keep only module wiring,
  dispatch, and default check routing.
- Kept command-specific argument shapes owned by their command modules.
- Preserved the public clap help/version surface.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Future CLI cleanup can focus on dispatch ergonomics and separating default-check
routing from explicit command dispatch.

