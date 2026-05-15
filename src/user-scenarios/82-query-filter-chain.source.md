# US-82 — Multiple ?filter terms compose with AND

Filter to status=better OR best — proof's filter is AND-only across repeated
terms, so this scenario uses a single `!=` filter that excludes only the
baseline. The remaining three rows render as-is.

```proof:table
md://src/user-scenarios/data/models.md#:table:0?filter=status!=baseline
```
