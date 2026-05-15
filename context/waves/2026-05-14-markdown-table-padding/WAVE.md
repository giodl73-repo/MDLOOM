---
wave: markdown-table-padding
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# Markdown Table Padding

## Mission

Remove `md_table_cell_padding` noise where table rows are intentionally ignored
for extra columns, or cells cannot add padding without widening the declared
column.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - Corpus sample classification | DONE | Samples were over-split math rows like `|G|` and compact cells with no spare width. |
| 02 - Padding scope repair | DONE | Padding now skips body rows whose column count does not match the header. |
| 03 - No-room cell policy | DONE | Markdown table cells now mirror ASCII cell behavior: warn only when padding can fit. |
| 04 - Corpus and validation gate | DONE | MAXIM stays at `0` errors and `md_table_cell_padding` drops to `0`. |

## Gates

- Ignored extra-column body rows do not emit padding warnings.
- Full cells do not warn solely because no padding can fit.
- Cells with spare width still warn.
- MAXIM remains error-free.

## Closeout

See `CLOSE.md`.
