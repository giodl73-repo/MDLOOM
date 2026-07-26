# Actual File Counts Close

## Outcome

Made CLI summary counts match the files actually selected by the runner:

- Added `Runner::file_count()` using the same include/exclude matcher as
  `Runner::run()`.
- Updated `mdloom check` and `mdloom stats` directory summaries to use the runner
  count instead of a separate markdown-extension approximation.
- Removed the old duplicate `count_files` helper from `main.rs`.

## Tests Added

- `binary_stats_file_count_honors_include_exclude`
- `binary_check_summary_file_count_honors_include_exclude`

## Validation

- `cargo test binary_stats_file_count_honors_include_exclude`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

If future commands need both diagnostics and file count, consider a single
`Runner::run_with_summary()` result to avoid collecting the file list twice.
