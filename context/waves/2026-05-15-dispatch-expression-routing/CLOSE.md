# Dispatch Expression Routing Closeout

## Mission

Improve dispatch readability now that command routing lives in `src/dispatch.rs`.

## Changes

- Refactored `dispatch::run` so the command routing is a single match
  expression.
- Removed repeated early `return` statements from each command branch.
- Moved the implicit default-check route into the `None` branch of the same
  expression.
- Kept explicit command behavior and default check behavior unchanged.

## Validation

- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test binary_draft_command_writes_plan`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Future dispatch work can focus on command grouping or reducing config override
plumbing, but the routing shell should remain behavior-light.

