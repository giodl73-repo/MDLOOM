# proof dashboard — Fixed-Width ASCII Canvas Compositor

> **Status**: Design — not yet implemented.

---

## What it is

A dashboard is a **fixed-width ASCII canvas** compiled from a `.dashboard.source.md` file.
Unlike a flowing document, every element in a dashboard has an explicit position and size.
The output is a single fenced block of exactly `width × height` characters.

Dashboards are the top-level artifact for terminal UIs, reports, and data displays.
The IceLines NHL app uses dashboards as the data layer for every TUI screen.

---

## Source format (`.dashboard.source.md`)

```markdown
---
dashboard:
  width: 120
  height: 40
  title: "EDM 2025-26 — Team Dashboard"
---

# Regions

## header
x: 0, y: 0, width: 120, height: 3

## forwards-tree
x: 0, y: 3, width: 40, height: 20

## stats-chart
x: 82, y: 3, width: 38, height: 20

## player-table
x: 0, y: 24, width: 120, height: 16

# Content

```proof:region name=header
proof:element kind=label value="EDMONTON OILERS" width=40
proof:element kind=sparkline width=20 no-chrome
md://stats/2025.md#edm:table:0?select=date,pts
```

```proof:region name=forwards-tree
proof:tree kind=org name="Player" parent="Line" label="Score"
md://reports/edm-forwards.md#depth:table:0
```

```proof:region name=stats-chart
proof:chart kind=bar no-chrome
md://stats/2025.md#edm-leaders:table:0
```

```proof:region name=player-table
proof:row foreach=player in md://stats/2025.md#edm:table:0 width=120
  proof:element kind=label field=name width=24
  proof:element kind=value field=pts_82 format="{:.1}" width=6
  proof:element kind=mini-bar field=pts_82 max=200 width=20 no-chrome
  proof:element kind=sparkline width=10 no-chrome field=career_arc
  proof:element kind=badge field=expiry_type width=5
  proof:element kind=delta field=improvement format="{:+.2}" width=6
```
```

---

## Canvas model

A dashboard is a 2D grid of `width` columns × `height` rows. Every character cell
has an explicit position. Regions tile the canvas; their content is rendered and
clipped to their declared bounding box.

```
(0,0)────────────────────────────────────────────────(120,0)
│   header   [0,0 120×3]                                    │
├────────────────────────────────────────────────────────────┤
│ forwards   [0,3 40×20] │ defense [41,3 40×20] │ stats [82,3 38×20] │
├────────────────────────────────────────────────────────────┤
│   player-table   [0,24 120×16]                             │
(0,40)──────────────────────────────────────────────(120,40)
```

---

## Regions

Each region declares `x`, `y`, `width`, `height`. The compiler renders the
`proof:region` content into that bounding box, clipping at the boundary.

Region content is any combination of:
- `proof:element` — micro-element primitive
- `proof:row` — horizontal element compositor
- `proof:tree` — tree diagram
- `proof:chart` — chart (rendered without fence, `no-chrome` implied within regions)
- Plain text / markdown headings (rendered as literal text)

All content within a region uses `no-chrome` by default — the region boundary is
the container, not a fence.

---

## Compilation

```bash
proof compile report.dashboard.source.md
# → report.dashboard.md

proof compile report.dashboard.source.md --width 80 --height 24
# → report.dashboard.md (canvas scaled to 80×24)
```

Output format:

````markdown
<!-- proof:compiled from="proof:dashboard" title="EDM 2025-26 — Team Dashboard" -->
```dashboard
EDMONTON OILERS                      ▁▂▅▇█▆▄▃▂▄  Team Score: 927.0
────────────────────────────────────────────────────────────────────────────────
Forwards                Defense              Goals/82
├── Line 1              ├── Pair 1           McDavid  ████████████████████  138
│   ├── C: McDavid  138 │   ├── Bouchard  95  Kucherov ███████████████████  130
│   ├── LW: Hyman    73 │   └── Ekholm    65  Draisait ██████████████████   117
│   └── RW: Drais   116 ├── Pair 2
...                     ...
────────────────────────────────────────────────────────────────────────────────
Player                  Pts/82  ████████████████████  Trend      Type  Δ
Connor McDavid           138.0  ████████████████████  ▁▂▅▇█▆▄  UFA   +0.19
Nikita Kucherov          130.2  ███████████████████   ▃▅▆▇█▇▅  UFA   +0.12
```
<!-- /proof:compiled -->
````

