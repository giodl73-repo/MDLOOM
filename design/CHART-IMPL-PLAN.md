# mdloom chart — Implementation Plan

> **Spec**: `design/CHART-SPEC.md`
> **Status**: ✅ Implemented — `src/chart/`. All ten roster kinds live: bar, line, area, stacked-bar, waterfall, scatter, heatmap, candlestick, gantt, timeline. `sankey` is intentionally out of scope (see CHART-SPEC for rationale). The wave plan below is now historical context, not a forward plan.
> **Implemented**: `mdloom:chart` compile directive live. `mdloom chart check` and `mdloom chart generate` CLI stubs present.

---

## Codebase conventions (established patterns to follow)

| Pattern | Source |
|---------|--------|
| `Check` trait | `src/checks/mod.rs` — `fn name() -> &'static str`, `fn check(path, content) -> Vec<Diagnostic>` |
| Config struct | `src/config.rs` — `#[derive(Debug, Deserialize, Clone)]`, `Default` impl, wired into `MdloomConfig` |
| Compile directive | `src/compile.rs` — `Directive` enum variant, `mdloom_directive_kind()` match arm, `collect_directives()` branch, `compile_file()` arm, `format_*_block()` output |
| CLI subcommand | `src/main.rs` — `Command` enum variant + `cmd_*()` function; `clap` derive pattern |
| Visual width | `src/layout.rs` → `visual_width()` (already handles box-drawing and CJK) |

**Config merge semantics**: scalar fields — child wins. The `ascii_chart` config block should follow the same pattern as `ascii_barchart` in `config.rs:565–594`.

---

## Module layout

```
src/
  chart/
    mod.rs            — pub use all submodules; ChartKind enum
    ticks.rs          — tick interval algorithm (shared by all axis-bearing kinds)
    schema.rs         — source table parser (shared across all kinds)
    table.rs          — Wave 1: Category 1 kinds
    graph.rs          — Wave 2: Category 2 kinds
    flow.rs           — Wave 3: Category 3 kinds
  checks/
    ascii_chart.rs    — unified Check impl (dispatches to chart/ by kind)
  commands/
    chart.rs          — CLI surface (check + generate subcommands)
```

`src/checks/mod.rs` gains `pub mod ascii_chart;`.
`src/commands/chart.rs` is registered in `main.rs`.

---

## Wave 1 — Table charts

**Target file**: `src/chart/table.rs`
**Estimated LOC**: ~650
**Depends on**: nothing (self-contained)

### What to build

Validate and generate the eight Category 1 kinds. All share a common render model: one row per label, visual bar/block/dot encoding, no coordinate system.

#### Migration: `bar`

- Move `AsciiBarchartCheck` logic into `table.rs` as `validate_bar()` and `generate_bar()`.
- Keep `src/checks/ascii_barchart.rs` intact — it continues emitting `ascii_barchart_*` codes.
- New unified path emits `ascii_chart_scale` (aliased meaning) alongside or instead of `ascii_barchart_scale`. Both fire during the overlap window.
- `AsciiChartConfig::kind` = `"bar"` routes to `validate_bar`.

#### New kinds

**`stacked-bar`**
Schema: `item | v1 | v2 | v3 [| ...]`
Fill char order: `█ ░ ▒ ▓ ·` cycling.
Validation: for each row, `round(vN / sum * max_bar_width)` segments must sum to bar width ±1.
Diagnostic: `ascii_chart_stacked_sum`.
Generation: render each row as concatenated fill-char runs, append legend line.

**`bullet`**
Schema: `metric | actual | target | max`
Validation: bar width = `round(actual / max * chart_width)`; target marker `|` at `round(target / max * chart_width)`.
Diagnostic: `ascii_chart_scale` (bar off) — no separate bullet code needed.
Generation: render fill bar, insert `|` at target column.

**`lollipop`**
Schema: `item | value [| max]`
Same auto-scale logic as `bar`. Render as `─────●` where stem length = scaled value.
Configurable `marker` attribute (default `●`).

