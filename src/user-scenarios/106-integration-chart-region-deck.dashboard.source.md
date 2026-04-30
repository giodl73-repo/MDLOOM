---
dashboard:
  width: 80
  height: 16
  regions:
    title: { x: 0, y: 0, width: 80, height: 2 }
    metric_a: { x: 0, y: 2, width: 40, height: 7 }
    metric_b: { x: 40, y: 2, width: 40, height: 7 }
    trend: { x: 0, y: 9, width: 80, height: 7 }
---

```proof:region name=title
PROOF release-health dashboard
```

```proof:region name=metric_a
Tests passing:
proof:element kind=value value="793" width=10 no-chrome
Build warnings:
proof:element kind=value value="0" width=10 no-chrome
```

```proof:region name=metric_b
Chart kinds:
proof:element kind=value value="10" width=10 no-chrome
Slide layouts:
proof:element kind=value value="8" width=10 no-chrome
```

```proof:region name=trend
proof:chart kind=bar width=70 no-chrome
v0.5: 540
v0.6: 711
v0.7: 793
```

Full integration: dashboard with a header, two metric panels (each with
elements), and a chart trend region across the bottom.
