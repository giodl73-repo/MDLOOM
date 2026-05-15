# proof figure — Named ASCII Art Figures with Image Import

> **Status**: ✅ Implemented. DaVinci pinning lives end-to-end (`proof pin`, `proof check --daVinci`, `[[davinci]]` in proof.toml, inline `pin=` on `proof:include`, plus `regex` match alongside `pattern`/`contains-text`/`equals` invariants). Image→ASCII engine built (`src/figure/`) — all seven dither modes (density, block, half-block, quarter-block, braille, binary, edge) + ten shape masks. Full figure validation pipeline (FIGURE-001 through FIGURE-006 diagnostics) wired. Build via `--features figure` for image-import support.

---

## What it is

A **figure** is a named, addressable, pinnable unit of ASCII art. Figures are:

- **Larger than symbols** (multi-line, structural diagrams or artwork)
- **Smaller than charts/trees** (no data binding — purely visual)
- **Importable from images** — PNG, JPG, SVG → ASCII art conversion
- **Pinnable with DaVinci invariants** — structure protected against drift

Examples: team logos, animal mascots, geometric shapes, architectural diagrams,
portraits, icons at sizes too large for the symbol library.

---

## Figure files

A figure file is a `.md` file whose code blocks are annotated with
`<!-- proof:figure -->` HTML comment markers:

```markdown
<!-- proof:figure id="edm-logo" kind="figure.logo" -->
```
        ████████████████
      ██░░░░░░░░░░░░░░░░██
    ██░░    ██████████  ░░░░██
    ██░░  ██▓▓▓▓▓▓▓▓▓██  ░░██
    ██░░ ██▓▓ EDMONTON ▓▓██ ░██
    ██░░  ██▓▓▓▓▓▓▓▓▓██  ░░██
      ██░░    ██████████  ░░██
        ████████████████
```
<!-- /proof:figure -->
```

The HTML comment is **outside** the code fence — markdown renderers hide it,
proof indexes it. A figure file may contain multiple figures.

Address via mdpath:
```
md://figures/nhl/edm-logo.md#edm-logo:0
md://figures/animals/bear.md#bear-stop:0
```

---

## `proof:figure import=` — image to ASCII

Convert any raster image (PNG, JPG, GIF, BMP) or vector (SVG) to ASCII art.

```bash
proof figure import logos/EDM.png --id edm-logo --width 40 --height 20
proof figure import logos/EDM.png --id edm-logo --dither block --edge
proof figure import photos/McDavid.jpg --id mcdavid-portrait --width 60 --dither braille
proof figure import icons/bear.png --id bear --shape octagon --label "STOP" --width 20
```

### Image sources

| Format | Extension | Notes |
|--------|-----------|-------|
| PNG | `.png` | Preferred — lossless, supports transparency |
| JPEG | `.jpg`, `.jpeg` | Lossy — fine for photos |
| GIF | `.gif` | First frame used |
| BMP | `.bmp` | |
| SVG | `.svg` | Rasterized at `--svg-scale` (default 4×) before conversion |
| URL | `https://...` | Fetched and cached (with `--allow-fetch`) |

### Dither modes

The `--dither` flag selects the character mapping algorithm:

| Mode | Characters | Pixels/char | Best for |
|------|-----------|-------------|---------|
| `density` | ` .:-=+*#%@` | 1 | Simple line art, text |
| `block` | ` ░▒▓█` | 1 | Logos, icons, solid shapes |
| `half-block` | ` ▀▄█` | 2 (top/bottom) | Better vertical resolution |
| `quarter-block` | ` ▘▝▖▗▌▐▀▄█ ` | 4 subpixels | High-fidelity small images |
| `braille` | `⠀–⣿` (256 chars) | 2×4 = 8 | Photos, portraits, fine detail |
| `binary` | `█ ` (threshold) | 1 | Silhouettes, logos at small size |
| `edge` | `─│╱╲` | 1 | Outline-only mode |

