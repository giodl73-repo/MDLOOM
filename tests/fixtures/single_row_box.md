# Single Row Box — No Errors

The minimum valid box: one content row between two borders.

```
+--------+
| single |
+--------+
```

Unicode single row:

```
┌────────┐
│ single │
└────────┘
```

Two single-row boxes stacked (tests bottom-border-not-top-of-new-box):

```
┌────────┐
│ first  │
└────────┘
┌────────┐
│ second │
└────────┘
```
