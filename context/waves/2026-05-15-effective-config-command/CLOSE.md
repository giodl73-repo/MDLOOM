# Effective Config Command Close

## Outcome

Implemented the documented `proof config [PATH]` behavior:

- `proof config [PATH]` now prints the resolved effective config as TOML.
- Auto mode resolves PATH through normal config cascade.
- Explicit `--config` mode prints the supplied config with defaults and skips
  auto-cascade.
- Config structs now derive `Serialize` so effective config output is generated
  from the real data model rather than a hand-written summary.

## Tests Added

- `binary_config_prints_effective_cascaded_config`
- `binary_config_honors_explicit_config_override`

## Validation

- `cargo test binary_config_prints_effective_cascaded_config`
- `cargo test binary_config_honors_explicit_config_override`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

`proof config` now exposes the current effective model. A future raw/effective
config split should keep this command backed by the effective, post-resolution
view.
