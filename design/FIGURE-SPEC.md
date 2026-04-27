# proof figure — Named ASCII Art Figures with Image Import

> **Status**: Design — not yet implemented (DaVinci pinning partial via proof pin).

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
| `--allow-fetch` | false | Allow fetching remote image URLs |
| `--output-file` | — | Write to file instead of stdout |

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

Available shapes: `circle`, `octagon` (stop sign), `shield` (NHL/heraldry),
`star` (5-point), `heart`, `diamond`, `hexagon`, `pentagon`, `rounded-rect`.

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

In `.source.md` files, use `proof:figure` as a directive to embed a named figure:

```
```proof:figure
md://figures/nhl/edm-logo.md#edm-logo:0
```
```

This is identical to `proof:include` — the figure is resolved, validated against
DaVinci invariants, and embedded with traceability. The distinction is semantic:
`proof:include` for any element, `proof:figure` for figures specifically (enables
`COMPILE-003` warning when figure has no DaVinci pin).

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
