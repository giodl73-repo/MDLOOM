# Low-volume Markdown Signal

## Mission

Close small remaining proof-owned Markdown false positives after the ASCII signal
waves.

## Scope

- Treat inline math/function notation shaped like `[X](t)`, `[A,A](X,Y)`, and
  `[m/n](x)` as non-link syntax for file-target verification.
- Allow comparison matrices to use a blank top-left row-label corner.
- Allow the row-label corner separator to be compact (`--`) while preserving
  short-separator warnings for named columns.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| Classify low-volume Markdown | done | Sampled links, heading hierarchy, H1 count, missing tables, and table warnings. |
| Math-link notation | done | Removed false `link_broken_target` warnings for inline notation. |
| Comparison table corners | done | Removed empty-header and short-corner-separator warnings for row-label matrix corners. |
| Validate corpus impact | done | MAXIM warning total dropped from 1053 to 1036 with zero errors. |

## Gates

- Link checker regressions pass.
- Markdown table regressions pass.
- MAXIM corpus stays at zero errors.

