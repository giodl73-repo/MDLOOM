# ELEMENT-IMPL-PLAN — proof:element + proof:row

> Prerequisite: MAPPING-SPEC partial impl (FieldMap, parse_md_table, parse_json_source) already in
> `src/tree/schema.rs`. Waves 2 and 3 consume these directly.

---

## Wave 1 — `src/element/` rendering module (450 LOC)

**Goal**: Pure rendering functions for all 6 element kinds. No I/O. No source resolution.
Every function takes pre-fetched data (a `&str` scalar or `&[f64]` series) and config,
returns a `String` of exactly `width` visual characters.

### New files

```
src/element/
  mod.rs        — ElementConfig, ElementOutput, render_element(), pub re-exports
  value.rs      — render_value(), render_delta()
  sparkline.rs  — render_sparkline(), bucket_series(), level_char()
  mini_bar.rs   — render_mini_bar()
```

### Key structs / functions

**`src/element/mod.rs`**

```rust
pub enum ElementKind { Value, Delta, Sparkline, MiniBar, Label, Badge }

pub struct ElementConfig {
    pub kind: ElementKind,
    pub width: usize,        // exact character budget; 0 = auto (caller must set before render)
    pub align: Align,        // re-use layout::Align (Left/Right/Center)
    pub format: String,      // Rust fmt spec, e.g. "{:.1}"
    pub no_chrome: bool,
    pub max: Option<f64>,    // mini-bar scale reference
    pub fill_char: char,     // default '█'
    pub empty_char: char,    // default '░'
}

pub fn render_element(kind: &ElementKind, data: &ElementData, cfg: &ElementConfig)
    -> Result<String, ElementError>

pub enum ElementData {
    Scalar(f64),
    Text(String),
    Series(Vec<f64>),
}
```

**`src/element/value.rs`**

```rust
pub fn render_value(val: f64, cfg: &ElementConfig) -> String
pub fn render_delta(val: f64, cfg: &ElementConfig) -> String
// Both: format → align_in_width() → pad/truncate to cfg.width
```

**`src/element/sparkline.rs`**

```rust
const SPARK_CHARS: [char; 8] = ['▁','▂','▃','▄','▅','▆','▇','█'];

pub fn render_sparkline(series: &[f64], cfg: &ElementConfig) -> String
// bucket_series() — if series.len() > width: mean-aggregate into width buckets
//                   if series.len() < width: repeat-fill to width
// level_char() — linear map [min, max] → SPARK_CHARS[0..7]
```

**`src/element/mini_bar.rs`**

```rust
pub fn render_mini_bar(val: f64, max: f64, cfg: &ElementConfig) -> String
// fill_count = round(val / max * width).clamp(0, width)
// String of fill_count fill_chars + (width - fill_count) empty_chars
```

**Shared helper in `mod.rs`**

```rust
fn align_in_width(s: &str, width: usize, align: Align) -> String
// Pad/truncate to exactly `width` visual chars.
// Truncate: trim Unicode-aware from right, append '…' only if truncating label/badge kinds.
// Padding: spaces on left (right), right (left), or split (center, tie-break: extra right).
// Uses layout::visual_width — do NOT reimplement.
```

### Invariant enforcement (in render_element)

| Invariant | Enforcement point |
|-----------|------------------|
| E-1 | `assert_eq!(visual_width(&out), cfg.width)` in debug; return `ElementError::WidthExceeded` in release |
| E-2 | Caller must pass `ElementData::Scalar` for value/delta; render rejects `Series` |
| E-3 | `level_char()` maps range [min,max] linearly to `SPARK_CHARS[0..7]`; tested exhaustively |
| E-4 | `render_mini_bar` fill proportion = `val / max`; off-by-one allowed (clamp) |
| E-5 | `no_chrome=true` path returns raw string — no fence or HTML |
| E-6 | `align_in_width` unit-tested for left/right/center with odd/even widths |

### Output modes

- `no_chrome=false` (default): caller in compile.rs wraps in `format_element_block(uri, inner)` — same pattern as `format_include_block`.
- `no_chrome=true`: `render_element` returns the raw string; compile.rs emits it directly as the replacement text.

### Tests (25+)

| # | What |
|---|------|
| 1–2 | `render_value` — integer format, 1-decimal format |
| 3–4 | `render_value` align=right, align=center |
| 5–6 | `render_delta` — positive sign, negative U+2212 |
| 7 | `render_delta` — right-align in width=8 |
| 8 | `render_sparkline` — 5-value series into width=5: min→▁, max→█ |
| 9 | `render_sparkline` — series longer than width: bucket aggregation |
| 10 | `render_sparkline` — series shorter than width: repeat-fill |
| 11 | `render_sparkline` — single-value series (all same → all ▁) |
| 12 | `render_mini_bar` — 50% fill: half █, half ░ |
| 13 | `render_mini_bar` — 100% fill: all █ |
| 14 | `render_mini_bar` — 0% fill: all ░ |
| 15 | `render_mini_bar` — val > max (clamp to full) |
| 16 | `render_label` — short string, left-aligned, padded |
| 17 | `render_label` — string longer than width, truncated with … |
| 18 | `render_badge` — short enum value, right-padded |
| 19 | `align_in_width` — center, even width, tie-break extra space right |
| 20 | `align_in_width` — center, odd width |
| 21 | `align_in_width` — right align |
| 22 | E-1: output width exactly equals cfg.width for every kind |
| 23 | E-3: sparkline min value always maps to ▁ char |
| 24 | E-3: sparkline max value always maps to █ char |
| 25 | `render_element` rejects `ElementData::Series` for kind=Value (E-2) |
| 26 | `render_sparkline` width=1: single char |
| 27 | visual_width of sparkline output = cfg.width (block chars measured at 1) |

