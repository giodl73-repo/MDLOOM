# Spec Generate Command Module Closeout

## Mission

Continue the command-module split by extracting the invariant suggestion command,
`mdloom spec-generate`.

## Changes

- Added `src/cmd_spec_generate.rs`.
- Moved mdpath URI resolution, ID derivation, static invariant generation,
  optional AI-assisted invariant generation, output-file handling, and summary
  rendering out of `main.rs`.
- Kept global explicit-config behavior by loading config inside the command
  module with the same `load_config_for_path` helper used by other commands.
- Preserved the existing CLI regression that verifies `spec-generate` emits
  DaVinci TOML.

## Validation

- `cargo test cli_spec_generate_outputs_toml`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`mdloom compile` and `mdloom layout` remain in `main.rs`; `layout` is smaller,
while `compile` may deserve a command module plus helper extraction.
