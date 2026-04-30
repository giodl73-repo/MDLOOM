# proof figure — Implementation Plan

> **Status**: Substantially implemented — see FIGURE-SPEC.md for the authoritative status. `src/figure/` contains the dither modes, shape clipping, and image→ASCII pipeline. DaVinci pinning lives in `src/davinci.rs`. Wave numbering below is now historical context, not a forward plan.
> **Spec**: FIGURE-SPEC.md
> **Exit criterion (full)**: `proof figure import logos/EDM.png --shape shield --label "EDM" --width 20`
> produces a valid figure file; `proof:include kind=figure` embeds it with FIGURE-005 if unpinned.

---

## Wave 0 — Pre-conditions (before starting Wave 1)

- Modify `src/layout.rs::visual_width()` to add Braille (U+2800–U+28FF) to the width=1
  override list, alongside box-drawing and geometric shapes. This is required so braille
  dither output measures correctly in column layouts.
- Add test: `assert_eq!(visual_width("⠿"), 1); // U+28FF Braille`
- This touches a file outside `src/figure/` but is a blocking pre-condition.
  Estimated: 5 LOC + 1 test.

---

## Already added to Cargo.toml (dependency additions complete)

```toml
image = { version = "0.25", default-features = false, features = ["png", "jpeg", "gif", "bmp"], optional = true }
resvg = { version = "0.44", optional = true }

[features]
figure = ["image"]
svg = ["figure", "resvg"]
```

Wave 1 must be compiled with `cargo build --features figure`. Wave 4 SVG support requires
`cargo build --features svg`.

`unicode-width` is already present (used by `visual_width` in `layout.rs`).
`serde_json` is already present (used for `.proof-fetch-lock.json`).

---

## Wave 1 — Image-to-ASCII engine (~450 LOC)

**Files**: `src/figure/mod.rs`, `src/figure/dither.rs`, `src/figure/shape.rs`

### Key structs

```rust
// src/figure/mod.rs
pub struct ImportOptions {
    pub width: u32,               // default 40
    pub height: Option<u32>,      // None = auto (preserve aspect)
    pub dither: DitherMode,       // default Block
    pub edge: bool,
    pub invert: bool,
    pub threshold: u8,            // binary mode, default 128
    pub contrast: f32,            // default 1.0
    pub gamma: f32,               // default 1.0
    pub bg_char: char,            // default ' '
    pub shape: Option<ShapeKind>,
    pub label: Option<String>,
    pub label_pos: LabelPos,      // Center | Top | Bottom
    pub allow_fetch: bool,
    pub svg_scale: u32,           // default 4
}

pub enum DitherMode { Density, Block, HalfBlock, QuarterBlock, Braille, Binary, Edge }
pub enum ShapeKind { Circle, Octagon, Shield, Star, Heart, Diamond, Hexagon, Pentagon, RoundedRect }
pub enum LabelPos { Center, Top, Bottom }

// src/figure/dither.rs
pub struct DitherContext<'a> {
    pub gray: &'a image::GrayImage,
    pub width: u32,
    pub height: u32,
    pub opts: &'a ImportOptions,
}

pub fn dither(ctx: &DitherContext) -> Vec<String>;  // returns ASCII rows
```

### Key functions

```rust
// src/figure/mod.rs
pub fn import_image(path: &Path, opts: &ImportOptions) -> Result<String>
// Returns raw ASCII string (no fence). Pipeline:
//   1. load_image(path, opts) → DynamicImage
//   2. apply_gamma_contrast(img, opts) → DynamicImage
//   3. resize to (width, height) with aspect-ratio preservation
//   4. convert to GrayImage
//   5. if opts.shape.is_some() → apply shape mask (shape.rs)
//   6. dither(ctx) → Vec<String> (dither.rs)
//   7. if opts.label.is_some() → apply_label_overlay(rows, opts)
//   8. join rows with '\n'

pub fn load_image(path: &Path, opts: &ImportOptions) -> Result<image::DynamicImage>
// Handles: PNG/JPG/GIF/BMP via image crate; .svg via resvg feature (Wave 4)
// Remote URLs: requires opts.allow_fetch; fetches, checks .proof-fetch-lock.json
//
// SVG feature guard: all code paths that reference `resvg` types must be inside
// `#[cfg(feature = "svg")]` blocks. The non-feature build (`cargo build --features figure`
// without `svg`) must compile cleanly. In load_image(), add a `#[cfg(feature = "svg")]`
// arm for `.svg` extension handling; the default arm returns Err("SVG requires --features svg").

