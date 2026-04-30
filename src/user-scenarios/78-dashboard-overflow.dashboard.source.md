---
dashboard:
  width: 40
  height: 4
  regions:
    main: { x: 0, y: 0, width: 40, height: 4 }
---

```proof:region name=main
Line 1
Line 2
Line 3
Line 4
Line 5 — this exceeds height=4 → DASHBOARD-005
Line 6 — even more overflow
```

Demonstrates DASHBOARD-005 overflow warning — the region declares height=4
but the body has 6 lines. Proof clips and reports.
