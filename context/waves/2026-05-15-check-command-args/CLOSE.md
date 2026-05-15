# Check Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the `proof check` argument shape
into the check command module.

## Changes

- Added `cmd_check::Args` with the clap argument definitions for `proof check`.
- Replaced the inline `Command::Check { ... }` fields in `main.rs` with a tuple
  variant that references `cmd_check::Args`.
- Kept default check path resolution and global flag handling in `main.rs`.
- Preserved the existing check help and file-count regressions.

## Validation

- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Other command argument groups can be migrated module-by-module where the move
reduces `main.rs` without obscuring the public CLI structure.
