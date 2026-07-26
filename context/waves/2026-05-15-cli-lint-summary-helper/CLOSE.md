# CLI Lint Summary Helper Closeout

## Mission

Remove duplicated CLI orchestration for linting paths now that the runner owns
file-vs-directory summary behavior.

## Changes

- Added `lint_paths(paths, config_override)` in `main.rs`.
- The helper loads the effective config for each input, builds the appropriate
  runner, calls `Runner::run_path_summary`, and aggregates diagnostics plus
  file counts into one `RunSummary`.
- Updated `mdloom check`, `mdloom stats`, and `mdloom draft` to use the helper.

## Validation

- Reuse the check/stats file-count regressions and full test suite.
- Draft uses the same helper but still builds its plan from diagnostics only.
- `cargo test runner_path_summary_counts_file_and_directory_inputs`
- `cargo test binary_stats_file_count_honors_include_exclude`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

This helper is a stepping stone toward splitting command handlers out of
`main.rs`: future `cmd_check`, `cmd_stats`, and `cmd_draft` modules can share
the same lint orchestration boundary instead of each reconstructing it.
