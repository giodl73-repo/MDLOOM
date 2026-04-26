# Nested Boxes — Inner Borders Generate Warnings (Expected)

An outer box containing inner boxes. The current detector pairing algorithm
cannot distinguish inner box borders from outer box content rows, so inner
box border lines generate ascii_box_col warnings (the inner `+` positions
don't align with the outer box's expected `|` positions).

This is EXPECTED behavior for nested diagrams. The test verifies:
1. The outer box itself is detected correctly (no panic)
2. Inner border lines generate warnings (not silently ignored)
3. proof does not panic or infinite-loop on complex nested diagrams

```
+------------------------------------------+
| OUTER BOX                                |
|  +----------------+  +----------------+  |
|  | inner left     |  | inner right    |  |
|  +----------------+  +----------------+  |
+------------------------------------------+
```

A simpler nested (inner box at the edges — column 1 matches):

```
+--------------------+
| outer              |
+----------+         |
| inner    |         |
+----------+         |
| outer    |         |
+--------------------+
```
