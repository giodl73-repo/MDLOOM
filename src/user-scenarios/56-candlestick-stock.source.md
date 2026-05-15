# US-56 — Candlestick: weekly OHLC ticker

Each row is one period: `label: open, high, low, close`. Up-periods (close
≥ open) render with `O` body; down-periods with `█`. The wick `│` spans
[low, high].

```proof:chart kind=candlestick width=40 height=10 title="ACME weekly"
Wk1: 100, 108, 95, 105
Wk2: 105, 112, 102, 110
Wk3: 110, 118, 108, 107
Wk4: 107, 115, 100, 113
Wk5: 113, 120, 110, 118
```

Wk3 closed below its open — the down-body is visually distinct from the
surrounding up-weeks.
