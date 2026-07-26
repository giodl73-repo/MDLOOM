# US-110 — Full document: prose + math + chart + tree + table

A single source file exercising five different directive kinds in one
document, the way a real authoring workflow would.

## Background

The bias-variance tradeoff governs the relationship between model capacity
and generalization. The expected error decomposes:

```mdloom:math
\text{Error} = \text{Bias}^2 + \text{Variance} + \sigma^2
```

## Empirical results

Validation loss across our four candidate models:

```mdloom:chart kind=bar width=50 label-field=model value-field=delta source=md://src/user-scenarios/data/models.md#:table:0
```

## Tradeoff hierarchy

```mdloom:tree kind=taxonomy
root: Tradeoffs
- Capacity
  - Model size
  - Training data volume
- Regularization
  - L2
  - Dropout
  - Early stopping
- Evaluation
  - Cross-validation
  - Held-out test set
```

## Models in scope

```mdloom:table
md://src/user-scenarios/data/models.md#:table:0
```

## See also

```mdloom:xref uri=md://docs/guides/01-math.md
```
