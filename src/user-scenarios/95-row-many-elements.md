# US-95 — Row with multiple elements per record

A row layout that pulls one record per row from the model table and lays
out three elements per record: name label, delta value (single number),
and a sparkline of val_loss. Element widths plus separators (1 char × 2)
must sum to the declared row width — the default separator is a single space.

<!-- proof:compiled from="proof:row" uri="md://src/user-scenarios/data/models.md#:table:0" -->
```
LSTM-baseline        0        ▅█▅▅▁▁▁▅█▅▅▁▁▁▅█▅▅▁▁▁▅
GRU-v2               1.3      █▅▅▁▁▁▁█▅▅▁▁▁▁█▅▅▁▁▁▁█
Transformer-S        3        █▅▅▅▁▁▁█▅▅▅▁▁▁█▅▅▅▁▁▁█
Transformer-L        5.1      ██▅▅▁▁▁██▅▅▁▁▁██▅▅▁▁▁█
Hybrid-CNN           2.7      █▅▅▅▁▁▁█▅▅▅▁▁▁█▅▅▅▁▁▁█
```
<!-- /proof:compiled -->

The `accuracy` column from the table isn't used here — its `89.1%` cells
include a `%` and don't parse as f64. Use it via `proof:element kind=label
field=accuracy` if you want it as text.
