# US-55 — Heatmap: weekly availability grid

Body syntax for heatmap cells is `row|col: value`. Rows and columns appear
in first-seen order. Five shading levels (` ░▒▓█`) bucket the values.

```proof:chart kind=heatmap width=60 title="Availability by day × time"
Mon|9am: 4
Mon|11am: 8
Mon|1pm: 6
Mon|3pm: 3
Tue|9am: 7
Tue|11am: 9
Tue|1pm: 7
Tue|3pm: 5
Wed|9am: 2
Wed|11am: 5
Wed|1pm: 8
Wed|3pm: 4
```

Higher values produce darker shading; the busiest cell (Tue 11am: 9) renders
at the maximum shading level.
