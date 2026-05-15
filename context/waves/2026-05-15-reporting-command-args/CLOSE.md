# Reporting Command Args Closeout

## Mission

Continue modularizing the CLI shell by moving the reporting command argument
shapes for `proof config`, `proof status`, and `proof stats` into their command
modules.

## Changes

- Added `cmd_config::Args` with the clap argument definitions for `proof config`.
- Added `cmd_status::Args` with the clap argument definitions for `proof status`.
- Added `cmd_stats::Args` with the clap argument definitions for `proof stats`.
- Replaced the inline `Command::Config { ... }`, `Command::Status { ... }`, and
  `Command::Stats { ... }` fields in `main.rs` with tuple variants that
  reference their command-module argument types.
- Kept default stats path handling in `main.rs` through the shared
  `paths_or_cwd` helper.
- Kept config, status, and stats behavior unchanged.

## Validation

- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_config_honors_explicit_config_override`
- `cargo test binary_status_command_reports_project_summary`
- `cargo test binary_stats_command_runs`
- `cargo test binary_stats_file_count_honors_include_exclude`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Continue migrating the few remaining command argument groups from `main.rs` into
their command modules in small, independently validated waves.

