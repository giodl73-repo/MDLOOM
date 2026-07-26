# Spec Generate Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the `mdloom spec-generate`
argument shape into the spec-generate command module.

## Changes

- Added `cmd_spec_generate::Args` with the clap argument definitions for
  `mdloom spec-generate`.
- Replaced the inline `Command::SpecGenerate { ... }` fields in `main.rs` with a
  tuple variant that references `cmd_spec_generate::Args`.
- Kept spec generation behavior in `cmd_spec_generate::run` unchanged.
- Preserved the spec-generate CLI regression that emits DaVinci TOML.

## Validation

- `cargo test cli_spec_generate_outputs_toml`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue migrating command argument groups from `main.rs` into their command
modules in small, independently validated waves.

