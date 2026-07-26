# Named Run Summary Closeout

## Mission

Finish the runner/reporting API cleanup by replacing positional tuple results
with a named summary type.

## Changes

- Added `RunSummary` with named `diagnostics` and `files_checked` fields.
- Replaced `Runner::run_with_count()` with `Runner::run_summary()`.
- Exported `RunSummary` from `mdloom_lib`.
- Updated `mdloom check` and `mdloom stats` to read named summary fields.

## Validation

- Reuse the file-count regressions from the prior runner summary wave:
  `binary_stats_file_count_honors_include_exclude` and
  `binary_check_summary_file_count_honors_include_exclude`.
- `cargo fmt`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

If runner reporting grows again, add fields to `RunSummary` rather than adding
parallel return values or duplicating directory walks.