**`sparkline`**
Schema: `period | value`
8-level block chars: `▁▂▃▄▅▆▇█` (min → max, with equal-width buckets).
Invariant C-8: `min(values)` maps to `▁`, `max(values)` maps to `█`.
Validation: scan each char in the sparkline string; verify it's a block char; verify relative ordering is monotone-consistent with values.
Generation: `(v - min) / (max - min) * 7` rounded → index into `['▁','▂','▃','▄','▅','▆','▇','█']`.
Output: single line, no axes.

**`gantt`**
Schema: `task | start | end | status`
Fill chars: `█` done, `▒` in-progress, `░` planned, `·` optional.
Allowed status values: `done`, `in-progress`, `planned`, `optional`/`deferred`.
Invariant C-5: no overlapping bars within a row (start < end; no row duplicates at same task).
Validation: check fill chars match declared status; check bar proportionality against time axis.
Diagnostic: `ascii_chart_scale` (proportionality), `ascii_chart_kind` (unknown status value as warning).
Generation: compute time axis range = `[min(start), max(end)]`; scale each task bar to chart_width.

**`heatmap`**
Schema: matrix table (first column = row labels, header row = column labels, cells = numeric values).
Invariant C-4: cells must use only chars from the declared 4-char shading scale.
Default scale: `░▒▓█` (low → high).
Validation: parse each cell; look up its shading char; verify it maps to the correct bucket given the declared min/max.
Diagnostic: `ascii_chart_shading`.
Generation: compute min/max across all cells; map each value to `floor((v - min) / (max - min) * 3)` → shading char index.

### Key structs

```rust
// src/chart/table.rs
pub struct BarConfig { pub width: usize, pub bar_char: char, pub tolerance: usize }
pub struct StackedSegment { pub label: String, pub value: f64, pub fill_char: char }
pub struct BulletRow { pub metric: String, pub actual: f64, pub target: f64, pub max: f64 }
pub struct SparklineRow { pub period: String, pub value: f64 }
pub struct GanttRow { pub task: String, pub start: f64, pub end: f64, pub status: GanttStatus }
pub struct HeatmapMatrix { pub row_labels: Vec<String>, pub col_labels: Vec<String>, pub cells: Vec<Vec<f64>> }

pub enum GanttStatus { Done, InProgress, Planned, Optional }

pub fn validate_bar(rows: &[BarRow], path: &Path, config: &BarConfig) -> Vec<Diagnostic>
pub fn generate_bar(data: &[BarData], config: &BarConfig) -> String
pub fn validate_stacked_bar(rows: &[Vec<StackedSegment>], path: &Path, chart_width: usize) -> Vec<Diagnostic>
pub fn generate_stacked_bar(data: &[Vec<StackedSegment>]) -> String
pub fn validate_bullet(rows: &[BulletRow], path: &Path, chart_width: usize) -> Vec<Diagnostic>
pub fn generate_bullet(rows: &[BulletRow], chart_width: usize) -> String
pub fn validate_lollipop(rows: &[BarData], path: &Path, marker: char, chart_width: usize) -> Vec<Diagnostic>
pub fn generate_lollipop(rows: &[BarData], marker: char, chart_width: usize) -> String
pub fn validate_sparkline(values: &[f64], chars: &str, path: &Path, line: usize) -> Vec<Diagnostic>
pub fn generate_sparkline(values: &[f64]) -> String
pub fn validate_gantt(rows: &[GanttRow], path: &Path, chart_width: usize) -> Vec<Diagnostic>
pub fn generate_gantt(rows: &[GanttRow], chart_width: usize) -> String
pub fn validate_heatmap(matrix: &HeatmapMatrix, shading: &str, path: &Path) -> Vec<Diagnostic>
pub fn generate_heatmap(matrix: &HeatmapMatrix, shading: &str) -> String
```

### Config addition (`src/config.rs`)

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct AsciiChartConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,
    pub kind: Option<String>,           // "bar" | "stacked-bar" | "line" | ...
    #[serde(default = "default_chart_width")]
    pub width: usize,                   // default 60
    #[serde(default = "default_chart_height")]
    pub height: usize,                  // default 20
    pub bar_char: Option<char>,         // default '█'
    pub fill_char: Option<char>,        // area fill, default '█'
    pub shading: Option<String>,        // heatmap scale, default "░▒▓█"
    pub marker: Option<char>,           // lollipop/scatter marker
    pub bins: Option<usize>,            // histogram explicit bin count
    #[serde(default = "default_prop_tolerance")]
    pub proportionality_tolerance: usize,
}
```

Add `ascii_chart: AsciiChartConfig` to `MdloomConfig`. Merge: child wins (same as `ascii_barchart`).

### `src/checks/ascii_chart.rs`

```rust
pub struct AsciiChartCheck { pub config: AsciiChartConfig }

