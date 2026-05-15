# US-57 — Gantt: release schedule

Each row is one task: `label: start, end, status`. Status codes: 0=Done,
1=InProgress, 2=Planned, 3=Optional — each picks a different shading glyph.

<!-- proof:compiled from="proof:chart" -->
```
                   v1.0 release schedule
                   0                                      11
Spec freeze       │ ████████
Build core        │     ███████████████
Review pass       │                ▒▒▒▒▒▒▒▒▒▒▒
Doc author        │                   ▒▒▒▒▒▒▒▒▒▒▒▒
Beta release      │                          ░░░░░░░░░
GA prep           │                              ░░░░░░░░
Stretch features  │                       │││││││││││││││││││
```
<!-- /proof:compiled -->

Overlapping tasks are visually distinct because they live on different rows;
status shading separates done work from planned and optional.
