# Pin Command Module Closeout

## Mission

Continue the command-module split by extracting the DaVinci pinning command,
`proof pin`.

## Changes

- Added `src/cmd_pin.rs`.
- Moved DaVinci URI resolution and config mutation out of `main.rs`.
- Preserved the existing pin behavior: resolve the supplied `md://` URI, append
  a `[[davinci]]` entry, keep duplicate-id detection, and print follow-up
  guidance.
- Added a CLI regression that verifies `proof pin` appends the expected
  DaVinci entry to `proof.toml`.

## Validation

- `cargo test binary_pin_appends_davinci_entry`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Larger commands remain in `main.rs`; `proof check` is the next natural
extraction target because it owns more output/rendering decisions than the small
commands completed so far.
