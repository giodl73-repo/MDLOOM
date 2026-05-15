---
dashboard:
  width: 60
  height: 8
  regions:
    main: { x: 0, y: 0, width: 60, height: 8 }
---

```proof:region name=main
Status: [sym:checkmark] all green
proof:symbol checkmark size=2
proof:shape name=badge label="MVP" width=12
```

Three different glyph kinds in one region — inline `[sym:name]` expansion
in literal text, then a multi-line proof:symbol block, then a proof:shape.
