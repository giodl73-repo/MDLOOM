---
wave: box-column-signal
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# Box Column Signal

## Mission

Reduce `ascii_box_col` noise caused by valid row separators and embedded inner
borders being reported as bottom-border column mismatches.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - Corpus sample classification | DONE | Samples were mostly row separators (`├──┬──┤`) and inner component boxes inside an outer box. |
| 02 - Bottom-border policy | DONE | Extra bottom junctions are allowed; only missing top columns can warn, and only when border edges match. |
| 03 - Regression coverage | DONE | Added row-separator and embedded-inner-border tests while preserving zero-row mismatch coverage. |
| 04 - Corpus and validation gate | DONE | MAXIM stays at `0` errors and `ascii_box_col` drops sharply. |

## Gates

- Valid row separators do not warn solely because they add internal junctions.
- Embedded inner borders do not emit bottom-border column diffs for the outer box.
- Mismatched zero-row boxes still report width errors.
- MAXIM remains error-free.

## Closeout

See `CLOSE.md`.
