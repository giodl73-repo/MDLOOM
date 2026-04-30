# US-69 — Org tree from data table

Driven by the model-evaluation table — `status` becomes the parent column,
`model` becomes the node name. Proof synthesizes parent nodes for any
status that isn't itself listed as a row.

```proof:tree kind=org source=md://src/user-scenarios/data/models.md#:table:0 name=model parent=status
```

`baseline`, `better`, `good`, `best` are synthesized as parent categories
with the matching models grouped underneath.
