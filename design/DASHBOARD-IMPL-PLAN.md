# DASHBOARD-IMPL-PLAN — mdloom:dashboard canvas compositor

> Prerequisite: ELEMENT-IMPL-PLAN Wave 1 (element rendering module) must be complete before
> Wave 3. Waves 1 and 2 are independent of element.

---

## Wave 1 — Canvas model (250 LOC)

**Goal**: A fixed-width character grid that other waves paste content into and serialize.
Pure data structure — no parsing, no I/O, no directive knowledge.

### New files

```
src/dashboard/
  mod.rs       — pub re-exports
  canvas.rs    — Canvas struct, paste(), render()
```

### Key structs / functions

**`src/dashboard/canvas.rs`**

```rust
pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Vec<char>>,  // height rows × width cols, initialized to ' '
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self

    /// Paste content_lines into the bounding box starting at (x, y).
    /// Lines longer than the box width are truncated (no '…' — hard clip).
    /// Lines shorter than box width are NOT padded into the canvas — only written chars.
    /// Lines beyond box height are silently dropped (D-5 warning emitted by caller).
    pub fn paste(&mut self, x: usize, y: usize, width: usize, height: usize, content_lines: &[String])

    /// Serialize canvas to a String: each row is exactly `self.width` chars, newline-terminated.
    /// Invariant D-6: every row in the output is exactly `width` chars wide.
    pub fn render(&self) -> String
}
```

Implementation notes:
- `cells` is a flat `Vec<char>` indexed as `row * width + col` — simpler than `Vec<Vec<char>>` for
  bounds checking.
- `paste`: for each `(row_idx, line)` in `content_lines.iter().take(height).enumerate()`:
  - target row = y + row_idx; if >= self.height → skip
  - for each `char` in line (unicode-aware, using `visual_width` for double-wide detection):
    - target col = x + visual_col; if >= x + width or >= self.width → stop
    - write char to `cells[target_row * self.width + target_col]`
- `render`: join rows with `\n`. Every row is exactly `self.width` chars because cells are
  initialized to `' '` and never truncated shorter than `width`.

### Tests (10)

| # | What |
|---|------|
| 1 | `Canvas::new(10, 3).render()` → 3 rows of 10 spaces each |
| 2 | D-6: every row in `render()` output is exactly `width` chars |
| 3 | `paste` at (0,0): content appears at top-left |
| 4 | `paste` at (5,1): content starts at column 5, row 1 |
| 5 | `paste` clips content wider than box width — no overflow into adjacent region |
| 6 | `paste` clips content taller than box height — extra lines dropped |
| 7 | Two `paste` calls to non-overlapping regions don't bleed into each other |
| 8 | `paste` at x=0, width=5 with line "ABCDEFGH" → only "ABCDE" written |
| 9 | `paste` beyond canvas height → panic-free, lines silently dropped |
| 10 | `paste` with empty content_lines → canvas unchanged |

**Exit criterion**: `cargo test -p mdloom-lib dashboard::canvas::` is clean.
`Canvas::new(80, 24).render()` produces a 24-line, 80-char-per-line string.

---

## Wave 2 — Region parser (300 LOC)

**Goal**: Parse the YAML front-matter and `## region-name` / `x:, y:, width:, height:` declarations
from a `.dashboard.source.md` file. Validate D-2 (bounds) and D-3 (no overlap).

### New file

**`src/dashboard/region.rs`**

```rust
pub struct DashboardMeta {
    pub width: usize,
    pub height: usize,
    pub title: String,
}

pub struct RegionDecl {
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

pub struct DashboardDecl {
    pub meta: DashboardMeta,
    pub regions: Vec<RegionDecl>,
    pub content_start_line: usize,  // line index where "# Content" section begins
}

/// Parse front-matter + region declarations from a .dashboard.source.md string.
/// Returns DashboardDecl or a Vec<DashboardError>.
pub fn parse_dashboard_decl(source: &str) -> Result<DashboardDecl, Vec<DashboardError>>

pub enum DashboardError {
    MissingFrontMatter,
    MalformedFrontMatter(String),
    RegionOutOfBounds { name: String, code: &'static str, detail: String },
    RegionOverlap { a: String, b: String },
    UnknownRegion(String),  // content references a region not in declarations
}
```

