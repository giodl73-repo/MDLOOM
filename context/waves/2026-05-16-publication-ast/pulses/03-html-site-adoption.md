---
wave: publication-ast
pulse: 03
date: 2026-05-16
status: todo
depends_on: ["publication-ast/pulse-02"]
governing_roles: ["COMPOSE", "BOOK", "STAGE", "BENCH"]
---

# Pulse 03: HTML/site adoption

## Mission

Move HTML and static-site rendering onto the publication AST and apply visible
theme tokens through CSS variables.

## Scope inventory

- Source artifacts:
  - `src/publish.rs`
  - `src/publication.rs`
  - `README.md`
  - `docs/guides/07-compile.md`
  - `src/guides/07-compile.source.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - HTML files
  - `proof-site.json`

## Pre-implementation scout

- Inspect current `markdown_to_html_document` and `write_static_site` behavior.
- Identify assertions that must remain unchanged.
- Define visible CSS theme variables for font, color, spacing, and code blocks.

## Deliverables checklist

- [ ] Render HTML body from `PublicationDocument`.
- [ ] Add CSS theme token mapping for built-in themes.
- [ ] Apply site navigation theme styling.
- [ ] Preserve escaping behavior and current HTML output guarantees.
- [ ] Add tests for themed HTML and site index output.

## Validation gates

- `cargo fmt --check`
- `cargo test html_backend_renders_common_markdown_blocks`
- `cargo test static_site_helper_sorts_pages_and_writes_index_manifest`
- `cargo test binary_compile_target_html_writes_html_document`
- `cargo test binary_compile_target_site_writes_static_site`
- `cargo test --test integration_tests`
- `git diff --check`

## Non-goals

- Do not add asset bundling, search, deployment, or JavaScript.

## Evidence

- Pending.
