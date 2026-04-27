# SLIDE-IMPL-PLAN — proof:slide ASCII presentation composer

> Wave 1 (parser) is independent of the dashboard canvas.

---

## Wave 1 — Slide parser and canvas (~350 LOC)

**Goal**: Parse `.slides.source.md` files into a `Vec<Slide>`. Implement front-matter
disambiguation, slide separation, and per-slide canvas allocation. No rendering — pure
data extraction and validation.

### New files

```
src/slide/
  mod.rs       — pub re-exports, SlideDoc, Slide, SlideMeta, compile_slides()
  parser.rs    — parse_slide_doc(), front_matter parsing, slide separation
  canvas.rs    — Lightweight canvas: Vec<char>, paste(region, lines), render() → String
```

### Key structs / functions

**`src/slide/mod.rs`**

```rust
pub struct SlideMeta {
    pub width: usize,          // default: 120
    pub height: usize,         // default: 34
    pub theme: SlideTheme,     // minimal | box | none
    pub show_numbers: bool,
    pub font_width: usize,     // 1 = ASCII, 2 = wide-char
    pub max_bullets: usize,    // default: 6
    pub max_depth: usize,      // default: 4
}

pub enum SlideTheme { Minimal, Box, None }

pub struct Slide {
    pub index: usize,          // 1-indexed
    pub layout: SlideLayout,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub body_content: String,  // raw body lines (after front-matter attrs)
    pub notes_content: String, // raw proof:notes block content (may be empty)
    pub source_line: usize,    // line in source where this slide begins
}

pub enum SlideLayout {
    Title,
    TitleContent,
    TwoColumn { ratio: (u8, u8) },
    Section,
    ContentCaption,
    Comparison,
    Stats,
    Blank,
}

pub struct SlideDoc {
    pub meta: SlideMeta,
    pub slides: Vec<Slide>,
}
```

**`src/slide/parser.rs`**

```rust
/// Parse a .slides.source.md source string into SlideDoc.
/// Disambiguation rule: if line 0 == "---", enter YAML front-matter mode;
/// next "---" is the closer (not a slide separator). All subsequent "---"
/// on their own line are slide separators.
pub fn parse_slide_doc(source: &str) -> Result<SlideDoc, Vec<SlideError>>

pub enum SlideError {
    MalformedFrontMatter(String),
    InvalidRatio { slide: usize, raw: String },  // SLIDE-002
    SlideOutOfRange { requested: usize, count: usize }, // SLIDE-006
    UnknownLayout { slide: usize, name: String },
}

/// Parse YAML front-matter block (hand-parsed, no serde_yaml).
/// Recognizes: width, height, theme, show-numbers, font-width, max-bullets, max-depth.
fn parse_front_matter(block: &str) -> Result<SlideMeta, String>

/// Split body text into slides at bare "---" lines.
/// Returns Vec of raw slide strings (one per slide).
fn split_slides(body: &str) -> Vec<(usize, String)>  // (source_line, content)

/// Parse a single slide block: extract ```proof:slide ...``` fence attrs and body.
fn parse_slide(raw: &str, index: usize, source_line: usize) -> Result<Slide, SlideError>

/// Parse layout= and ratio= from proof:slide info string.
fn parse_slide_attrs(info: &str) -> (SlideLayout, Option<String>, Option<String>)

/// Extract proof:notes block content from slide body; return (body_without_notes, notes).
fn extract_notes(body: &str) -> (String, String)
```

**`src/slide/mod.rs`** — `derive_output_path` extension:

```rust
// In compile.rs derive_output_path — checked before .source.md suffix:
if let Some(stem) = name.strip_suffix(".slides.source.md") {
    let out_name = format!("{}.slides.md", stem);
    Some(source.parent().unwrap_or(Path::new(".")).join(out_name))
}
```

**SlideCanvas** — implemented in `src/slide/canvas.rs` (Wave 1). This eliminates the
cross-plan dependency on DASHBOARD-IMPL-PLAN. The slide canvas is simpler than the dashboard
canvas and does not need to wait. `src/slide/mod.rs` and `layout.rs` import from
`slide::canvas::SlideCanvas`, not from `dashboard::canvas::Canvas`. Each slide gets its own
`SlideCanvas::new(meta.width, meta.height)`.

