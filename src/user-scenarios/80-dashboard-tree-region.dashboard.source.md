---
dashboard:
  width: 60
  height: 10
  regions:
    main: { x: 0, y: 0, width: 60, height: 10 }
---

```proof:region name=main
Site structure:
proof:tree kind=taxonomy
root: docs
- guides
  - getting-started
  - reference
- api
  - reference
- examples
```

Tree directive inside a region — full tree connectors render into the
canvas with the region as the bounding box.
