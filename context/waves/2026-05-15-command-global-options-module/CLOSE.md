# Command Global Options Module Closeout

## Mission

Separate command execution context from the CLI parser module.

## Changes

- Added `cmd_context::GlobalOptions`.
- Moved global option construction and accessors out of `cli.rs`.
- Updated config-aware command modules and dispatch to import global options from
  the command context module.
- Kept `cli.rs` focused on clap parser types and the dispatch input boundary.
- Preserved all global option behavior.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_stats_command_runs`
- `cargo test cli_compile_output_dir_flag`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Command execution context should live outside parser modules. If additional
shared command context appears, add it to command context modules rather than
growing `cli.rs`.

