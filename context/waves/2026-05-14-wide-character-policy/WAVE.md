---
wave: wide-character-policy
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# Wide Character Policy

## Mission

Make the implemented `ascii_char` behavior match its documented config
contract: strict by default, but suppress wide-character diagnostics when a
corpus explicitly sets `error_on_wide = false` for intentional wide content.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - Corpus character classification | DONE | MAXIM samples were mostly emoji status marks plus CJK/Korean/Japanese guide examples. |
| 02 - Config contract repair | DONE | `error_on_wide = false` now suppresses wide-character diagnostics instead of downgrading them to warnings. |
| 03 - Regression coverage | DONE | Added focused unit test for intentional wide content suppression. |
| 04 - Corpus and validation gate | DONE | MAXIM stays at `0` errors and `ascii_char_range` drops to `0`. |

## Gates

- Default `error_on_wide = true` behavior remains strict.
- `error_on_wide = false` suppresses wide content while `ascii_box` remains
  responsible for visual-width alignment.
- README/SPEC/schema docs agree with implementation.

## Closeout

See `CLOSE.md`.
