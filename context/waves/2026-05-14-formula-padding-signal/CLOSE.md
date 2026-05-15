---
wave: formula-padding-signal
date_close: 2026-05-14
status: complete
---

# Formula Padding Signal - Closeout

## Delivered

| Area | Result |
|---|---|
| Cell scope | `ascii_cell_padding` now runs only when a content row is inside an active bordered box. |
| Delimiters | Cell delimiters continue to come from border junction columns, not arbitrary `|` characters. |
| Regression | Added `cell_padding_ignores_absolute_value_formula_without_box_border`. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before wave | After |
|---|---:|---:|
| Errors | 0 | 0 |
| Warnings | 7847 | 7222 |
| `ascii_cell_padding` | 2518 | 1894 |

## Carry-Forwards

1. Remaining MAXIM signal is dominated by `ascii_box_col`.
2. `ascii_connector_drift` still needs classification; many samples look like
   alternating or decorative connector columns.
3. Sibling `mdpath` warnings remain external to proof.
