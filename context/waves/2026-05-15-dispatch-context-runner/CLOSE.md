# Dispatch Context Runner Closeout

## Mission

Make `dispatch::run` a parse-and-delegate shell by moving command/default routing
behind the dispatch context.

## Changes

- Added `DispatchContext::run(command)` as the single routing entry inside
  dispatch.
- Reduced top-level `dispatch::run` to parse CLI input, extract the command, build
  context, and delegate.
- Kept explicit command routing and default-check behavior unchanged.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_draft_command_writes_plan`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Keep dispatch orchestration shallow: parse at the top, route inside context, and
push command-specific behavior into command modules.

