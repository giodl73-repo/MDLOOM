# Library Lint Orchestration Closeout

## Mission

Turn the CLI-local lint summary helper into a reusable library boundary so
future command-module extraction does not have to move orchestration logic again.

## Changes

- Added `src/lint.rs`.
- Moved config loading for lint inputs into `load_config_for_path`.
- Moved path aggregation into `lint_paths`.
- Kept explicit `--config` semantics inside the shared boundary:
  explicit configs use `Runner::new_with_config`; automatic configs use normal
  runner cascade.
- Exported `lint_paths` from `proof_lib`.
- Updated `main.rs` to import the library helpers and removed the local
  duplicates.

## Validation

- `cargo fmt`
- `cargo test runner_path_summary_counts_file_and_directory_inputs`
- `cargo test binary_stats_file_count_honors_include_exclude`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The next command-split wave can move `cmd_check`, `cmd_stats`, or `cmd_draft`
into a module while depending on `proof_lib::lint::lint_paths` instead of
recreating config/runner orchestration.
