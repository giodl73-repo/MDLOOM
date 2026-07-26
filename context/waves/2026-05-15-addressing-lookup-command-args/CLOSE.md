# Addressing Lookup Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the related `mdloom resolve` and
`mdloom depends` argument shapes into their command modules.

## Changes

- Added `cmd_resolve::Args` with the clap argument definitions for
  `mdloom resolve`.
- Added `cmd_depends::Args` with the clap argument definitions for
  `mdloom depends`.
- Replaced the inline `Command::Resolve { ... }` and `Command::Depends { ... }`
  fields in `main.rs` with tuple variants that reference their command-module
  argument types.
- Kept resolve and reverse-dependency behavior unchanged.
- Preserved the resolve and depends JSON CLI regressions.

## Validation

- `cargo test binary_resolve_prints_json_for_heading`
- `cargo test binary_depends_prints_json_references`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue migrating command argument groups from `main.rs` into their command
modules in small, independently validated waves.

