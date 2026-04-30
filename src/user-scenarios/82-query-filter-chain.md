# US-82 — Multiple ?filter terms compose with AND

Filter to status=better OR best — proof's filter is AND-only across repeated
terms, so this scenario uses a single `!=` filter that excludes only the
baseline. The remaining three rows render as-is.

<!-- proof:compiled from="md://src/user-scenarios/data/models.md#:table:0?filter=status!=baseline" -->
```
| model | accuracy | delta | val_loss | status |
|---|---|---|---|---|
| GRU-v2 | 90.4% | +1.3 | 3,2,2,1,1,1,1 | better |
| Transformer-S | 92.1% | +3.0 | 3,2,2,2,1,1,1 | good |
| Transformer-L | 94.2% | +5.1 | 3,3,2,2,1,1,1 | best |
| Hybrid-CNN | 91.8% | +2.7 | 3,2,2,2,1,1,1 | good |
```
<!-- /proof:compiled -->
