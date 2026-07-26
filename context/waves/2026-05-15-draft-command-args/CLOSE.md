# Draft Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the `mdloom draft` argument shape
into the draft command module.

## Changes

- Added `cmd_draft::Args` with the clap argument definitions for `mdloom draft`.
- Replaced the inline `Command::Draft { ... }` fields in `main.rs` with a tuple
  variant that references `cmd_draft::Args`.
- Kept default path handling in `main.rs` through the shared `paths_or_cwd`
  helper.
- Kept draft-plan generation behavior in `cmd_draft::run` unchanged.
- Preserved the draft CLI regression that writes a DraftPlan-shaped JSON file.

## Validation

- `cargo test binary_draft_command_writes_plan`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue migrating command argument groups from `main.rs` into their command
modules in small, independently validated waves.