// src/figure/dither.rs
pub fn dither_density(ctx: &DitherContext) -> Vec<String>   // " .:-=+*#%@"
pub fn dither_block(ctx: &DitherContext) -> Vec<String>     // " ░▒▓█"
pub fn dither_half_block(ctx: &DitherContext) -> Vec<String>// " ▀▄█" — 2 rows/char
pub fn dither_quarter_block(ctx: &DitherContext) -> Vec<String>  // 4-subpixel
pub fn dither_braille(ctx: &DitherContext) -> Vec<String>   // U+2800-U+28FF, 2×4 cells
pub fn dither_binary(ctx: &DitherContext) -> Vec<String>    // "█ " threshold
pub fn dither_edge(ctx: &DitherContext) -> Vec<String>      // "─│╱╲" via Sobel

// src/figure/shape.rs
pub struct ShapeMask {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<bool>,        // true = inside shape, false = background
}

pub fn build_mask(kind: ShapeKind, width: u32, height: u32) -> ShapeMask
pub fn apply_mask(img: &image::GrayImage, mask: &ShapeMask, bg: u8) -> image::GrayImage
pub fn enforce_minimum_size(kind: &ShapeKind, width: u32) -> Result<()>
// Errors on: octagon<14, circle<10, shield<12, star<8
```

### Braille width note

Braille characters (U+2800–U+28FF) must be forced to width=1 in `visual_width()` in `layout.rs`. The range `0x2800..=0x28FF` is currently NOT covered by the existing override (which covers up to `0x25FF`). Add it alongside the existing box-drawing/geometric-shapes arm:

```rust
// layout.rs visual_width() — add to the existing match arm:
|| (0x2800..=0x28FF).contains(&cp)  // Braille patterns
```

This is required for correct column layout; terminals that render Braille at width=2 will display wider than declared — FIGURE-004 warns when `--dither braille` is used.

### Diagnostic emission

| Condition | Code | Severity |
|-----------|------|----------|
| File not found / unreadable | FIGURE-001 | error |
| Aspect ratio change > 20% with explicit `--height` | FIGURE-002 | warning |
| Shape clip produced empty output | FIGURE-003 | error |
| `--dither braille` used | FIGURE-004 | warning |
| Remote URL without `--allow-fetch` | FIGURE-006 | error |

### Estimated LOC breakdown

| File | LOC |
|------|-----|
| `src/figure/mod.rs` | ~130 |
| `src/figure/dither.rs` | ~200 |
| `src/figure/shape.rs` | ~120 |

### Tests (25+)

```
test_density_dither_produces_ascii_chars
test_block_dither_uses_block_chars
test_half_block_reduces_height_by_half
test_braille_dither_uses_braille_range
test_binary_dither_threshold_splits_black_white
test_edge_dither_detects_edges
test_dither_invert_reverses_brightness
test_octagon_mask_rejects_width_13   // FIGURE-003
test_circle_mask_rejects_width_9
test_shield_mask_rejects_width_11
test_star_mask_rejects_width_7
test_octagon_mask_at_minimum_size_14
test_circle_mask_at_minimum_size_10
test_shape_mask_bg_char_fills_outside
test_label_overlay_center_positioning
test_label_overlay_top_positioning
test_label_overlay_bottom_positioning
test_label_truncated_at_frame_width
test_braille_chars_are_width_1_in_visual_width   // regression for layout.rs fix
test_import_options_defaults
test_aspect_ratio_warning_fires_on_override      // FIGURE-002
test_remote_url_without_allow_fetch_errors       // FIGURE-006
test_load_image_png_succeeds
test_load_image_unknown_format_errors            // FIGURE-001
test_gamma_contrast_applied_before_dither
```

### Exit criterion

`import_image(path, &ImportOptions::default())` returns a non-empty ASCII string
for a 1×1 black PNG.

**Fixture note:** Create test images programmatically using `image::GrayImage::new(1, 1)` —
do not require filesystem fixture files. This keeps tests self-contained and avoids binary
test files in the repo.

---

## Wave 2 — Figure file format + catalog (~200 LOC)

**Files**: `src/figure/catalog.rs`

### Figure file format

```
<!-- proof:figure id="edm-logo" kind="figure.logo" -->
```ascii
<raw ascii art>
```
<!-- /proof:figure -->
```

The `<!-- proof:figure -->` comment sits **outside** the code fence. The fence has
no info string (plain triple-backtick). Multiple figures may appear in one `.md` file.

### Key structs

```rust
pub struct FigureMarker {
    pub id: String,
    pub kind: String,             // "figure.logo", "figure.illustration", etc.
    pub file: PathBuf,
    pub line_start: usize,        // line of <!-- proof:figure -->
    pub line_end: usize,          // line of <!-- /proof:figure -->
    pub width: u32,               // columns in widest ASCII line
    pub height: u32,              // line count
    pub content: String,          // raw ASCII between fences
}

