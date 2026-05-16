# Publish backend role review

Date: 2026-05-16

## Roles

- SOURCE
- COMPOSE
- BOOK
- BENCH
- SCHEMA
- STAGE
- OFFICE

## Findings

| Role | Finding | Resolution |
|---|---|---|
| SOURCE | The plan correctly keeps every backend behind resolved Markdown/source compilation instead of letting targets parse directives independently. | Keep this as a backend invariant. |
| COMPOSE | Static site, PDF, DOCX, and PPTX need target-specific layout claims; visual equivalence must not be implied by compile success. | Specs now separate output-shape gates from pixel/layout fidelity. |
| BOOK | Static site is the main corpus-author backend; it needs navigation and site manifest claims before search/deploy work. | Site pulse keeps hosting/search out of scope. |
| BENCH | Backend support must mean tests for output shape and manifest target, not only command success. | Each pulse names integration and smoke gates. |
| SCHEMA | JSON report bundle needs an explicit stable schema before code. | Pulse 02 defines `proof.publish.json_report.v1` before implementation. |
| STAGE | PPTX must be reviewed as a real presentation, not just a generated file. | PPTX pulse includes density/hierarchy and STAGE fidelity gates. |
| OFFICE | Existing roles did not cover native DOCX/PPTX package correctness. | Added OFFICE role and wired it into DOCX/PPTX governance. |

## Carry-forwards

- JSON report bundle should avoid serializing unstable internal Rust structs.
- DOCX/PPTX tests must inspect OOXML ZIP/XML parts directly and must not require
  Microsoft Office in CI.
- PPTX is not supported until package, structure, and presentation gates pass.
