# Check Path Helper Closeout

## Mission

Keep command path defaulting in command helper modules instead of individual
command implementations.

## Changes

- Added `cmd_paths::check_paths_or_cwd`.
- Updated `cmd_check` to use the shared helper for both explicit `mdloom check`
  and default-check routing.
- Removed the duplicate private path fallback helper from `cmd_check`.
- Preserved check behavior: command paths win, top-level default-check paths are
  honored next, and current directory remains the final fallback.

## Validation

- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_stats_command_runs`
- `cargo test runner_path_summary_counts_file_and_directory_inputs`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Keep command-specific path fallback helpers in `cmd_paths` when the behavior is
shared across dispatch routes or differs from ordinary cwd fallback.
