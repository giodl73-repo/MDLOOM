---
wave: publication-ast
pulse: 07
date: 2026-05-16
status: todo
depends_on: ["publication-ast/pulse-01"]
governing_roles: ["SCHEMA", "COMPOSE", "BOOK", "BENCH"]
---

# Pulse 07: Theme config surface

## Mission

Expose built-in theme selection through CLI/config and record theme selection in
publish artifacts.

## Scope inventory

- Source artifacts:
  - `src/cmd_compile.rs`
  - config schema/types
  - `src/publish.rs`
  - `README.md`
  - `design/SPEC.md`
  - `docs/specs/publication-ast.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - `.proof/artifacts.json`
  - `*.proof-report.json`
  - target outputs

## Pre-implementation scout

- Inspect compile arg/config patterns.
- Decide precedence: CLI `--theme` over `[publish].theme` over built-in default.
- Decide manifest/report field shape.

## Deliverables checklist

- [ ] Add `proof compile --theme <name>` for publish targets.
- [ ] Add `[publish].theme` config field for built-in themes.
- [ ] Validate unknown themes with a registered diagnostic/error.
- [ ] Record theme name in artifact manifest and JSON report.
- [ ] Update docs and tests.

## Validation gates

- `cargo fmt --check`
- `cargo test binary_compile_theme_flag_selects_builtin_theme`
- `cargo test binary_compile_unknown_theme_fails_loudly`
- `cargo test --test integration_tests`
- `git diff --check`

## Non-goals

- Do not parse arbitrary custom theme tables yet.

## Evidence

- Pending.
