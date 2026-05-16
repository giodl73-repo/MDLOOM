---
wave: publish-backends
pulse: 02
date: 2026-05-16
status: todo
depends_on: ["publish-backends/pulse-01"]
governing_roles: ["SCHEMA", "SOURCE", "SIGNAL", "BENCH"]
---

# Pulse 02: JSON report bundle

## Mission

Add a `json-report` publish target that emits a stable machine-readable bundle
for CI, agents, and integrations.

## Scope inventory

- Source artifacts:
  - `src/cmd_compile.rs`
  - `src/publish.rs`
  - `src/artifact.rs`
  - `README.md`
  - `design/SPEC.md`
  - `docs/specs/publish-backends.md`
  - `tests/integration_tests.rs`
- Generated/user artifacts:
  - `*.proof-report.json`
  - `.proof/artifacts.json` entries with `target = "json-report"`

## Pre-implementation scout

- Inspect compile result fields available after Markdown resolution.
- Compare with Pebble output to avoid duplicating its retrieval-focused schema.
- Define `proof.publish.json_report.v1` before writing code.

## Deliverables checklist

- [ ] Add `json-report` to compile target enum and output derivation.
- [ ] Emit resolved document metadata, sections, diagnostics, dependencies,
      artifact summary, and source frontmatter summary.
- [ ] Preserve normal compile diagnostics and manifest behavior.
- [ ] Add integration tests for output shape and manifest target.
- [ ] Update README/SPEC/spec docs.

## Validation gates

- `cargo fmt --check`
- `cargo test binary_compile_target_json_report_writes_bundle`
- `cargo test --test integration_tests`
- `proof compile <fixture>.source.md --target json-report -o <out>.proof-report.json`
- `git diff --check`

## Non-goals

- Do not replace Pebble.
- Do not add remote upload, CI annotations, or dashboards.
- Do not serialize unstable internal Rust structs directly.

## Evidence

- Pending.
