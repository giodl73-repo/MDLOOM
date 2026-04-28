# proof Dashboard — Fixed-Canvas Terminal UIs

A dashboard is a fixed-size ASCII canvas composed from named regions. Unlike
slides (which flow top-to-bottom) or prose (which wraps), a dashboard gives
you pixel-level control over position. Each region is a rectangle declared in
YAML front-matter with exact x/y coordinates and dimensions. The dashboard
compositor places each region's content into its bounding box and clips anything
that overflows.

Use dashboards for terminal status boards, monitoring displays, data summaries,
and any output where visual layout is as important as content. They compile to
a `.dashboard.md` file that renders cleanly in any terminal or fixed-width
context.

---

## Basic dashboard structure

A dashboard source file is a `.dashboard.source.md` file with two parts:
YAML front-matter declaring the canvas and regions, and `proof:region` blocks
providing content for each region.

The front-matter declares the canvas dimensions and names every region with
its position (`x`, `y`) and size (`width`, `height`). Positions are
zero-indexed from the top-left corner. The compositor validates that regions
don't overlap and that content fits within each region's bounds.

```yaml
---
dashboard:
  width: 80
  height: 20
  title: "My Dashboard"
  regions:
    header: { x: 0, y: 0, width: 80, height: 3 }
    left:   { x: 0, y: 3, width: 40, height: 14 }
    right:  { x: 40, y: 3, width: 40, height: 14 }
    footer: { x: 0, y: 17, width: 80, height: 3 }
---
```

---

## Region content

Each `proof:region` block fills one named region. The body of the block is a
mini-document that supports the same directives as any compiled source file:
`proof:element`, `proof:tree`, `proof:symbol`, `proof:shape`, `proof:bullets`,
and literal text lines. Content is clipped to the region's declared width and
height — if your content is taller than the region, it's truncated.

---

## Example: proof stats dashboard

This shows a typical 60-column stats board with a title header, three metric
columns, and a footer.

```yaml
---
dashboard:
  width: 60
  height: 14
  title: "proof stats"
  regions:
    title:  { x: 0, y: 0,  width: 60, height: 2  }
    tests:  { x: 0, y: 2,  width: 20, height: 5  }
    mods:   { x: 20, y: 2, width: 20, height: 5  }
    loc:    { x: 40, y: 2, width: 20, height: 5  }
    footer: { x: 0, y: 9,  width: 60, height: 2  }
---
```

---

## Canvas coordinate system

The canvas uses a standard screen coordinate system: origin at the top-left,
`x` increasing rightward (columns), `y` increasing downward (rows). All values
are zero-indexed.

```
(0,0) ────────────────────→ x (columns)
  │
  │   ┌──────────────────┐
  │   │  header (y=0)    │
  │   ├──────────────────┤
  │   │  left  |  right  │
  │   │  (y=3) |  (y=3)  │
  │   ├──────────────────┤
  │   │  footer (y=17)   │
  │   └──────────────────┘
  ↓
  y (rows)
```

When positioning regions, think of `x` as the left edge and `y` as the top
edge of the region rectangle. Two regions overlap if their rectangles
intersect — proof reports this as a `DASHBOARD-003` error.

---

## Diagnostic codes

| Code | Meaning |
|------|---------|
| `DASHBOARD-001` | Region declared in front-matter but no `proof:region` block provides content |
| `DASHBOARD-002` | Region content exceeds its bounding box (clipped) |
| `DASHBOARD-003` | Two regions overlap — adjust positions or sizes |

---

## Region attributes

The `proof:region name=X` directive takes one required attribute:

| Attribute | Required | Description |
|-----------|----------|-------------|
| `name` | yes | Must match a key declared in the YAML `regions:` map |

The region's position and size are entirely determined by the front-matter
declaration — you can't override them in the `proof:region` block.

---

## Dashboard vs slide

Both dashboards and slides produce fixed-width ASCII output, but they solve
different problems:

| | Dashboard | Slide |
|-|-----------|-------|
| Layout | Pixel-precise x/y coordinates | Flow-based zones (title + body) |
| Content | Any proof: directives | Slide body directives |
| File suffix | `.dashboard.source.md` | `.slides.source.md` |
| Use case | Terminal UIs, status boards | Presentations, decks |
| Output | Single canvas | Sequence of slides with separators |

Use dashboards when you need to control exactly where content appears. Use
slides when you want a clean authoring experience with automatic layout.
