# Command Path Accessors Closeout

## Mission

Keep command path extraction owned by command modules instead of exposing path
fields to dispatch.

## Changes

- Added `take_paths()` accessors to check, compile, draft, and stats command
  argument structs.
- Narrowed draft, stats, and compile path fields so dispatch no longer reaches
  into those command internals.
- Updated CLI path default helpers to consume command-owned path accessors.
- Preserved existing cwd defaulting and top-level `proof PATH` check routing.

## Validation

- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_draft_command_writes_plan`
- `cargo test binary_stats_command_runs`
- `cargo test cli_proof_version_exits_zero`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue moving any remaining dispatch-time command data shaping behind
module-owned adapters before tightening command argument visibility further.

