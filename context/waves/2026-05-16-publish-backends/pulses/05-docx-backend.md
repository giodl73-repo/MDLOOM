---
wave: publish-backends
pulse: 05
date: 2026-05-16
status: done
depends_on: ["publish-backends/pulse-04"]
governing_roles: ["COMPOSE", "BOOK", "SCHEMA", "OFFICE", "BENCH"]
---

# Pulse 05: DOCX backend

## Mission

Add a DOCX target that turns resolved Markdown into an editable Word-processing
document with stable basic structure.

## Scope inventory

- Source artifacts:
  - `src/cmd_compile.rs`
  - `src/publish.rs`
  - `Cargo.toml`
  - `README.md`
  - `design/SPEC.md`
  - `docs/specs/publish-backends.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - `*.docx`
  - `.proof/artifacts.json` entries with `target = "docx"`

## Pre-implementation scout

- Evaluate Rust DOCX writing crates and whether generated archives are stable
  enough for tests.
- Define the first supported Markdown block set.
- Decide how to inspect DOCX output in tests without Microsoft Word.
- Use OFFICE to review package parts, content types, relationships, numbering,
  and editable document XML before calling DOCX supported.

## Deliverables checklist

- [x] Add `docx` target dispatch and output derivation.
- [x] Support title, headings, paragraphs, lists, tables, fenced code, links, and
      metadata.
- [x] Add archive/content tests that inspect document XML for expected structure.
- [x] Add OFFICE review evidence for OOXML package validity and editability.
- [x] Preserve diagnostics and manifest behavior.
- [x] Update README/SPEC/spec docs.

## Validation gates

- `cargo fmt --check`
- `cargo test binary_compile_target_docx_writes_docx`
- `cargo test --test integration_tests`
- `proof compile <fixture>.source.md --target docx -o <out>.docx`
- `git diff --check`

## Non-goals

- Do not require Word to be installed.
- Do not implement tracked changes, comments, corporate templates, complex page
  sections, or full style customization.
- Do not promise round-trip preservation after manual Word edits.

## Evidence

- Added `proof compile --target docx` with `.docx` output derivation and manifest
  target records.
- Generated native OOXML package parts: `[Content_Types].xml`, `_rels/.rels`,
  core/app properties, `word/document.xml`, relationships, styles, and numbering.
- OFFICE review: output is a ZIP/XML package with content types and
  relationships, editable WordprocessingML paragraphs/tables, style references,
  and native numbering IDs for bullets and ordered lists; CI inspects package
  parts without requiring Microsoft Word.
- Validation:
  - `cargo fmt --check`
  - `cargo test binary_compile_target_docx_writes_docx`
  - `cargo test --test integration_tests`
  - `cargo test`
  - `cargo build`
  - `proof compile <fixture>.source.md --target docx -o <out>.docx`
  - `git diff --check`
