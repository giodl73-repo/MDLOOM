# US-103 — Lint cluster: typo + missing pin

Multiple lint signals in one file:

- `[sym:checkmrk]` typo → SYMBOL-001 with did-you-mean
- proof:include pin reference that doesn't match a `[[davinci]]` entry

Status: [sym:checkmrk] all green.

```proof:include pin=nonexistent-pin
md://src/data/features.md#:table:0
```
