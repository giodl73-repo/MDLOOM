# US-108 — Query params chained into a chart from a numeric column

End-to-end: filter the model table to non-baseline rows, take the top two,
chart the `accuracy` column. Note: the model table's accuracy values
include `%` so they don't parse as plain f64; this scenario substitutes a
hypothetical numeric `score` column to demonstrate the composition syntax.

```text
proof:chart kind=bar width=50 label-field=model value-field=score
            source=md://stats.md#:table:0?filter=status!=baseline&top=2
```

What renders: a 2-bar chart with the top two non-baseline models. Equivalent
queries can be composed for any numeric column the source table provides.
