---
wave: publication-ast
pulse: 01
date: 2026-05-16
status: done
depends_on: []
governing_roles: ["COMPOSE", "SCHEMA", "BOOK", "BENCH"]
---

# Pulse 01: AST and theme contract

## Mission

Add the typed Rust publication AST and theme token contract without changing
backend behavior yet.

## Scope inventory

- Source artifacts:
  - `src/publication.rs` or `src/publication/mod.rs`
  - `src/lib.rs`
  - `docs/specs/publication-ast.md`
  - `tests/integration_tests.rs` if contract tests need crate-level access
- Generated/user artifacts:
  - None.

## Pre-implementation scout

- Inspect current publish helper block parsing in `src/publish.rs`.
- Inspect exported modules in `src/lib.rs`.
- Identify existing theme/config structs before introducing publish theme types.

## Deliverables checklist

- [x] Add typed `PublicationDocument`, `PublicationBlock`, `PublicationInline`,
      and `PublicationTheme` structures.
- [x] Add built-in theme definitions for `plain`, `professional`, and `dense`.
- [x] Add schema/version constants and stable theme names.
- [x] Add L0 tests for defaults, theme lookup, serialization, and block
      construction.
- [x] Export the module from `proof_lib`.
- [x] Keep backend behavior unchanged in this pulse.

## Validation gates

- `cargo fmt --check`
- `cargo test publication_theme_lookup_returns_builtin_tokens`
- `cargo test publication_ast_serializes_schema_and_blocks`
- `cargo test`
- `cargo build`
- `git diff --check`

## Non-goals

- Do not migrate backends in this pulse.
- Do not add CLI/config theme selection yet.
- Do not add custom user-defined theme parsing yet.

## Evidence

- Added `src/publication.rs` with `proof.publication_ast.v1`, typed document,
  block, inline, note, list item, and theme structures.
- Added built-in `plain`, `professional`, and `dense` theme token definitions
  covering fonts, colors, spacing, typography, and slide defaults.
- Exported `proof_lib::publication` without migrating any backend behavior yet.
- Validation:
  - `cargo fmt --check`
  - `cargo test publication_theme_lookup_returns_builtin_tokens`
  - `cargo test publication_ast_serializes_schema_and_blocks`
  - `cargo test`
  - `cargo build`
  - `git diff --check`
