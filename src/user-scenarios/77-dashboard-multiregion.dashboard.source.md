---
dashboard:
  width: 80
  height: 14
  regions:
    header: { x: 0, y: 0, width: 80, height: 3 }
    left: { x: 0, y: 3, width: 40, height: 11 }
    right: { x: 40, y: 3, width: 40, height: 11 }
---

```proof:region name=header
SPEC HEALTH — every "✅ Implemented" status now means it
```

```proof:region name=left
Specs claimed:
proof:element kind=value value=12 width=8 no-chrome
Specs ⚡ Partial:
proof:element kind=value value=2 width=8 no-chrome
Specs 🔲 Not started:
proof:element kind=value value=0 width=8 no-chrome
```

```proof:region name=right
proof:chart kind=bar width=35 no-chrome
v0.5: 9
v0.6: 11
v0.7: 14
```

Two side-by-side panels under a header strip — left has labels + values,
right has a bar chart growth trend.
