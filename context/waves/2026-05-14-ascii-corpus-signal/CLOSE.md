---
wave: ascii-corpus-signal
date_close: 2026-05-14
status: complete
---

# ASCII Corpus Signal - Closeout

## Delivered

| Area | Result |
|---|---|
| Cell padding | `ascii_flow` now derives cell delimiters from active border junction columns. Interior math/prose bars are ignored unless they line up with real box columns. |
| Regression | Added `cell_padding_ignores_math_pipes_inside_single_cell_box`. |
| Safety | `split_cells` slices only at `char_indices`-derived delimiter boundaries. Tabs use 4-column expansion in new delimiter-column logic. |
| Risk control | Rejected a broader nested-box pairing change because it changed MAXIM from warning-only to 244 errors. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before ASCII wave | After |
|---|---:|---:|
| Errors | 0 | 0 |
| Warnings | 13323 | 10903 |
| `ascii_cell_padding` | 4918 | 2518 |
| `ascii_box_col` | 3737 | 3733 |

## Review Notes

PARSE review found no retained panic-risk string slicing in the touched flow
path: delimiter splits use character-boundary byte offsets. PIXEL review
confirmed `is_border_line` still requires at least two junction characters and
does not fire on prose markdown table rows. Nested/side-by-side box pairing
remains the largest known ASCII signal problem, but needs a dedicated detector
design because the first attempted fix surfaced real corpus errors.

## Carry-Forwards

1. Design nested/side-by-side box detection so inner borders are not mistaken
   for outer borders without masking real malformed diagrams.
2. Classify `ascii_char_range` in MAXIM; it is now the largest remaining warning
   family alongside `ascii_box_col`.
3. Sibling `mdpath` warnings remain external to proof.
