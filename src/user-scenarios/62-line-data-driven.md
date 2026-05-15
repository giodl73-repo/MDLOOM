# US-62 — Line chart from data table

Chart drawn from a markdown table — `model` drives x-axis labels, `delta`
provides the numeric series. The `delta` column has values like `+0.0`,
`+1.3`, `+5.1`; f64 parses each cell directly.

<!-- proof:compiled from="proof:chart" -->
```
                Accuracy delta across models
5.10 ┤                                      //●\\
     ┤                                  ////     \\\\
     ┤                              ////             \\\\\
     ┤                        ///●//                      \\●
     ┤                  //////
     ┤           ///●///
     ┤     //////
   0 ┤ ●///
     └┬────────────┬────────────┬────────────┬─────────────┬
      LSTM-baseline       Transformer-STransformerHybrid-CNN
```
<!-- /proof:compiled -->

Note: the table's `accuracy` column won't parse here because of the `%`
suffix, and `val_loss` won't parse because each cell is a comma-separated
sparkline series. `delta` is the single-value numeric column.
