# Backfill Review Role Closeout

## Mission

Review the role system against the expanded compiler/typesetter and reverse
backfill vision, then add missing review coverage.

## Changes

- Confirmed the existing roles already cover most of the expanded compiler and
  typesetter surface:
  - SOURCE for source/output document model and compile UX.
  - COMPOSE/PRESS/STAGE/PANEL for rendering, publishing, slides, and dashboards.
  - CACHE/PARSE/SCHEMA/BENCH/SIGNAL for compiler correctness, config semantics,
    tests, performance, and diagnostic quality.
- Added BACKFILL as the missing reverse-adoption specialist.
- Updated `.roles/ROLE.md` from twelve to thirteen roles.
- Added BACKFILL to tiebreaker ranking, role tensions, and usage guidance.
- Added `.roles/backfill.md` with lenses for round-trip fidelity, extraction
  confidence, provenance, cutover, and adoption speed.

## Validation

- `git --no-pager diff --check`

## Carry-forward

Use BACKFILL whenever reviewing `proof backfill`, markdown-to-source migration,
extraction confidence, round-trip comparison, or source-of-truth cutover plans.
