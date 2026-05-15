# Arrow and Barchart Signal Close

## Outcome

Closed proof-owned false positives in two remaining ASCII families:

- `ascii_arrow_gap` now ignores prose/source arrows such as `->` and decorative
  spaced axis rulers while still warning on isolated breaks in Unicode arrow
  bodies.
- `ascii_barchart` now targets plain-text diagram fences and skips boxed
  multi-panel drawings, adjacent texture/pattern runs, and typed programming
  fences.
- Barchart value-format detection now treats `ms`/`s`/`m` as durations only
  when attached to a numeric value.

## Corpus Result

MAXIM (`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`):

- Before wave: 1354 warnings, 0 errors.
- After wave: 1142 warnings, 0 errors.
- `ascii_arrow_gap`: 102 -> 7.
- `ascii_barchart_*`: 141 -> 24.

## Carry-forward

Remaining dominant warnings are schema/content policy (`md_missing_section`,
`md_missing_pattern`) and strict cell-padding style (`ascii_cell_padding`), not
obvious proof-side detector bugs from this wave.

