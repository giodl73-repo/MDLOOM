# Action Dispatch Adapters Closeout

## Mission

Continue reducing `main.rs` command dispatch boilerplate by letting action
command modules consume their own argument structs.

## Changes

- Updated `cmd_fix::run` to accept `cmd_fix::Args` directly.
- Updated `cmd_pin::run` to accept `cmd_pin::Args` directly.
- Updated `cmd_tree::run` to accept `cmd_tree::Args` directly.
- Simplified `main.rs` dispatch for `Command::Fix`, `Command::Pin`, and
  `Command::Tree` so it no longer destructures their fields.
- Kept fix, pin, and tree behavior unchanged.

## Validation

- `cargo test binary_fix_dry_run_writes_nothing`
- `cargo test binary_pin_appends_davinci_entry`
- `cargo test binary_tree_generate_prints_dirtree`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Apply the same module-owned dispatch adapter pattern to path/config-heavy
commands where shared path defaulting remains clear.

