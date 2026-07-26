# Compile Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the `mdloom compile` argument shape
into the compile command module.

## Changes

- Added `cmd_compile::Args` with the clap argument definitions for
  `mdloom compile`.
- Replaced the inline `Command::Compile { ... }` fields in `main.rs` with a
  tuple variant that references `cmd_compile::Args`.
- Kept default compile path normalization in `main.rs` while command behavior
  remains in `cmd_compile`.
- Preserved compile help and output-dir routing regressions.

## Validation

- `cargo test binary_help_documents_progress_only_for_compile`
- `cargo test cli_compile_output_dir_flag`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The remaining command variants can be converted to module-owned clap argument
structs in small batches, prioritizing larger variants first.
