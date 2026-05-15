---
wave: box-column-signal
date_close: 2026-05-14
status: complete
---

# Box Column Signal - Closeout

## Delivered

| Area | Result |
|---|---|
| Row separators | Bottom borders may add internal junctions without warning. |
| Embedded borders | Bottom-column diff warnings are skipped when top/bottom border edges do not match. |
| Preservation | Width errors and missing expected top-column warnings remain intact. |
| Tests | Added coverage for row separators, embedded inner borders, and retained zero-row mismatch behavior. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before wave | After |
|---|---:|---:|
| Errors | 0 | 0 |
| Warnings | 6632 | 2953 |
| `ascii_box_col` | 3733 | 61 |

## Carry-Forwards

1. Remaining MAXIM signal is mostly `ascii_cell_padding`.
2. The remaining 61 `ascii_box_col` warnings should be sampled separately; they
   are no longer dominated by row-separator/embedded-border noise.
3. Sibling `mdpath` warnings remain external to proof.
