---
wave: formula-padding-signal
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# Formula Padding Signal

## Mission

Reduce remaining `ascii_cell_padding` noise where mathematical notation uses
vertical bars for absolute values, cardinalities, or norms outside any bordered
ASCII box.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - Formula sample classification | DONE | MAXIM samples included `|G|`, `|Orb(x)|`, `|B(theta)|`, and beam-pattern formulas. |
| 02 - Active border requirement | DONE | Cell padding now requires a detected border-derived delimiter column set. |
| 03 - Regression coverage | DONE | Added formula and in-box math-pipe tests. |
| 04 - Corpus and validation gate | DONE | MAXIM stays at `0` errors while `ascii_cell_padding` drops again. |

## Gates

- Real bordered boxes still produce padding warnings.
- Absolute-value formulas outside bordered boxes are ignored.
- MAXIM remains error-free.

## Closeout

See `CLOSE.md`.
