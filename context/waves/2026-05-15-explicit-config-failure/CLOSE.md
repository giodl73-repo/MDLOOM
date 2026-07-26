# Explicit Config Failure Close

## Outcome

Made explicit `--config` authoritative across CLI paths that load mdloom config:

- `load_config` now returns `Result<MdloomConfig>`.
- Explicit config load errors are propagated with context instead of warning and
  falling back to discovered/default config.
- Check, stats, draft, compile, watch, pin-list, and spec-generate config loads
  now propagate explicit config failures.

## Tests Added

- `binary_missing_config_override_fails_loudly`
- `binary_invalid_config_override_fails_loudly`

Existing `binary_stats_honors_config_override` continues to cover the successful
explicit-config path.

## Validation

- `cargo test binary_missing_config_override_fails_loudly`
- `cargo test binary_invalid_config_override_fails_loudly`
- `cargo test binary_stats_honors_config_override`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The deeper raw/effective config split remains the preferred architecture for
future cascade work. This wave only removes the unsafe explicit-config fallback.
