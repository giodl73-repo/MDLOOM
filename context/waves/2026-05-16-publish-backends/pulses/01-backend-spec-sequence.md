---
wave: publish-backends
pulse: 01
date: 2026-05-16
status: done
depends_on: []
governing_roles: ["SOURCE", "COMPOSE", "BOOK", "BENCH"]
---

# Pulse 01: Backend spec and sequence

## Mission

Specify the publish backend family and sequence implementation so JSON report
bundle, static site, PDF, DOCX, and PPTX can be added without blurring the
existing compile graph.

## Scope inventory

- Source artifacts:
  - `docs/specs/publish-backends.md`
  - `context/waves/2026-05-16-publish-backends/WAVE.md`
  - `context/waves/PHASES.md`
  - `README.md`
  - `design/SPEC.md`
- Generated/user artifacts:
  - Publish backend wave plan.

## Pre-implementation scout

- Confirm current target support is `md`, `html`, and `mdport`.
- Confirm HTML and Mdport docs describe their scoped support.
- Confirm future targets should include JSON report bundle, static site, PDF,
  DOCX, and PPTX while deferring LaTeX.

## Deliverables checklist

- [x] Add publish backend spec.
- [x] Add active publish-backends wave.
- [x] Sequence target implementation pulses.
- [x] Update public README/SPEC pointers.

## Validation gates

- `git diff --check`

## Non-goals

- Do not implement a backend in this pulse.
- Do not change `mdloom compile` behavior in this pulse.
- Do not add LaTeX to the wave.

## Evidence

- `docs/specs/publish-backends.md` records current and planned target contracts.
- `context/waves/2026-05-16-publish-backends/WAVE.md` records the pulse
  sequence.
