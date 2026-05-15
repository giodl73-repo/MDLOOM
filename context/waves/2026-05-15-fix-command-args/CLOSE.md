# Fix Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the `proof fix` argument shape
into the fix command module.

## Changes

- Added `cmd_fix::Args` with the clap argument definitions for `proof fix`.
- Replaced the inline `Command::Fix { ... }` fields in `main.rs` with a tuple
  variant that references `cmd_fix::Args`.
- Kept fix plan loading, dry-run/apply behavior, confidence parsing, and
  verification behavior in `cmd_fix::run` unchanged.
- Preserved the fix CLI regression that verifies dry-run mode writes nothing.

## Validation

- `cargo test binary_fix_dry_run_writes_nothing`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Only the tree command wrapper still has inline clap structure in `main.rs`; move
that final wrapper into `cmd_tree` in a follow-up wave.