pub struct FigureCatalog {
    pub figures: Vec<FigureMarker>,
}
```

### Key functions

```rust
// src/figure/catalog.rs
pub fn parse_figure_markers(content: &str, file: &Path) -> Vec<FigureMarker>
// Scans for <!-- proof:figure id="..." kind="..." --> ... <!-- /proof:figure -->
// Extracts content from the fenced block inside

pub fn index_directory(dir: &Path) -> Result<FigureCatalog>
// Walks dir, calls parse_figure_markers on each .md file

pub fn write_figure_file(marker: &FigureMarker, ascii: &str, output: &Path) -> Result<()>
// Writes full markdown wrapper:
//   <!-- proof:figure id="..." kind="..." -->
//   ```
//   <ascii>
//   ```
//   <!-- /proof:figure -->

pub fn figures_command(dir: &Path, kind_filter: Option<&str>) -> Result<()>
// proof figures . — lists all indexed figures with metadata
```

### CLI output format (`proof figures .`)

```
figures/nhl/edm-logo.md#edm-logo:0
  kind:     figure.logo
  size:     40×20
  pinned:   no

figures/animals/bear-stop.md#bear-stop:0
  kind:     figure.illustration
  size:     20×10
  pinned:   no
```

### `proof figure preview <uri>`

Resolves uri via `mdpath::parse` + `mdpath::resolve` (same pattern as `compile.rs`
`resolve_uri`), then prints content to stdout. No fence wrapping in terminal output.

### `--output-file` format

When `--output-file path.md` is passed to `proof figure import`, writes via
`write_figure_file`. When `--output-file -` is passed, writes raw ASCII to stdout
without wrapper.

### Tests (catalog)

```
test_parse_single_figure_marker
test_parse_multiple_figures_in_one_file
test_parse_marker_extracts_correct_id_and_kind
test_parse_marker_measures_width_and_height
test_parse_ignores_unclosed_markers
test_parse_ignores_non_figure_html_comments
test_index_directory_finds_all_md_files
test_write_figure_file_roundtrips_through_parse
test_output_file_minus_writes_raw_ascii
```

### Exit criterion

`parse_figure_markers` correctly extracts id, kind, content, width, height from a
two-figure test file; `index_directory` returns 2 entries.

---

## Wave 3 — Template-based generation (~300 LOC)

**Files**: `src/figure/generate.rs`, `src/figure/animals/*.txt`

### Built-in animals (5 core)

Stored as `src/figure/animals/{name}.txt`, embedded at compile time via `include_str!`:

| Name | Description |
|------|-------------|
| `bear` | Standing bear silhouette |
| `eagle` | Spread-wing eagle |
| `wolf` | Wolf head facing forward |
| `puck` | Abstract circular puck shape |
| `flame` | Abstract flame |

Each file is a ~20-line ASCII art block, no fence wrapper. Full set adds ~8 KB to binary.

### Key structs

```rust
pub enum GenerateKind {
    Animal,
    LogoBadge,
    Shape,
}

pub struct GenerateOptions {
    pub kind: GenerateKind,
    pub name: Option<String>,        // animal name or shape name
    pub text: Option<String>,        // logo-badge primary text
    pub subtitle: Option<String>,    // logo-badge subtitle
    pub shape: Option<ShapeKind>,    // clip animal/badge into shape
    pub size: u32,                   // shape size 1-5
    pub id: String,                  // figure id
    pub output_file: Option<PathBuf>,
}

pub struct AnimalRegistry {
    animals: HashMap<&'static str, &'static str>,   // name → include_str! content
}
```

### Key functions

```rust
// src/figure/generate.rs
pub fn generate(opts: &GenerateOptions) -> Result<String>
// Dispatches to generate_animal, generate_logo_badge, or generate_shape
// Returns raw ASCII string

pub fn generate_animal(name: &str, shape: Option<ShapeKind>, label: Option<&str>) -> Result<String>
// Looks up animal in AnimalRegistry, optionally clips into shape

pub fn generate_logo_badge(text: &str, subtitle: Option<&str>, shape: ShapeKind, size: u32) -> Result<String>
// Centers text + subtitle inside shape template at given size

pub fn generate_shape(name: ShapeKind, size: u32) -> Result<String>
// Pure geometric ASCII art scaled by size multiplier

pub fn sanitize_filename(raw: &str) -> String
// lowercase, spaces→hyphens, strip non-[a-z0-9-_]
// "EDM" → "edm", "St. Louis" → "st-louis"

impl AnimalRegistry {
    pub fn new() -> Self   // populated with include_str! calls
    pub fn get(&self, name: &str) -> Option<&'static str>
    pub fn list(&self) -> Vec<&'static str>
}
```

### Filename sanitization rules

1. Lowercase all characters
2. Replace spaces and underscores with hyphens
3. Strip all characters that are not `[a-z0-9-]`
4. Collapse consecutive hyphens to one
5. Strip leading/trailing hyphens

`"EDM"` → `"edm"`, `"St. Louis"` → `"st-louis"`, `"L.A. Kings"` → `"la-kings"`

### Tests

```
test_animal_bear_renders_non_empty
test_animal_eagle_renders_non_empty
test_animal_wolf_renders_non_empty
test_animal_puck_renders_non_empty
test_animal_flame_renders_non_empty
test_animal_unknown_name_errors
test_logo_badge_size_1_produces_output
test_logo_badge_size_3_produces_output
test_logo_badge_text_appears_in_output
test_logo_badge_subtitle_appears_in_output
test_sanitize_edm_becomes_edm
test_sanitize_st_louis_becomes_st_minus_louis
test_sanitize_la_kings_becomes_la_minus_kings
test_sanitize_strips_trailing_hyphens
test_sanitize_collapses_consecutive_hyphens
test_generate_shape_star_size_1
test_generate_shape_star_size_5
test_generate_shape_shield_size_3
test_animal_registry_list_contains_all_5
```

### Exit criterion

`generate_animal("bear", None, None)` returns a non-empty string containing no
fence markers. `sanitize_filename("St. Louis")` returns `"st-louis"`.

---

## Wave 4 — proof:include kind=figure + cache + CLI (~200 LOC)

**Files**: `src/compile.rs` (extend), `src/commands/figure.rs` (new CLI surface)

### compile.rs changes

#### 1. New directive variant

Add `IncludeFigure` to the `Directive` enum:

```rust
IncludeFigure {
    uri: String,
    attrs: FigureAttrs,    // dither, width, height, shape, label, invert, contrast, gamma
    line_start: usize,
    line_end: usize,
},
```

#### 2. Directive parsing

Extend `proof_directive_kind` and `collect_directives` to recognize
`` ```proof:include kind=figure ``:

```rust
// proof_directive_kind — add:
else if rest.starts_with("include") {
    // Check for kind=figure attribute in the info string — token-exact match required
    if rest.split_whitespace().any(|tok| tok == "kind=figure") { Some("include_figure") }
    else { Some("include") }
}
```

**Implementation note:** Do NOT use `info_string.contains("kind=figure")` — this would
false-positive on `kind=figure-sequence` or similar future attributes. Use token-exact
matching: split the info string on whitespace and check for the exact token `"kind=figure"`
as a complete element. Pattern: `info_string.split_whitespace().any(|tok| tok == "kind=figure")`.

The body of an `include_figure` directive is a single URI line (same as `include`).
The info string carries directive attributes: `kind=figure dither=block width=40 ...`

#### 3. FigureAttrs struct

```rust
#[derive(Debug, Default)]
pub struct FigureAttrs {
    pub dither: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub shape: Option<String>,
    pub label: Option<String>,
    pub invert: bool,
    pub contrast: Option<f32>,
    pub gamma: Option<f32>,
}

impl FigureAttrs {
    fn parse(attrs_str: &str) -> Self { /* same key=value scanner as LayoutAttrs */ }

    fn cache_hash(&self) -> u64 { /* FNV or std DefaultHasher over all fields */ }
}
```

#### 4. Cache key extension

The compile cache key for `kind=figure` incorporates directive attributes that
affect rendered output. Hash these alongside the source `resolve_key` before
computing `compile_key`:

```
compile_key = sha256(resolve_key + "|" + hex(figure_attrs.cache_hash()))
```

This is an extension to the standard three-tier model in COMPILE-SPEC.md.
Changing `dither`, `width`, `height`, `shape`, `label`, `invert`, `contrast`,
or `gamma` produces a cache miss and re-renders.

#### 5. FIGURE-005 warning

When a `proof:include kind=figure` directive resolves a URI that has no DaVinci
pin in the config, emit FIGURE-005:

```rust
// After validate_davinci — if no davinci entry matched this uri:
if !config.davinci.iter().any(|e| uri_matches(&e.uri, uri)) {
    violations.push(CompileViolation {
        code: "FIGURE-005",
        severity: ViolationSeverity::Warning,
        uri: uri.clone(),
        ...
        message: "figure has no DaVinci pin — use `proof pin` to protect it".to_string(),
    });
}
```

Plain `proof:include` (without `kind=figure`) does NOT trigger FIGURE-005.

#### 6. Remote URL lock file (`.proof-fetch-lock.json`)

When `--allow-fetch` is used and the URI is a remote URL, `load_image` writes a
lock file adjacent to the output file:

```json
{
  "url": "https://example.com/logo.png",
  "fetch_time": "2026-04-26T18:00:00Z",
  "sha256": "abc123..."
}
```

On subsequent compiles, the lock file is checked. If SHA-256 matches cached file,
cache hits. If remote content changes, cache misses and regenerates. Pinned figures
that include a remote source must have their lock file committed.

### src/commands/figure.rs — CLI surface

```rust
pub enum FigureSubcommand {
    Import {
        image: PathBuf,
        id: Option<String>,
        output_file: Option<PathBuf>,
        opts: ImportOptions,
    },
    Generate {
        opts: GenerateOptions,
    },
    Preview {
        uri: String,
    },
}

