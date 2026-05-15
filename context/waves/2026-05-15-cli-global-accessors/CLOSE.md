# CLI Global Accessors Closeout

## Mission

Keep the root CLI parser and global option bundle encapsulated after command
routing moved into dispatch.

## Changes

- Made root `Cli` parser fields private; only `Cli::into_parts()` exposes parsed
  command, top-level paths, and globals.
- Made `GlobalOptions` fields private.
- Added explicit accessors for global config, format, errors-only, no-fail, and
  output options.
- Updated dispatch and check global adaptation to use those accessors instead of
  reading fields directly.

## Validation

- `cargo test cli_proof_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_config_prints_effective_cascaded_config`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Dispatch is now close to pure routing. Future waves can move config-aware command
adapters into command modules to reduce the remaining dispatch/context coupling.

