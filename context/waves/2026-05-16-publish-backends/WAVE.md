---
wave: publish-backends
date_open: 2026-05-16
status: active
source_request: "Make a spec and plan for PPTX, PDF, DOCX, static site, and JSON report bundle; hold off on LaTeX."
---

# Wave: Publish backends

## Mission

Extend PROOF's compile graph from Markdown, HTML, and Pebble into a planned
publish backend family: JSON report bundle, static site, PDF, DOCX, and PPTX.
Keep every backend behind the same source-resolution pipeline so target-specific
renderers never bypass directive compilation, diagnostics, manifests, or cache
policy.

## Claim boundary

PROOF owns source compilation, resolved Markdown, publish target dispatch,
artifact manifests, backend diagnostics, and target-specific output generation.
PROOF does not own hosting/deployment, browser pixel equivalence, office-suite
round-trip fidelity, corporate templates, slide animation systems, search
ranking, or LaTeX in this wave.

## Inputs

- Existing `proof compile --target md|html|pebble`.
- Existing artifact manifest `.proof/artifacts.json`.
- Existing publish helpers in `src/publish.rs`.
- Existing slide source concepts for future PPTX grounding.
- `docs/specs/publish-backends.md`.
- `.roles/office.md` for native DOCX/PPTX package review.

## Pulse status

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | Backend spec and sequence | done | Captured supported targets, planned target contracts, invariants, and pulse sequence. |
| 02 | JSON report bundle | done | Added `proof.publish.json_report.v1` as a machine-readable compile/report target for CI and agents. |
| 03 | Static site backend | done | Added multi-page HTML site generation with navigation index and `proof-site.json` manifest. |
| 04 | PDF backend | done | Rendered existing HTML publish output to deterministic PDF artifacts with manifest integration. |
| 05 | DOCX backend | todo | Emit editable Word-processing documents from resolved Markdown. |
| 06 | PPTX backend | todo | Emit native editable PowerPoint decks from explicit slide-oriented sources, with package/XML, bullet, notes, and STAGE fidelity gates. |

## Validation gates

- `cargo fmt --check`
- `cargo test`
- `cargo test --test integration_tests`
- Per-backend CLI smoke commands named in each pulse.
- `git diff --check`

## Done criteria

- Every planned backend has a pulse with scope, gates, non-goals, and output
  claims.
- `md`, `html`, and `pebble` remain the stable baseline targets.
- `json-report`, `site`, `pdf`, `docx`, and `pptx` each become supported only
  after command surface, integration tests, manifest entries, docs, and
  diagnostics are implemented.
- LaTeX remains explicitly deferred.

## Non-goals

- Do not implement LaTeX in this wave.
- Do not bypass source directive compilation for any backend.
- Do not promise pixel-identical browser/PDF output.
- Do not infer slide decks from arbitrary prose without an explicit slide
  boundary.
- Do not implement PPTX as screenshots, rasterized text, or HTML embedded in
  slides.
- Do not require Microsoft Office, PowerPoint, or Word to be installed for
  normal tests.

## Closeout/lessons

- Pending.
