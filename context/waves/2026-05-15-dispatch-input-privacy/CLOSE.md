# Dispatch Input Privacy Closeout

## Mission

Keep the named CLI dispatch boundary explicit without exposing its fields
outside the CLI module.

## Changes

- Made `DispatchInput` fields private.
- Added accessors for top-level paths and global options.
- Added a consuming `take_command()` method for dispatch routing.
- Changed `DispatchContext` to borrow `DispatchInput` instead of owning loose
  copies of its fields.
- Preserved command routing and default-check behavior.

## Validation

- `cargo test cli_proof_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_help_documents_progress_only_for_compile`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Keep parse-boundary state private to the CLI module; add named accessors for
new dispatch needs rather than exposing fields directly.

