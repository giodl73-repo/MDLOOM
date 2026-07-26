---
wave: publish-backends
pulse: 05a
date: 2026-05-16
status: done
depends_on: ["publish-backends/pulse-05"]
governing_roles: ["BENCH", "SCHEMA", "OFFICE", "COMPOSE"]
---

# Pulse 05a: Backend coverage hardening

## Mission

Close the L0/L1 coverage gaps discovered after JSON report, static site, PDF,
and DOCX support landed, before starting PPTX.

## Scope inventory

- Source artifacts:
  - `src/publish.rs`
  - `tests/integration_tests.rs`
  - `context/waves/2026-05-16-publish-backends/WAVE.md`
- Generated/user artifacts:
  - Temporary test DOCX/static-site outputs only.

## Pre-implementation scout

- Inventory existing publish backend tests by L0/L1/L2.
- Identify helper-level gaps for DOCX package parts and static-site sorting/index
  behavior.
- Identify a compile-pipeline L1 test that proves publish helpers consume
  resolved Markdown instead of source directives.

## Deliverables checklist

- [x] Add L0 DOCX helper/package tests.
- [x] Add L0 static-site helper tests.
- [x] Add L1 resolved-compile-output publish helper test.
- [x] Preserve existing L2 CLI backend tests.
- [x] Update wave evidence.

## Validation gates

- `cargo fmt --check`
- `cargo test docx_backend_writes_native_ooxml_package_parts`
- `cargo test static_site_helper_sorts_pages_and_writes_index_manifest`
- `cargo test publish_backends_consume_resolved_compile_output`
- `cargo test --test integration_tests`
- `cargo test`
- `git diff --check`

## Non-goals

- Do not change backend behavior.
- Do not start PPTX implementation in this pulse.
- Do not require Microsoft Office, browsers, or PDF readers for validation.

## Evidence

- Added L0 DOCX helper coverage that opens the generated ZIP package and checks
  content types, relationships, core/app properties, `word/document.xml`, styles,
  numbering, headings, links, tables, code text, and bullet/decimal numbering.
- Added L0 static-site helper coverage that verifies page sorting, generated
  `mdloom-site.json`, generated `index.html`, and HTML escaping in navigation.
- Added L1 compile-pipeline coverage proving publish helpers consume resolved
  Markdown from `compile_file`, after a `mdloom:toc` directive has been expanded,
  across HTML, JSON report, PDF, and DOCX helpers.
- Existing L2 CLI backend tests remain in place for `html`, `mdport`,
  `json-report`, `site`, `pdf`, and `docx`.
- Validation:
  - `cargo fmt --check`
  - `cargo test docx_backend_writes_native_ooxml_package_parts`
  - `cargo test static_site_helper_sorts_pages_and_writes_index_manifest`
  - `cargo test publish_backends_consume_resolved_compile_output`
  - `cargo test --test integration_tests`
  - `cargo test`
  - `cargo build`
  - `git diff --check`
