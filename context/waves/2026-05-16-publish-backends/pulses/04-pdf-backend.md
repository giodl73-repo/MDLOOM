---
wave: publish-backends
pulse: 04
date: 2026-05-16
status: done
depends_on: ["publish-backends/pulse-03"]
governing_roles: ["COMPOSE", "BOOK", "BENCH"]
---

# Pulse 04: PDF backend

## Mission

Add a PDF target that renders the existing resolved HTML publish output into a
portable human-readable artifact.

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
  - `*.pdf`
  - `.proof/artifacts.json` entries with `target = "pdf"`

## Pre-implementation scout

- Evaluate Rust-native and CLI-backed HTML-to-PDF options available in CI.
- Prefer deterministic, testable output over perfect layout.
- Define how missing renderer dependencies fail.

## Deliverables checklist

- [x] Add `pdf` target dispatch and output derivation.
- [x] Render from the same HTML path used by `--target html`.
- [x] Preserve diagnostics and manifest behavior.
- [x] Add integration tests that validate PDF creation and target manifest entry.
- [x] Update README/SPEC/spec docs with PDF scope and limits.

## Validation gates

- `cargo fmt --check`
- `cargo test binary_compile_target_pdf_writes_pdf`
- `cargo test --test integration_tests`
- `proof compile <fixture>.source.md --target pdf -o <out>.pdf`
- `git diff --check`

## Non-goals

- Do not create a second Markdown-to-PDF renderer.
- Do not claim exact browser or print-engine equivalence.
- Do not require paid/proprietary tools in normal validation.

## Evidence

- Added `proof compile --target pdf`.
- PDF output is generated from the same resolved HTML publish output used by
  `--target html`.
- Added deterministic dependency-free PDF bytes with metadata and basic text
  rendering for CI.
- Added integration coverage for PDF creation and manifest target.
- Validation passed:
  - `cargo fmt --check`
  - `cargo test binary_compile_target_pdf_writes_pdf`
  - `cargo test --test integration_tests`
  - `proof compile <fixture>.source.md --target pdf -o <out>.pdf`
  - `git diff --check`
