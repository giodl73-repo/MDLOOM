---
name: proof-pulse
description: "Execute one proof wave pulse end to end with scout, edits, documentation, and validation."
tags: [proof, pulse, execute, quality, validation]
---

# proof-pulse

Execute a pulse from the active proof wave.

## Usage

```text
/proof-pulse next
/proof-pulse 02
```

## Procedure

1. Resolve the active wave from `context/waves/PHASES.md`.
2. If `next`, choose the first pulse with `status: todo`.
3. Read the pulse file completely.
4. Run every command in `Pre-implementation Scout`.
5. Implement deliverables using existing proof patterns.
6. Update docs and the pulse checklist.
7. Update `WAVE.md` pulse table.
8. Run validation from the pulse.
9. Run `git diff --check`.

## Default proof Validation

```powershell
cargo test
cargo test --test integration_tests
cargo build
git diff --check
```

If a validation command fails, record the exact failure and keep the pulse open
or blocked.

## Completion Report

Report pulse number and title, files changed, gates completed, validation
commands and result, and carry-forwards.

## Rules

- A "clean" claim must name the gate that made it clean.
- ASCII/SVG quality changes must be proof-checked or manually inspected before
  being called done.
- Public behavior changes must update `README.md`, `design/SPEC.md`, or both.
- History integration happens through wave closeout and `CHANGELOG.md`, not
  ad hoc edits inside unrelated pulses.
