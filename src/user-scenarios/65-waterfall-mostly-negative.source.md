# US-65 — Waterfall with mostly-negative deltas

Stress test: a waterfall where the deltas drag the running total well below
the starting baseline. The min y-axis bound auto-extends below zero.

```proof:chart kind=waterfall width=60 title="Cash burn"
Open balance: 1000
Engineering: -400
Sales: -250
Marketing: -300
Support: -100
Closing: 0
```

The Closing total = -50 — proof should render the closing bar below zero
without truncation.
