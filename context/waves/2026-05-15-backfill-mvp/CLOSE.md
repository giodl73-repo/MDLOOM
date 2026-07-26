# Backfill MVP Closeout

## Mission

Give existing markdown projects a safe first-day adoption bridge into mdloom
source ownership.

## Changes

- Added `mdloom backfill`.
- Added `mdloom_lib::backfill` with report types and literal-first source
  generation.
- Mirrors existing `.md` files to `.source.md` candidates under
  `--output-source`.
- Adds source provenance frontmatter:
  - `tags: [backfill]`
  - `ops: [backfill]`
  - `content_tags: [markdown]`
  - `mdloom_original: "..."`
- Writes `backfill-report.json` with scan/generation/round-trip summary and
  per-file entries.
- Supports `--literal-first`, `--report`, `--output-source`, and
  `--check-roundtrip`.
- Round-trip mode compiles generated source and compares compiled output to the
  original markdown.
- Updated README and SPEC CLI reference for the implemented MVP surface.

## Validation

- `cargo test binary_backfill_literal_generates_source_and_report`
- `cargo test cli_mdloom_version_exits_zero`
- `cargo test binary_help_documents_progress_only_for_compile`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Next backfill work should add classifiers and confidence evidence without
changing the literal-first round-trip contract. Semantic extraction should remain
opt-in until report grouping and golden tests are in place.
