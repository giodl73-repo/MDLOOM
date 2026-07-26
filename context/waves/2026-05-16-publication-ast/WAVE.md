---
wave: publication-ast
date_open: 2026-05-16
status: active
source_request: "Get the right publication AST and invest in themes, fonts, colors, and better backend presentation quality."
---

# Wave: Publication AST and themes

## Mission

Introduce a shared publication AST and theme token system so MDLOOM's publish
backends move from correct first-scope artifacts toward professionally styled,
consistent, target-native outputs.

## Claim boundary

MDLOOM owns source resolution, publication semantics, theme tokens, and
target-native mappings. MDLOOM does not yet own full desktop-publishing fidelity,
browser-equivalent PDF layout, corporate template import, rich PPTX animation
systems, chart/media embedding, or LaTeX.

## Inputs

- Supported publish targets: `md`, `html`, `pebble`, `json-report`, `site`,
  `pdf`, `docx`, and `pptx`.
- Existing publish helpers in `src/publish.rs`.
- Existing slide parser/model in `src/slide/`.
- Existing artifact manifest `.mdloom/artifacts.json`.
- `docs/specs/publish-backends.md`.
- New spec: `docs/specs/publication-ast.md`.
- Roles: COMPOSE, BOOK, SCHEMA, STAGE, OFFICE, BENCH.

## Pulse status

| Pulse | Title | Status | Outcome |
|------:|-------|--------|---------|
| 01 | AST and theme contract | done | Added exported typed publication AST/theme module with built-in theme tokens and L0 contract tests. |
| 02 | Markdown AST extraction | done | Added resolved Markdown extraction into PublicationDocument blocks/inlines with heading path metadata and L0/L1 coverage. |
| 03 | HTML/site adoption | todo | Render HTML and static site pages from the AST with CSS theme tokens. |
| 04 | JSON/Pebble/PDF adoption | todo | Feed report summaries, Pebble chunks, and PDF text output from AST sections. |
| 05 | DOCX theme mapping | todo | Map AST/theme into DOCX styles, numbering, fonts, colors, and spacing. |
| 06 | PPTX theme mapping | todo | Map slide AST/theme into PPTX text, bullets, dimensions, fonts, colors, and notes. |
| 07 | Theme config surface | todo | Add built-in theme selection via CLI/config with manifest/report evidence. |
| 08 | Visual quality gates | todo | Add fixtures and role panels for backend presentation quality and regression coverage. |

## Validation gates

- `cargo fmt --check`
- Focused unit/integration tests named in each pulse.
- `cargo test --test integration_tests`
- `cargo test`
- `cargo build`
- `cargo clippy -- -D warnings`
- `git diff --check`

## Done criteria

- Publication AST and theme token structs are documented and tested.
- Markdown-family backends share AST extraction instead of each reinventing block
  parsing.
- DOCX and PPTX use theme tokens for native Office styles/theme mappings.
- HTML/site and PDF expose visible theme improvements.
- CLI/config theme selection is documented and recorded in manifest/report
  evidence.
- The visual quality role review identifies remaining non-goals honestly.

## Non-goals

- Do not import external Word/PowerPoint templates in this wave.
- Do not promise browser pixel parity or print-engine PDF equivalence.
- Do not implement animations, transitions, embedded video/audio, or advanced
  PPTX chart rendering.
- Do not add LaTeX.
- Do not remove or break existing backend command surfaces.

## Closeout/lessons

- Pending.
