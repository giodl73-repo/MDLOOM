# CLI Global Options Closeout

## Mission

Continue reducing dispatch plumbing in `main.rs` by grouping root-level global
options into one context value.

## Changes

- Added a `GlobalOptions` dispatch context for root `--config`, `--format`,
  `--errors-only`, `--no-fail`, and `--output` values.
- Updated command dispatch to pass `globals.config` to commands that need the
  optional config override.
- Updated `run_check` to accept `&GlobalOptions` instead of a long parameter
  list.
- Kept command behavior and public CLI flags unchanged.

## Validation

- `cargo test cli_proof_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Future CLI cleanup can continue reducing command dispatch boilerplate where
command modules can own small adapter functions without hiding public CLI
behavior.

