# Width Mismatch — Should Report Errors

The bottom border is one character wider than the top.

```
+----------+----------+
| cell one | cell two |
+----------+----------++
```

A content row that's too wide:

```
+------+------+
| foo  | bar  |
| baz   | qux |
+------+------+
```