**Front-matter parsing** — the YAML block is `---\ndashboard:\n  width: N\n  height: N\n---`.
Parse with a minimal hand-written parser (not a full YAML library) since the schema is fixed:
- Find the `---` opening and closing delimiters.
- Extract `width:`, `height:`, `title:` lines inside the `dashboard:` block.
- Fall back to serde_yaml only if already a dependency — otherwise hand-parse.

**Region declaration parsing** — after the front-matter, scan for lines matching `## <name>`:
- The next non-empty lines are `x: N, y: N, width: N, height: N` (comma-separated on one line
  per the spec example, or one key per line — accept both).
- Accumulate into `Vec<RegionDecl>`.

**Validation**:

```rust
fn validate_bounds(decl: &RegionDecl, meta: &DashboardMeta) -> Option<DashboardError>
// D-2: x + width <= meta.width → DASHBOARD-001
//      y + height <= meta.height → DASHBOARD-002

fn validate_no_overlap(regions: &[RegionDecl]) -> Vec<DashboardError>
// D-3: for every pair (a, b): check bounding boxes are disjoint
// Disjoint iff: a.x+a.width <= b.x OR b.x+b.width <= a.x OR
//               a.y+a.height <= b.y OR b.y+b.height <= a.y
```

### Tests (12)

| # | What |
|---|------|
| 1 | Minimal valid front-matter: width, height, title parsed correctly |
| 2 | Missing front-matter → `MissingFrontMatter` error |
| 3 | Malformed width (non-integer) → `MalformedFrontMatter` |
| 4 | Two non-overlapping regions parsed: names, coordinates correct |
| 5 | Region with x + width > canvas width → DASHBOARD-001 error |
| 6 | Region with y + height > canvas height → DASHBOARD-002 error |
| 7 | Two overlapping regions → DASHBOARD-003 error |
| 8 | Regions touching at edge (not overlapping) → no error |
| 9 | Region at (0,0) exactly filling canvas → valid |
| 10 | `content_start_line` points to line after last `## region-name` block |
| 11 | Region `x:, y:, width:, height:` on one comma-separated line → parsed |
| 12 | Region properties on separate lines → parsed |

**Exit criterion**: `parse_dashboard_decl` correctly parses the example from DASHBOARD-SPEC.md
(4 regions: header, forwards-tree, stats-chart, player-table). D-2 and D-3 errors surface as
`DashboardError` variants.

---

## Wave 3 — `mdloom:region` directive + canvas compositor (400 LOC)

**Goal**: Recognize `mdloom:region` blocks in `.dashboard.source.md`, render each region's content
using the existing compile pipeline, paste into Canvas, serialize the full output.

### New files / changes

**`src/dashboard/mod.rs`** — add `compile_dashboard()`:

```rust
pub fn compile_dashboard(
    source_text: &str,
    source_path: &Path,
    root: &Path,
    config: &MdloomConfig,
    width_override: Option<usize>,
    height_override: Option<usize>,
    region_filter: Option<&str>,
    no_chrome: bool,
) -> Result<DashboardResult>

pub struct DashboardResult {
    pub output: String,                      // serialized canvas (or single region)
    pub violations: Vec<CompileViolation>,
    pub regions_rendered: usize,
}
```

Implementation:

