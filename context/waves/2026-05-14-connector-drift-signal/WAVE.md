---
wave: connector-drift-signal
date_open: 2026-05-14
date_close: 2026-05-14
status: complete
---

# Connector Drift Signal

## Mission

Reduce `ascii_connector_drift` false positives where vertical bars in timelines,
formulas, and labeled drawings were compared as if they were flowchart connector
lines.

## Pulses

| Pulse | Status | Notes |
|---|---|---|
| 01 - Drift sample classification | DONE | MAXIM samples included oxygen timelines, `degree | n` formulas, and architectural section labels. |
| 02 - Connector-only policy | DONE | Drift checks now only compare lines that contain connectors and connector glyphs/whitespace/arrows only. |
| 03 - Positive/negative coverage | DONE | Added tests for timeline/formula suppression and true connector-only drift. |
| 04 - Corpus and validation gate | DONE | MAXIM stays at `0` errors and `ascii_connector_drift` drops to `0`. |

## Gates

- Timeline and formula pipes do not warn.
- Connector-only drift still warns.
- MAXIM remains error-free.

## Closeout

See `CLOSE.md`.
