# Column Misalignment — Should Report Errors

The inner | is at column 7, but the border's + is at column 8.
One fewer space in first cell shifts everything by one.

```
+------+------+
| good | good |
| bad |  bad  |
+------+------+
```

A row where the inner | appears two columns early:

```
+--------+--------+
| good   | good   |
| ba  |     good  |
+--------+--------+
```
