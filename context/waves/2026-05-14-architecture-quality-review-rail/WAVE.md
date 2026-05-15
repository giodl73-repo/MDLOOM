---
wave: architecture-quality-review-rail
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
source_request: "quality and arch review; fix findings; adopt waves and pulses; backfill history from route, apportionment, and maxim"
---

# Architecture and Quality Review Rail

## Mission

Turn the architecture/quality review into durable proof execution practice:
backfill wave/pulse history, repair stale public contracts, add missing tests,
clean local warnings, and leave proof with an active planning rail.

## Claim Boundary

This wave may edit proof skills, wave context, docs, tests, and local warning
cleanup. It may not rewrite release history wholesale, mutate sibling projects,
or change MAXIM corpus content except through a later named pulse.

## Inputs

| Input | Source |
|---|---|
| Spec review findings | `design/SPEC.md`, `src/config.rs`, review-spec skill |
| Pitfall review findings | `design/pitfalls/*.md`, review-pitfalls skill |
| ASCII review findings | `src/checks/ascii_box.rs`, `src/checks/ascii_flow.rs`, review-ascii skill |
| Wave model | `C:\src\route\waves\`, `C:\src\maxim\context\waves\`, `C:\src\apportionment\.claude\waves.json` |
| Release history | `CHANGELOG.md` |

## Pulse Status

| Pulse | Status | Evidence |
|---|---|---|
| 01 - Wave/pulse system backfill | DONE | proof-native skills, `.claude/waves.json`, `context/waves/PHASES.md` |
| 02 - Docs and spec contract repair | DONE | `README.md`, `design/SPEC.md`, pitfall docs |
| 03 - Coverage and warning cleanup | DONE | Rust tests for review gaps; local warnings removed |
| 04 - Validation and closeout | DONE | cargo tests/build, proof smoke checks, `CLOSE.md` |

## Validation Gates

```powershell
cargo test
cargo test --test integration_tests
cargo build
git diff --check
```

## Done Criteria

- proof has project-native wave, pulse, and plan skills.
- Backfilled waves connect current execution to release history without
  rewriting `CHANGELOG.md`.
- README and SPEC match implemented CLI/config/output behavior.
- Pitfall docs cite existing tests or explicitly name missing tests.
- Missing coverage from the review is added where implementation already exists.
- Local proof warnings are cleaned or intentionally carried forward.

## Non-Goals

- Do not bulk-edit the MAXIM corpus during this wave.
- Do not change mdpath unless a separate wave/pulse owns sibling-repo cleanup.
- Do not make breaking config semantics changes without a dedicated schema
  compatibility pulse.

## Closeout

See `CLOSE.md`.