**Note:** Braille characters (U+2800–U+28FF) are forced to width=1 in `visual_width()`, consistent with how box-drawing characters are handled. This is required for correct column layout. Terminals that render Braille at width=2 will display output as wider than declared — `FIGURE-004` warns when `--dither braille` is used.

### Generation options

| Option | Default | Description |
|--------|---------|-------------|
| `--width N` | 40 | Output width in chars |
| `--height N` | auto | Output height (default: preserve aspect ratio) |
| `--dither` | `block` | Character mapping algorithm |
| `--edge` | false | Detect and draw edges only (combine with dither) |
| `--invert` | false | Invert brightness (dark background) |
| `--threshold N` | 128 | Binary threshold (0-255) for `binary` mode |
| `--color` | `mono` | `mono`, `ansi256`, `truecolor` |
| `--bg-char` | ` ` | Character to use for background/transparent areas |
| `--contrast N` | 1.0 | Contrast multiplier before conversion |
| `--gamma N` | 1.0 | Gamma correction |
| `--shape` | none | Clip to shape before conversion (`octagon`, `circle`, `shield`, `star`, `heart`, `diamond`) |
| `--label TEXT` | none | Overlay label text (centered) |
| `--label-pos` | `center` | `center`, `top`, `bottom` |
| `--svg-scale N` | 4 | Rasterization scale for SVG input |
| `--allow-fetch` | false | Allow fetching remote image URLs (see Content locking note below) |
| `--output-file` | — | Write to file instead of stdout (see Output format note below) |

**Content locking:** When a remote URL is fetched, proof writes a `.proof-fetch-lock.json` alongside the output file recording `{url, fetch_time, sha256_of_response}`. On subsequent compiles, the lock file is checked — if the SHA-256 still matches, the cache hits. If the remote content changes, the cache misses and the figure regenerates. Pinned figures (`proof pin`) that include a remote source must have their lock file committed.

**`--output-file` format:** `--output-file` writes the **full markdown wrapper** including `<!-- proof:figure id="..." -->` marker, fenced code block, and `<!-- /proof:figure -->` closer — the same format as figure files. The raw ASCII content is always wrapped. Use `--output-file -` to write raw ASCII to stdout without the wrapper.

### Shape clipping

`--shape` clips the image to a geometric shape before ASCII conversion.
Useful for logos inside shields, badges inside circles, etc.

```bash
# Bear inside a stop-sign octagon with STOP label
proof figure import animals/bear.png \
    --shape octagon \
    --label "STOP" \
    --label-pos bottom \
    --width 20 \
    --dither block \
    --id bear-stop
```

Output:
```
   ████████████████
  ██░░░░░░░░░░░░░░██
 ██░░  ███░░░███  ░░██
██░░░ █████████████ ░░██
██░░░ ██▓▓▓▓▓▓▓▓█ ░░░██
██░░░  ██▓▓▓▓▓██  ░░░██
 ██░░   █████████  ░░██
  ██░░░░░░░░░░░░░░░░██
   ████  S T O P  ████
    ████████████████
```

*(Illustrative — actual output varies by source image, dither mode, and shape geometry)*

Available shapes: `circle`, `octagon` (stop sign), `shield` (NHL/heraldry),
`star` (5-point), `heart`, `diamond`, `hexagon`, `pentagon`, `rounded-rect`.

**Minimum sizes:** Shapes degrade at small widths. Recommended minimums: `circle` ≥ 10, `octagon` ≥ 14, `shield` ≥ 12, `star` ≥ 8. Below these thresholds, diagonal corners collapse to single characters and the shape may be unrecognizable. `FIGURE-003` fires on empty output but not on degraded-but-non-empty output — authors should preview at the target size.

---

## Named figure generation (without import)

For figures that don't come from images, use the figure template system:

```bash
# Generate a team logo badge from team name
proof figure generate --kind logo-badge --text "EDM" --subtitle "OILERS" \
    --shape shield --width 20 --id edm-badge

# Generate an animal in a shape
proof figure generate --kind animal --name bear --shape octagon --label "STOP" \
    --width 20 --id bear-stop

# Generate a geometric shape
proof figure generate --kind shape --name star --size 5 --id large-star
```

### Built-in figure kinds

| Kind | Description | Key options |
|------|-------------|-------------|
| `logo-badge` | Text in a decorative shape | `--text`, `--subtitle`, `--shape`, `--style` |
| `animal` | ASCII art animal | `--name` (bear, eagle, lion, wolf, moose...) |
| `shape` | Pure geometric ASCII shape | `--name`, `--size`, `--fill` |
| `portrait` | Human silhouette (stick figure or abstract) | — |
| `banner` | Decorative text banner | `--text`, `--style` |
| `seal` | Circular seal/emblem | `--text`, `--motto`, `--icon` |

**Animal storage:** Built-in animal templates are embedded in the binary as static string data (Rust `include_str!` at compile time). Each animal is a ~20-line ASCII art block stored in `src/figure/animals/*.txt`. The full set adds ~8KB to the binary — acceptable for a CLI tool.

### Built-in animals

Common animals useful for mascots, stop signs, team logos:
`bear`, `eagle`, `lion`, `wolf`, `moose`, `goose`, `penguin`, `shark`, `whale`,
`fox`, `tiger`, `horse`, `hawk`, `panther`, `coyote`, `duck`, `blue-jay`,
`flame` (abstract), `kraken` (tentacles)

---

## `proof figures .` — figure catalog

List all figures in scope with their metadata:

```bash
proof figures .
proof figures figures/nhl/
proof figures --kind logo
```

Output:
```
figures/nhl/edm-logo.md#edm-logo:0
  label:    EDM OILERS logo
  kind:     figure.logo
  size:     40×20
  pinned:   yes (edm-logo, protection=error)
  invariants: box-count min=1, contains-text "EDMONTON"
  included by: slides/team-edm.slides.source.md:14

figures/animals/bear-stop.md#bear-stop:0
  label:    Bear in stop-sign octagon
  kind:     figure.illustration
  size:     20×10
  pinned:   no
  included by: (none)
```

---

## Figure kinds

The `kind` attribute on `<!-- proof:figure -->` classifies the figure for
validation and spec-generate suggestions:

| Kind | Validation | spec-generate suggests |
|------|-----------|----------------------|
| `figure.logo` | box-count, brand text | contains-text, box-width range |
| `figure.flowchart` | connector grammar, box alignment | box-count, arrow count |
| `figure.illustration` | line count range | line-count min/max |
| `figure.diagram` | box alignment (via ascii_box) | box-count, col-count |
| `figure.portrait` | line count, aspect ratio | line-count, box-width |
| `figure.symbol` | single box or no boxes | line-count max=N |

---

## DaVinci pinning for figures

Generated figures can be pinned with `proof pin` or `proof spec-generate`:

```bash
# Generate and immediately pin
proof figure import logos/EDM.png --id edm-logo --width 40 | \
    proof pin md://figures/edm-logo.md#edm-logo:0 --id edm-logo --protection error

# Or: generate spec, review, then pin
proof spec-generate md://figures/edm-logo.md#edm-logo:0 --id edm-logo
```

Typical invariants for a logo figure:
```toml
[[davinci]]
id = "edm-logo"
uri = "md://figures/nhl/edm-logo.md#edm-logo:0"
protection = "error"

  [[davinci.invariant]]
  rule = "line-count"
  min = 8
  max = 22

  [[davinci.invariant]]
  rule = "contains-text"
  value = "EDM"

  [[davinci.invariant]]
  rule = "box-width"
  min = 16
  max = 44
```

---

## Integration with proof compile

### Cache key

**Cache key:** The compile cache key for a figure include must incorporate directive attributes that affect output: `{resolve_key_of_source, dither, width, height, shape, label, invert, contrast, gamma}`. Changing any of these attributes produces a cache miss. Directive attributes are hashed alongside the source resolve_key before computing the compile_key — this is an extension to the standard three-tier model in COMPILE-SPEC.md.

