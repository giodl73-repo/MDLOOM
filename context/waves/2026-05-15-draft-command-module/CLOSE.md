# Draft Command Module Closeout

## Mission

Continue the command-module split by extracting `proof draft`, which now shares
library lint orchestration with `check` and `stats`.

## Changes

- Added `src/cmd_draft.rs`.
- Moved the `draft` command implementation out of `main.rs`.
- Kept CLI dispatch, output text, and draft-plan JSON generation behavior
  unchanged.
- Added an E2E regression that verifies `proof draft -o <path> <input>` writes a
  DraftPlan-shaped JSON file.

## Validation

- `cargo test binary_draft_command_writes_plan`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`cmd_check` is the largest remaining lint-facing command extraction. It should
reuse `proof_lib::lint::lint_paths` and leave formatting/output helpers either
in main for now or in a later renderer module.
