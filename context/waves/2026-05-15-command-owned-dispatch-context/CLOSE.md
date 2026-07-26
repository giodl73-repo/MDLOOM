# Command-Owned Dispatch Context Closeout

## Mission

Make the dispatch routing context own the selected command along with globals
and default-check paths.

## Changes

- Added `command: Option<Command>` to `DispatchContext`.
- Changed `dispatch::run` to build an owned context and call `run()` directly.
- Kept parser-to-dispatch conversion behind `DispatchContext::from_cli` and
  `DispatchContext::from_input`.
- Preserved all explicit command routing and default-check behavior.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_stats_command_runs`
- `cargo test cli_compile_output_dir_flag`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Keep `dispatch::run` as a shallow shell. Future routing state should be added to
the owned dispatch context rather than threaded through positional arguments.