In `.source.md` files, use `proof:include kind=figure` to embed a named figure:

```
```proof:include kind=figure
md://figures/nhl/edm-logo.md#edm-logo:0
```
```

Use `proof:include kind=figure` to embed a named figure. This is the preferred form — it triggers `FIGURE-005` (no DaVinci pin warning) when the `kind=figure` attribute is present, while plain `proof:include` without `kind=` does not. Authors using `proof:include` without `kind=figure` will not receive the pin reminder.

---

## Integration with proof layout

Side-by-side figures:

```bash
proof layout \
    "md://figures/nhl/edm-logo.md#edm-logo:0" \
    "md://figures/nhl/cgy-logo.md#cgy-logo:0" \
    --gap 6 \
    --labels "Edmonton" "Calgary"
```

---

## NHL logo generation

IceLines ships 32 team logo figures generated via:

```bash
# Generate all 32 NHL team badges from team data
proof figure generate --kind logo-badge \
    --source md://data/nhl-teams.md#teams:table:0 \
    --text-field abbrev \
    --subtitle-field city \
    --shape shield \
    --width 20 \
    --output-dir figures/nhl/
```

Source table (`nhl-teams.md`):
```markdown
| abbrev | city | team | colors |
|--------|------|------|--------|
| EDM | Edmonton | Oilers | navy,orange |
| CGY | Calgary | Flames | red,yellow |
| VAN | Vancouver | Canucks | blue,green |
...
```

Each team gets a `figures/nhl/{abbrev}-logo.md` with a `proof:figure id="{abbrev}-logo"` block.

**Filename sanitization:** The `--text-field` value (e.g., `abbrev`) is used as the output filename stem. proof sanitizes it: lowercase, spaces → hyphens, non-alphanumeric (except `-_`) → stripped. `EDM` → `edm-logo.md`, `St. Louis` → `st-louis-logo.md`. The sanitized stem is also used as the figure `id`. Authors should verify sanitized names are unique across the team table.

---

## CLI summary

```bash
proof figure import <image>          # convert image to ASCII figure
proof figure generate --kind <kind>  # generate from template
proof figures [path]                 # list/catalog figures in scope
proof figure preview <uri>           # show figure in terminal
```

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `FIGURE-001` | error | Image file not found or unreadable |
| `FIGURE-002` | warning | Image aspect ratio significantly changed by width/height override |
| `FIGURE-003` | error | `--shape` clip produced empty output (image too small for shape) |
| `FIGURE-004` | warning | `--dither braille` used — requires terminal braille font support |
| `FIGURE-005` | warning | Figure has no DaVinci pin — use `proof pin` to protect it |
| `FIGURE-006` | error | `--allow-fetch` required for remote image URL |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/figure/mod.rs` | Figure catalog, file indexing |
| `src/figure/import.rs` | Image → ASCII conversion engine |
| `src/figure/dither.rs` | Dither algorithms (block, braille, half-block, edge) |
| `src/figure/shape.rs` | Geometric clipping masks |
| `src/figure/generate.rs` | Template-based figure generation (animals, badges) |
| `src/commands/figure.rs` | CLI surface (import, generate, figures catalog) |

### Rust dependencies needed

| Crate | Purpose |
|-------|---------|
| `image` | PNG/JPG/GIF/BMP loading, resizing, grayscale |
| `resvg` | SVG rasterization |
| `unicode-width` | Already present — used for output width measurement |

---

## See also

- [Symbol Spec](./symbol-spec.md) — single chars and small shapes (< 5 lines)
- [Compile Spec](./compile-spec.md) — proof:figure directive, DaVinci validation
- [Layout Spec](./layout-spec.md) — side-by-side figure composition
- [Slide Spec](./slide-spec.md) — figures in slide body
- [Dashboard Spec](./dashboard-spec.md) — figures in dashboard regions
