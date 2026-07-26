# Backfill Classifiers Closeout

## Mission

Make the backfill report useful for adoption planning without changing
literal-first source generation.

## Changes

- Added advisory block inventory to `mdloom_lib::backfill`.
- Reports now include aggregate and per-file block counts for:
  - prose
  - fenced blocks
  - markdown tables
  - ASCII table candidates
  - chart-like blocks
  - diagram-like blocks
  - ambiguous blocks
- Reports now include evidence strings with source line hints for detected
  candidate blocks.
- Kept generated `.source.md` output literal-first; classifiers only affect the
  JSON report.
- Updated README and SPEC to document the classifier report surface.
- Added integration coverage for markdown tables, fenced ASCII tables,
  chart-like blocks, and diagram-like blocks.

## Validation

- `cargo test binary_backfill_literal_generates_source_and_report`
- `cargo test binary_backfill_report_classifies_candidate_blocks`
- `cargo fmt && cargo test && cargo build && git --no-pager diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Structured extraction should consume these report classifications only behind
explicit extraction flags and continue preserving literal fallback metadata until
round-trip gates pass.
