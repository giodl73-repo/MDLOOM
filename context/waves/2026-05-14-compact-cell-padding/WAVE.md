---
wave: compact-cell-padding
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# Compact Cell Padding

## Mission

Reduce remaining `ascii_cell_padding` warnings where table content exactly fills
the declared cell width and adding padding would require changing the box width.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - Remaining padding classification | DONE | Samples were compact cells like `Ultrasonic`, `High-freq`, matrix rows, and width-constrained labels. |
| 02 - No-room padding policy | DONE | Padding warnings are skipped when `trimmed_width + 2 * min_pad > cell_width`. |
| 03 - Regression coverage | DONE | Added coverage for full-width compact cells and retained warnings when spare width exists. |
| 04 - Corpus and validation gate | DONE | MAXIM stays at `0` errors and `ascii_cell_padding` drops sharply. |

## Gates

- Full-width compact cells do not warn.
- Cells with spare width still warn when padding is missing.
- MAXIM remains error-free.

## Closeout

See `CLOSE.md`.
