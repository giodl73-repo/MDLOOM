# US-64 — Bar chart with all-zero data

Edge case: every value is zero. The bar area should render empty (no fill
characters); chart should not panic on division-by-zero.

<!-- mdloom:compiled from="mdloom:chart" -->
```
A  │                                    0
B  │                                    0
C  │                                    0
```
<!-- /mdloom:compiled -->
