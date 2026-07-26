# Pin List Command Module Closeout

## Mission

Continue the command-module split by extracting the small DaVinci listing
command, `mdloom pin-list`.

## Changes

- Added `src/cmd_pin_list.rs`.
- Moved DaVinci-entry listing out of `main.rs`.
- Preserved explicit `--config` behavior through
  `mdloom_lib::lint::load_config_for_path`.
- Added a CLI regression that verifies registered DaVinci entries are rendered.

## Validation

- `cargo test binary_pin_list_prints_registered_davinci_entries`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The larger `mdloom pin` command still lives in `main.rs`; it is a natural
follow-up once URI/config mutation behavior is covered by focused tests.
