---
wave: publish-backends
pulse: 03
date: 2026-05-16
status: todo
depends_on: ["publish-backends/pulse-02"]
governing_roles: ["COMPOSE", "BOOK", "SIGNAL", "BENCH"]
---

# Pulse 03: Static site backend

## Mission

Add a static site backend that compiles a source tree into navigable HTML pages
plus a site manifest.

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
  - Site output directory.
  - `index.html`
  - Page HTML files.
  - `proof-site.json`
  - `.proof/artifacts.json` entries with `target = "site"`

## Pre-implementation scout

- Inspect directory compile/output-dir behavior.
- Decide whether `site` is a compile target or a separate `proof site` wrapper
  over compile.
- Identify minimal navigation from heading/title/path metadata.

## Deliverables checklist

- [ ] Add command surface for site generation.
- [ ] Compile each source through resolved Markdown then HTML.
- [ ] Generate index/navigation from source titles and output paths.
- [ ] Emit a site manifest with pages, source paths, diagnostics, and generation
      metadata.
- [ ] Add integration tests for multi-page output and manifest.
- [ ] Update README/SPEC/spec docs.

## Validation gates

- `cargo fmt --check`
- `cargo test binary_compile_target_site_writes_static_site`
- `cargo test --test integration_tests`
- `proof compile <fixture-dir> --target site --output-dir <site-dir>`
- `git diff --check`

## Non-goals

- Do not deploy or host the site.
- Do not add search ranking or CROP graph cuts.
- Do not claim browser pixel equivalence.
- Do not make watch mode target-aware in this pulse.

## Evidence

- Pending.