impl Check for AsciiChartCheck {
    fn name(&self) -> &'static str { "ascii_chart" }
    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic> {
        // 1. code_block_mask() (reuse from ascii_barchart pattern)
        // 2. detect kind from config or heuristic
        // 3. dispatch: validate_bar | validate_stacked_bar | validate_sparkline | ...
    }
}
```

`src/checks/mod.rs` adds: `pub mod ascii_chart;`

### Tests (`src/chart/table.rs` — `#[cfg(test)]`)

Target: 30+ unit tests. Cover:

| Test area | Count |
|-----------|-------|
| `generate_bar` proportionality, tolerance ±1, clamping | 4 |
| `validate_bar` scale error, clean pass | 3 |
| `stacked_bar` segment sum pass/fail, legend format | 4 |
| `bullet` bar pass, target marker position, missed/exceeded | 3 |
| `lollipop` stem length, marker char | 2 |
| `sparkline` 8 levels correct, min→`▁`, max→`█`, single value | 4 |
| `gantt` status fill chars, overlap detection, proportionality | 4 |
| `heatmap` shading scale, min/max mapping, wrong char detected | 4 |
| Existing `ascii_barchart` tests still pass (no regression) | 2+ |

**Wave 1 exit criterion**: `cargo test` green; existing `ascii_barchart_*` tests pass without modification; 30+ new tests for Wave 1 kinds.

---

## Wave 2 — Graph charts

**Target file**: `src/chart/graph.rs`
**Estimated LOC**: ~900
**Depends on**: `chart/ticks.rs` (build first in this wave)

### Build `src/chart/ticks.rs` first (~120 LOC)

The tick interval algorithm from the spec, tested in isolation.

```rust
pub struct TickResult {
    pub step: f64,
    pub first_tick: f64,
    pub ticks: Vec<f64>,
    pub label_width: usize,    // max label string width (for padding)
}

pub fn compute_ticks(min: f64, max: f64) -> TickResult
```

Algorithm (exact as spec §Tick interval algorithm):
1. `range = max - min`
2. `step_raw = range / 6.0`
3. Round up to nearest nice value from `[1, 2, 5, 10, 25, 50, 100, 250, 500, 1000, ...]`.
4. Count ticks = `ceil((max - first_tick) / step) + 1`. If outside [5, 8], step up/down one nice entry.
5. First tick = `ceil(min / step) * step`.
6. Label width = max width of formatted tick strings (handles `-10` vs `10` difference).

Nice sequence: generate as powers of 10 times each of `{1, 2, 5}` up to `1e9`.

**Ticks unit tests** (10 tests):

| Test | Input | Expected |
|------|-------|---------|
| Range [0, 10] | step = 2, ticks 0..10 | 6 ticks |
| Range [-4, 4] | step = 1 or 2, 5–8 ticks | pass |
| Range [0, 100] | step = 20, 6 ticks | pass |
| Range [-3.14, 3.14] | nice step | 5–8 ticks |
| Range [95, 105] | step = 2 or 5 | 5–8 ticks |
| Range [0, 1] | step = 0.2 | 6 ticks |
| Range [1000, 10000] | step = 2000 | 5 ticks |
| Asymmetric [-1, 9] | step = 2 | 5–6 ticks |
| Single-value range | fallback | no panic |
| Label width for negatives | range [-10, 10] | label_width = 3 |

### Core graph structs

```rust
// src/chart/graph.rs
pub struct ChartGrid {
    pub width: usize,   // columns
    pub height: usize,  // rows
    cells: Vec<Vec<char>>,
}

impl ChartGrid {
    pub fn new(width: usize, height: usize) -> Self    // fill with ' '
    pub fn place(&mut self, col: usize, row: usize, ch: char)
    pub fn render(&self) -> Vec<String>                // row 0 = top of grid
}

pub struct Axis {
    pub min: f64,
    pub max: f64,
    pub ticks: TickResult,
    pub label: String,
}

pub struct DataPoint { pub x: f64, pub y: f64, pub series: String, pub label: Option<String> }
pub struct DataSeries { pub name: String, pub points: Vec<DataPoint>, pub marker: char }

pub struct GraphConfig {
    pub width: usize,           // default 60
    pub height: usize,          // default 20
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
    pub x_label: String,
    pub y_label: String,
    pub interpolate: bool,      // connect points (line/area)
    pub fill_char: char,        // area fill
    pub bins: Option<usize>,    // histogram
}
```