pub fn run_figure(cmd: FigureSubcommand, root: &Path) -> Result<()>
pub fn run_figures_catalog(dir: &Path, kind_filter: Option<&str>) -> Result<()>
```

The `proof figures` (catalog list) subcommand lives at the top-level alongside
`proof figure` (singular — import/generate/preview).

### Tests (Wave 4)

```
test_proof_directive_kind_detects_include_figure
test_collect_directives_include_figure_parses_attrs
test_figure_attrs_cache_hash_changes_with_dither
test_figure_attrs_cache_hash_changes_with_width
test_figure_attrs_cache_hash_stable_for_identical_opts
test_figure_005_fires_for_unpinned_kind_figure_include
test_figure_005_does_not_fire_for_plain_include
test_figure_005_does_not_fire_for_pinned_figure
test_fetch_lock_written_on_remote_fetch
test_fetch_lock_cache_hit_on_matching_sha256
test_fetch_lock_cache_miss_on_changed_sha256
test_compile_figure_embeds_content_with_traceability_comment
```

### Exit criterion (full wave)

```bash
proof figure import logos/EDM.png \
    --shape shield \
    --label "EDM" \
    --width 20 \
    --id edm-logo \
    --output-file figures/nhl/edm-logo.md
```

Produces `figures/nhl/edm-logo.md` with valid `<!-- proof:figure -->` markers and
ASCII content. A `.source.md` containing:

```
```proof:include kind=figure
md://figures/nhl/edm-logo.md#edm-logo:0
```
```

compiles successfully and emits FIGURE-005 warning (figure unpinned).

---

## Cross-wave constraints

### `visual_width` fix (Wave 0 pre-condition — must precede Wave 1)

See **Wave 0 — Pre-conditions** section above for full details. Code reference for `src/layout.rs`:

```rust
pub fn visual_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if (0x2190..=0x21FF).contains(&cp)   // arrows
                || (0x2500..=0x259F).contains(&cp)  // box-drawing + block elements
                || (0x25A0..=0x25FF).contains(&cp)  // geometric shapes
                || (0x2800..=0x28FF).contains(&cp)  // Braille patterns  ← ADD THIS
            {
                1
            } else {
                UnicodeWidthChar::width(c).unwrap_or(1)
            }
        })
        .sum()
}
```

### Module registration (`src/lib.rs`)

Add `pub mod figure;` alongside existing module declarations so `src/figure/` is
compiled. `src/figure/mod.rs` re-exports `ImportOptions`, `import_image`,
`DitherMode`, `ShapeKind`.

### `Directive` enum `line_start`/`line_end` arm

Add `IncludeFigure` to both `line_start()` and `line_end()` match arms in `compile.rs`.

---

## LOC summary

| Wave | Files | Est. LOC |
|------|-------|----------|
| 1 | figure/mod.rs, dither.rs, shape.rs | ~450 |
| 2 | figure/catalog.rs | ~200 |
| 3 | figure/generate.rs, animals/*.txt (data) | ~300 |
| 4 | compile.rs (extend), commands/figure.rs | ~200 |
| **Total** | | **~1,150** |

Tests: 25+ per wave = 60–70 test functions total across `#[cfg(test)]` blocks in each file.
