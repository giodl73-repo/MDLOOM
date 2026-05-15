---
wave: compact-cell-padding
date_close: 2026-05-14
status: complete
---

# Compact Cell Padding - Closeout

## Delivered

| Area | Result |
|---|---|
| Padding policy | `ascii_cell_padding` only warns when a cell has room for the configured padding. |
| Width handling | The room check uses visual width so wide glyphs and tabs are measured consistently. |
| Regression | Added `cell_padding_allows_full_cells_with_no_room_for_padding`. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before wave | After |
|---|---:|---:|
| Errors | 0 | 0 |
| Warnings | 2953 | 1628 |
| `ascii_cell_padding` | 1894 | 569 |

## Carry-Forwards

1. Remaining MAXIM warning families are now small enough for manual sampling:
   `ascii_cell_padding`, markdown table padding, barchart warnings, and schema
   missing-section/pattern warnings.
2. Sibling `mdpath` warnings remain external to proof.
