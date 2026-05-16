---
wave: publication-ast
pulse: 05
date: 2026-05-16
status: todo
depends_on: ["publication-ast/pulse-02"]
governing_roles: ["OFFICE", "BOOK", "SCHEMA", "BENCH"]
---

# Pulse 05: DOCX theme mapping

## Mission

Render DOCX from the publication AST and map theme tokens into native Word styles,
numbering, fonts, colors, and spacing.

## Scope inventory

- Source artifacts:
  - `src/publish.rs`
  - `src/publication.rs`
  - `docs/specs/publication-ast.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - `*.docx`

## Pre-implementation scout

- Inspect current DOCX package parts and tests.
- Identify minimum Word style mappings for headings, normal text, code, lists,
  tables, and document metadata.
- Review OFFICE role constraints for native package validity.

## Deliverables checklist

- [ ] Generate DOCX document XML from AST blocks.
- [ ] Map built-in theme fonts/colors into `word/styles.xml`.
- [ ] Preserve native numbering and editable table/code text.
- [ ] Add package/XML tests for style and theme mappings.
- [ ] Preserve manifest behavior.

## Validation gates

- `cargo fmt --check`
- `cargo test docx_backend_writes_native_ooxml_package_parts`
- `cargo test binary_compile_target_docx_writes_docx`
- `cargo test --test integration_tests`
- `git diff --check`

## Non-goals

- Do not import `.dotx` templates.
- Do not implement tracked changes/comments.

## Evidence

- Pending.
