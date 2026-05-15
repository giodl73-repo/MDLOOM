# US-64 — Bar chart with all-zero data

Edge case: every value is zero. The bar area should render empty (no fill
characters); chart should not panic on division-by-zero.

<!-- proof:compiled from="proof:chart" -->
```
A  │                                    0
B  │                                    0
C  │                                    0
```
<!-- /proof:compiled -->
