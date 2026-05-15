---
wave: architecture-quality-review-rail
pulse: 02
date: 2026-05-14
status: done
depends_on: [01]
governing_roles: [schema, signal]
---

# Pulse 02 - Docs and Spec Contract Repair

## Mission

Make public docs and the design spec match the implementation discovered by the
review.

## Scope Inventory

| Area | Files |
|---|---|
| Public docs | `README.md` |
| Contract spec | `design/SPEC.md` |
| Pitfalls | `design/pitfalls/*.md` |

## Deliverables

- [x] Fix README fix-pipeline commands to use `draft`/`--plan`.
- [x] Update SPEC output naming from `context` to implemented `rich`.
- [x] Document implemented check families and diagnostic codes.
- [x] Update invariant test statuses and remaining backlog.
- [x] Make pitfall docs traceability honest for missing tests.

## Validation Gates

```powershell
cargo test --test integration_tests
git diff --check
```

## Non-Goals

- Do not make compatibility-breaking schema changes in this pulse.
- Do not claim warning-free build until pulse 03 validates it.
