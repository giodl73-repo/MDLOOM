# US-56 — Candlestick: weekly OHLC ticker

Each row is one period: `label: open, high, low, close`. Up-periods (close
≥ open) render with `O` body; down-periods with `█`. The wick `│` spans
[low, high].

<!-- mdloom:compiled from="mdloom:chart" -->
```
              ACME weekly
120 ┤                                   │
    ┤                  │                O
    ┤                  │       │        O
    ┤         │        │       O        O
    ┤ │       O        █       O        │
    ┤ O       O        █       O
    ┤ O       │                │
    ┤ O                        │
    ┤ │
 95 ┤ │
    └┬───────┬────────┬───────┬────────┬
```
<!-- /mdloom:compiled -->

Wk3 closed below its open — the down-body is visually distinct from the
surrounding up-weeks.
