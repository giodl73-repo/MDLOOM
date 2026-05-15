# CLI Dispatch Module Closeout

## Mission

Move command routing out of `main.rs` so the binary entry point only parses and
delegates.

## Changes

- Added `src/dispatch.rs` to own command routing.
- Moved explicit command dispatch, default check routing, and shared path default
  usage out of `main.rs`.
- Updated `main.rs` to declare modules, parse `Cli`, and call `dispatch::run`.
- Kept parser ownership in `cli.rs` and command behavior ownership in command
  modules.
- Kept public CLI behavior unchanged.

## Validation

- `cargo test cli_proof_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_draft_command_writes_plan`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`main.rs` is now parse-and-delegate only. Future CLI architecture work should
focus on dispatch readability and shared command adapter patterns in
`dispatch.rs`.

