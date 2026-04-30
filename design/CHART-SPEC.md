# proof chart — ASCII Chart Composer and Validator

> **Status**: ✅ Implemented — `src/chart/`. Kinds live: bar, line, area, stacked-bar, waterfall, scatter, heatmap, candlestick, gantt, timeline. **Removed from scope:** `sankey` — proportional flow diagrams quantize poorly to fixed-width character cells (flow widths can't be sub-cell, and overlapping/crossing flows are unrepresentable without color). Authors who need flow visualizations should use `kind=stacked-bar` for level transitions or render an SVG via an external tool and embed via `proof:include`.

---

## Namespace

Two code namespaces coexist during the transition period:

**`ascii_barchart_*`** — existing codes emitted by `src/checks/ascii_barchart.rs`.
These are preserved unchanged so existing suppression comments and CI configs
continue to work.

**`ascii_chart_*`** — new codes used by the unified chart system covering all
kinds (line, scatter, heatmap, area, stacked-bar, waterfall, histogram, bullet,
lollipop, candlestick, sankey, timeline, sparkline, gantt, and the migrated bar).

**Deprecation path for bar:** When the `bar` kind is migrated into the unified
chart system, `ascii_barchart_scale` will be aliased to `ascii_chart_scale` and
the old code deprecated with a migration warning. Until then, both codes may
fire for bar charts depending on which checker runs first.

---

## What it is

`proof chart` validates existing ASCII charts in code blocks, generates charts
from source data at `md://` URIs, and embeds them via the `proof:chart` compile
directive. All chart kinds share a common source schema (markdown table) and
a common generation pipeline.

---

## Three categories of chart

Charts in ASCII documentation fall into three structurally distinct categories.
The category determines the rendering pipeline, the source schema, and which
invariants apply.

### Category 1 — Table charts

Data displayed in labelled rows with visual bar/dot/block encoding. **No axes.**
Each row is a self-contained entry; the chart is essentially a rich table.

| Kind | ASCII form | Source schema key |
|------|-----------|-------------------|
| `bar` | `████████` horizontal bars | `item \| value [\| max]` |
| `bar.vertical` | column bars | `item \| value [\| max]` |
| `stacked-bar` | `████░░▒▒` segmented bars | `item \| v1 \| v2 \| v3` |
| `bullet` | bar + target marker `\|` | `metric \| actual \| target \| max` |
| `lollipop` | `─────●` stem + dot | `item \| value [\| max]` |
| `sparkline` | `▁▂▄▇█▄▂` inline trend | `period \| value` |
| `gantt` | `░░████▒▒░` schedule bars | `task \| start \| end \| status` |
| `heatmap` | `░▒▓█` shading grid | matrix table (rows × columns) |

### Category 2 — Graph charts

Data plotted on a 2D coordinate system with explicit **x / y axes**. Supports
first-quadrant (all-positive) and four-quadrant (negative ranges) modes.

| Kind | ASCII form | Source schema key |
|------|-----------|-------------------|
| `line` | points + interpolated connectors | `x \| y [\| series \| label]` |
| `scatter` | points only, no connectors | `x \| y [\| series \| label]` |
| `area` | filled region under a line | `x \| y [\| series]` |
| `histogram` | equal-width bins, count y-axis | `value` (auto-bucketed) or `bin_start \| bin_end \| count` |
| `candlestick` | OHLC range + body | `date \| open \| high \| low \| close` |

### Category 3 — Flow and structure charts

Charts that show **relationships, sequences, or cumulative flow** — not raw data
on axes, not simple row-per-entry tables.

| Kind | ASCII form | Source schema key |
|------|-----------|-------------------|
| `waterfall` | offset bars showing deltas | `step \| delta \| label` |
| `timeline` | `────●────●────` | `date \| event [\| label]` |
| `sankey` | `══╗ ╠══` proportional flows | `from \| to \| value` |

---

> **pie** is in a separate [Experimental](#experimental--limited-viability) section.
> For composition data, use `bar` or `stacked-bar` instead.

---

---

## 2D graphs — axes, quadrants, ranges

The `line`, `scatter`, and future `contour` kinds render on a 2D axis system.
The axis mode is determined by the declared ranges.

### First-quadrant mode (all values ≥ 0)

When `x_min = 0` and `y_min = 0` (the default), the origin sits at the
bottom-left corner:

```
y
│
5 +         *
4 +      *
3 +    *
2 +  *
1 +*
──┼──────────── x
  1  2  3  4  5
```

### Four-quadrant mode (negative ranges)

When `x_min < 0` or `y_min < 0`, the origin moves to the interior and all
four quadrants are rendered. This supports:
- Trigonometric plots (sin, cos, tan)
- Phase diagrams (control theory, complex analysis)
- Physics force vectors
- Economic supply/demand with surplus/deficit

```
          y
          │
        3 +    *
        2 +  *   *
        1 +*       *
──────────┼───────────── x
  -4 -3 -2│-1  1  2  3  4
       -1 +         *
       -2 +       *   *
       -3 +    *
          │
```

**Label-width alignment:** Negative tick labels (e.g., `-10`, 3 chars) are wider
than positive labels (e.g., `10`, 2 chars). The renderer must compute the
maximum tick label width on each axis and pad all labels to that width for
alignment. Failure to do so causes the axis line to shift position between
positive and negative regions.

**Asymmetric x-range origin:** When `x_min` and `x_max` have different absolute
values, the origin column is at:

```
origin_col = round(abs(x_min) / (abs(x_min) + x_max) * chart_width)
```

For the y-axis, symmetrically:

```
origin_row = round(y_max / (y_max + abs(y_min)) * chart_height)
```

Axis configuration:

```toml
# In proof:chart directive attributes or source YAML front-matter
x_min = -4
x_max = 4
y_min = -3
y_max = 3
x_label = "x"
y_label = "f(x)"
```

### Tick interval algorithm

For any axis with range `[min, max]`:

1. Compute `range = max - min`.
2. Compute `step_raw = range / 6` (targeting 6 intervals → ~7 ticks).
3. Round `step_raw` up to the nearest **nice** value from the sequence:
   `1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, …` (powers of 10 × {1, 2, 5}).
4. The resulting tick count must fall in **[5, 8]**. If it falls outside this
   range, adjust by stepping up or down one entry in the nice sequence.
5. Align the first tick to a multiple of the chosen step that is ≥ `min`.

This algorithm ensures ticks land on round numbers and axes never look crowded
or sparse. Invariant C-7 validates this.

### Axis rendering

| Element | Character(s) |
|---------|-------------|
| Y-axis | `│` |
| X-axis | `─` |
| Origin | `┼` (4-quadrant) or `└` (1st quadrant) |
| Tick marks | `+` at regular intervals |
| Axis labels | numeric labels at tick positions |
| Point markers | `*` (default), `●`, `○`, `·`, `+`, or custom |
| Line segments | `─` (horizontal), `│` (vertical), `/` `\` (diagonal) |

---

## Source schema and field mapping

All chart kinds read from a source addressed via `md://`. Field binding
(explicit overrides, auto-detection, query parameters, row selectors, type
coercion, numeric formatting) follows **[MAPPING-SPEC.md](./mapping-spec.md)**.

Per-kind roles are listed below. The source format defaults to GFM markdown
table; override with `format=json` for JSON array sources.

### 2D graph (line / scatter)

```markdown
| x | y | series | label |
|---|---|--------|-------|
| 0 | 0 | A | origin |
| 1 | 1 | A | |
| 2 | 4 | A | |
| 3 | 9 | A | |
| -1 | 1 | B | |
| -2 | 4 | B | |
```

- `x`, `y`: numeric coordinates
- `series`: optional group name. If absent, all rows belong to a single implicit
  series with marker `*`. Multiple series use distinct markers in order:
  `*`, `●`, `○`, `+`, `·`. When two series overlap at the same grid cell, the
  marker of the later-defined series wins and both are listed in the legend.
- `label`: optional annotation placed next to the point

### Bar chart

```markdown
| item | value | max |
|------|-------|-----|
| Go | 87 | 100 |
| Rust | 94 | 100 |
| Python | 72 | 100 |
| C++ | 65 | 100 |
```

- `item`: row label
- `value`: the data value (determines bar length)
- `max`: optional. If absent, proof auto-scales to `max(value)` across all rows.
  If present, bars exceeding `max` are clamped and flagged with `ascii_chart_scale`.
- Optional: `color` column (future: ANSI color blocks)

### Timeline

```markdown
| date | event | label |
|------|-------|-------|
| 1970 | Unix created | AT&T Bell Labs |
| 1991 | Linux kernel | Linus Torvalds |
| 2000 | Go conceived | Google |
| 2015 | Rust 1.0 | Mozilla |
```

Generated:
```
1970     1991          2000  2015
  │        │             │     │
──●────────●─────────────●─────●──────►
  Unix     Linux         Go    Rust 1.0
  AT&T     Torvalds      Ggl   Mozilla
```

- `date`: numeric year, or ISO date `YYYY-MM-DD`
- `event`: marker label above the axis
- `label`: secondary label below (optional)

### Sparkline

```markdown
| month | value |
|-------|-------|
| Jan | 12 |
| Feb | 18 |
| Mar | 9 |
| Apr | 24 |
| May | 31 |
| Jun | 27 |
```

Generated inline: `▃▄▂▆█▇` (8-level Unicode block characters)

Sparklines are designed to appear **inline within a table cell** or as a
compact one-line trend indicator. They render as a single line of block
chars with no axes.

### Heatmap

```markdown
| | Mon | Tue | Wed | Thu | Fri |
|---|-----|-----|-----|-----|-----|
| 9am | 12 | 8 | 15 | 20 | 5 |
| 12pm | 30 | 25 | 28 | 35 | 22 |
| 3pm | 18 | 20 | 24 | 16 | 30 |
| 6pm | 5 | 8 | 10 | 6 | 4 |
```

Generated (4-level shading: `░▒▓█`):
```
        Mon  Tue  Wed  Thu  Fri
9am      ▒    ▒    ▒    ▒    ░
12pm     █    ▓    ▓    █    ▓
3pm      ▒    ▒    ▓    ▒    ▓
6pm      ░    ░    ░    ░    ░
```

### Gantt

```markdown
| task | start | end | status |
|------|-------|-----|--------|
| Design | 1 | 3 | done |
| Implementation | 3 | 7 | done |
| Testing | 6 | 9 | in-progress |
| Release | 9 | 10 | planned |
```

Generated (weeks 1-10):
```
         1  2  3  4  5  6  7  8  9  10
Design   ████░░░░░░░░░░░░░░░░░░░░░░░░
Impl.    ░░░████████████░░░░░░░░░░░░░
Testing  ░░░░░░░░░░░████▒▒▒░░░░░░░░░
Release  ░░░░░░░░░░░░░░░░░░░░░░░███░
```

Fill characters:
- `█` done / complete
- `▒` in-progress
- `░` planned / future
- `·` optional / deferred

### Area chart

Fills the region under a line chart. Same source schema as `line`
(`x | y | series | label`). The area under each point is filled with the
`fill-char` attribute (default `█`).

```
y
4 |   *
3 | * █ *
2 |*█████*
1 |███████*
──┼────────── x
```

Attribute: `fill-char` — one of `░`, `▒`, `▓`, `█` (default `█`).
Multiple series each get their own fill character in the same order as
point markers (`*`, `●`, `○`, `+`, `·`).

### Stacked-bar chart

Multiple value columns stacked in a single horizontal bar. Each column gets a
fill character. A legend below the chart maps fill char → column name.

Source schema:

```markdown
| item | v1 | v2 | v3 |
|------|----|----|-----|
| Go | 60 | 20 | 15 |
| Rust | 70 | 13 | 12 |
| Python | 45 | 25 | 25 |
```

Generated:

```
Go     ████████████████████░░░░░░░░▒▒▒▒▒
Rust   █████████████████████████░░░▒▒▒▒
Python ██████████████░░░░░░░░▒▒▒▒▒▒▒▒▒▒

Legend: █ = v1   ░ = v2   ▒ = v3
```

Fill character assignment order: `█`, `░`, `▒`, `▓`, `·`. Columns beyond
five reuse the sequence with a warning. The total bar width is proportional
to `sum(v1 + v2 + v3)` relative to the row with the largest total.

Diagnostic code: `ascii_chart_stacked_sum` — emitted if a stacked bar's
segments don't add up to the expected total (rounding ±1 char allowed).

### Waterfall chart

Shows cumulative change. Each bar starts where the previous bar ended.
Useful for P&L breakdowns, budget deltas, and stage-by-stage flows.

Source schema:

```markdown
| step | delta | label |
|------|-------|-------|
| Start | 100 | |
| +Q1 | 40 | |
| -Q2 | -15 | |
| +Q3 | 30 | |
| End | 155 | total |
```

The first and last rows are rendered as full bars from zero (totals).
Middle rows are deltas: positive extends right, negative retracts left.

Generated:

```
Start   ████████
+Q1             ████████████
-Q2                     ████
+Q3                         ████████
End     ████████████████████████████
```

The `label` column is optional; if present, it appears to the right of the bar.

Diagnostic code: `ascii_chart_waterfall_balance` — emitted if the final total
bar does not equal `start + sum(deltas)` (rounding ±1 char allowed).

### Histogram

A bar chart where bars represent equal-width bins and the y-axis shows count
(frequency). Can be driven by pre-binned data or raw values that proof buckets
automatically.

**Pre-binned source schema:**

```markdown
| bin_start | bin_end | count |
|-----------|---------|-------|
| 0 | 10 | 5 |
| 10 | 20 | 12 |
| 20 | 30 | 8 |
```

**Raw source schema (auto-binning):**

```markdown
| value |
|-------|
| 14 |
| 7 |
| 22 |
```

When using raw data, proof computes bin count using **Sturges' rule**:
`bins = ceil(log2(n) + 1)`. Override with attribute `bins=N`.

Bins are always equal-width. No gaps are rendered between adjacent bars
(unlike a bar chart). X-axis labels show bin boundaries.

### Bullet chart

A bar with a target marker line. Used for KPI and performance displays.

Source schema:

```markdown
| metric | actual | target | max |
|--------|--------|--------|-----|
| Revenue | 82 | 75 | 100 |
| Margin | 63 | 70 | 100 |
```

Generated:

```
Revenue  ████████████████████████████|         (exceeded)
Margin   ██████████████████████|████████        (missed)
```

The target `|` marker sits at `target / max * chart_width`. The fill bar
extends to `actual / max * chart_width`. When actual > target the bar
visually passes the marker; when actual < target the bar stops before it.

Attribute `max` is optional — defaults to the largest `max` value across
all rows (so all metrics share the same scale).

### Lollipop chart

A cleaner alternative to bar charts. A horizontal stem (`─`) with a circular
marker (`●` by default) at the data value. Reduces ink compared to filled bars.

Source schema: same as `bar` (`item | value | max`).

Generated:

```
Go     ─────────────────────────────●
Rust   ────────────────────────────────────●
Python ────────────────────●
```

Attribute `marker` — default `●`. Other options: `○`, `◉`, `*`, `+`.
The `max` column is optional (same auto-scale rule as `bar`).

### Candlestick chart

OHLC (open / high / low / close) chart for financial or time-series data.
The high-to-low range is shown as a vertical stem (`│`). The open-to-close
body is shown as a filled block (`▓`).

Source schema:

```markdown
| date | open | high | low | close |
|------|------|------|-----|-------|
| Mon | 100 | 115 | 95 | 110 |
| Tue | 110 | 120 | 105 | 108 |
```

Generated (vertical, one column per date):

```
High  │    │
      │    │
Open  ┤▓▓  │
      │▓▓  ┤▓▓
Close └▓▓  │▓▓
      │    └▓▓
Low   │
```

Orientation: vertical by default (time on x-axis, price on y-axis).
Future attribute `orient=horizontal` for rotated display.

### Sankey diagram

Flow diagram showing volume through stages using width-proportional bars.
Each flow's width is proportional to its `value`. Useful for budget flows,
energy balances, and pipeline funnels.

Source schema:

```markdown
| from | to | value |
|------|----|-------|
| Source A | Output X | 40 |
| Source A | Output Y | 20 |
| Source B | Output X | 30 |
| Source C | Output Y | 10 |
```

Generated (approximate ASCII proportional flows):

```
Source A ══════════╗
Source B ════╗     ╠══════ Output X
Source C ═╗  ╠═════╣
          ╚══╝     ╚══════ Output Y
```

The width of each flow segment is `round(value / total * chart_width)`.
Nodes are labeled on the left (sources) and right (sinks). Multi-hop flows
(source → intermediate → sink) are supported; proof lays out intermediate
nodes between source and sink columns.

Diagnostic code: `ascii_chart_sankey_balance` — emitted if the sum of
outgoing flows from a node does not equal the sum of incoming flows
(i.e., flow is not conserved at intermediate nodes).

---

## Experimental / limited viability

### Pie chart

> **Not recommended.** ASCII pie charts are fundamentally limited — they cannot
> represent angular geometry faithfully in a character grid. Use `bar` or
> `stacked-bar` instead for composition displays. `pie` is included for
> completeness and may be removed in a future version.

proof renders pie charts as labeled wedge text rather than a geometric arc:

```markdown
| slice | value | label |
|-------|-------|-------|
| Rust | 35 | Systems |
| Python | 28 | Data/ML |
| Go | 20 | Cloud |
| Other | 17 | Other |
```

Generated (text-layout approximation):

```
┌─────────────────────────────────────┐
│  ████████████  Rust    35%  Systems │
│  ████████      Python  28%  Data/ML │
│  ██████        Go      20%  Cloud   │
│  █████         Other   17%  Other   │
└─────────────────────────────────────┘
```

proof warns if pie is used with < 3 slices or > 8 slices.

---

## CLI commands

```bash
# Validate an existing chart code block
proof chart check [--kind bar|line|scatter|area|stacked-bar|waterfall|histogram|bullet|lollipop|candlestick|sankey|...] <uri>

# Generate a chart from source data
proof chart generate --kind bar md://data/perf.md#results:table:0
proof chart generate --kind line --x-min -4 --x-max 4 --y-min -3 --y-max 3 \
    md://math/sin-cos.md#data:table:0
proof chart generate --kind area --fill-char ░ md://data/volume.md#:table:0
proof chart generate --kind stacked-bar md://data/breakdown.md#:table:0
proof chart generate --kind waterfall md://finance/pnl.md#deltas:table:0
proof chart generate --kind histogram --bins 10 md://data/raw.md#:table:0
proof chart generate --kind bullet md://kpis/q4.md#metrics:table:0
proof chart generate --kind lollipop md://data/perf.md#results:table:0
proof chart generate --kind candlestick md://finance/ohlc.md#:table:0
proof chart generate --kind sankey md://finance/budget.md#flows:table:0
proof chart generate --kind timeline md://history/computing.md#timeline:table:0
proof chart generate --kind sparkline md://metrics/monthly.md#traffic:table:0
proof chart generate --kind gantt md://project/plan.md#schedule:table:0
proof chart generate --kind heatmap md://data/activity.md#heatmap:table:0

# Output to file or stdout
proof chart generate --kind bar md://data.md#:0 -o charts/perf.md
```

---

## The `proof:chart` directive (compile mode)

````markdown
```proof:chart kind=bar width=40
md://data/benchmarks.md#results:table:0
```
````

````markdown
```proof:chart kind=line x-min=-3.14 x-max=3.14 y-min=-1 y-max=1 points=40
md://math/sinusoid.md#sin-data:table:0
```
````

````markdown
```proof:chart kind=timeline
md://history/unix.md#milestones:table:0
```
````

### Directive attributes

| Attribute | Kinds | Default | Description |
|-----------|-------|---------|-------------|
| `kind` | all | required | Chart type |
| `width` | bar, line, scatter, area, stacked-bar, waterfall, histogram, bullet, lollipop, candlestick | 60 | Chart width in columns |
| `height` | line, scatter, area, heatmap, candlestick | 20 | Chart height in rows |
| `x-min` | line, scatter, area, histogram | 0 | X-axis minimum |
| `x-max` | line, scatter, area, histogram | auto | X-axis maximum |
| `y-min` | line, scatter, area | 0 | Y-axis minimum |
| `y-max` | line, scatter, area | auto | Y-axis maximum |
| `x-label` | line, scatter, area, histogram | x | X-axis label |
| `y-label` | line, scatter, area, histogram | y | Y-axis label |
| `points` | line, scatter, area | all | Number of plotted points |
| `interpolate` | line, area | true | Connect points with line segments |
| `marker` | line, scatter, lollipop | `*` / `●` | Point marker character |
| `shading` | heatmap | `░▒▓█` | 4-char shading scale low→high |
| `bar-char` | bar | `█` | Bar fill character |
| `fill-char` | area | `█` | Fill character under line (`░`, `▒`, `▓`, `█`) |
| `bins` | histogram | Sturges | Number of equal-width bins (default: `ceil(log2(n)+1)`) |
| `show-axis` | all | true | Render axis lines |
| `show-labels` | all | true | Render axis tick labels |

---

## Invariants

| Invariant | Claim |
|-----------|-------|
| C-1 | Bar lengths are proportional to values (within ±1 char rounding) |
| C-2 | All bars in a chart use the same scale (max value = full width) |
| C-3 | Timeline events are sorted left-to-right by date |
| C-4 | Heatmap cells use the declared shading scale, min→max maps to first→last char |
| C-5 | Gantt bars are non-overlapping for the same row |
| C-6 | 2D graph: origin `┼` is at coordinates (0,0) in four-quadrant mode |
| C-7 | 2D graph: axis ticks use consistent equal intervals; tick count is 5–8 per axis; step is a "nice" value (see Tick interval algorithm); labels are padded to uniform width |
| C-8 | Sparkline: 8-level block chars, min value → `▁`, max value → `█` |
| C-9 | Pie: slice values sum to 100% (or normalized to 100%) |
| C-10 | Stacked-bar: sum of all segments equals the total bar length (±1 char) |
| C-11 | Waterfall: final total bar equals start + sum(deltas) (±1 char) |
| C-12 | Histogram: all bins are equal width; no gaps between adjacent bars |
| C-13 | Sankey: flow is conserved at intermediate nodes (in = out, ±1 char) |

---

## Diagnostic codes

**`ascii_barchart_*` codes** (existing checker — `src/checks/ascii_barchart.rs`):

| Code | Severity | Meaning |
|------|----------|---------|
| `ascii_barchart_scale` | error | Bar length not proportional to value (legacy bar checker) |

**`ascii_chart_*` codes** (unified chart system):

| Code | Severity | Meaning |
|------|----------|---------|
| `ascii_chart_scale` | error | Bar or segment length not proportional to value; also emitted when a bar exceeds a declared `max` and is clamped |
| `ascii_chart_origin` | error | 2D graph origin not at (0,0) in 4-quadrant mode |
| `ascii_chart_sort` | error | Timeline events not in chronological order |
| `ascii_chart_sum` | error | Pie slices don't sum to 100% |
| `ascii_chart_shading` | error | Heatmap shading chars not from declared scale |
| `ascii_chart_stacked_sum` | error | Stacked-bar segments don't add up to expected total (±1 char) |
| `ascii_chart_waterfall_balance` | error | Waterfall final total ≠ start + sum(deltas) |
| `ascii_chart_sankey_balance` | error | Sankey flow not conserved at an intermediate node |
| `ascii_chart_kind` | warning | Chart kind not declared — cannot validate |
| `ascii_chart_pie_count` | warning | Pie chart has < 3 or > 8 slices |
| `ascii_chart_deprecated_pie` | warning | `pie` kind used — consider `bar` or `stacked-bar` |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/checks/ascii_barchart.rs` | Existing bar chart validation (legacy `ascii_barchart_*` codes) |
| `src/chart/bar.rs` | Unified bar chart (emits `ascii_chart_*` codes; replaces legacy on migration) |
| `src/chart/line.rs` | Line/scatter 2D graph generation |
| `src/chart/area.rs` | Area chart (fill under line) |
| `src/chart/stacked_bar.rs` | Stacked-bar generation and segment validation |
| `src/chart/waterfall.rs` | Waterfall chart (cumulative delta bars) |
| `src/chart/histogram.rs` | Histogram binning and bar generation |
| `src/chart/bullet.rs` | Bullet chart with target marker |
| `src/chart/lollipop.rs` | Lollipop chart (stem + marker) |
| `src/chart/candlestick.rs` | OHLC candlestick chart |
| `src/chart/sankey.rs` | Sankey flow diagram |
| `src/chart/heatmap.rs` | Heatmap shading generation |
| `src/chart/timeline.rs` | Timeline generation |
| `src/chart/sparkline.rs` | Sparkline block-char encoding |
| `src/chart/gantt.rs` | Gantt bar generation |
| `src/chart/ticks.rs` | Shared tick interval algorithm (used by all axis-bearing kinds) |
| `src/chart/schema.rs` | Source table parsing shared across kinds |
| `src/commands/chart.rs` | CLI surface |

---

## See also

- [Tree Spec](./tree-spec.md) — ASCII trees (dirtree, org, taxonomy, etc.)
- [Layout Spec](./layout-spec.md) — compose multiple charts side by side
- [Compile Spec](./compile-spec.md) — `proof:chart` directive in compile mode
