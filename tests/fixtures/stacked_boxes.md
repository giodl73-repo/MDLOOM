# Stacked Boxes — No Errors

Two or more boxes connected by arrow/connector lines. The bottom border (`└──┘`)
of the first box must NOT be treated as the top of a new phantom box — that was
the Pattern C false positive eliminated by the can_open_box() check.

Unicode stacked (3 boxes with connectors):

```
┌───────────┐
│ Box One   │
└───────────┘
      │
      ▼
┌───────────┐
│ Box Two   │
└───────────┘
      │
      ▼
┌───────────┐
│ Box Three │
└───────────┘
```

ASCII stacked:

```
+----------+
|  Step 1  |
+----------+
     |
     v
+----------+
|  Step 2  |
+----------+
     |
     v
+----------+
|  Step 3  |
+----------+
```
