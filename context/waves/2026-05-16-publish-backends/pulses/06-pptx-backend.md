---
wave: publish-backends
pulse: 06
date: 2026-05-16
status: todo
depends_on: ["publish-backends/pulse-05"]
governing_roles: ["COMPOSE", "STAGE", "SCHEMA", "OFFICE", "BENCH"]
---

# Pulse 06: PPTX backend

## Mission

Add a native PPTX target for explicit slide-oriented PROOF sources without
guessing decks from arbitrary prose or rasterizing slide content.

## Scope inventory

- Source artifacts:
  - `src/cmd_compile.rs`
  - `src/publish.rs`
  - Existing slide source/compiler modules.
  - `Cargo.toml`
  - `README.md`
  - `design/SPEC.md`
  - `docs/specs/publish-backends.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - `*.pptx`
  - `.proof/artifacts.json` entries with `target = "pptx"`

## Pre-implementation scout

- Inspect existing `.slides.source.md` compile behavior and slide directive
  model.
- Evaluate Rust PPTX writing options and, if needed, direct OOXML package
  generation.
- Define the minimal native slide source contract for deck generation.
- Identify how to validate generated `ppt/slides/slide*.xml`,
  `ppt/notesSlides/notesSlide*.xml`, relationships, and content types without
  requiring PowerPoint.
- Use STAGE's density guidance: one clear message per slide, bounded bullets,
  readable hierarchy, and no overloaded defaults.

## Deliverables checklist

- [ ] Add `pptx` target dispatch and output derivation.
- [ ] Require explicit slide-oriented source boundaries.
- [ ] Emit title/content slides with native editable text placeholders or text
      boxes, not images.
- [ ] Emit native bullets and numbered lists with bounded nesting.
- [ ] Emit fenced code as monospace editable text runs.
- [ ] Emit speaker notes parts when source notes are available.
- [ ] Add OOXML package tests that inspect slide XML, notes XML, relationships,
      and `[Content_Types].xml`.
- [ ] Add OFFICE review evidence for native package validity and editability.
- [ ] Add STAGE-oriented fixture coverage for bullet density and title/body
      hierarchy.
- [ ] Preserve diagnostics and manifest behavior.
- [ ] Update README/SPEC/spec docs.

## Validation gates

- `cargo fmt --check`
- `cargo test binary_compile_target_pptx_writes_deck`
- `cargo test pptx_ooxml_package_contains_native_bullets_and_notes`
- `cargo test --test integration_tests`
- `proof compile <fixture>.slides.source.md --target pptx -o <out>.pptx`
- `git diff --check`

## Non-goals

- Do not infer slide decks from arbitrary prose.
- Do not render slides as screenshots, rasterized text, SVG-only images, or HTML
  embedded inside slide frames.
- Do not require PowerPoint to be installed.
- Do not implement animations, transitions, charts, embedded media, complex
  themes, brand templates, or advanced layout engines in the first pulse.
- Do not add LaTeX.

## Evidence

- Pending.
