# Bottom-Close Borders — No Box Errors

Tests that `└──┘` and `╰──╯` corners cannot open a new box when they appear
between two stacked diagram elements. Without the can_open_box() guard, proof
would detect a phantom box from the bottom border of one element to the top
border of the next, generating hundreds of false width/column errors.

Linear stacked — three real boxes, zero phantom boxes:

```
┌─────────────────────┐
│  Stage 1: Parse     │
└─────────────────────┘
           │
           ▼
┌─────────────────────┐
│  Stage 2: Validate  │
└─────────────────────┘
           │
           ▼
┌─────────────────────┐
│  Stage 3: Output    │
└─────────────────────┘
```

ASCII version:

```
+---------------------+
|  Stage 1: Parse     |
+---------------------+
           |
           v
+---------------------+
|  Stage 2: Validate  |
+---------------------+
           |
           v
+---------------------+
|  Stage 3: Output    |
+---------------------+
```
