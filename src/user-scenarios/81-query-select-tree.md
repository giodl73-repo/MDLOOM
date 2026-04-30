# US-81 — ?select project columns into a tree source

<!-- proof:compiled from="proof:tree kind=org" uri="md://src/user-scenarios/data/models.md#:table:0?select=model,status" -->
```org
baseline
└── LSTM-baseline
better
└── GRU-v2
good
├── Transformer-S
└── Hybrid-CNN
best
└── Transformer-L
```
<!-- /proof:compiled -->

Same data as US-69 but the URI now projects only the two columns the tree
actually consumes. The other columns (accuracy, delta, val_loss) drop out
before the tree generator sees the table.