---

## `proof:region` directive

```
```proof:region name=player-table
[content here]
```
```

- `name` — matches a declared region in the front-matter
- Content is rendered into the declared bounding box
- If content overflows `width` → line truncated with `…`
- If content overflows `height` → lines clipped at boundary
- If content underflows → padded with spaces to fill the box

---

## IceLines integration

Every TUI screen is a `.dashboard.source.md` file in `~/.icelines/dashboards/`.

The TUI runtime:
1. Measures terminal: `$COLUMNS × $LINES`
2. Calls `proof compile screen.dashboard.source.md --width $COLUMNS --height $LINES`
3. Reads the compiled ASCII string
4. Renders into a ratatui `Paragraph` widget (no further processing — the ASCII IS the UI)
5. On terminal resize: recompiles with new dimensions

```bash
icelines report team EDM        # compiles and prints team dashboard
icelines report standings        # league standings dashboard
icelines report player McDavid   # player profile dashboard
```

Each dashboard template is:
- User-editable (plain text `.dashboard.source.md`)
- Version-controlled
- Validated by proof (DaVinci invariants, element budget checks)
- Data-bound via mdpath (stable across schema renames)

Adding a new field to a player row = editing the template, not Rust code.

---

## Canvas compositor algorithm

```
proof compile dashboard.source.md
    │
    ├── 1. Parse front-matter (width, height, title)
    │
    ├── 2. Parse regions (x, y, width, height per named region)
    │
    ├── 3. Validate regions (D-2, D-3: bounds + no overlap)
    │
    ├── 4. For each region in declaration order:
    │       ├── Render content into a width×height text buffer
    │       ├── Clip at region boundary
    │       └── Paste into the canvas at (x, y)
    │
    ├── 5. Render canvas to string (width × height chars, newline-terminated rows)
    │
    └── 6. Wrap in fence + traceability comment
```

---

## CLI flags

| Flag | Description |
|------|-------------|
| `--width N` | Override canvas width (for terminal sizing) |
| `--height N` | Override canvas height |
| `--region name` | Render only one region (for partial updates) |
| `--no-chrome` | Suppress fence and traceability comment (raw canvas only) |

---

## DaVinci invariants

| Invariant | Claim |
|-----------|-------|
| D-1 | Each `proof:row` element widths + separators = declared row width |
| D-2 | Every region: `x + width ≤ canvas width`, `y + height ≤ canvas height` |
| D-3 | No two regions overlap (bounding boxes are disjoint) |
| D-4 | Every `proof:element kind=value` resolves to a scalar |
| D-5 | `foreach` loop count matches source table row count |
| D-6 | Total canvas is exactly `width × height` characters (no jagged lines) |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `DASHBOARD-001` | error | Region `x + width` exceeds canvas width |
| `DASHBOARD-002` | error | Region `y + height` exceeds canvas height |
| `DASHBOARD-003` | error | Two regions overlap |
| `DASHBOARD-004` | error | Named region in content has no front-matter declaration |
| `DASHBOARD-005` | warning | Region content overflows declared height — lines clipped |
| `DASHBOARD-006` | warning | Region content underflows declared width — padded with spaces |

---

## What proof needs to implement this

| Component | Status |
|-----------|--------|
| `proof:element` directive | Planned — ELEMENT-SPEC.md |
| `proof:row` compositor | Planned — ELEMENT-SPEC.md |
| `proof:region` directive | Planned — this spec |
| Canvas compositor engine | Planned |
| `--width N --height N` compile flags | Planned |
| `org` tree with field mapping | ✅ Done (Wave 3) |
| `sparkline` / `mini-bar` generation | Planned — CHART-SPEC.md Wave 1 |
| `no-chrome` flag | Planned |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/dashboard/mod.rs` | Canvas compositor |
| `src/dashboard/canvas.rs` | Fixed-width character grid |
| `src/dashboard/region.rs` | Region parsing and content rendering |
| `src/compile.rs` | proof:region directive handling |
| `src/element/mod.rs` | proof:element rendering |
| `src/element/row.rs` | proof:row compositor |

---

## See also

- [Element Spec](./element-spec.md) — `proof:element` and `proof:row` primitives
- [Chart Spec](./chart-spec.md) — chart generation used inside regions
- [Tree Spec](./tree-spec.md) — tree generation used inside regions
- [Compile Spec](./compile-spec.md) — base compilation pipeline
