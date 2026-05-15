# US-101 — Lint catches irregular tree indent (T-3)

This source intentionally contains a tree with mixed 2- and 3-space indent
to exercise the T-3 lint added for v0.7.

```dirtree
project/
  ├── src/
   ├── one.rs
   └── two.rs
  └── README.md
```

Running `proof check` on this file should fire `tree_indent` warnings on
the inconsistent rows. Compile passes (lint warnings don't fail compile).
