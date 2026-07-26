# CLI Dispatch Input Closeout

## Mission

Make the CLI-to-dispatch boundary explicit by replacing positional tuple output
with a named dispatch input.

## Changes

- Added `cli::DispatchInput` with named `command`, `top_level_paths`, and
  `globals` fields.
- Replaced `Cli::into_parts()` with `Cli::into_dispatch()`.
- Updated dispatch to destructure the named boundary type instead of relying on
  tuple order.
- Tightened `cmd_init::run` visibility to `pub(crate)`.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_init_command_writes_default_config`
- `cargo test binary_check_summary_file_count_honors_include_exclude`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

The dispatch boundary is now named. If more dispatch context appears, prefer
adding fields to `DispatchInput` or command-owned adapters over adding positional
tuples.