1. `parse_dashboard_decl(source_text)` → `DashboardDecl` (or emit errors and bail)
2. Apply width/height overrides to `meta`
3. `Canvas::new(meta.width, meta.height)`
4. For each `mdloom:region name=<X>` block in the content section:
   - Look up `RegionDecl` by name → emit DASHBOARD-004 if not found
   - If `region_filter` is set and name != filter → skip
   - Extract the body of the `mdloom:region` block (lines between ` ``` ` delimiters)
   - Run inner directives through `compile_file()` logic:
     - Build a synthetic `.source.md` from the region body
     - Call `collect_directives()` on it; run each directive's compile arm
     - All content inside a region uses `no_chrome=true` implicitly
   - Collect rendered lines
   - If content_lines.len() > region.height → emit DASHBOARD-005 (warning), clip
   - If any line visual_width < region.width → will be padded by canvas init (DASHBOARD-006 warning)
   - `canvas.paste(region.x, region.y, region.width, region.height, &content_lines)`
5. `canvas.render()` → raw string
6. Wrap in fence + traceability comment (unless `no_chrome`)

**`src/compile.rs`** — add `Directive::Region`:

```rust
Region {
    name: String,
    body_lines: Vec<String>,  // raw lines inside the mdloom:region block
    line_start: usize,
    line_end: usize,
}
```

Extend `mdloom_directive_kind()`:

```rust
else if rest.starts_with("region") { Some("region") }
```

Extend `collect_directives()` — new `"region"` arm:
- Extract `name=` from info string
- Collect body lines until closing ` ``` `

Extend `compile_file()` — detect `.dashboard.source.md` files:
- If `source_path` ends with `.dashboard.source.md`: call `compile_dashboard()` instead of
  the normal directive loop.
- Otherwise: existing behavior (for non-dashboard `.source.md` files containing `mdloom:region`
  as an inline embed — future use).

**Diagnostic emission**:

| Code | Trigger |
|------|---------|
| DASHBOARD-001 | `x + width > canvas_width` |
| DASHBOARD-002 | `y + height > canvas_height` |
| DASHBOARD-003 | Two regions overlap |
| DASHBOARD-004 | `mdloom:region name=X` has no matching declaration |
| DASHBOARD-005 | Region content overflows declared height |
| DASHBOARD-006 | Region content line underflows declared width |

### Tests (15)

| # | What |
|---|------|
| 1 | `compile_dashboard` with 2 regions → canvas contains both regions' content |
| 2 | Region content does not bleed into adjacent region (Canvas isolation) |
| 3 | DASHBOARD-004 emitted when `mdloom:region name=missing` has no declaration |
| 4 | DASHBOARD-005 emitted when region content is taller than declared height |
| 5 | DASHBOARD-006 emitted when region content line is narrower than declared width |
| 6 | `region_filter` set: only specified region rendered, rest left blank |
| 7 | Output wrapped in `<!-- mdloom:compiled from="mdloom:dashboard" -->` fence |
| 8 | `no_chrome=true`: output is raw canvas string, no fence |
| 9 | Width override replaces front-matter width |
| 10 | Height override replaces front-matter height |
| 11 | `derive_output_path` for `foo.dashboard.source.md` → `foo.dashboard.md` |
| 12 | Inner `mdloom:element kind=label value="X" width=10` inside region renders correctly |
| 13 | Inner `mdloom:row` inside region: each row line pasted at correct y offset |
| 14 | `compile_file` detects `.dashboard.source.md` and routes to `compile_dashboard` |
| 15 | Canvas with 0 region content → all spaces (blank canvas, no panic) |

**Exit criterion**: a `.dashboard.source.md` with 2 declared regions and 2 `mdloom:region` blocks
compiles to a fixed-width canvas with correct region placement. The two regions' content doesn't
bleed into each other.

---

## Wave 4 — CLI flags + IceLines integration (150 LOC)

**Goal**: `--width N`, `--height N`, `--region name`, `--no-chrome` flags on `mdloom compile`.
`derive_output_path` handles the `.dashboard.source.md → .dashboard.md` suffix.

### Changes to existing files

**`src/main.rs`** — extend `Command::Compile`:

```rust
Compile {
    paths: Vec<PathBuf>,
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,
    #[arg(long)]
    check: bool,
    #[arg(long)]
    root: Option<PathBuf>,
    /// Override canvas width (dashboard only)
    #[arg(long)]
    width: Option<usize>,
    /// Override canvas height (dashboard only)
    #[arg(long)]
    height: Option<usize>,
    /// Render only one named region
    #[arg(long)]
    region: Option<String>,
    /// Raw output — suppress fence and traceability comment
    #[arg(long)]
    no_chrome: bool,
}
```

Pass `width`, `height`, `region`, `no_chrome` through `cmd_compile()` → `compile_file()` or
`compile_dashboard()`.

**`src/compile.rs`** — `derive_output_path` already handles `.source.md`. Add `.dashboard.source.md`:

```rust
pub fn derive_output_path(source: &Path) -> Option<PathBuf> {
    let name = source.file_name()?.to_str()?;
    if let Some(stem) = name.strip_suffix(".dashboard.source.md") {
        let out_name = format!("{}.dashboard.md", stem);
        Some(source.parent().unwrap_or(Path::new(".")).join(out_name))
    } else if let Some(stem) = name.strip_suffix(".source.md") {
        let out_name = format!("{}.md", stem);
        Some(source.parent().unwrap_or(Path::new(".")).join(out_name))
    } else {
        None
    }
}
```

Note: `.dashboard.source.md` must be checked first — it's a longer suffix and would otherwise
match `.source.md` incorrectly.

**`src/compile.rs`** — `compile_file` signature extension:

```rust
pub fn compile_file(
    source_path: &Path,
    output_path: &Path,
    root: &Path,
    config: &MdloomConfig,
    // New optional params for dashboard:
    width_override: Option<usize>,
    height_override: Option<usize>,
    region_filter: Option<&str>,
    no_chrome: bool,
) -> Result<CompileResult>
```

For non-dashboard files, `width_override`, `height_override`, `region_filter`, `no_chrome` are
ignored (or `no_chrome` suppresses the traceability wrapper — consistent with element behavior).

### Tests (8)

| # | What |
|---|------|
| 1 | `derive_output_path("team.dashboard.source.md")` → `"team.dashboard.md"` |
| 2 | `derive_output_path("doc.source.md")` → `"doc.md"` (existing behavior unchanged) |
| 3 | `derive_output_path("team.dashboard.source.md")` does not match `.source.md` suffix first |
| 4 | `cmd_compile` with `--width 80 --height 24` passes overrides to `compile_dashboard` |
| 5 | `cmd_compile` with `--region header` renders only the header region |
| 6 | `cmd_compile` with `--no-chrome` suppresses fence in output |
| 7 | `cmd_compile` on a non-dashboard `.source.md` — `--width` / `--height` flags are no-ops |
| 8 | `mdloom compile team.dashboard.source.md --width 80 --height 24` exit code 0 on clean input |

**Exit criterion**: `mdloom compile dashboard.source.md --width 80 --height 24` compiles a
80×24 canvas. `mdloom compile dashboard.source.md --region header` emits only the header region
content. Both exit 0 on clean input.

---

## Module structure summary

```
src/dashboard/
  mod.rs        — compile_dashboard(), DashboardResult
  canvas.rs     — Canvas, paste(), render()
  region.rs     — DashboardDecl, RegionDecl, parse_dashboard_decl(), validate_bounds(), validate_no_overlap()
```

**Total estimated LOC**: ~250 (Wave 1) + ~300 (Wave 2) + ~400 (Wave 3) + ~150 (Wave 4) = **~1,100 LOC** excluding tests.

---

## Cross-cutting notes

- `visual_width` from `src/layout.rs` is the only width measurer. Canvas column tracking for
  paste must use it per-char (double-wide CJK occupies 2 columns; sparkline block chars are 1).
- `collect_directives()` and the compile arm dispatch are extended in-place — no new pass is
  needed. The dashboard route branches at `compile_file()` entry based on the `.dashboard.source.md`
  filename suffix.
- Inner directives inside a `mdloom:region` body run through the same `collect_directives()` +
  per-arm compile path. Region content is treated as a synthetic `.source.md` string, not a file.
  Pass `no_chrome=true` implicitly for all inner renders (the region boundary is the container).
- `CompileViolation` is reused for both ELEMENT-* and DASHBOARD-* codes. No new result type.
- IceLines integration: no mdloom changes required beyond Wave 4 CLI flags. IceLines calls
  `mdloom compile screen.dashboard.source.md --width $COLUMNS --height $LINES` and reads stdout.
  The `--no-chrome` flag gives IceLines the raw canvas string for direct terminal rendering.
- `walkdir` already traverses `.source.md` files in `cmd_compile`. Extend the filter to also
  collect `.dashboard.source.md` files (they already end in `.source.md`, so the existing
  `s.ends_with(".source.md")` check catches them — no change needed).
