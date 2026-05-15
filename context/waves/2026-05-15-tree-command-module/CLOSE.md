# Tree Command Module Closeout

## Mission

Continue the command-module split by extracting the tree generation command
family, `proof tree`.

## Changes

- Added `src/cmd_tree.rs`.
- Moved the `TreeAction` clap subcommand enum out of `main.rs` with the tree
  command implementation.
- Moved dirtree generation, schema-driven tree generation, source resolution,
  output-file handling, and tree-specific helper imports out of `main.rs`.
- Added a CLI regression that runs `proof tree generate` against a temporary
  directory and verifies dirtree output.

## Validation

- `cargo test binary_tree_generate_prints_dirtree`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`proof spec-generate`, `proof compile`, and `proof layout` remain in `main.rs`;
`spec-generate` is the next medium-sized extraction candidate.
