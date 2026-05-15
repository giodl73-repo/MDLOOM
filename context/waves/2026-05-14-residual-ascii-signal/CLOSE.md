# Residual ASCII Signal Close

## Outcome

Closed the remaining proof-owned ASCII detector noise from the prior corpus
pass:

- `ascii_barchart_*` dropped to zero by excluding non-chart row shapes and
  allowing stacked bars with mixed default fill characters.
- `ascii_box_col` dropped to one apparent real table drift by excluding
  top-border connector anchors, bottom connector ports, spanning rows, and ASCII
  tree branches from bottom-border column comparisons.
- `ascii_arrow_gap` now stops scanning at multi-space layout gaps and ignores
  bidirectional scale rulers, leaving only apparent real broken arrow bodies.

## Corpus Result

MAXIM (`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`):

- Before wave: 1142 warnings, 0 errors.
- After wave: 1053 warnings, 0 errors.
- `ascii_barchart_*`: 24 -> 0.
- `ascii_box_col`: 61 -> 1.
- `ascii_arrow_gap`: 7 -> 2.

## Carry-forward

Remaining high-volume warning families are `ascii_cell_padding` strict style and
MAXIM schema/content policy (`md_missing_section`, `md_missing_pattern`).
`ascii_unclosed_fence` appears to be real corpus content debt.

