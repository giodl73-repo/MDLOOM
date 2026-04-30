# US-65 — Waterfall with mostly-negative deltas

Stress test: a waterfall where the deltas drag the running total well below
the starting baseline. The min y-axis bound auto-extends below zero.

<!-- proof:compiled from="proof:chart" -->
```
                         Cash burn
Open balance  │   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 1000
Engineering   │                         ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  600
Sales         │                ▒▒▒▒▒▒▒▒▒▒                 350
Marketing     │     ▒▒▒▒▒▒▒▒▒▒▒▒                           50
Support       │ ▒▒▒▒▒                                     -50
Closing       │ ▓                                         -50
```
<!-- /proof:compiled -->

The Closing total = -50 — proof should render the closing bar below zero
without truncation.
