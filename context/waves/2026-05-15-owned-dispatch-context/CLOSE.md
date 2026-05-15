# Owned Dispatch Context Closeout

## Mission

Make command routing own the parsed execution context instead of borrowing from a
partially consumed CLI parser boundary.

## Changes

- Changed `DispatchContext` to own top-level default-check paths and global
  command options.
- Added `DispatchInput` take-style accessors for command, top-level paths, and
  globals.
- Updated dispatch routing to construct `(DispatchContext, command)` from one
  consumed `DispatchInput`.
- Preserved command routing, global option propagation, and default-check path
  behavior.

## Validation

- `cargo test cli_proof_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_stats_command_runs`
- `cargo test cli_compile_output_dir_flag`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Keep parser output consumption explicit at the dispatch boundary. New routing
state should be owned by dispatch or command context modules, not borrowed from
CLI parser structs.
