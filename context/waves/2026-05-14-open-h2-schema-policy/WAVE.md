---
wave: open-h2-schema-policy
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# Open H2 Schema Policy

## Mission

Reduce corpus warning floods caused by broad schemas by making required H2
sections enforce presence without also turning the schema into a closed-world H2
allowlist.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - Required H2 open-world policy | DONE | `required_h2` and `required_h2_all` require headings but do not warn on other H2s. |
| 02 - Explicit optional H2 allowlist | DONE | `optional_h2` remains the explicit switch for `md_unexpected_section`. |
| 03 - Corpus signal check | DONE | MAXIM warning count dropped from 30,658 to 13,323. |

## Gates

- Unit tests cover required-only open-world behavior.
- Existing optional allowlist behavior remains tested.
- README and SPEC describe the policy.

## Closeout

See `CLOSE.md`.
