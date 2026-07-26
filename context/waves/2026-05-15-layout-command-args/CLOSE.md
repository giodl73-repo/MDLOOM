# Layout Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the `mdloom layout` argument shape
into the layout command module.

## Changes

- Added `cmd_layout::Args` with the clap argument definitions for `mdloom layout`.
- Replaced the inline `Command::Layout { ... }` fields in `main.rs` with a tuple
  variant that references `cmd_layout::Args`.
- Kept layout behavior in `cmd_layout::run` unchanged.
- Preserved the layout CLI regression that composes two file sources.

## Validation

- `cargo test binary_layout_composes_file_sources`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue migrating command argument groups from `main.rs` into their command
modules in small, independently validated waves.
