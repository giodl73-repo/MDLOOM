# Resolve Command Module Closeout

## Mission

Continue the command-module split by extracting the mdpath lookup command,
`mdloom resolve`.

## Changes

- Added `src/cmd_resolve.rs`.
- Moved mdpath URI parsing, element resolution, text rendering, and JSON
  rendering out of `main.rs`.
- Preserved existing root handling: `--root` when supplied, current directory
  otherwise.
- Added a CLI regression that resolves a heading URI in JSON format and verifies
  the resolved element metadata.

## Validation

- `cargo test binary_resolve_prints_json_for_heading`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`mdloom depends` is the next smallest remaining command extraction and shares the
same mdpath-facing command surface.
