# Reverse Backfill Spec Closeout

## Mission

Define how existing markdown systems can adopt mdloom quickly by generating
reviewable `.source.md` candidates from current `.md` artifacts.

## Changes

- Added `mdloom backfill` as the reverse compiler/adoption bridge in
  `design/SPEC.md`.
- Defined the backfill pipeline: inventory/classify, extract candidates,
  generate source, compile, compare, and report.
- Added conservative extraction classes for literal markdown, ASCII figures,
  ASCII tables, markdown tables, chart-like blocks, repeated patterns, and
  ambiguous blocks.
- Documented quick adoption with a safe upgrade path: mirror, inspect, improve,
  automate, adopt.
- Added planned CLI options including `--literal-first`, `--check-roundtrip`,
  and `--cutover-plan`.
- Added backlog items for the reverse/backfill command, classifiers, cutover
  plans, review skill, migration guide, and round-trip golden tests.

## Validation

- `git --no-pager diff --check`

## Carry-forward

Backfill should prioritize round-trip fidelity and reviewability before semantic
extraction. Teams should be able to use mdloom automation against existing
markdown before committing to generated artifacts as the source of truth.