### Quadrant mode and origin placement

```rust
fn origin_col(x_min: f64, x_max: f64, chart_width: usize) -> usize {
    if x_min >= 0.0 { 0 }
    else if x_max <= 0.0 { chart_width - 1 }
    else { (x_min.abs() / (x_min.abs() + x_max) * chart_width as f64).round() as usize }
}

fn origin_row(y_min: f64, y_max: f64, chart_height: usize) -> usize {
    // row 0 = top; origin row increases downward
    if y_min >= 0.0 { chart_height - 1 }  // first-quadrant: origin at bottom
    else if y_max <= 0.0 { 0 }
    else { (y_max / (y_max + y_min.abs()) * chart_height as f64).round() as usize }
}
```

### Data-to-grid coordinate mapping

```rust
fn data_to_col(x: f64, x_min: f64, x_max: f64, chart_width: usize) -> usize
fn data_to_row(y: f64, y_min: f64, y_max: f64, chart_height: usize) -> usize
```

### Kinds

**`line` / `scatter`**
Place each point at `(data_to_col(x), data_to_row(y))` with series marker.
Line: after placing points, scan adjacent pairs; connect with `─` (dy=0), `│` (dx=0), `/` or `\` (diagonal). Multi-step diagonals use bresenham.
Tie-break for overlapping markers: later series wins; both appear in legend.
Diagnostic `ascii_chart_origin`: in 4-quadrant mode, verify `┼` is at the grid cell for (0,0).
Diagnostic `ascii_chart_sort`: (not applicable to scatter/line — used for timeline).

**`area`**
Same as `line` but after drawing the line, fill every cell between the line and the x-axis with `fill_char`. Multi-series: each series uses its own fill char in marker order.

**`histogram`**
Two input modes:
- Pre-binned (`bin_start | bin_end | count`): use directly.
- Raw (`value`): apply Sturges rule `bins = ceil(log2(n) + 1)` (or `config.bins` override); compute equal-width bins from min to max; count values per bin.

Invariant C-12: all bins equal width, no gaps. Validate by checking `bin_end[i] == bin_start[i+1]`.
Render as vertical bars on a 2D grid (y = count, x = bin position). No gap between adjacent bars.
Diagnostic: `ascii_chart_scale` if rendered bar height doesn't match count proportion.

**`candlestick`**
Schema: `date | open | high | low | close`.
Render vertically (time on x-axis, price on y-axis).
For each date column:
- `│` from `data_to_row(high)` to `data_to_row(low)` — the wick.
- `▓` from `data_to_row(min(open,close))` to `data_to_row(max(open,close))` — the body.
Validation: verify `low ≤ open ≤ high` and `low ≤ close ≤ high` (OHLC ordering invariant).
Diagnostic: `ascii_chart_scale` for body/wick mis-rendering.

### Axis rendering

After placing data, overlay axes on the grid:

```rust
fn draw_axes(grid: &mut ChartGrid, x_axis: &Axis, y_axis: &Axis,
             origin_col: usize, origin_row: usize, four_quadrant: bool)
