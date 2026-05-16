---
wave: publication-ast
pulse: 08
date: 2026-05-16
status: todo
depends_on: ["publication-ast/pulse-03", "publication-ast/pulse-05", "publication-ast/pulse-06", "publication-ast/pulse-07"]
governing_roles: ["STAGE", "BOOK", "OFFICE", "BENCH"]
---

# Pulse 08: Visual quality gates

## Mission

Add review fixtures and role evidence that make publication quality visible and
regression-testable across HTML/site, PDF, DOCX, and PPTX.

## Scope inventory

- Source artifacts:
  - `tests/fixtures/`
  - `tests/integration_tests.rs`
  - `context/waves/2026-05-16-publication-ast/panels/`
  - docs as needed
- Generated/user artifacts:
  - temporary backend outputs

## Pre-implementation scout

- Select one report-like fixture and one deck-like fixture.
- Define STAGE/BOOK/OFFICE criteria that can be tested mechanically and reviewed
  manually without Office/browser dependencies.
- Identify what remains qualitative carry-forward.

## Deliverables checklist

- [ ] Add representative report fixture covering headings, lists, tables, code,
      links, and notes.
- [ ] Add representative slide fixture covering title/content hierarchy, bullets,
      code, and speaker notes.
- [ ] Add mechanical tests for theme/style presence across outputs.
- [ ] Add role review panel with STAGE, BOOK, OFFICE, and BENCH findings.
- [ ] Update closeout carry-forwards.

## Validation gates

- `cargo fmt --check`
- `cargo test publication_visual_quality_fixture_outputs`
- `cargo test --test integration_tests`
- `cargo test`
- `cargo build`
- `cargo clippy -- -D warnings`
- `git diff --check`

## Non-goals

- Do not require visual snapshot tooling or external Office/browser/PDF apps.
- Do not claim pixel-perfect output.

## Evidence

- Pending.
