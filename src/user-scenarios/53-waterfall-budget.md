# US-53 — Waterfall: project budget reconciliation

Start with the planned budget, walk through approved deltas, land on the
final figure. `▓` marks Start and End, `█` is a positive delta, `▒` is
negative.

<!-- proof:compiled from="proof:chart" -->
```
                      Budget walk ($K)
Plan        │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓       500
Hardware    │                                     ███████ 580
Staffing    │                                  ▒▒▒▒▒▒▒▒▒▒ 460
Vendor      │                               ▒▒▒▒          415
Q3 reserve  │                               ███           445
Final       │                                 ▓           445
```
<!-- /proof:compiled -->

The bars float at the running-total y-coordinate so each delta visually
reads as "what changed from the prior level."
