# Generation Dispatch Adapters Closeout

## Mission

Continue reducing `main.rs` command dispatch boilerplate by letting generation
and summary command modules consume their own argument structs.

## Changes

- Updated `cmd_draft::run` to accept `cmd_draft::Args` directly alongside the
  dispatch-normalized paths.
- Updated `cmd_stats::run` to accept `cmd_stats::Args` directly alongside the
  dispatch-normalized paths.
- Updated `cmd_spec_generate::run` to accept `cmd_spec_generate::Args` directly.
- Simplified `main.rs` dispatch for `Command::Draft`, `Command::Stats`, and
  `Command::SpecGenerate`.
- Kept path defaulting for draft and stats in `main.rs`, where shared CLI path
  semantics are already centralized.
- Kept draft, stats, and spec-generate behavior unchanged.

## Validation

- `cargo test binary_draft_command_writes_plan`
- `cargo test binary_stats_command_runs`
- `cargo test cli_spec_generate_outputs_toml`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Apply the same module-owned dispatch adapter pattern to compile and layout once
their path/output routing can stay readable.

