# Check Dispatch Flags Closeout

## Mission

Continue simplifying the CLI shell after command argument modularization by
moving check-specific dispatch flag aggregation into the check command module.

## Changes

- Added `cmd_check::Flags` as the normalized check-specific option state.
- Added `cmd_check::Args::flags()` to derive runtime flags from clap arguments.
- Replaced mutable `main.rs` locals for `--daVinci`, `--by-code`,
  `--deduplicate`, and `--unused` with a single `cmd_check::Flags` value.
- Kept default check path routing and global CLI options in `main.rs`.
- Kept check behavior and output unchanged.

## Validation

- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

With command argument shapes and check flag normalization complete, future CLI
cleanup can focus on dispatch ergonomics and separating default-check routing
from command dispatch.