```

- Draw `─` along the x-axis row from col 0 to col width-1.
- Draw `│` along the y-axis column from row 0 to row height-1.
- Place `┼` (4-quadrant) or `└` (first-quadrant) at origin.
- Place `+` tick marks at computed tick positions on each axis.
- Place padded numeric labels at each tick (label_width from `TickResult`).

Label placement: y-axis labels to the LEFT of the `│` column, right-aligned to `label_width`. X-axis labels BELOW the `─` row, centered on tick column.

### Tests (`src/chart/graph.rs` + `src/chart/ticks.rs`)

Target: 40+ unit tests total across ticks and graph.

| Test area | Count |
|-----------|-------|
| Tick algorithm (in `ticks.rs`) | 10 |
| `origin_col` / `origin_row` for 1st/4th quadrant, asymmetric | 5 |
| `data_to_col` / `data_to_row` boundary values | 4 |
| `ChartGrid::place` + `render` round-trip | 3 |
| Line chart: single series, two series overlap tie-break | 4 |
| Scatter: points placed at correct grid coordinates | 3 |
| Area: fill below line, multi-series fill chars | 3 |
| Histogram: pre-binned pass, raw Sturges binning, equal-width invariant | 4 |
| Candlestick: wick + body placement, OHLC ordering error | 3 |
| Axis draw: `┼` at origin in 4-quadrant, `└` in 1st-quadrant | 3 |

**Wave 2 exit criterion**: `mdloom chart generate --kind line --x-min -3 --x-max 3 --y-min -2 --y-max 2` produces a valid 4-quadrant chart with `┼` at (0,0), tick labels padded to uniform width, 5–8 ticks per axis. 40+ tests green.

---

## Wave 3 — Flow charts

**Target file**: `src/chart/flow.rs`
**Estimated LOC**: ~480
**Depends on**: Wave 1 (shares `code_block_mask`), Wave 2 `ticks.rs` (timeline date axis)

### `waterfall`

Schema: `step | delta | label`.
First row and last row are totals (full bars from 0); middle rows are deltas.
Running offset tracks cumulative position.

```rust
pub struct WaterfallRow {
    pub step: String,
    pub delta: f64,
    pub label: Option<String>,
    pub is_total: bool,
}

pub fn validate_waterfall(rows: &[WaterfallRow], path: &Path, chart_width: usize) -> Vec<Diagnostic>
pub fn generate_waterfall(rows: &[WaterfallRow], chart_width: usize) -> String
```

Invariant C-11: `final_total_bar_width == round(start + sum(deltas) / max_abs_val * chart_width)` ±1.
Diagnostic: `ascii_chart_waterfall_balance`.

Rendering: for each row, compute bar start column (`offset`) and bar width (`delta_scaled`). Positive delta extends right from offset; negative delta retracts left. Label placed after bar.

### `timeline`

Schema: `date | event [| label]`.
Date parsing: integer year or ISO `YYYY-MM-DD`. Convert to comparable f64 (year + fractional day).
Invariant C-3: events sorted left-to-right by date.
Diagnostic: `ascii_chart_sort` (emitted per out-of-order pair).

```rust
pub fn validate_timeline(rows: &[TimelineRow], path: &Path, chart_width: usize) -> Vec<Diagnostic>
pub fn generate_timeline(rows: &[TimelineRow], chart_width: usize) -> String
```

Generation:
- Compute date range; map each date to a column via `data_to_col`.
- Render axis line: `────●────●────►`.
- Place `●` at each event column.
- Alternate event labels above/below the axis line to avoid overlap.

### `sankey`

Schema: `from | to | value`.

```rust
pub struct SankeyFlow { pub from: String, pub to: String, pub value: f64 }
pub struct SankeyNode { pub name: String, pub total_in: f64, pub total_out: f64 }

pub fn validate_sankey(flows: &[SankeyFlow], path: &Path) -> Vec<Diagnostic>
pub fn generate_sankey(flows: &[SankeyFlow], chart_width: usize) -> String
```

Flow width: `round(value / total * chart_width / 2)` (half for each side).
Node balance: for intermediate nodes, `total_in` must equal `total_out` ±1 char.
Diagnostic: `ascii_chart_sankey_balance`.
Rendering: sources on left, sinks on right. Each flow rendered as `═` bar scaled to value. Box-drawing connectors `╗ ╠ ╣ ╚` join flow lines to node columns.

Source nodes: labeled on left, sorted by descending `total_out`.
Sink nodes: labeled on right, sorted by descending `total_in`.
Multi-hop flows: intermediate nodes laid out in center column(s).

### Tests (`src/chart/flow.rs`)

Target: 10+ tests per kind (30+ total).

| Test area | Count |
|-----------|-------|
| `waterfall` balance pass, balance fail, positive+negative deltas | 4 |
| `waterfall` generation: offset bars, label placement | 3 |
| `timeline` sort pass, sort fail (out-of-order), same-date tie-break | 4 |
| `timeline` generation: axis rendering, label alternation | 3 |
| `sankey` balance pass at intermediate node, balance fail | 3 |
| `sankey` generation: flow width proportional to value | 3 |
| Date parsing: year-only, ISO date, comparison ordering | 3 |

**Wave 3 exit criterion**: 30+ flow chart tests green; each kind generates recognizable output for a minimal 3-row source table; balance/sort diagnostics fire on malformed inputs.

---

## Wave 4 — Generation from `md://` source schemas

