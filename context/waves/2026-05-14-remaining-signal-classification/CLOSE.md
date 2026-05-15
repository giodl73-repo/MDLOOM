# Remaining Signal Classification Close

## Outcome

No additional broad proof-side detector wave was found after the latest cleanup.
The remaining MAXIM warnings are now primarily content/schema/style carry-forward:

- `ascii_cell_padding` (567): mostly true compact style in bordered boxes where
  MAXIM has `check_cell_padding = true`.
- `md_missing_section` (346): root MAXIM schema requires `Decision Cheat Sheet`
  in files that do not yet have that section.
- `md_missing_pattern` (100): root MAXIM schema recommends at least one fenced
  code block/landscape diagram in files that do not yet have one.
- `ascii_unclosed_fence` (12): appears to be real unclosed code fence content debt.
- Remaining low-volume warnings (`ascii_arrow_gap`, `ascii_box_col`,
  `md_missing_table`, H1/hierarchy) look content/schema-owned after sampling.

## Corpus Result

MAXIM remains at:

- 1036 warnings.
- 0 errors.

## Carry-forward

Future reductions should be explicit MAXIM policy/content work rather than proof
detector suppression:

- decide whether all content guides should require `Decision Cheat Sheet`;
- decide whether biography/history families should require landscape diagrams;
- decide whether MAXIM wants strict ASCII cell padding or a looser corpus policy;
- fix real unclosed fences and remaining document-structure warnings in MAXIM.

