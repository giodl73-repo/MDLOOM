---
dashboard:
  width: 60
  height: 12
  regions:
    title: { x: 0, y: 0, width: 60, height: 2 }
    chart: { x: 0, y: 2, width: 60, height: 10 }
---

```proof:region name=title
PROOF release health
```

```proof:region name=chart
proof:chart kind=sparkline width=50 no-chrome
v0.1: 12
v0.2: 15
v0.3: 20
v0.4: 35
v0.5: 60
v0.6: 78
v0.7: 100
```

A title region above and a chart region below — the inner directive renders
into the canvas at the region's coordinates.
