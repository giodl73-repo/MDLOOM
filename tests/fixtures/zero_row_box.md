# Adjacent Borders — Should Be Handled Gracefully

Two border lines with no content rows between them. proof should detect this
as a box with zero content rows. The borders must have matching widths.

```
+----------+
+----------+
```

Mismatched adjacent borders — should report width error:

```
+----------+
+--------+
```
