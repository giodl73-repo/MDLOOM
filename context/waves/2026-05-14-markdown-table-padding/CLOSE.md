---
wave: markdown-table-padding
date_close: 2026-05-14
status: complete
---

# Markdown Table Padding - Closeout

## Delivered

| Area | Result |
|---|---|
| Extra-column rows | Rows ignored by `ignore_extra_body_cols` are no longer checked for padding. |
| Full cells | Cells whose trimmed content already fills the available width do not warn. |
| Regression | Added tests for over-split math rows, no-room cells, and spare-width warnings. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before wave | After |
|---|---:|---:|
| Errors | 0 | 0 |
| Warnings | 1628 | 1354 |
| `md_table_cell_padding` | 274 | 0 |

## Carry-Forwards

1. Remaining MAXIM warnings are now dominated by schema presence checks and
   intentionally strict ASCII barchart/arrow/padding families.
2. Sibling `mdpath` warnings remain external to proof.
