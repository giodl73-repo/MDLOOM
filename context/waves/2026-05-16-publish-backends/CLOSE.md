# Publish Backends Closeout

Closed: 2026-05-16

## Outcome

PROOF now supports a scoped publish backend family behind `proof compile`:

- `json-report` for machine-readable compile/report bundles.
- `site` for local static HTML sites with page navigation and site manifests.
- `pdf` for deterministic portable PDF artifacts.
- `docx` for native editable Word-processing OOXML documents.
- `pptx` for native editable PowerPoint OOXML decks from explicit
  `.slides.source.md` inputs.

The baseline targets `md`, `html`, and `pebble` remain intact. LaTeX stayed
deferred.

## Shipped pulses

| Pulse | Result |
|---:|---|
| 01 | Captured backend contracts, sequencing, invariants, and non-goals. |
| 02 | Added `proof.publish.json_report.v1` with resolved Markdown, sections, refs, diagnostics, metadata, and compile counts. |
| 03 | Added static site generation: pages, `index.html`, `proof-site.json`, and manifest entries. |
| 04 | Added deterministic dependency-free PDF generation from resolved HTML output. |
| 05 | Added native DOCX generation with OOXML package/content tests. |
| 05a | Hardened L0/L1 coverage for DOCX, static site, and resolved-output publish contracts. |
| 06 | Added native PPTX generation with explicit slide-source boundaries, native bullets/numbering, code text, notes, OOXML package tests, and manifest integration. |

## Validation evidence

Each implementation pulse ran its focused backend tests plus the repository gates
named in the pulse. The final PPTX pulse also ran:

- `cargo fmt --check`
- `cargo test binary_compile_target_pptx_writes_deck`
- `cargo test pptx_ooxml_package_contains_native_bullets_and_notes`
- `cargo test --test integration_tests`
- PPTX CLI smoke compile
- `cargo test`
- `cargo build`
- `git diff --check`
- `cargo clippy -- -D warnings`

## Lessons

- Publish backends should stay target-scoped and honest: supported means command
  surface, manifest entries, docs, tests, and explicit non-goals.
- OFFICE was necessary for DOCX/PPTX because STAGE can judge presentation
  usefulness but not OOXML package correctness.
- PPTX must remain slide-source-driven. Rejecting arbitrary prose keeps deck
  generation intentional and prevents accidental shallow conversions.
- JSON report and Pebble serve different jobs: JSON report is verbose for CI and
  integrations; Pebble stays compact for retrieval/context transfer.

## Carry-forwards

- Richer PPTX layout, themes, charts, images/media, animations, transitions, and
  brand templates remain future work.
- DOCX tracked changes, comments, corporate templates, complex sections, and full
  style customization remain future work.
- Browser/print-engine-equivalent PDF rendering remains out of scope for the
  first PDF backend.
- LaTeX remains deferred until a separate typesetting contract is planned.
