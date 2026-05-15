# US-69 — Org tree from data table

Driven by the model-evaluation table — `status` becomes the parent column,
`model` becomes the node name. Proof synthesizes parent nodes for any
status that isn't itself listed as a row.

<!-- proof:compiled from="proof:tree kind=org" uri="md://src/user-scenarios/data/models.md#:table:0" -->
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

`baseline`, `better`, `good`, `best` are synthesized as parent categories
with the matching models grouped underneath.
