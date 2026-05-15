---
wave: architecture-quality-review-rail
pulse: 03
date: 2026-05-14
status: done
depends_on: [02]
governing_roles: [bench, parse]
---

# Pulse 03 - Coverage and Warning Cleanup

## Mission

Backfill tests for the review gaps and remove local proof warnings that obscure
real regressions.

## Deliverables

- [x] Add AD-09 heading-format tests for `## Title ##` and `## Gotchas from C#`.
- [x] Add AD-10 table parser tests for escaped pipes, code spans, and SQL concat.
- [x] Add CLI test for `proof init` creating `proof.toml`.
- [x] Keep stale `old_string`, dry-run, and reverse-line-order fix tests covered.
- [x] Remove local proof crate/test warnings where safe.

## Validation Gates

```powershell
cargo test
cargo test --test integration_tests
cargo build
```

## Non-Goals

- Do not edit sibling `mdpath` warnings in this pulse.
- Do not change table parsing behavior beyond locking existing intended behavior.
