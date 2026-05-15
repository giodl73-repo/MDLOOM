# Init Command Module Closeout

## Mission

Continue the command-module split by extracting the self-contained
`proof init` command.

## Changes

- Added `src/cmd_init.rs`.
- Moved default `proof.toml` creation out of `main.rs`.
- Updated dispatch to call `cmd_init::run`.
- Reused the existing init E2E regression.

## Validation

- `cargo test binary_init_creates_default_proof_toml`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The small command modules are now mostly separated. Remaining high-value splits
include `cmd_check`, `cmd_fix`, and the compile command family.
