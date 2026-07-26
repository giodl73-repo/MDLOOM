# Status Command Module Closeout

## Mission

Continue the command-module split by extracting `mdloom status`, including its
small cached-check JSON helper.

## Changes

- Added `src/cmd_status.rs`.
- Moved source/compiled/stale counting, last-check cache rendering, and config
  summary rendering out of `main.rs`.
- Updated dispatch to call `cmd_status::run`.
- Added a CLI smoke regression that verifies `mdloom status <dir>` reports the
  project summary fields.

## Validation

- `cargo test binary_status_command_reports_project_summary`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`cmd_check` remains the largest lint-facing extraction. The status split removes
another self-contained block from `main.rs` without touching check rendering.
