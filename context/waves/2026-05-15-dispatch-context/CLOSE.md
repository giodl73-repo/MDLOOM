# Dispatch Context Closeout

## Mission

Name the shared routing context inside dispatch so command selection does not
pass loose globals and top-level paths around.

## Changes

- Added a private `DispatchContext` wrapper in `dispatch.rs`.
- Bundled global options and top-level default-check paths into the context.
- Routed config-aware commands through `context.globals()`.
- Routed explicit and default check paths through context methods.
- Preserved command behavior while making the dispatch match read as command
  selection plus context forwarding.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_pin_list_prints_registered_davinci_entries`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

If dispatch accumulates more helper behavior, keep it on `DispatchContext` or
move it into command-owned adapters rather than adding ad hoc local plumbing.

