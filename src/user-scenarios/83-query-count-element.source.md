# US-83 — ?count + mdloom:row pulls a synthetic count cell

`?count` synthesizes a one-cell table with column `count`. Wrap a
mdloom:row over that synthetic table to surface the value as a mdloom:element.

Total models tracked:

```mdloom:row source=md://src/user-scenarios/data/models.md#:table:0?count width=8
  mdloom:element kind=value field=count width=8
```

Models that improved on baseline:

```mdloom:row source=md://src/user-scenarios/data/models.md#:table:0?filter=status!=baseline&count width=8
  mdloom:element kind=value field=count width=8
```
