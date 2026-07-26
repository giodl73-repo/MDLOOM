# Status Global Config Adapter Closeout

## Mission

Bring `mdloom status` into the config-aware command adapter pattern so explicit
global config overrides are reflected in status summaries.

## Changes

- Added `cmd_status::run_with_globals`.
- Routed `Command::Status` through global options in dispatch.
- Preserved default status behavior when no explicit config is supplied.
- Added a regression that `mdloom --config <path> status <dir>` reports the
  explicit config's section schema count.

## Validation

- `cargo test binary_status_command_reports_project_summary`
- `cargo test binary_status_command_honors_explicit_config`
- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test cli_mdloom_version_exits_zero`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Any command that reads runtime config should route through `GlobalOptions` so
`--config` remains authoritative across the CLI surface.