**Exit criterion**: all 27 tests pass. No I/O, no mdpath dependency.
`cargo test -p proof-lib element::` is clean.

---

## Wave 2 — `proof:element` compile directive (300 LOC)

**Goal**: `proof:element` fenced blocks in `.source.md` files are recognized, source-resolved,
field-extracted, rendered via Wave 1, and replaced with output (fenced or raw per `no-chrome`).

### Changes to existing files

**`src/compile.rs`**

Add `Directive::Element` to the enum:

```rust
Element {
    kind: String,        // "value" | "delta" | "sparkline" | "mini-bar" | "label" | "badge"
    source: Option<String>,  // md:// URI (absent if value="literal" inline)
    field: Option<String>,
    inline_value: Option<String>,  // from value="..." attribute
    attrs: ElementAttrs,
    line_start: usize,
    line_end: usize,
}
```

Add `ElementAttrs` struct (parallel to `TreeAttrs`):

```rust
struct ElementAttrs {
    width: Option<usize>,
    align: String,         // "left" | "right" | "center"
    format: String,        // "{:.1}" etc.
    no_chrome: bool,
    max: Option<f64>,
    fill: char,
    empty: char,
}
```

Extend `proof_directive_kind()`:

```rust
else if rest.starts_with("element") { Some("element") }
```

Extend `collect_directives()` — new `"element"` arm:
- Parse info string: extract `kind=`, `field=`, `width=`, `align=`, `format=`, `no-chrome`, `max=`, `value=`
- First `md://` line in body = source URI

Extend `compile_file()` — new `Directive::Element` arm:
- If `inline_value` is set: skip source resolution, use literal
- Else: `resolve_uri(source, root)` → content string
- Parse source via `parse_md_table` or `parse_json_source` (detect from URI extension)
- Extract field column — emit `ELEMENT-005` if missing
- Coerce to `ElementData` — emit `ELEMENT-002` if value kind gets non-scalar
- Call `element::render_element()` — emit `ELEMENT-001` if width exceeded
- Wrap output: `no_chrome=true` → raw string; else `format_element_block(uri, rendered)`

New formatting helper:

```rust
fn format_element_block(uri: &str, rendered: &str) -> String {
    format!(
        "<!-- proof:compiled from=\"proof:element\" uri=\"{}\" -->\n```\n{}\n```\n<!-- /proof:compiled -->",
        uri, rendered
    )
}
```

Diagnostic codes emitted from compile arm:

| Code | Trigger |
|------|---------|
| ELEMENT-001 | `render_element` returns `WidthExceeded` |
| ELEMENT-002 | Field resolves to a list, not a scalar (for value/delta) |
| ELEMENT-003 | Sparkline series shorter than width (warning, render still proceeds) |
| ELEMENT-005 | `field=X` not found in source table headers |

### Tests (12)

| # | What |
|---|------|
| 1 | `collect_directives` recognizes `proof:element` kind=value |
| 2 | `collect_directives` recognizes `proof:element` kind=sparkline with no-chrome |
| 3 | `ElementAttrs::parse` — all keys parsed correctly |
| 4 | `ElementAttrs::parse` — `no-chrome` flag (no `=`) |
| 5 | `ElementAttrs::parse` — defaults |
| 6 | `proof_directive_kind` returns "element" for `\`\`\`proof:element` |
| 7 | E2E: `kind=value inline_value="42" width=4` → compiled `" 42 "` (no mdpath) |
| 8 | E2E: `kind=label inline_value="McDavid" width=8 align=left` → `"McDavid "` |
| 9 | E2E: `kind=badge inline_value="UFA" width=5` → `"UFA  "` |
| 10 | E2E: `no-chrome=true` → output has no fence, no HTML comment |
| 11 | E2E: `no-chrome=false` → output wrapped in `<!-- proof:compiled -->` fence |
| 12 | E2E: `field=X` with `X` absent from table → ELEMENT-005 violation emitted |

**Exit criterion**: `proof compile test.source.md` with a `proof:element kind=value inline_value=...`
block produces a compiled `.md` with the rendered value inline. `proof check` on a `proof:element`
where rendered output would exceed `width` emits ELEMENT-001.

---

## Wave 3 — `proof:row` compositor (350 LOC)

**Goal**: `proof:row foreach=X in md://... separator=" "` iterates source table rows and emits
one output line per row, with child `proof:element` blocks rendered as fixed-width columns.

### New file

**`src/element/row.rs`**

