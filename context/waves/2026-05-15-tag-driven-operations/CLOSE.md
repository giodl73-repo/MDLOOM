# Tag-Driven Operations Closeout

## Mission

Turn source frontmatter tags from passive metadata into explicit operation
selectors while preserving inclusive default behavior.

## Changes

- Added reusable `FrontmatterFilter` for exact-match source metadata filtering.
- Added opt-in filters to `proof check`:
  - `--tag <TAG>`
  - `--op <OP>`
  - `--content-tag <TAG>`
- Added the same filters to `proof compile`.
- Added the same filters to `proof stats`.
- Filters are additive: when multiple filters are supplied, a source must match
  all requested fields.
- Defaults remain behavior-safe: without filters, tags never exclude content.
- Compile manifests honor the filtered source set because filtering happens
  before artifact records are written.
- Added focused coverage for stats, check, compile, and the reusable filter
  matcher.
- Updated README, SPEC, session plan, and wave history.

## Validation

- `cargo test frontmatter_filter_requires_requested_fields`
- `cargo test binary_stats_tag_filter_limits_files`
- `cargo test binary_compile_tag_filter_limits_sources`
- `cargo test binary_check_tag_filter_limits_sources`
- `cargo fmt && cargo test && cargo build && git --no-pager diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Future config policy hooks may use these filters, but should stay opt-in until
selection semantics are stable across watch mode, manifests, and status views.
