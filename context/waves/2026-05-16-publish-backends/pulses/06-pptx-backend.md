---
wave: publish-backends
pulse: 06
date: 2026-05-16
status: done
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

- [x] Add `pptx` target dispatch and output derivation.
- [x] Require explicit slide-oriented source boundaries.
- [x] Emit title/content slides with native editable text placeholders or text
      boxes, not images.
- [x] Emit native bullets and numbered lists with bounded nesting.
- [x] Emit fenced code as monospace editable text runs.
- [x] Emit speaker notes parts when source notes are available.
- [x] Add OOXML package tests that inspect slide XML, notes XML, relationships,
      and `[Content_Types].xml`.
- [x] Add OFFICE review evidence for native package validity and editability.
- [x] Add STAGE-oriented fixture coverage for bullet density and title/body
      hierarchy.
- [x] Preserve diagnostics and manifest behavior.
- [x] Update README/SPEC/spec docs.

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

- Added `proof compile --target pptx` with `.pptx` output derivation and a
  `.slides.source.md` boundary guard so arbitrary prose is not treated as a deck.
- Generated native editable OOXML parts for presentation, slides, slide
  relationships, notes slides, notes relationships, masters, layouts, theme,
  core/app properties, and `[Content_Types].xml`.
- OFFICE review: output is a ZIP/XML package with expected presentation, slide,
  notes, layout, master, theme, relationship, and content-type parts; slide text,
  bullets, numbered lists, code, and notes are native XML text, not screenshots,
  SVG-only images, or embedded HTML frames.
- STAGE review: first-scope source contract stays explicit and defaults to
  title/content hierarchy with bounded bullet levels; richer layout, themes,
  charts, media, animations, and brand templates remain non-goals.
- Validation:
  - `cargo fmt --check`
  - `cargo test binary_compile_target_pptx_writes_deck`
  - `cargo test pptx_ooxml_package_contains_native_bullets_and_notes`
  - `cargo test --test integration_tests`
  - `proof compile <fixture>.slides.source.md --target pptx -o <out>.pptx`
  - `cargo test`
  - `cargo build`
  - `git diff --check`