**Target file**: `src/chart/schema.rs`
**Estimated LOC**: ~420
**Depends on**: Waves 1–3 (consumes all `generate_*` functions)

### Source table parser

Reuse the existing `mdpath` library to resolve `md://` URIs. The result is a markdown table string. Parse it with a GFM table parser.

```rust
pub struct SourceTable {
    pub headers: Vec<String>,   // lowercase, trimmed
    pub rows: Vec<Vec<String>>, // body rows
}

impl SourceTable {
    pub fn parse(content: &str) -> Result<Self, SchemaError>
    pub fn column(&self, name: &str) -> Option<Vec<&str>>
    pub fn numeric_column(&self, name: &str) -> Result<Vec<f64>, SchemaError>
    pub fn require_columns(&self, required: &[&str]) -> Result<(), SchemaError>
}

pub enum SchemaError {
    MissingColumn(String),
    NotNumeric { column: String, value: String },
    TooFewRows { kind: &'static str, got: usize },
    ParseError(String),
}
```

### Per-kind schema validators

Each kind's schema validator calls `table.require_columns(...)`, then calls the matching `generate_*()` function from the appropriate wave module.

```rust
pub fn generate_from_source(
    kind: &str,
    table: &SourceTable,
    config: &GraphConfig,      // unified config covering all attrs
) -> Result<String, SchemaError>
```

Dispatches to:
- `"bar"` → `validate_schema_bar(table)?; generate_bar(...)`
- `"stacked-bar"` → `validate_schema_stacked_bar(table)?; generate_stacked_bar(...)`
- `"line"` | `"scatter"` | `"area"` → `validate_schema_graph(table)?; generate_line(...)`
- `"histogram"` → `validate_schema_histogram(table)?; generate_histogram(...)`
- `"candlestick"` → `validate_schema_candlestick(table)?; generate_candlestick(...)`
- `"waterfall"` → `validate_schema_waterfall(table)?; generate_waterfall(...)`
- `"timeline"` → `validate_schema_timeline(table)?; generate_timeline(...)`
- `"sankey"` → `validate_schema_sankey(table)?; generate_sankey(...)`
- `"sparkline"` → `validate_schema_sparkline(table)?; generate_sparkline(...)`
- `"gantt"` → `validate_schema_gantt(table)?; generate_gantt(...)`
- `"heatmap"` → `validate_schema_heatmap(table)?; generate_heatmap(...)`
- `"bullet"` → `validate_schema_bullet(table)?; generate_bullet(...)`
- `"lollipop"` → `validate_schema_lollipop(table)?; generate_lollipop(...)`

### CLI: `src/commands/chart.rs`

```rust
// Hooked into main.rs as Command::Chart { sub: ChartCommand }
pub enum ChartCommand {
    Check { kind: Option<String>, uri: String },
    Generate {
        kind: String,
        uri: Option<String>,
        x_min: Option<f64>, x_max: Option<f64>,
        y_min: Option<f64>, y_max: Option<f64>,
        x_label: Option<String>, y_label: Option<String>,
        width: Option<usize>, height: Option<usize>,
        fill_char: Option<char>, bins: Option<usize>,
        output: Option<PathBuf>,
    },
}
```

`cmd_chart_generate()` resolves the `md://` URI via `mdpath`, parses the source table, calls `generate_from_source()`, wraps the result in a fenced code block, and writes to stdout or `-o` file.

`cmd_chart_check()` resolves the URI, locates the chart code block in the file, calls `AsciiChartCheck::check()`, and reports diagnostics.

### main.rs wiring

Add to `Command` enum:
```rust
Chart {
    #[command(subcommand)]
    sub: ChartSubcommand,
}
```

Add `mdloom_directive_kind()` match arm: `"chart"` → `Some("chart")`.

