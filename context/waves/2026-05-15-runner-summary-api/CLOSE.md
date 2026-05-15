# Runner Summary API Close

## Outcome

Reduced runner/reporting duplication introduced by actual file counts:

- Replaced separate `Runner::file_count()` + `Runner::run()` directory flows with
  `Runner::run_with_count()`.
- `run_with_count()` collects matching files once, returns the selected count,
  and lints that same file set in parallel.
- `proof check` and `proof stats` now use the one-pass API for directory inputs.

## Tests Reused

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

If more summary fields are needed, promote the tuple to a named `RunSummary`
struct rather than adding more parallel return values.
