# CLI Path Extraction Closeout

## Mission

Make dispatch path handling more declarative by centralizing command-path
extraction with CLI default semantics.

## Changes

- Added `cli::take_paths_or_cwd` for commands whose path vectors default to the
  current directory.
- Added `cli::take_check_paths_or_cwd` for explicit/default check routing, where
  top-level paths can supply the default check targets.
- Replaced repeated `std::mem::take(&mut args.paths)` calls in `dispatch.rs`
  with the new helpers.
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

Keep parser/path-default semantics in `cli.rs` and command routing in
`dispatch.rs`; future cleanup should improve routing readability without moving
behavior back into `main.rs`.