### Cache key

When used in compile mode (Wave 5), cache key = `hash(source_uri + kind + width + height + x_min + x_max + y_min + y_max + fill_char + bins + shading + marker)`. No standalone cache in Wave 4 — Tier 2/3 cache integration deferred to compile pipeline.

### Tests (`src/chart/schema.rs`)

Target: 15+ unit tests.

| Test area | Count |
|-----------|-------|
| `SourceTable::parse` valid GFM table | 3 |
| `SourceTable::require_columns` missing column error message | 2 |
| `SourceTable::numeric_column` non-numeric value error | 2 |
| Per-kind schema validation: missing column errors for 3 representative kinds | 6 |
| `generate_from_source` smoke test for each category | 3 |

**Wave 4 exit criterion**: `mdloom chart generate --kind bar md://data.md#:table:0` resolves, parses, and generates a valid bar chart; schema errors produce human-readable messages naming the missing/wrong column; all 3 category kinds generate from their respective source schemas.

---

## Wave 5 — `mdloom:chart` compile directive

**Target file**: additions to `src/compile.rs`
**Estimated LOC**: ~180 (incremental — new enum variant + arm + format function)
**Depends on**: Waves 1–4

### Directive syntax (from spec)

````markdown
```mdloom:chart kind=bar width=40
md://data/benchmarks.md#results:table:0
```
````

### `compile.rs` changes

**1. Extend `Directive` enum:**

```rust
Directive::Chart {
    uri: String,
    attrs: ChartAttrs,
    line_start: usize,
    line_end: usize,
}
```

**2. `ChartAttrs` struct:**

```rust
#[derive(Debug, Default)]
struct ChartAttrs {
    kind: String,
    width: usize,           // default 60
    height: usize,          // default 20
    x_min: Option<f64>,
    x_max: Option<f64>,
    y_min: Option<f64>,
    y_max: Option<f64>,
    x_label: Option<String>,
    y_label: Option<String>,
    fill_char: Option<char>,
    bins: Option<usize>,
    shading: Option<String>,
    marker: Option<char>,
    interpolate: bool,
}

impl ChartAttrs {
    fn parse(after_directive: &str) -> Self    // same key=value loop as LayoutAttrs::parse
    fn to_graph_config(&self) -> GraphConfig
}
```

**3. `mdloom_directive_kind()` match arm:**

```rust
else if rest.starts_with("chart") { Some("chart") }
```

**4. `collect_directives()` branch:**

```rust
"chart" => {
    let attrs_str = info_after_backticks
        .strip_prefix("mdloom:chart").unwrap_or("").trim().to_string();
    let attrs = ChartAttrs::parse(&attrs_str);
    let uri = body.iter().find_map(|l| {
        let t = l.trim();
        if !t.is_empty() { Some(t.to_string()) } else { None }
    }).unwrap_or_default();
    if !uri.is_empty() {
        directives.push(Directive::Chart { uri, attrs, line_start, line_end });
    }
}
```

**5. `compile_file()` arm:**

```rust
Directive::Chart { uri, attrs, .. } => {
    match resolve_uri(uri, root) {
        Ok((content, _fig_file)) => {
            // Parse source table
            match SourceTable::parse(&content) {
                Ok(table) => {
                    let graph_cfg = attrs.to_graph_config();
                    match generate_from_source(&attrs.kind, &table, &graph_cfg) {
                        Ok(chart_text) => {
                            resolved_count += 1;
                            format_chart_block(uri, &attrs.kind, &chart_text)
                        }
                        Err(e) => {
                            // COMPILE-007: schema validation error
                            violations.push(CompileViolation {
                                code: "COMPILE-007",
                                severity: ViolationSeverity::Error,
                                uri: uri.clone(),
                                figure_id: None,
                                invariant: String::new(),
                                message: format!("chart schema error: {}", e),
                                source_line: line_start + 1,
                            });
                            source_lines[line_start..=line_end].join("\n")
                        }
                    }
                }
                Err(e) => {
                    violations.push(CompileViolation { code: "COMPILE-007", ... });
                    source_lines[line_start..=line_end].join("\n")
                }
            }
        }
        Err(e) => { /* COMPILE-002 as usual */ ... }
    }
}
```

**6. `format_chart_block()`:**

