# CLI Path Defaults Closeout

## Mission

Keep CLI path default semantics with the parser-owned CLI module instead of the
dispatch shell.

## Changes

- Moved `paths_or_cwd` from `main.rs` into `src/cli.rs`.
- Moved `check_paths_or_cwd` from `main.rs` into `src/cli.rs`.
- Updated `main.rs` dispatch to import and use the CLI-owned path default
  helpers.
- Kept draft, stats, compile, explicit check, and default check path behavior
  unchanged.

## Validation

- `cargo test binary_draft_command_writes_plan`
- `cargo test binary_stats_command_runs`
- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

With parser decomposition and path defaults in `cli.rs`, `main.rs` can stay a
thin dispatcher over command modules and shared check routing.

