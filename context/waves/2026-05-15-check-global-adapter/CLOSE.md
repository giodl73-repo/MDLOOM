# Check Global Adapter Closeout

## Mission

Move the remaining check-specific global option adaptation out of `main.rs` and
into the check command module.

## Changes

- Added `cmd_check::Options::from_globals` to convert CLI global options into
  check runtime options.
- Added `cmd_check::run_with_globals` so both explicit and default check routes
  can call the check module directly.
- Removed the `run_check` helper from `main.rs`.
- Kept explicit `proof check`, implicit default check, and global flag behavior
  unchanged.

## Validation

- `cargo test cli_proof_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`main.rs` is now a thin dispatcher. Future cleanup should focus on readability
of command routing rather than moving behavior back into the shell.

