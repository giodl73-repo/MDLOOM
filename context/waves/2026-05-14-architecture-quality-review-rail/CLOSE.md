---
wave: architecture-quality-review-rail
date_close: 2026-05-14
status: complete
---

# Architecture and Quality Review Rail - Closeout

## Delivered

| Area | Result |
|---|---|
| Wave system | Added proof-native wave, pulse, and plan skills plus `.claude/waves.json`. |
| History backfill | Added `context/waves/PHASES.md` with backfilled waves from `CHANGELOG.md`. |
| Active rail | Opened and closed the architecture/quality review wave with four pulses. |
| Docs contract | Updated README fix-pipeline commands and refreshed SPEC check/output/backlog sections. |
| Pitfall traceability | Updated AD-09, AD-10, and SC-01 test references/gaps. |
| Coverage | Added tests for heading hash false positives, GFM table pipe parsing, and `proof init`. |
| Warning cleanup | Removed local proof crate/test warnings; sibling `mdpath` warnings remain external carry-forward. |

## Gates

```powershell
cargo test
cargo test --test integration_tests
cargo build
proof tests/fixtures/perfect_box.md
proof tests/fixtures/width_mismatch.md
proof --format json --no-fail tests/fixtures/width_mismatch.md
```

## Carry-Forwards

1. Schema compatibility: decide whether merge-sensitive booleans become
   `Option<bool>` so child configs can explicitly disable inherited checks.
2. Corpus signal: the MAXIM warning flood is mostly `md_unexpected_section`; run
   the next active wave against schema allowlists and directory section schemas.
3. Sibling warning cleanup: `mdpath` still emits six warnings during proof builds.
4. Fix pipeline line endings: add a CRLF-preservation test for `proof fix`.
