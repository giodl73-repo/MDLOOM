# US-102 — Lint catches dangling continuation (T-4)

The `src/` row has a level-1 continuation `│   │` underneath but no level-1
child node materializes — T-4 fires `tree_orphan`.

```dirtree
project/
├── src/
│   │
└── README.md
```
