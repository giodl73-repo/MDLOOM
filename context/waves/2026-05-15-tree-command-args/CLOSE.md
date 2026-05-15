# Tree Command Args Closeout

## Mission

Finish the remaining inline command wrapper by moving the `proof tree`
subcommand argument shape into the tree command module.

## Changes

- Added `cmd_tree::Args` with the clap subcommand wrapper for `proof tree`.
- Replaced the inline `Command::Tree { ... }` fields in `main.rs` with a tuple
  variant that references `cmd_tree::Args`.
- Kept the existing `TreeAction` subcommand enum and tree behavior in
  `cmd_tree`.
- Preserved the tree CLI regression that generates dirtree output.

## Validation

- `cargo test binary_tree_generate_prints_dirtree`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

All command-specific clap argument shapes now live in command modules; future
CLI cleanup can focus on dispatch ergonomics and default check routing.

