# Loaded Config Explicitness Closeout

## Mission

Separate parser-only TOML explicitness from mdloom's effective runtime config
without changing the public config schema or cascade behavior.

## Changes

- Removed `include_set` from `FilesConfig` and `enabled_set` from
  `MarkdownConfig`, so `MdloomConfig` is again a clean effective config shape.
- Added an internal `LoadedConfig` layer that carries `ConfigExplicitness` while
  resolving cascades.
- Updated cascade merge to use TOML-loaded explicitness for ambiguous defaults:
  `files.include = ["**/*.md"]` can still intentionally replace a parent include,
  and `markdown.enabled = false` can still intentionally disable inherited
  markdown checks.
- Kept the public `merge(parent, child)` helper for effective configs, with
  explicitness inferred from non-default child values.

## Tests

- Added regression coverage for explicit default `files.include` replacement
  through real TOML cascade resolution.
- Added regression coverage for explicit `markdown.enabled = false` overriding
  an enabled parent.
- Re-ran the existing config cascade regressions.

## Validation

- `cargo fmt`
- `cargo test config_`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

The next deeper config cleanup is a full raw/effective model where every TOML
section is represented as optional raw fields before resolving to the complete
runtime `MdloomConfig`.
