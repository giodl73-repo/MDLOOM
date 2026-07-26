# Layout Command Module Closeout

## Mission

Continue the command-module split by extracting the figure composition command,
`mdloom layout`.

## Changes

- Added `src/cmd_layout.rs`.
- Moved layout option parsing, file/mdpath source resolution, figure content
  extraction, composition, and output-file handling out of `main.rs`.
- Preserved existing behavior for empty source lists, alignment/direction
  parsing, `--root`, labels, wrapping, borders, and output routing.
- Added a CLI regression that composes two temporary file sources and verifies
  both appear in the layout output.

## Validation

- `cargo test binary_layout_composes_file_sources`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`mdloom compile` is the last large command implementation remaining in `main.rs`
and should be extracted with care because it includes normal and watch modes.
