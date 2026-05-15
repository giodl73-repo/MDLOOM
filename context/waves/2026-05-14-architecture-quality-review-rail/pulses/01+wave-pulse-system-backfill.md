---
wave: architecture-quality-review-rail
pulse: 01
date: 2026-05-14
status: done
depends_on: []
governing_roles: [schema, signal]
---

# Pulse 01 - Wave/Pulse System Backfill

## Mission

Install proof-native wave/pulse planning, using ROUTE's execution rail and
MAXIM's history bridge as the model.

## Scope Inventory

| Area | Files |
|---|---|
| Skills | `.claude/skills/proof-wave/`, `.claude/skills/proof-pulse/`, `.claude/skills/proof-plan/` |
| Wave config | `.claude/waves.json` |
| Wave rail | `context/waves/PHASES.md`, `context/waves/2026-05-14-architecture-quality-review-rail/WAVE.md` |

## Pre-implementation Scout

```powershell
Get-ChildItem C:\src\route\waves
Get-Content C:\src\maxim\context\waves\PHASES.md
```

## Deliverables

- [x] Add proof-native wave management skill.
- [x] Add proof-native pulse execution skill.
- [x] Add proof-native wave/pulse planning skill.
- [x] Add wave config and phase index.
- [x] Backfill release-history waves from `CHANGELOG.md`.
- [x] Open the active architecture/quality review wave.

## Validation Gates

```powershell
git diff --check
```

## Non-Goals

- Do not rewrite `CHANGELOG.md`.
- Do not copy domain-specific ROUTE or MAXIM gates.
- Do not create bulk content-editing scripts.