```rust
fn format_chart_block(uri: &str, kind: &str, chart_text: &str) -> String {
    format!(
        "<!-- mdloom:compiled from=\"mdloom:chart kind={}\" uris=\"{}\" -->\n```\n{}\n```\n<!-- /mdloom:compiled -->",
        kind, uri, chart_text
    )
}
```

### `layout_config_hash` extension

When computing the compile cache key, include all `ChartAttrs` fields in the hash alongside the source URI. This ensures re-generation when any attribute changes.

### Tests (`src/compile.rs`)

Target: 8+ new tests.

| Test area | Count |
|-----------|-------|
| `mdloom_directive_kind("```mdloom:chart kind=bar")` returns `Some("chart")` | 1 |
| `ChartAttrs::parse` with kind, width, x-min, x-max | 2 |
| `collect_directives` recognizes mdloom:chart block, extracts uri | 2 |
| `format_chart_block` traceability comment format | 1 |
| compile integration: Chart directive produces fenced output | 2 |

**Wave 5 exit criterion**: a `.source.md` file containing a `mdloom:chart kind=bar` directive with a valid `md://` source table compiles to a `.md` file with a fenced bar chart and correct traceability comment. Schema errors produce `COMPILE-007`. All existing compile tests pass.

---

## Inter-wave dependencies

```
Wave 1 (table.rs)
    ↓ bar generation used by schema.rs
Wave 2 (ticks.rs → graph.rs)
    ↓ graph generation used by schema.rs
Wave 3 (flow.rs)
    ↓ flow generation used by schema.rs
Wave 4 (schema.rs) — integrates Waves 1–3
    ↓ generate_from_source used by compile.rs
Wave 5 (compile.rs extension) — requires Wave 4
```

Waves 1–3 are independently buildable and testable. Wave 4 cannot start until the `generate_*` function signatures from Waves 1–3 are stabilized (interfaces can be finalized by end of Wave 2 for the table+graph kinds, then extended in Wave 3).

---

## LOC summary

| Wave | Primary files | Est. LOC |
|------|--------------|----------|
| 1 | `chart/table.rs`, `config.rs` (+), `checks/ascii_chart.rs` | 650 + 80 + 120 = 850 |
| 2 | `chart/ticks.rs`, `chart/graph.rs` | 120 + 900 = 1020 |
| 3 | `chart/flow.rs` | 480 |
| 4 | `chart/schema.rs`, `commands/chart.rs`, `main.rs` (+) | 420 + 220 + 60 = 700 |
| 5 | `compile.rs` (+) | 180 |
| **Total** | | **~3,230** |

---

## Test count summary

| Wave | Tests |
|------|-------|
| 1 | 30+ |
| 2 | 40+ |
| 3 | 30+ |
| 4 | 15+ |
| 5 | 8+ |
| **Total** | **123+** |

---

## Diagnostic codes (all waves)

| Code | Wave | Invariant | Meaning |
|------|------|-----------|---------|
| `ascii_chart_scale` | 1, 2 | C-1, C-2 | Bar/segment proportionality violation |
| `ascii_chart_stacked_sum` | 1 | C-10 | Stacked-bar segments don't sum correctly |
| `ascii_chart_shading` | 1 | C-4 | Heatmap cell uses wrong shading char |
| `ascii_chart_origin` | 2 | C-6 | 4-quadrant graph origin not at (0,0) |
| `ascii_chart_sort` | 3 | C-3 | Timeline events not chronological |
| `ascii_chart_waterfall_balance` | 3 | C-11 | Waterfall total ≠ start + sum(deltas) |
| `ascii_chart_sankey_balance` | 3 | C-13 | Sankey flow not conserved at node |
| `ascii_chart_kind` | 1–3 | — | Kind not declared; cannot validate |

Legacy codes preserved unchanged:

| Code | Checker |
|------|---------|
| `ascii_barchart_scale` | `src/checks/ascii_barchart.rs` |
| `ascii_barchart_char` | `src/checks/ascii_barchart.rs` |
| `ascii_barchart_pad` | `src/checks/ascii_barchart.rs` |
| `ascii_barchart_value` | `src/checks/ascii_barchart.rs` |
| `ascii_barchart_align` | `src/checks/ascii_barchart.rs` |
