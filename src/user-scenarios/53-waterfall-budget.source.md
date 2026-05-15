# US-53 — Waterfall: project budget reconciliation

Start with the planned budget, walk through approved deltas, land on the
final figure. `▓` marks Start and End, `█` is a positive delta, `▒` is
negative.

```proof:chart kind=waterfall width=60 title="Budget walk ($K)"
Plan: 500
Hardware: 80
Staffing: -120
Vendor: -45
Q3 reserve: 30
Final: 0
```

The bars float at the running-total y-coordinate so each delta visually
reads as "what changed from the prior level."
