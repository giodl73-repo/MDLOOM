# Stats Command Module Closeout

## Mission

Begin the command-module split with the most self-contained command:
`mdloom stats`.

## Changes

- Added `src/cmd_stats.rs`.
- Moved the `stats` implementation out of `main.rs`.
- Kept command dispatch, CLI argument parsing, and output behavior unchanged.
- Reused `mdloom_lib::lint::lint_paths` from the prior orchestration wave.

## Validation

- `cargo test binary_stats_command_runs`
- `cargo test binary_stats_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`cmd_check` and `cmd_draft` are natural follow-up extractions because they now
share the same library lint orchestration seam.