> **Cross-plan dependency resolved:** Wave 2 originally required DASHBOARD-IMPL-PLAN Wave 1
> to provide a Canvas. By implementing `src/slide/canvas.rs` in Wave 1, slide development is
> fully self-contained. If the dashboard Canvas is later unified, `SlideCanvas` can be replaced
> or aliased at that time.

### Tests (12)

| # | What |
|---|------|
| 1 | File with YAML front-matter: `---\nslides:\n  width: 80\n---` parsed, width=80 |
| 2 | Front-matter closer `---` is NOT counted as a slide separator (SL-7) |
| 3 | File without front-matter: first `---` is slide 2's separator, slide count correct |
| 4 | 3 slides separated by 2 `---` → `slides.len() == 3` (SL-7) |
| 5 | `layout=title` parsed to `SlideLayout::Title` |
| 6 | `layout=two-column ratio=60:40` parsed to `SlideLayout::TwoColumn { ratio: (60, 40) }` |
| 7 | `ratio=60:50` → `SlideError::InvalidRatio` (SLIDE-002: parts don't sum to 100) |
| 8 | Slide title from info string: `title="Foo Bar"` → `slide.title == Some("Foo Bar")` |
| 9 | `proof:notes` block extracted to `notes_content`; absent from `body_content` |
| 10 | `body_content` for title layout: subtitle, author, date parsed from YAML attrs |
| 11 | Empty file → SlideDoc with 1 empty slide, default SlideMeta |
| 12 | `parse_slide_doc` on 5-slide file → `slides.len() == 5`, indices 1..=5 |

**Exit criterion**: `cargo test -p proof-lib slide::parser::` is clean. `parse_slide_doc` on the
example from SLIDE-SPEC.md produces 5 slides with correct layouts and titles.

---

## Wave 2 — Layout renderers (~500 LOC)

> **Pre-condition:** `src/slide/canvas.rs` (Wave 1) must be complete. Wave 2 uses
> `SlideCanvas` from that file — no dependency on DASHBOARD-IMPL-PLAN. The cross-plan
> dependency is eliminated by implementing the canvas in Wave 1 (Option 1, chosen).

> **LOC note:** Estimate raised from ~400 to ~500 to account for `render_body_lines` stub
> (~20 LOC), full `apply_theme` with box-style border + `├──┤` mid-divider (~40 LOC), and
> `## col:` suppression in heading checks.

**Goal**: Render each `SlideLayout` variant to a populated `SlideCanvas`. Each renderer takes
a `&Slide`, a `&SlideMeta`, and a mutable `&mut SlideCanvas`. Title-bar sizing and body region
coordinates are layout-specific constants, not caller-configurable.

### New file

**`src/slide/layout.rs`**

```rust
/// Entry point: render a Slide onto a fresh Canvas using its declared layout.
pub fn render_slide(
    slide: &Slide,
    meta: &SlideMeta,
    violations: &mut Vec<CompileViolation>,
) -> Canvas

// ── Layout implementations ─────────────────────────────────

/// title: vertically + horizontally center title, subtitle, author, date.
/// No title bar. Centering is compositor-driven (not proof:centered).
fn render_title(slide: &Slide, canvas: &mut Canvas, meta: &SlideMeta)

/// title-content: 3-row title bar (row 0..2) + separator + body fills rest.
/// Title bar: left-padded by 1 space. Body region: rows 3..height.
fn render_title_content(
    slide: &Slide,
    canvas: &mut Canvas,
    meta: &SlideMeta,
    violations: &mut Vec<CompileViolation>,
)

/// two-column: 3-row title bar, body split into two columns.
/// Rounding: floor(width * left_ratio). Remainder to first (left) column.
/// Divider: │ drawn at x = left_width on rows title_bar_height..height.
fn render_two_column(
    slide: &Slide,
    canvas: &mut Canvas,
    meta: &SlideMeta,
    ratio: (u8, u8),
    violations: &mut Vec<CompileViolation>,
)

/// section: compositor-driven. Centers title (larger) and subtitle (smaller) vertically
/// and horizontally. No proof:centered required from author.
fn render_section(slide: &Slide, canvas: &mut Canvas, meta: &SlideMeta)

/// stats: dedicated renderer. No proof:columns. No ratio= or divider=.
/// Column width = floor(content_width / stat_count); remainder to rightmost.
/// Each stat block independently centered within its allocated column width.
fn render_stats(
    slide: &Slide,
    canvas: &mut Canvas,
    meta: &SlideMeta,
    violations: &mut Vec<CompileViolation>,
)

/// blank: no structure. Body lines pasted starting at (0, 0). All positioning
/// via proof: directives in the body.
fn render_blank(
    slide: &Slide,
    canvas: &mut Canvas,
    meta: &SlideMeta,
    violations: &mut Vec<CompileViolation>,
)
```

**Centering helpers** (used by `render_title` and `render_section`):

```rust
/// Center a line horizontally in `width`. Tie-break: extra space on right (SL-6).
fn center_line(line: &str, width: usize) -> String

/// Return the y offset to center `content_lines` vertically in `height`.
fn vertical_offset(content_lines: usize, height: usize) -> usize
```

**Theme application** — called after layout rendering, before serialization:

```rust
/// Apply theme chrome to canvas. `minimal`: draw ─── separator after title bar.
/// `box`: draw ┌──┐/│/└──┘ border; title bar delimited by ├──┤.
/// `none`: no-op.
pub fn apply_theme(canvas: &mut SlideCanvas, meta: &SlideMeta, title_bar_rows: usize)
```

**Wave 2 stub — `render_body_lines`**: This function is not complete in Wave 2 but is needed
by `render_title_content`, `render_two_column`, and `render_blank`. Add a stub that returns
the input lines unchanged:

```rust
pub(crate) fn render_body_lines(lines: &[&str], _width: usize) -> Vec<String> {
    lines.iter().map(|l| l.to_string()).collect()
}
```

Wave 3 replaces this stub with the full directive-dispatching implementation.

**Content overflow** — if body rendering exceeds `height - title_bar_rows` rows, emit
`SLIDE-003` (warning) and clip. Canvas `paste` handles clipping silently; the violation
is emitted by the layout renderer before calling `paste`.

**Ratio rounding** — `two_column` and `proof:columns`:

// The correct formula: `floor(content_width × ratio_a / (ratio_a + ratio_b))` for
// column A, remainder to column B.
// Spec example: 119 wide, 60:40, no divider → floor(71.4)=71, floor(47.6)=47, sum=118,
// remainder 1 → left gets 72, right stays 47.

```rust
fn split_ratio(total_width: usize, left_pct: u8, divider: bool) -> (usize, usize) {
    let usable = if divider { total_width - 1 } else { total_width };
    let left_floor = (usable as f64 * left_pct as f64 / 100.0).floor() as usize;
    let right_floor = (usable as f64 * (100 - left_pct) as f64 / 100.0).floor() as usize;
    let remainder = usable - left_floor - right_floor;
    (left_floor + remainder, right_floor)  // remainder to left (first) column
}
```

### Tests (14)

| # | What |
|---|------|
| 1 | `render_title` at 80×24: title centered horizontally (SL-6 tie-break) |
| 2 | `render_title` at 80×24: title+subtitle+author vertically centered |
| 3 | `render_title_content` at 80×24: row 0..2 contains title, row 3 is separator |
| 4 | `render_title_content` at 80×24: body starts at row 3, canvas is 24 rows |
| 5 | `render_section` at 80×24: title and subtitle vertically centered, no title bar |
| 6 | `render_two_column` 50:50 at 80 wide: each col is 40 chars |
| 7 | `render_two_column` 60:40 at 119 wide: left=72, right=47 (spec example) |
| 8 | `render_two_column` with divider=true at 80: left + 1 (divider) + right = 80 |
| 9 | `render_stats` 3 stats at 78 content width: widths are 26, 26, 26 |
| 10 | `render_stats` 3 stats at 79 content width: floor(79/3)=26; remainder 1 to rightmost → 26, 26, 27 |
| 11 | `render_blank`: body lines pasted from row 0 |
| 12 | Content overflow: SLIDE-003 emitted when body exceeds available rows |
| 13 | `apply_theme minimal`: separator line drawn at row title_bar_rows |
| 14 | `apply_theme box`: border chars present at canvas edges |

**Exit criterion**: `render_slide` on a `title-content` slide at 80×24 produces a `Canvas`
whose `render()` is exactly 24 lines of 80 chars each (SL-1). All 6 layout types render
without panic.

---

## Wave 3 — Slide-specific directives (~350 LOC)

**Goal**: Implement the directive renderers that appear inside slide body blocks:
`proof:bullets`, `proof:columns`, `proof:quote`, `proof:centered`, `proof:stat`,
`proof:callout`, `proof:divider`. Also handle `proof:notes` extraction and linting contract.

### New files

**`src/slide/bullets.rs`**

```rust
pub struct BulletConfig {
    pub max_depth: usize,     // from SlideMeta
    pub max_bullets: usize,   // from SlideMeta
    pub chars: [char; 4],     // ['•', '◦', '▸', '–']
    pub indent_per_level: usize, // default: 2
}

/// Render a proof:bullets block body to lines.
/// Input: raw lines after `proof:bullets` header (inside the fence).
/// Output: Vec<String> of rendered bullet lines.
/// Emits SLIDE-001 if bullet count > max_bullets.
/// Emits SLIDE-007 if depth > max_depth (renders at max_depth char).
pub fn render_bullets(
    body: &str,
    config: &BulletConfig,
    available_width: usize,
    violations: &mut Vec<CompileViolation>,
) -> Vec<String>

/// Parse indentation level from a bullet line (2 spaces per level).
fn parse_bullet_level(line: &str) -> (usize, &str)  // (level, text)
```

**`src/slide/columns.rs`**

```rust
pub struct ColumnsConfig {
    pub cols: usize,
    pub ratio: Vec<u8>,    // percentages summing to 100; equal split if empty
    pub divider: bool,
}

/// Parse ## col: sections from a proof:columns or two-column body.
/// Returns Vec of (col_index, content_lines).
/// Emits SLIDE-004 if two-column body has < 2 ## col: sections.
pub fn parse_col_sections(body: &str) -> Vec<(usize, Vec<String>)>

/// Render N column bodies side-by-side into a flat Vec<String>.
/// Column widths computed via split_ratio (per-pair for N>2, left-to-right).
/// Divider: │ inserted between adjacent columns if divider=true.
/// SL-3: column widths sum to content_width (minus divider chars).
pub fn render_columns(
    col_bodies: Vec<Vec<String>>,
    total_width: usize,
    config: &ColumnsConfig,
) -> Vec<String>
```

**`src/slide/inline.rs`**

```rust
/// proof:quote — centered block quote with curly-quote attribution.
/// Output is centered within available_width.
pub fn render_quote(body: &str, attribution: Option<&str>, available_width: usize) -> Vec<String>

/// proof:centered — horizontally center each non-empty line.
/// SL-6: tie-break extra space on right.
pub fn render_centered(body: &str, available_width: usize) -> Vec<String>

/// proof:stat — large value with label and optional sublabel.
/// SL-4: value right-aligned within width.
/// Emits SLIDE-005 warning if value is non-numeric.
pub fn render_stat(
    value: &str,
    label: Option<&str>,
    sublabel: Option<&str>,
    width: usize,
    violations: &mut Vec<CompileViolation>,
) -> Vec<String>

/// proof:callout — boxed or prefixed callout.
/// Styles: key=★, info=ℹ, warning=⚠, tip=→, note=◆
pub fn render_callout(body: &str, style: &str, available_width: usize) -> Vec<String>

/// proof:divider — horizontal rule.
/// Styles: thin=─, double=═, dotted=·, wave=~, approx=≈
pub fn render_divider(style: &str, width: usize) -> String
```

**proof:notes behavior**:

- `extract_notes` (Wave 1 parser) strips `proof:notes` block content from `body_content`
  into `slide.notes_content` before layout rendering. Notes are never present in the slide
  canvas.
- When `proof check` runs on a `.slides.source.md` file, notes content is passed through the
  full check pipeline. The check runner treats the notes block as inline body content. To
  suppress: `proof check --no-notes` (planned, not implemented in this plan).
- When compiling with `--format notes`, `compile_slides` emits `slide.notes_content` instead
  of the canvas render.
- SL-5 is enforced at the canvas level: `render_slide` never reads `notes_content`.

**Body directive dispatch** — inside `render_title_content`, `render_two_column`, and
`render_blank`, the body string is scanned for inline `proof:*` directives using a
`render_body_lines` function:

```rust
/// Render body content string into Vec<String>, dispatching proof: directives.
/// Directives are recognized as bare lines (not fenced) inside the slide body fence.
/// Returns rendered lines for pasting into the canvas body region.
fn render_body_lines(
    body: &str,
    available_width: usize,
    meta: &SlideMeta,
    violations: &mut Vec<CompileViolation>,
) -> Vec<String>
```

Directive dispatch order inside body: `proof:bullets`, `proof:columns`, `proof:quote`,
`proof:centered`, `proof:stat`, `proof:callout`, `proof:divider`. Lines not matching any
directive prefix are emitted as-is (plain text rendering).

`proof:chart`, `proof:tree`, and `proof:element` inside slide body are passed through
unchanged to the existing compile pipeline (their output is already plain text fenced blocks;
inside a slide body they render as text lines).

### Tests (15)

| # | What |
|---|------|
| 1 | `render_bullets`: 3 levels → •, ◦, ▸ chars |
| 2 | `render_bullets`: level > max_depth renders at max_depth char; SLIDE-007 emitted |
| 3 | `render_bullets`: > max_bullets items → SLIDE-001 warning emitted |
| 4 | `render_bullets`: 2-space indent per level preserved in output width |
| 5 | `parse_col_sections`: 2 `## col:` sections in body → 2 entries |
| 6 | `render_columns` 50:50 at 80 wide: each column's content fits in 40 chars |
| 7 | `render_columns` 60:40 with divider at 80: `│` at column 48 (floor(80*0.60)) |
| 8 | `render_columns` col widths sum to total_width minus divider count (SL-3) |
| 9 | `render_quote`: output contains `"` and `"`, attribution on `—` line |
| 10 | `render_centered`: single line centered, extra space on right (SL-6) |
| 11 | `render_stat`: value right-aligned within width (SL-4) |
| 12 | `render_stat`: non-numeric value → SLIDE-005 warning |
| 13 | `render_callout style=warning`: output prefixed with ⚠ |
| 14 | `render_divider style=double` at width=40 → 40 `═` chars |
| 15 | Notes excluded from canvas: `slide.notes_content` non-empty, canvas body empty of notes text (SL-5) |

**Exit criterion**: `cargo test -p proof-lib slide::bullets slide::columns slide::inline` is
clean. A slide body containing `proof:bullets` (3 levels) + `proof:divider` renders without
panic; output lines have correct bullet chars and divider.

---

## Wave 4 — CLI + compile directive (~200 LOC)

**Goal**: Wire the slide pipeline into `compile.rs` and `main.rs`. Implement the
`proof:slide` directive handler, `--slide N`, `--format`, `--theme` CLI flags, and
SLIDE-* diagnostic codes. Handle `.slides.source.md` path routing.

### Changes to existing files

**`src/compile.rs`** — add `Directive::Slide` variant:

```rust
Slide {
    layout: String,
    attrs_raw: String,   // full info string after "proof:slide"
    body_lines: Vec<String>,
    line_start: usize,
    line_end: usize,
}
```

Extend `proof_directive_kind`:

```rust
else if rest.starts_with("slide") { Some("slide") }
```

Extend `collect_directives` — new `"slide"` arm: extract `layout=`, `title=`, `ratio=`
from info string; collect body lines until closing ` ``` `.

Extend `compile_file` — detect `.slides.source.md`:

```rust
if source_path ends with ".slides.source.md" {
    return compile_slide_doc(source_text, meta, flags, violations);
}
```

Add `compile_slide_doc` in `src/slide/mod.rs`:

```rust
pub fn compile_slide_doc(
    source_text: &str,
    source_path: &Path,
    slide_filter: Option<usize>,       // --slide N (1-indexed)
    width_override: Option<usize>,
    height_override: Option<usize>,
    format: SlideFormat,               // compiled | notes | json
    theme_override: Option<SlideTheme>,
    no_chrome: bool,
) -> Result<SlideCompileResult>

pub enum SlideFormat { Compiled, Notes, Json }

pub struct SlideCompileResult {
    pub output: String,
    pub violations: Vec<CompileViolation>,
    pub slides_rendered: usize,
}
```

Implementation:
1. `parse_slide_doc(source_text)` → `SlideDoc`; apply overrides to `meta`.
2. Validate `slide_filter` → emit SLIDE-006 if N > slide count.
3. For each slide (or the one slide if `slide_filter` set):
   - `render_slide(slide, meta, violations)` → `Canvas`
   - `apply_theme(canvas, meta, title_bar_rows)`
   - Match `format`: `Compiled` → `canvas.render()`, `Notes` → `slide.notes_content`,
     `Json` → JSON object.
4. Assemble output:
   - `Compiled` all slides: join with `SLIDE N ──── N/total` headers and `\n` between.
   - Wrap in `<!-- proof:compiled from="proof:slides" count=N -->` ... `<!-- /proof:compiled -->`
     (unless `no_chrome`).

**`src/main.rs`** — extend `Command::Compile`:

```rust
/// Render only slide N (1-indexed)
#[arg(long)]
slide: Option<usize>,

/// Output format: compiled (default) | notes | json
#[arg(long, default_value = "compiled")]
format: String,

/// Theme override: minimal | box | none
#[arg(long)]
theme: Option<String>,
```

Pass `slide`, `format`, `theme` through `cmd_compile` → `compile_file` →
`compile_slide_doc`.

**`src/compile.rs`** — `derive_output_path` — check `.slides.source.md` before `.source.md`:

```rust
pub fn derive_output_path(source: &Path) -> Option<PathBuf> {
    let name = source.file_name()?.to_str()?;
    if let Some(stem) = name.strip_suffix(".slides.source.md") {
        return Some(source.parent().unwrap_or(Path::new(".")).join(format!("{}.slides.md", stem)));
    }
    if let Some(stem) = name.strip_suffix(".dashboard.source.md") {
        return Some(source.parent().unwrap_or(Path::new(".")).join(format!("{}.dashboard.md", stem)));
    }
    if let Some(stem) = name.strip_suffix(".source.md") {
        return Some(source.parent().unwrap_or(Path::new(".")).join(format!("{}.md", stem)));
    }
    None
}
```

**SLIDE-* diagnostic codes** — emitted as `CompileViolation` entries using the existing
`ViolationSeverity` enum (no new types):

| Code | Severity | Trigger site |
|------|----------|-------------|
| `SLIDE-001` | Warning | `render_bullets`: bullet count > max_bullets |
| `SLIDE-002` | Error | `parse_slide_doc`: ratio parts don't sum to 100 |
| `SLIDE-003` | Warning | layout renderers: body overflows available height |
| `SLIDE-004` | Error | `parse_col_sections`: two-column has < 2 `## col:` sections |
| `SLIDE-005` | Warning | `render_stat`: value is non-numeric |
| `SLIDE-006` | Error | `compile_slide_doc`: `--slide N` > slide count |
| `SLIDE-007` | Warning | `render_bullets`: depth > max_depth |

**Integration with existing directives** — `proof:chart`, `proof:tree`, and `proof:element`
inside a slide body fence are passed through `collect_directives` and compiled by their
existing arms before the body lines are assembled for `render_body_lines`. The slide body
is a synthetic `.source.md` string; the existing compile pipeline handles nested directives.

### Tests (12)

| # | What |
|---|------|
| 1 | `derive_output_path("deck.slides.source.md")` → `"deck.slides.md"` |
| 2 | `derive_output_path("deck.slides.source.md")` does NOT match `.source.md` first |
| 3 | `derive_output_path("doc.source.md")` → `"doc.md"` (existing behavior) |
| 4 | `compile_slide_doc` on 3-slide doc → output contains 3 SLIDE N headers |
| 5 | `--slide 2` on 3-slide doc → only slide 2 rendered, no SLIDE 1/3 headers |
| 6 | `--slide 5` on 3-slide doc → SLIDE-006 error emitted |
| 7 | `--format notes` → `slide.notes_content` emitted, no canvas content (SL-5) |
| 8 | `--format notes` with no notes → empty output per slide, no panic |
| 9 | `--theme box` overrides front-matter theme; border chars present in output |
| 10 | SLIDE-002 emitted for `ratio=60:50` — error, compile halts |
| 11 | SLIDE-004 emitted for two-column slide with one `## col:` section |
| 12 | `proof compile edm-preview.slides.source.md --slide 2` exits 0, output is 34 lines of 120 chars (SL-1) |

**Exit criterion**: `proof compile edm-preview.slides.source.md --slide 2` renders a valid
`title-content` slide with `proof:bullets` and a chart. Output is exactly
`meta.width × meta.height` characters per line. All SLIDE-* codes are reachable via test
inputs.

---

## Module structure summary

```
src/slide/
  mod.rs        — SlideDoc, Slide, SlideMeta, SlideLayout, SlideTheme,
                  compile_slide_doc(), SlideCompileResult, SlideFormat
  canvas.rs     — SlideCanvas: Vec<char>, new(width, height), paste(region, lines),
                  render() → String  [Wave 1 — eliminates dashboard cross-dependency]
  parser.rs     — parse_slide_doc(), parse_front_matter(), split_slides(),
                  parse_slide(), parse_slide_attrs(), extract_notes()
  layout.rs     — render_slide(), render_title(), render_title_content(),
                  render_two_column(), render_section(), render_stats(),
                  render_blank(), apply_theme(), render_body_lines() [stub in W2, full in W3],
                  center_line(), vertical_offset(), split_ratio()
  bullets.rs    — render_bullets(), BulletConfig, parse_bullet_level()
  columns.rs    — render_columns(), parse_col_sections(), ColumnsConfig
  inline.rs     — render_quote(), render_centered(), render_stat(),
                  render_callout(), render_divider()
```

**Total estimated LOC**: ~350 (Wave 1) + ~500 (Wave 2) + ~350 (Wave 3) + ~200 (Wave 4) = **~1,400 LOC** excluding tests.

---

## Cross-cutting notes

- `slide::canvas::SlideCanvas` (Wave 1, `src/slide/canvas.rs`) is used by `layout.rs` and
  `compile_slide_doc`. One `SlideCanvas::new(meta.width, meta.height)` per slide. No
  dependency on `dashboard::canvas::Canvas` — the slide canvas is self-contained.
- `visual_width` from `src/layout.rs` is the only column-width measurer. `center_line`,
  `split_ratio`, and `render_columns` all use it for char-level measurement.
- `## col:` sections inside a `proof:slide layout=two-column` or `proof:columns` fence are
  structural delimiters, not document headings. The slide body compiler must suppress
  `md_h1_count` and heading checks for lines matching `## col:*` inside these fences.
- `proof:notes` linting: the check runner in `src/runner.rs` needs a way to receive the
  extracted notes text. The simplest approach: `parse_slide_doc` returns `notes_content`
  per slide; the check command joins all notes blocks and runs `runner.lint_content` on the
  aggregate. Planned `--no-notes` flag would skip this step.
- `CompileViolation` is reused unchanged. SLIDE-* codes are new string literals, not a new
  enum variant.
- `derive_output_path` checks `.slides.source.md` before `.source.md` — same pattern as
  the planned `.dashboard.source.md` check. All three suffixes must be ordered longest-first.
- IceLines integration: no proof changes beyond Wave 4 CLI flags. IceLines calls
  `proof compile team.slides.source.md --slide N --width $COLUMNS --height $LINES --no-chrome`
  and renders the raw canvas string directly to the terminal.
- `walkdir` in `cmd_compile` already catches `.slides.source.md` files because they end in
  `.source.md` — no filter change needed.
