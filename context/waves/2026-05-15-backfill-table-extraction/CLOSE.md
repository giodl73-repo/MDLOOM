# Backfill Table Extraction Closeout

## Mission

Begin structured reverse compilation without weakening the literal-first adoption
contract.

## Changes

- Added `mdloom backfill --extract-tables`.
- Added table extraction plumbing through `cmd_backfill` and
  `mdloom_lib::backfill`.
- Reused the existing markdown table parser so extraction skips fenced code
  blocks consistently with markdown table checks.
- Writes high-confidence markdown pipe tables to sibling sidecar files named
  `<stem>.tables.json`.
- Sidecar data includes schema version, original markdown path, table id, source
  line, heading context, headers, and trimmed row cells.
- Backfill reports now include `summary.tables_extracted` and per-file
  extraction entries with kind, sidecar path, confidence, line, row count, and
  column count.
- Kept generated `.source.md` output literal-first even when extraction is
  enabled.
- Updated README and SPEC for the implemented table extraction surface.

## Validation

- `cargo test binary_backfill_extract_tables_writes_sidecar_data`
- `cargo test binary_backfill_report_classifies_candidate_blocks`
- `cargo test binary_backfill_literal_generates_source_and_report`
- `cargo fmt && cargo test && cargo build && git --no-pager diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Next extraction work should handle ASCII tables and chart-like blocks behind
explicit flags with confidence thresholds, fallback provenance, and round-trip
review gates before changing generated markdown.
