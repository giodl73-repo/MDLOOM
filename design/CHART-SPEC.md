# proof chart — ASCII Chart Composer and Validator

> **Status**: Design — not yet implemented.

---

## What it is

`proof chart` validates existing ASCII charts in code blocks, generates charts
from source data at `md://` URIs, and embeds them via the `proof:chart` compile
directive. All chart kinds share a common source schema (markdown table) and
a common generation pipeline.

---

## Chart kinds

| Kind | ASCII form | Typical use |
|------|-----------|------------|
| `bar` | `████████` horizontal bars | rankings, comparisons |
| `bar.vertical` | stacked columns | distributions |
| `line` | plotted points + interpolation | trends, time series |
| `scatter` | points on 2D axes, no lines | correlation, clustering |
| `heatmap` | `░▒▓█` shading grid | density, correlation matrix |
| `timeline` | `────●────●────` | history, schedule |
| `sparkline` | `▁▂▄▇█▄▂` inline trend | in-table summary |
| `gantt` | `░░░████░░` schedule bars | project planning |
| `pie` | labeled wedge text | composition (limited ASCII) |

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

## Source schema — shared format

All chart kinds read from a **markdown table** at the source `md://` address.
The table structure is kind-specific but follows GFM table format.

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
- `series`: optional group name (for multi-series plots, different markers per series)
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
- `max`: the full-scale value (determines chart width reference)
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

### Pie chart

ASCII pie charts are approximate at best. proof renders them as labeled
wedge text rather than a geometric arc:

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

Pie charts in ASCII are fundamentally limited — for meaningful composition
display, use a bar chart instead. proof warns if pie is used with < 3 slices
or > 8 slices.

---

## CLI commands

```bash
# Validate an existing chart code block
proof chart check [--kind bar|line|scatter|...] <uri>

# Generate a chart from source data
proof chart generate --kind bar md://data/perf.md#results:table:0
proof chart generate --kind line --x-min -4 --x-max 4 --y-min -3 --y-max 3 \
    md://math/sin-cos.md#data:table:0
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
| `width` | bar, line, scatter | 60 | Chart width in columns |
| `height` | line, scatter, heatmap | 20 | Chart height in rows |
| `x-min` | line, scatter | 0 | X-axis minimum |
| `x-max` | line, scatter | auto | X-axis maximum |
| `y-min` | line, scatter | 0 | Y-axis minimum |
| `y-max` | line, scatter | auto | Y-axis maximum |
| `x-label` | line, scatter | x | X-axis label |
| `y-label` | line, scatter | y | Y-axis label |
| `points` | line, scatter | all | Number of plotted points |
| `interpolate` | line | true | Connect points with line segments |
| `marker` | line, scatter | `*` | Point marker character |
| `shading` | heatmap | `░▒▓█` | 4-char shading scale low→high |
| `bar-char` | bar | `█` | Bar fill character |
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
| C-7 | 2D graph: axis tick spacing is consistent (equal intervals) |
| C-8 | Sparkline: 8-level block chars, min value → `▁`, max value → `█` |
| C-9 | Pie: slice values sum to 100% (or normalized to 100%) |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `ascii_chart_scale` | error | Bar length not proportional to value |
| `ascii_chart_origin` | error | 2D graph origin not at (0,0) in 4-quadrant mode |
| `ascii_chart_sort` | error | Timeline events not in chronological order |
| `ascii_chart_sum` | error | Pie slices don't sum to 100% |
| `ascii_chart_shading` | error | Heatmap shading chars not from declared scale |
| `ascii_chart_kind` | warning | Chart kind not declared — cannot validate |
| `ascii_chart_pie_count` | warning | Pie chart has < 3 or > 8 slices |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/checks/ascii_barchart.rs` | Existing bar chart validation (extend) |
| `src/chart/line.rs` | Line/scatter 2D graph generation |
| `src/chart/heatmap.rs` | Heatmap shading generation |
| `src/chart/timeline.rs` | Timeline generation |
| `src/chart/sparkline.rs` | Sparkline block-char encoding |
| `src/chart/gantt.rs` | Gantt bar generation |
| `src/chart/schema.rs` | Source table parsing shared across kinds |
| `src/commands/chart.rs` | CLI surface |

---

## See also

- [Tree Spec](./tree-spec.md) — ASCII trees (dirtree, org, taxonomy, etc.)
- [Layout Spec](./layout-spec.md) — compose multiple charts side by side
- [Compile Spec](./compile-spec.md) — `proof:chart` directive in compile mode
