# Command Args Visibility Closeout

## Mission

Tighten command module encapsulation after dispatch stopped destructuring most
command argument structs.

## Changes

- Narrowed remaining `pub` fields on `cmd_compile::Args` to `pub(crate)`.
- Narrowed remaining `pub` fields on `cmd_layout::Args` to `pub(crate)`.
- Narrowed `cmd_check::Args`, `cmd_check::Flags`, and `cmd_check::Options`
  fields so check internals are no more visible than dispatch requires.
- Kept clap parsing, command dispatch, and command behavior unchanged.

## Validation

- `cargo test cli_proof_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test cli_compile_output_dir_flag`
- `cargo test binary_layout_composes_file_sources`
- `cargo test binary_check_summary_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue preferring module-owned adapters and narrow visibility as command
modules stabilize.

