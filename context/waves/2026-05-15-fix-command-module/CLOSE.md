# Fix Command Module Closeout

## Mission

Continue the command-module split by extracting the AI-assisted fix application
command, `proof fix`.

## Changes

- Added `src/cmd_fix.rs`.
- Moved fix plan loading, DraftPlan-to-FixPlan conversion, confidence parsing,
  dry-run/apply reporting, signal-check option wiring, and post-apply
  verification out of `main.rs`.
- Preserved existing exit semantics for invalid confidence levels and remaining
  verification errors.
- Preserved the existing CLI regression that verifies dry-run mode writes
  nothing.

## Validation

- `cargo test binary_fix_dry_run_writes_nothing`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`proof resolve` and `proof depends` are now the smallest remaining command
extractions; they share mdpath URI/root behavior and can be split in either
order.
