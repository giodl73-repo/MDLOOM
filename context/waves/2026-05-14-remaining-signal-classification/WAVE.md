# Remaining Signal Classification

## Mission

Classify the remaining MAXIM warning families after proof-side detector cleanup
and decide whether another safe implementation wave exists.

## Scope

- Sample dominant `ascii_cell_padding` warnings.
- Sample broad schema warnings (`md_missing_section`, `md_missing_pattern`).
- Sample low-volume residual diagnostics after Markdown/link cleanup.
- Record proof-owned versus MAXIM-owned carry-forward.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| Padding classification | done | Remaining `ascii_cell_padding` is mostly real compact bordered-box style; separator-row false positives were already fixed. |
| Schema classification | done | `md_missing_section` and `md_missing_pattern` follow MAXIM root policy requiring `Decision Cheat Sheet` and a landscape diagram/code block. |
| Low-volume classification | done | Residual arrow, box, unclosed fence, H1, hierarchy, and missing-table warnings look content/schema-owned after sampling. |
| Distribution review | done | Remaining warnings cluster by MAXIM subject areas, not by a single proof parsing failure. |

## Gates

- No MAXIM content mutations.
- No proof behavior changes without a confirmed detector false positive.

