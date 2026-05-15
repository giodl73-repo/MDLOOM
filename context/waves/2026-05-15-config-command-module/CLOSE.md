# Config Command Module Closeout

## Mission

Continue the command-module split by extracting the small, stabilized
`proof config` command.

## Changes

- Added `src/cmd_config.rs`.
- Moved effective-config printing out of `main.rs`.
- Preserved automatic cascade behavior for `proof config [PATH]`.
- Preserved explicit `--config` behavior through
  `proof_lib::lint::load_config_for_path`.

## Validation

- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_config_honors_explicit_config_override`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`cmd_check` remains the main lint-facing extraction. Small command extractions
can continue first, but `check` is the highest-value split once output helpers
are ready to move.
