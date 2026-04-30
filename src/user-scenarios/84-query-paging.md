# US-84 — ?skip + ?top combine for SQL-style paging

Skip the first model (baseline), then take the next two. Result: GRU-v2 +
Transformer-S.

<!-- proof:compiled from="md://src/user-scenarios/data/models.md#:table:0?skip=1&top=2" -->
```
| model | accuracy | delta | val_loss | status |
|---|---|---|---|---|
| GRU-v2 | 90.4% | +1.3 | 3,2,2,1,1,1,1 | better |
| Transformer-S | 92.1% | +3.0 | 3,2,2,2,1,1,1 | good |
```
<!-- /proof:compiled -->
