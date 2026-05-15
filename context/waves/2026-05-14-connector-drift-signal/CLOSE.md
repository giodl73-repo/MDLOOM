---
wave: connector-drift-signal
date_close: 2026-05-14
status: complete
---

# Connector Drift Signal - Closeout

## Delivered

| Area | Result |
|---|---|
| Detector scope | `ascii_connector_drift` now only compares connector-only vertical lines. |
| False positives | Timelines, formulas, and labeled drawings with `|` are ignored by drift checking. |
| Regression | Added negative coverage for timeline/formula pipes and positive coverage for true connector-only drift. |
| Docs | SPEC and schema reference now describe connector-only drift semantics. |

## Corpus Impact

`proof stats --by-code --config C:\src\maxim\proof.toml C:\src\maxim\`:

| Metric | Before wave | After |
|---|---:|---:|
| Errors | 0 | 0 |
| Warnings | 7222 | 6632 |
| `ascii_connector_drift` | 590 | 0 |

## Carry-Forwards

1. Remaining MAXIM signal is dominated by `ascii_box_col` and
   `ascii_cell_padding`.
2. Nested/side-by-side box detection still needs a dedicated design wave.
3. Sibling `mdpath` warnings remain external to proof.
