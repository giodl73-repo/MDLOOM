# US-68 — Directory tree with excludes

Render the proof source tree, hiding compiled artifacts and the cache
directory. Demonstrates the `exclude` glob list.

```proof:tree kind=dirtree root=src max_depth=2 exclude=target,*.lock
```
