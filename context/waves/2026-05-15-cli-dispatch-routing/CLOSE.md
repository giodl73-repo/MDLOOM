# CLI Dispatch Routing Closeout

## Mission

Continue simplifying `main.rs` by separating explicit command dispatch from the
default check route.

## Changes

- Added a `run(Cli)` entry point so `main()` only parses and delegates.
- Destructured `Cli` once before dispatch, avoiding follow-up matching against
  the parsed command state.
- Routed explicit `mdloom check` and implicit default check through a shared
  `run_check` helper.
- Kept global options, top-level default check paths, and command behavior
  unchanged.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Future CLI cleanup can continue reducing dispatch boilerplate where command
modules can own small adapter functions without hiding public CLI behavior.

