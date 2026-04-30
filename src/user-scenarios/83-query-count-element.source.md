# US-83 — ?count + proof:row pulls a synthetic count cell

`?count` synthesizes a one-cell table with column `count`. Wrap a
proof:row over that synthetic table to surface the value as a proof:element.

Total models tracked:

```proof:row source=md://src/user-scenarios/data/models.md#:table:0?count width=8
  proof:element kind=value field=count width=8
```

Models that improved on baseline:

```proof:row source=md://src/user-scenarios/data/models.md#:table:0?filter=status!=baseline&count width=8
  proof:element kind=value field=count width=8
```
