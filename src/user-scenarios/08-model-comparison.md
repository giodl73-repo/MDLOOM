# Model Comparison — Q1 Evaluation

Comparing 5 variants of our sequence model on the validation set.
Higher accuracy and lower loss trend (downward sparkline) is better.

```mdloom:row source=md://src/user-scenarios/data/models.md foreach=row separator=" │ "
mdloom:element kind=label field=model width=20
mdloom:element kind=label field=accuracy width=10
mdloom:element kind=label field=delta width=8
mdloom:element kind=sparkline field=val_loss width=16
mdloom:element kind=badge field=status width=10
```

---

## Winner: Transformer-L

The large transformer ($d_{model} = 512$, $N = 8$ layers) achieves best
accuracy at $94.2\%$, with validation loss:

<!-- mdloom:compiled from="mdloom:math" -->
```
     1  N               
L = -─  ∑  yi \log \hati
     N i=1              
```
<!-- /mdloom:compiled -->

converging to $0.187$ after 50 epochs — a $\Delta = -0.041$ improvement
over the baseline LSTM.
