# Spec Registry Boundary Close

## Outcome

Created a diagnostic-code registry boundary so emitted mdloom diagnostics have a
single discoverable contract:

- Added `src/diagnostic_registry.rs` with code, default severity, owner family,
  and description entries.
- Exported the registry through `mdloom_lib`.
- Added an invariant test that scans source string literals and fails when a
  diagnostic-like code is not registered.
- Updated `design/SPEC.md` to document the registry, larger command surface,
  compile/render diagnostic families, raw/effective config direction, and the
  new registry invariant.

## Design Decision

The registry is intentionally descriptive first. Existing checks still emit their
current string codes, while tests now prevent new undocumented codes from being
introduced. A later wave can migrate emitters to constants from the registry if
that becomes valuable.

## Validation

- `cargo test invariant_all_source_diagnostic_codes_are_registered`
- `cargo test`
- `cargo build`
- `git diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
build/test; this wave did not change sibling repository code.

## Carry-forward

- Split raw TOML config from resolved effective config.
- Add typed rich-context contracts for table, link, chart, compile, and markdown
  diagnostics.
- Extract large command and compile directive modules after the registry and spec
  contract stabilize.
