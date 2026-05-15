# US-99 — Side-by-side layout of three figures

`proof:layout` composes multiple figures into a single ASCII collage with
gaps and labels. Useful in slides and dashboards where you want a row of
visualizations.

```proof:layout gap=4 align=top labels="One Two Three"
md://src/data/features.md#:table:0
md://src/data/features.md#:table:0
md://src/data/features.md#:table:0
```

The same source repeated three times — purely to demonstrate the layout
geometry. Real usage would pull three different figures.
