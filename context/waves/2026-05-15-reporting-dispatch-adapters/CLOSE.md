# Reporting Dispatch Adapters Closeout

## Mission

Continue reducing `main.rs` command dispatch boilerplate by letting reporting
command modules consume their own argument structs.

## Changes

- Updated `cmd_config::run` to accept `cmd_config::Args` directly.
- Updated `cmd_status::run` to accept `cmd_status::Args` directly.
- Simplified `main.rs` dispatch for `Command::Config` and `Command::Status` so
  it no longer destructures their fields.
- Kept effective-config and status summary behavior unchanged.

## Validation

- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_config_honors_explicit_config_override`
- `cargo test binary_status_command_reports_project_summary`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

Apply the same module-owned dispatch adapter pattern to other self-contained
commands where it reduces boilerplate without obscuring CLI behavior.

