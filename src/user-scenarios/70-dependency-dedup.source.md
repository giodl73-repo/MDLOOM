# US-70 — Dependency tree with shared subtree

When a package appears under multiple parents, the second occurrence is
marked `(deduped ↑ N)` pointing at the first appearance — keeping the
output bounded for diamond-shaped graphs.

```proof:tree kind=dependency
| package | depends_on | version |
|---------|------------|---------|
| app | core | 1.0 |
| app | utils | 0.4 |
| utils | core | 1.0 |
| core | — | 1.0 |
```

`core` is depended on by both `app` and `utils`, but only renders once;
the second occurrence shows the dedup marker.
