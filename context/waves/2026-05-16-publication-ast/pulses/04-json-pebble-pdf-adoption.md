---
wave: publication-ast
pulse: 04
date: 2026-05-16
status: todo
depends_on: ["publication-ast/pulse-02"]
governing_roles: ["SCHEMA", "SIGNAL", "BENCH"]
---

# Pulse 04: JSON/Pebble/PDF adoption

## Mission

Use the publication AST as the shared source for JSON report section summaries,
Pebble chunk boundaries where practical, and PDF text output.

## Scope inventory

- Source artifacts:
  - `src/publish.rs`
  - `src/publication.rs`
  - `docs/specs/publication-ast.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - `*.proof-report.json`
  - `*.pebble.json`
  - `*.pdf`

## Pre-implementation scout

- Compare current JSON section extraction and Pebble section extraction.
- Identify what PDF text extraction loses today.
- Decide which JSON additions are backwards-compatible.

## Deliverables checklist

- [ ] Derive JSON report section summaries from AST headings.
- [ ] Add AST/theme summary fields without breaking existing report consumers.
- [ ] Keep Pebble compact and compatible; only adopt AST where it improves
      section fidelity.
- [ ] Render PDF text lines from AST blocks rather than HTML text stripping.
- [ ] Add regression tests for all three targets.

## Validation gates

- `cargo fmt --check`
- `cargo test json_report_backend_serializes_compile_bundle`
- `cargo test pdf_backend_writes_valid_pdf_bytes_from_html`
- `cargo test binary_compile_target_json_report_writes_bundle`
- `cargo test binary_compile_target_pebble_writes_ai_context_pack`
- `cargo test binary_compile_target_pdf_writes_pdf`
- `cargo test --test integration_tests`
- `git diff --check`

## Non-goals

- Do not make Pebble verbose.
- Do not implement print-engine PDF layout in this pulse.

## Evidence

- Pending.
