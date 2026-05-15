# US-110 — Full document: prose + math + chart + tree + table

A single source file exercising five different directive kinds in one
document, the way a real authoring workflow would.

## Background

The bias-variance tradeoff governs the relationship between model capacity
and generalization. The expected error decomposes:

<!-- proof:compiled from="proof:math" -->
```
Error = Bias² + Variance + σ²
```
<!-- /proof:compiled -->

## Empirical results

Validation loss across our four candidate models:

<!-- proof:compiled from="proof:chart" -->
```
LSTM-baseline  │                                  0
GRU-v2         │ ███████                       1.30
Transformer-S  │ █████████████████                3
Transformer-L  │ █████████████████████████████ 5.10
Hybrid-CNN     │ ███████████████               2.70
```
<!-- /proof:compiled -->

## Tradeoff hierarchy

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
Tradeoffs
├── Capacity
│   ├── Model size
│   └── Training data volume
├── Regularization
│   ├── L2
│   ├── Dropout
│   └── Early stopping
└── Evaluation
    ├── Cross-validation
    └── Held-out test set
```
<!-- /proof:compiled -->

## Models in scope

<!-- proof:compiled from="md://src/user-scenarios/data/models.md#:table:0" -->
```
model | accuracy | delta | val_loss | status
------- | ---------- | ------- | ---------- | --------
LSTM-baseline | 89.1% | +0.0 | 2,3,2,2,1,1,1 | baseline
GRU-v2 | 90.4% | +1.3 | 3,2,2,1,1,1,1 | better
Transformer-S | 92.1% | +3.0 | 3,2,2,2,1,1,1 | good
Transformer-L | 94.2% | +5.1 | 3,3,2,2,1,1,1 | best
Hybrid-CNN | 91.8% | +2.7 | 3,2,2,2,1,1,1 | good
```
<!-- /proof:compiled -->

## See also

<!-- proof:compiled from="proof:xref" -->
*See: [01 math](docs/guides/01-math.md)*
<!-- /proof:compiled -->
