# proof dashboard — Fixed-Width ASCII Canvas Compositor

> **Status**: ✅ Implemented — `src/dashboard/`. Canvas, region parser, compositor, proof:region compile directive all live. Inner directives inside `proof:region` bodies (proof:chart, proof:tree, proof:row, proof:element, proof:symbol, proof:shape, proof:math) are recursively rendered and pasted into the canvas with `no-chrome` semantics. **Author syntax**: directive lines inside a region are *fenceless* (`proof:chart kind=bar` on its own line followed by data lines), not nested triple-backtick fences — markdown's outer fence would close at the inner backticks. DASHBOARD-001 through DASHBOARD-006 diagnostics wired.

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
  regions:
    header:       { x: 0,  y: 0,  width: 120, height: 3  }
    forwards:     { x: 0,  y: 3,  width: 40,  height: 20 }
    defense:      { x: 41, y: 3,  width: 40,  height: 20 }
    stats:        { x: 82, y: 3,  width: 38,  height: 20 }
    player-table: { x: 0,  y: 24, width: 120, height: 16 }
---

```proof:region name=header
proof:element kind=label value="EDMONTON OILERS" width=40
proof:element kind=sparkline width=20 no-chrome source=md://stats/2025.md#edm:table:0?select=date,pts
```

```proof:region name=forwards
proof:tree kind=org name="Player" parent="Line" label="Score"
md://reports/edm-forwards.md#depth:table:0
```

```proof:region name=stats
proof:chart kind=bar no-chrome
md://stats/2025.md#edm-leaders:table:0
```

```proof:region name=player-table
proof:row source=md://stats/2025.md#edm:table:0 width=120
  proof:element kind=label field=name width=24
  proof:element kind=value field=pts_82 format="{:.1}" width=6
  proof:element kind=mini-bar field=pts_82 max=200 width=20 no-chrome
  proof:element kind=sparkline width=10 no-chrome field=career_arc
  proof:element kind=label style=badge field=expiry_type width=5
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

### Canvas coordinate system

Coordinates are **0-indexed**: column 0 is the leftmost character, row 0 is the top line. A region at `x: 41` starts at column index 41 (0-based). Canvas cells not covered by any region are filled with a **space** character (`' '`). The defense region example in the diagram (`[41,3 40×20]`) is a separate region from forwards (`[0,3 40×20]`) — column 40 (the border between them at 0-index) belongs to neither; it is a space. Authors who want a visual border between regions declare a narrow separator region.

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

Dashboard compile cache key includes: `source_parse_key`, all region `source` resolve_keys, layout config hash, **canvas_width, canvas_height**. Changing `--width` or `--height` produces a cache miss even if source data is unchanged.

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

### Region content parsing

Inside a `proof:region` fenced block, lines are parsed as follows:

- A line starting with `proof:element`, `proof:tree`, `proof:chart`, `proof:row` (after optional leading spaces) is a **directive line** — processed as a proof: directive
- All other lines are **literal content** — rendered verbatim

Directive lines do NOT use fenced blocks inside a region. They are single-line directives with their source URI on the next line:

```
proof:element kind=label value="EDMONTON OILERS" width=40 no-chrome
proof:sparkline width=20 no-chrome source=md://stats/2025.md#edm:table:0?select=date,pts
```

The `proof:region` block itself is the container fence. No nested fences.

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
| `--explain` | Write a JSON traceability manifest alongside the compiled output showing which source URI produced each canvas region |

---

## DaVinci invariants

| Invariant | Claim |
|-----------|-------|
| D-1 | Each `proof:row` element widths + separators = declared row width |
| D-2 | Every region: `x + width ≤ canvas width`, `y + height ≤ canvas height` |
| D-3 | No two regions overlap (bounding boxes are disjoint) |
| D-4 | Every `proof:element kind=value` resolves to a scalar |
| D-5 | `proof:row` loop count (via `source=`) matches source table row count |
| D-6 | Total canvas is exactly `width × height` characters (no jagged lines) |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `DASHBOARD-001` | error | Region `x + width` exceeds canvas width |
| `DASHBOARD-002` | error | Region `y + height` exceeds canvas height |
| `DASHBOARD-003` | error | Two regions overlap |
| `DASHBOARD-004` | error | Named region in content has no front-matter declaration |
| `DASHBOARD-005` | warning | Region content overflows declared height — lines N..M clipped (N lines lost); use --explain to see clipped content |
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

---

## Spec Clarifications (from scenario findings)

- **F63** (region fill): Rows in a region not covered by content remain as space characters. No padding, borders, or fill lines are added.
- **F64** (canvas init char): `Canvas::new` initializes all cells to ASCII space (U+0020). The fill character is fixed — not configurable.
- **F65** (overlap complexity): AABB overlap check is O(N²) over region pairs. For dashboards with up to ~20 regions this is acceptable. Large dashboards (50+ regions) may optimize with a spatial index in future.
- **F66** (adjacent not overlapping): Adjacent regions are NOT overlapping. Region at x=0,width=40 occupies columns 0-39. Region at x=40,width=40 occupies columns 40-79. They share no cell. DASHBOARD-003 only fires when rectangles have at least one cell in common.
- **F67** (element placement in region): Elements rendered inside a region are left-aligned within the region by default. Element `width` must be ≤ region `width` — oversized elements are clipped at the region boundary.
- **F70** (tree overflow in region): If a tree's output is taller than the region height, excess lines are silently clipped by the canvas paste operation. No DASHBOARD-002 is emitted for this case (content-overflow is expected).

