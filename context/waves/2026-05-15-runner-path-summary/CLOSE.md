# Runner Path Summary Closeout

## Mission

Move file-vs-directory lint summary behavior into the runner boundary so CLI
commands do not duplicate input-shape branching.

## Changes

- Added `Runner::run_path_summary(path)` for single files and directories.
- Kept `Runner::run()` as a compatibility convenience over `run_summary()`.
- Updated `mdloom check`, `mdloom stats`, and `mdloom draft` to consume
  `RunSummary` through the unified path API.
- Added a regression that verifies file inputs count as one checked file while
  directory inputs count only runner-selected markdown files.

## Validation

- `cargo test runner_path_summary_counts_file_and_directory_inputs`
- `cargo test binary_stats_file_count_honors_include_exclude`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

If commands need richer run metadata, add it to `RunSummary` and keep the
file-vs-directory input semantics centralized in `Runner::run_path_summary`.
