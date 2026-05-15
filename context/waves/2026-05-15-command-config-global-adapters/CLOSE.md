# Command Config Global Adapters Closeout

## Mission

Move remaining global config plumbing out of dispatch and into config-aware
command modules.

## Changes

- Added command-owned global adapters for compile, config, draft, pin-list,
  spec-generate, and stats.
- Made each adapter pass the global config override to its module-private
  implementation.
- Updated dispatch to route by command without passing raw `globals.config()`
  into individual command implementations.
- Preserved explicit `--config` behavior and command output.

## Validation

- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_stats_command_runs`
- `cargo test binary_draft_command_writes_plan`
- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_pin_list_prints_registered_davinci_entries`
- `cargo test binary_spec_generate_static_outputs_toml`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Dispatch is now mostly command selection plus context forwarding. A later wave
can consider a `DispatchContext` wrapper if more shared context appears.

