---
wave: publication-ast
pulse: 02
date: 2026-05-16
status: done
depends_on: ["publication-ast/pulse-01"]
governing_roles: ["COMPOSE", "SCHEMA", "BENCH"]
---

# Pulse 02: Markdown AST extraction

## Mission

Convert resolved Markdown into the shared publication AST with enough structure
to support existing publish backend claims.

## Scope inventory

- Source artifacts:
  - `src/publication.rs` or `src/publication/markdown.rs`
  - `src/publish.rs`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - None.

## Pre-implementation scout

- Compare current HTML, JSON report, PDF, DOCX, and Pebble Markdown parsing.
- Identify block types currently supported by all or most backends.
- Decide how to preserve heading IDs and paths.

## Deliverables checklist

- [x] Add resolved-Markdown to `PublicationDocument` extraction.
- [x] Support headings, paragraphs, links, inline code, lists, code blocks, and
      tables.
- [x] Preserve document title, heading paths, stable IDs, and metadata hooks.
- [x] Add L0 extraction tests for nested lists, links, code, tables, and headings.
- [x] Add L1 test proving extraction happens after directive resolution.

## Validation gates

- `cargo fmt --check`
- `cargo test publication_markdown_extracts_common_blocks`
- `cargo test publication_ast_uses_resolved_compile_output`
- `cargo test`
- `git diff --check`

## Non-goals

- Do not migrate all backends yet.
- Do not implement images/media/citations beyond typed placeholders.

## Evidence

- Added `PublicationDocument::from_resolved_markdown` for resolved Markdown
  extraction into headings, paragraphs, nested lists, fenced code blocks,
  tables, and rich inline spans.
- Heading extraction preserves stable slug IDs and stores path metadata under
  `heading_path.<id>`.
- Added L0 coverage with `publication_markdown_extracts_common_blocks`.
- Added L1 coverage with `publication_ast_uses_resolved_compile_output`,
  proving AST extraction consumes compiled Markdown after `mdloom:toc`
  resolution rather than raw source directives.
- Validation:
  - `cargo fmt --check`
  - `cargo test publication_markdown_extracts_common_blocks`
  - `cargo test publication_ast_uses_resolved_compile_output`
  - `cargo test`
  - `cargo test --test integration_tests`
  - `cargo build`
  - `cargo clippy -- -D warnings`
  - `git diff --check`
