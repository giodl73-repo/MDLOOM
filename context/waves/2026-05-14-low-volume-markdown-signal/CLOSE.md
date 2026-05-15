# Low-volume Markdown Signal Close

## Outcome

Closed residual low-volume Markdown false positives:

- `link_broken_target`: 3 -> 0 by skipping inline math/function notation.
- `md_table_empty_header`: 8 -> 0 by allowing blank row-label corners.
- `md_table_separator_invalid`: 4 -> 0 by allowing compact row-label corner separators.

## Corpus Result

MAXIM (`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`):

- Before wave: 1053 warnings, 0 errors.
- After wave: 1036 warnings, 0 errors.

## Carry-forward

Remaining warnings are dominated by `ascii_cell_padding` strict style and
MAXIM schema/content policy (`md_missing_section`, `md_missing_pattern`).
The remaining small families (`ascii_arrow_gap`, `ascii_box_col`,
`ascii_unclosed_fence`, H1/hierarchy, missing table) appear content- or
schema-owned after sampling.

