---
wave: publication-ast
pulse: 06
date: 2026-05-16
status: todo
depends_on: ["publication-ast/pulse-01"]
governing_roles: ["OFFICE", "STAGE", "SCHEMA", "BENCH"]
---

# Pulse 06: PPTX theme mapping

## Mission

Map slide publication/theme tokens into native PPTX dimensions, fonts, colors,
bullet indentation, title/body hierarchy, code text, and notes.

## Scope inventory

- Source artifacts:
  - `src/publish.rs`
  - `src/publication.rs`
  - `src/slide/`
  - `docs/specs/publication-ast.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - `*.pptx`

## Pre-implementation scout

- Inspect current PPTX helper and package tests.
- Identify STAGE defaults for title/body font sizes and bullet density.
- Identify OFFICE theme/package parts that should carry font/color tokens.

## Deliverables checklist

- [ ] Map theme tokens into PPTX theme XML and slide text run properties.
- [ ] Use slide theme tokens for aspect ratio, title size, body size, and bullet
      indentation.
- [ ] Keep notes native and editable.
- [ ] Add tests for theme XML, slide font/color tokens, and bullet levels.
- [ ] Preserve `.slides.source.md` boundary guard.

## Validation gates

- `cargo fmt --check`
- `cargo test pptx_ooxml_package_contains_native_bullets_and_notes`
- `cargo test binary_compile_target_pptx_writes_deck`
- `cargo test --test integration_tests`
- `git diff --check`

## Non-goals

- Do not add animations, transitions, charts, embedded media, or brand templates.

## Evidence

- Pending.
