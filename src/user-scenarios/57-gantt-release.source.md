# US-57 — Gantt: release schedule

Each row is one task: `label: start, end, status`. Status codes: 0=Done,
1=InProgress, 2=Planned, 3=Optional — each picks a different shading glyph.

```proof:chart kind=gantt width=60 title="v1.0 release schedule"
Spec freeze: 0, 2, 0
Build core: 1, 5, 0
Review pass: 4, 7, 1
Doc author: 5, 8, 1
Beta release: 7, 9, 2
GA prep: 8, 10, 2
Stretch features: 6, 11, 3
```

Overlapping tasks are visually distinct because they live on different rows;
status shading separates done work from planned and optional.