```rust
pub struct RowConfig {
    pub source_uri: String,
    pub var_name: String,    // "player" in foreach=player in md://...
    pub separator: String,   // default " "
    pub elements: Vec<RowElement>,
}

pub struct RowElement {
    pub kind: ElementKind,
    pub field: String,
    pub attrs: ElementAttrs,
}

pub fn render_row(
    row_data: &HashMap<String, String>,  // one source row
    row_cfg: &RowConfig,
    root: &Path,
) -> Result<String>
// Renders one line: join render_element() calls with separator.
// Validates R-1: sum of element widths + separators × (n-1) = declared width.

pub fn render_row_foreach(
    source_rows: &[HashMap<String, String>],
    row_cfg: &RowConfig,
    root: &Path,
) -> Result<Vec<String>>
// Returns one rendered String per source row.
```

R-1 validation:

```rust
fn validate_r1(elements: &[RowElement], separator_len: usize, declared_width: Option<usize>)
    -> Option<ElementError>
// Computes: sum(e.attrs.width) + separator_len * (n-1)
// If declared_width is Some and sum != declared_width → ElementError::RowWidthMismatch
```

### Changes to existing files

**`src/element/mod.rs`** — add `pub mod row;`

**`src/compile.rs`**

Add `Directive::Row` to enum:

```rust
Row {
    source_uri: String,
    var_name: String,
    separator: String,
    declared_width: Option<usize>,
    elements: Vec<RowElement>,  // parsed from body lines
    no_chrome: bool,
    line_start: usize,
    line_end: usize,
}
```

Extend `proof_directive_kind()`:

```rust
else if rest.starts_with("row") { Some("row") }
```

Extend `collect_directives()` — new `"row"` arm:
- Parse `foreach=VAR in URI` from info string after `proof:row`
- Parse `separator=`, `width=`, `no-chrome` from remaining attrs
- Parse body lines: each line starting with `proof:element` becomes a `RowElement`

Extend `compile_file()` — new `Directive::Row` arm:
- Resolve source URI via `resolve_uri`
- Parse table → `Vec<HashMap<String,String>>`
- Emit ELEMENT-007 (MAPPING-007) if 0 rows
- Call `row::render_row_foreach()` → Vec of lines
- Validate R-1 → emit ELEMENT-004 if mismatch
- Join lines with `\n`, wrap in element block (or raw if no-chrome)

### Tests (15)

| # | What |
|---|------|
| 1 | `render_row` with 3 elements → output length = sum(widths) + 2 separators |
| 2 | `render_row` — label + value + mini-bar: each column at correct offset |
| 3 | `validate_r1` — correct sum, no error |
| 4 | `validate_r1` — sum exceeds declared_width → error |
| 5 | `validate_r1` — sum less than declared_width → error |
| 6 | `render_row_foreach` — 3-row source → 3 output lines |
| 7 | `render_row_foreach` — field not in row → ELEMENT-005 propagated |
| 8 | `collect_directives` — `proof:row foreach=p in md://x` → Directive::Row parsed |
| 9 | `collect_directives` — body `proof:element` lines parsed as RowElements |
| 10 | `collect_directives` — `separator=" "` default |
| 11 | `collect_directives` — `separator=","` explicit |
| 12 | E2E compile: `proof:row foreach=p in ...` with 2 source rows → 2 output lines |
| 13 | E2E compile: R-1 violation in source.md → ELEMENT-004 emitted |
| 14 | Column pinning: element N always starts at sum of widths 1..N-1 + separators |
| 15 | `render_row` no-chrome: output has no fence |

**Exit criterion**: a `.source.md` with a `proof:row foreach=player in md://stats.md#edm:table:0`
directive compiles to a block with one row line per player. R-1 violation in the directive
(element widths don't sum to declared width) emits ELEMENT-004 and blocks compilation.

---

## Module structure summary

```
src/element/
  mod.rs        — ElementKind, ElementConfig, ElementData, ElementError, render_element()
  value.rs      — render_value(), render_delta()
  sparkline.rs  — render_sparkline(), bucket_series(), level_char()
  mini_bar.rs   — render_mini_bar()
  row.rs        — RowConfig, RowElement, render_row(), render_row_foreach(), validate_r1()
```

**Total estimated LOC**: ~450 (Wave 1) + ~300 (Wave 2) + ~350 (Wave 3) = **~1,100 LOC** excluding tests.

---

## Cross-cutting notes

- `visual_width` from `src/layout.rs` is the canonical width measurer — import it, don't copy it.
  Block chars (sparkline) and box-drawing (mini-bar fill) are already measured at 1 column.
- `parse_md_table` and `parse_json_source` from `src/tree/schema.rs` are the source parsers.
  Import them into element wave 2/3 — no new parser needed.
- `ElementAttrs::parse()` follows the same token-by-token pattern as `TreeAttrs::parse()` and
  `LayoutAttrs::parse()`. Keep the three implementations consistent.
- `no-chrome` stripping is purely at the compile.rs wrapping layer — `render_element()` itself
  always returns raw chars; it never adds fences.
- `Directive` enum `line_start()`/`line_end()` pattern: add Element and Row arms to both methods.
