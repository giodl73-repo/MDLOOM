# proof slide — ASCII Presentation Composer

> **Status**: ✅ Implemented — `src/slide/`. All 8 layouts live: title, title-content, two-column, section, content-caption (title + body + caption strip from `subtitle:`), comparison (2×2 quadrant grid via `## q:tl/tr/bl/br` markers; axis labels deferred), stats, blank + agenda. proof:bullets, proof:ol, proof:columns, proof:quote, proof:centered, proof:right, proof:stat, proof:callout, proof:divider, proof:notes, proof:reveal all wired. Footer, progress-bar, show-numbers in front-matter. Default two-column ratio 60:40.

---

## What it is

`proof slide` compiles `.slides.source.md` files into fixed-width ASCII slide
decks. Each slide is a `width × height` canvas with **flow layout** — not
absolute positioning. Unlike dashboards (spatial, data-dense), slides are
**semantic and presentation-oriented**: they have titles, bodies, bullets,
quotes, and speaker notes.

---

## How it differs from the dashboard

| | Dashboard | Slide |
|--|-----------|-------|
| Layout model | Absolute x/y positions | Flow (title → body → footer) |
| Primary use | Data display, TUI screens | Presentations, reports |
| Key primitives | `proof:element`, `proof:row` | `proof:bullets`, `proof:columns`, `proof:quote` |
| Multiple pages | No | Yes — `---` separates slides |
| Speaker notes | No | Yes — `proof:notes` excluded from output |
| Centering | Per-element | First-class layout concept |
| Orientation | Any ratio | Landscape (16:9 typical) |

---

## Slide layouts

### 1. `title` — Opening slide

Full-slide title with optional subtitle and author. Content is vertically and
horizontally centered.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│                                                                              │
│                      EDM 2025-26 Season Preview                              │
│                   A data-driven look at the Oilers                           │
│                                                                              │
│                            Gio Della-Libera                                  │
│                            April 2026                                        │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 2. `title-content` — Title with body (default)

Title bar at top, body fills the remainder. The most common layout.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ McDavid: By the Numbers                                                      │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  • Career points per 82: 138.0 — highest in NHL history                     │
│  • 2025-26 pace: 0.94 points per shift                                      │
│  • Corsi For % at 5v5: 62.3% (top 0.1% of forwards)                        │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 3. `two-column` — Side-by-side comparison

Body split into two columns. Configurable ratio (default 60:40).

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ McDavid vs Kucherov — 2025-26                                               │
├───────────────────────────────────────┬──────────────────────────────────────┤
│ McDavid                               │ Kucherov                             │
│                                       │                                      │
│  Pts/82:   138.0  ████████████████    │  Pts/82:   130.2  ███████████████   │
│  Goals:     52    ██████████          │  Goals:     43    ████████          │
│  Assists:   86    █████████████████   │  Assists:   87    █████████████████ │
│                                       │                                      │
│  Contract: 8yr × $12.5M              │  Contract: 8yr × $11.5M             │
│  Status:   UFA 2026                   │  Status:   UFA 2026                 │
└───────────────────────────────────────┴──────────────────────────────────────┘
```

### 4. `section` — Section divider

Large title, optional subtitle. Used as a visual break between presentation sections.

**Compositor-driven:** The `section` layout automatically centers the `title` and (if provided) `subtitle` both vertically and horizontally. Authors do not use `proof:centered` — the layout renderer applies centering. This cannot be overridden within a section layout; use `blank` layout with `proof:centered` for custom alignment.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│                                                                              │
│                              ── Part 2 ──                                   │
│                                                                              │
│                            Defensive Corps                                   │
│                                                                              │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 5. `content-caption` — Content with annotation

Main content area with a smaller caption strip at the bottom.

### 6. `comparison` — 2×2 matrix

Four quadrants with labels on axes. Used for strategic matrices (2×2 grids).

### 7. `stats` — Large-number highlight

One or more large statistics with labels, centered. Used for impact statements.

**Renderer:** `stats` layout uses its own dedicated renderer (not `proof:columns`). It does not support `ratio=` or `divider=` attributes. SL-3 does not apply — column widths are computed as `floor(content_width / stat_count)` with remainders distributed to the rightmost stat. Each stat block is independently centered within its allocated width.

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Key Numbers                                                                  │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│              138.0            62.3%            $12.5M                        │
│           Pts per 82        Corsi For        Cap Hit/yr                     │
│                                                                              │
│           #1 all-time     Top 0.1% fwd     League max                      │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

### 8. `blank` — No structure

Full canvas, author places all content manually using proof: directives.

---

## Source format (`.slides.source.md`)

### Parser disambiguation rules

**`---` disambiguation:** The parser uses these rules in order:
1. If the file begins with `---` on line 1, the parser enters YAML front-matter mode and reads until the next `---` (the closer). The closer is NOT a slide separator.
2. After the front-matter closer (or immediately if the file does not begin with `---`), the parser enters slide mode. Every subsequent `---` on its own line is a slide separator.
3. Files with no front-matter: the first `---` is slide 2's separator. Slide 1 begins at line 1.

SL-7 counts `---` separators in slide mode only — the front-matter closer is excluded.

```markdown
---
slides:
  width: 120
  height: 34
  theme: minimal          # minimal | box | none
  show-numbers: true      # slide numbers in footer
  font-width: 1           # 1 = ASCII, 2 = wide-char
---

```proof:slide layout=title
title: "EDM 2025-26 Season Preview"
subtitle: "A data-driven look at the Oilers"
author: "Gio Della-Libera"
date: "April 2026"
```

---

```proof:slide layout=title-content title="McDavid: By the Numbers"
proof:bullets
- Career points per 82: 138.0 — highest in NHL history
  - Previous record: Gretzky 89.9 (1985-86)
  - Active comparison: Kucherov 130.2 (2024-25)
- 2025-26 pace: 0.94 points per shift
- Corsi For % at 5v5: 62.3% (top 0.1% of forwards)
```

---

```proof:slide layout=two-column title="McDavid vs Kucherov"
## col:left
proof:stat field=pts_82 format="{:.1}" label="Pts/82" source=md://stats.md#mcdavid[row=0]
proof:mini-bar field=pts_82 max=200 width=30
## col:right
proof:stat field=pts_82 format="{:.1}" label="Pts/82" source=md://stats.md#kucherov[row=0]
proof:mini-bar field=pts_82 max=200 width=30
```

---

```proof:slide layout=section
title: "Part 2 — Defensive Corps"
```

---
```

---

## Slide-specific directives

### `proof:bullets`

Hierarchical bullet list. Indent with 2 spaces per level.

```
proof:bullets
- Top-level point
  - Second level (◦)
    - Third level (▸)
- Another top point
```

Bullet characters (configurable): `•` (level 1), `◦` (level 2), `▸` (level 3), `–` (level 4+).

**Max bullets per slide: 4** (configurable via `max-bullets` in slide front-matter — the **30-second rule**: more than 4 bullets cannot reasonably be read aloud in the typical 30-second slide pacing). Authors who genuinely need more should opt in explicitly:

```yaml
---
slides:
  width: 120
  height: 34
  max-bullets: 8     # raise the threshold for this deck
---
```

When a `proof:bullets` block exceeds `max-bullets`, the compiler emits **SLIDE-001** as a warning and prepends an HTML comment to the compiled output:

```
<!-- SLIDE-WARN SLIDE-001 slide=3: bullet 5 exceeds max_bullets 4 -->
SLIDE 3 ──────────────────────────────────────── 3/12
...
```

This makes the warning visible to readers of the compiled deck, not just to authors who run `proof compile` in a terminal. SLIDE-001 is non-blocking — the deck still compiles and writes successfully.

**Counter scope:** the bullet counter is per-`proof:bullets` block, not per-slide. A two-column slide with 3 bullets in each column does not trigger SLIDE-001 at threshold 4 — each column's `proof:bullets` is counted independently.

**Depth limit:** `max-depth=4` (configurable) limits nesting. Levels beyond `max-depth` are rendered at the deepest defined bullet char. `SLIDE-007: bullet depth exceeds max-depth` (planned warning).

### `proof:quote`

Centered block quote with attribution.

```
proof:quote attribution="Connor McDavid"
I want to win. Everything else is secondary.
```

Rendered with `"` and `"` (curly quotes) and a `—` attribution line, centered in the content area.

### `proof:columns`

Splits the content area into N columns. Column bodies are written under `## col:` prefixed headings.

**Note:** Column sections use `## col:` prefix (H2 level, not H1) to avoid triggering `md_h1_count` checks. The compiler recognizes `## col:` inside a `proof:columns` or `proof:slide layout=two-column` fence as a structural delimiter, not a document heading — heading rules are suppressed for these markers.

```
```proof:slide layout=blank title="Comparison"
proof:columns cols=2 ratio=60:40 divider=true
## col:left
proof:bullets
- Strengths
- More strengths
## col:right
proof:tree kind=org source=md://team.md#:table:0
```
```

`ratio=60:40` — first column gets 60% of width, second gets 40%.
`divider=true` — draws a `│` separator between columns.

**Rounding rule:** Column widths are computed as `floor(content_width × ratio_fraction)`. Any remaining columns (due to integer rounding) are added to the **first** column. Example: 119 cols at 60:40 → floor(71.4)=71, floor(47.6)=47, remainder 1 → first column gets 72, second gets 47. This is consistent with the principle that the primary (left) column is the anchor.

### `proof:centered`

Centers content horizontally within the current region. Used for impact text.

```
proof:centered
THE BEST PLAYER IN THE WORLD
```

### `proof:stat`

Renders a large number with a label below. Can be used standalone or in a `proof:columns` for multi-stat layouts.

```
proof:stat value=138.0 label="Pts per 82" sublabel="#1 all-time" width=20
```

### `proof:callout`

Highlighted box with a style indicator. Useful for key takeaways or warnings.

```
proof:callout style=key
McDavid's contract expires June 2026 — largest free agent in NHL history.
```

Styles: `key` (`★`), `info` (`ℹ`), `warning` (`⚠`), `tip` (`→`), `note` (`◆`).

### `proof:divider`

Horizontal rule across the content width.

```
proof:divider style=thin    # ─────────────────────
proof:divider style=double  # ═════════════════════
proof:divider style=dotted  # ·····················
proof:divider style=wave    # ~~~~~~~~~~~~~~~~~~~~ (see note)
```

**Note:** `style=wave` uses `~` chars which may render as strikethrough delimiters in some markdown previewers (GFM extensions). Source files should not be previewed as raw markdown; the compiled `.slides.md` output is the canonical form. Alternative: `style=approx` uses `≈≈≈` (U+2248) instead.

### `proof:notes`

Speaker notes — rendered in a separate `notes:` section, excluded from slide output.

```
proof:notes
Talk about the contract situation here. Mention that his agent is Pat Brisson.
The comparison to Gretzky is the key talking point — use it.
```

**Check-time behavior:** When `proof check` runs on `.slides.source.md`, notes content IS linted — it passes through the full check pipeline including line-length and heading rules. This ensures notes quality for `proof compile --format notes` output. To suppress linting on notes, use `proof check --no-notes` (planned).

---

## Compilation

```bash
proof compile deck.slides.source.md
# → deck.slides.md  (all slides in one file, separated by ─── dividers)

proof compile deck.slides.source.md --slide 3
# → render only slide 3
# --slide is 1-indexed: --slide 1 is the first slide, --slide N where N > slide count emits SLIDE-006

proof compile deck.slides.source.md --width 80 --height 24
# → terminal-sized output (override front-matter dimensions)

proof compile deck.slides.source.md --format notes
# → output speaker notes only (one per slide)

proof compile deck.slides.source.md --format json
# → slides as JSON array (for programmatic consumption)
```

Output format (single compiled file with slide separators):

````markdown
<!-- proof:compiled from="proof:slides" count=5 title="EDM Preview" -->
```slides
SLIDE 1 ─────────────────────────────────────────────────── 1/5

             EDM 2025-26 Season Preview
          A data-driven look at the Oilers

                  Gio Della-Libera
                    April 2026

SLIDE 2 ─────────────────────────────────────────────────── 2/5

McDavid: By the Numbers
────────────────────────────────────────────────────────────
  • Career points per 82: 138.0 — highest in NHL history
    ◦ Previous record: Gretzky 89.9 (1985-86)
  • 2025-26 pace: 0.94 points per shift
  ...
```
<!-- /proof:compiled -->
````

---

## Theming

| Theme | Style |
|-------|-------|
| `minimal` | No borders. Title separated by `───` rule. Clean whitespace. |
| `box` | Each slide wrapped in `┌──┐ │ └──┘` border. Title in top `├──┤` band. |
| `none` | Raw content only. No chrome at all. |

---

## IceLines integration

IceLines pre-game reports and briefings use slide decks:

```bash
icelines slides team EDM --width 120 --height 34
# → compiles and streams team deck slide by slide

icelines slides player McDavid
# → player profile slide deck (6 slides)
```

Slide navigation in the TUI: `→`/`←` advances slides. `n` opens speaker notes.

---

## Invariants

| Invariant | Claim |
|-----------|-------|
| SL-1 | Each slide output is exactly `width × height` characters |
| SL-2 | `proof:bullets` level N uses the declared bullet char for that level |
| SL-3 | `proof:columns ratio=A:B` column widths sum to content width (minus divider if present) |
| SL-4 | `proof:stat` value is right-aligned within `width` |
| SL-5 | `proof:notes` content is never present in non-notes output |
| SL-6 | `proof:centered` output is horizontally centered (tie-break: extra space on right) |
| SL-7 | Slide count matches the number of `---` separators + 1 |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `SLIDE-001` | warning | Bullet list exceeds `max-bullets` — recommend splitting slide |
| `SLIDE-002` | error | Column ratios don't sum to 100 (e.g. `ratio=60:50`) |
| `SLIDE-003` | warning | Content overflows slide height — lines clipped |
| `SLIDE-004` | error | `layout=two-column` has only one `# Column` section |
| `SLIDE-005` | warning | `proof:stat` value is non-numeric |
| `SLIDE-006` | error | `--slide N` references a slide that doesn't exist |
| `SLIDE-007` | warning | Bullet depth exceeds `max-depth` (planned) |

---

## What proof needs to implement this

| Component | Status |
|-----------|--------|
| Slide parser (front-matter + `---` separators) | Planned |
| Flow layout engine (title bar + body) | Planned |
| `proof:bullets` renderer | Planned |
| `proof:columns` compositor | Planned |
| `proof:quote`, `proof:centered`, `proof:stat` | Planned |
| `proof:callout`, `proof:divider` | Planned |
| `proof:notes` extraction | Planned |
| Canvas per-slide (reuse dashboard Canvas) | Planned |
| `proof compile --slide N --format notes` flags | Planned |
| Field mapping (per MAPPING-SPEC.md) | ✅ Designed |
| `proof:chart`, `proof:tree` inside slides | ✅ Done (reuse) |

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/slide/mod.rs` | Slide deck parser, layout engine |
| `src/slide/layout.rs` | Title, two-column, section, stats layouts |
| `src/slide/bullets.rs` | Hierarchical bullet rendering |
| `src/slide/columns.rs` | N-column compositor |
| `src/slide/inline.rs` | quote, centered, stat, callout, divider |
| `src/compile.rs` | proof:slide directive handling |

---

## See also

- [Dashboard Spec](./dashboard-spec.md) — absolute-position canvas (data display, TUI)
- [Element Spec](./element-spec.md) — micro-elements used inside slide content
- [Chart Spec](./chart-spec.md) — charts embeddable in slide body
- [Tree Spec](./tree-spec.md) — trees embeddable in slide body
- [Mapping Spec](./mapping-spec.md) — field binding for data-driven slides

---

## Spec Clarifications (from scenario findings)

These clarifications resolve ambiguities surfaced during scenario testing. They are normative — implementations must conform.

### F47 — Title slide blank-row padding

In a `title` layout at `height = H`, the renderer emits all declared fields (title, subtitle, author, date) in order, then pads remaining rows with blank lines (each line is exactly `width` spaces). Total rendered lines always equals `H`, satisfying SL-1.

Fields not provided in the slide front-matter are skipped (no blank slot reserved); the remaining rows absorb the extra space.

### F48 — Overlong slide titles are clipped

If a title string's `visual_width` exceeds the slide `width`, it is clipped via `clip_to_width()` — the function appends `…` (U+2026) when truncation occurs. Slide titles are **never wrapped** to a second line; the title bar is always exactly one row.

This applies to all layouts that render a title (`title`, `title-content`, `section`, `stats`, etc.).

### F49 — Bullet indent width

Default bullet indent is `indent_width = 2` spaces per level. Configurable per slide deck via the front-matter:

```yaml
---
slides:
  width: 120
  height: 34
  indent-width: 4
---
```

`indent-width` is a slide-deck-level setting only — it is **not** currently configurable in `proof.toml`. Each level adds `indent_width` spaces of leading whitespace before the bullet character.

### F50 — Tabs in bullet source

Tab characters in bullet source lines are normalized to 2 spaces **before** indent-level detection runs. A single leading tab therefore counts as exactly one indent level (when `indent_width = 2`).

This normalization happens in the parser; downstream code never sees tabs in bullet text.

### F51 — Two-column remainder distribution

For a `proof:columns` block with `ratio=N:M` at total content width `width`:

```
left_w  = floor(width × N / (N + M))
right_w = width - left_w
```

The remainder column always goes to the **right** column.

> **Note:** This supersedes the earlier "Rounding rule" stated under `proof:columns` ("remainder added to the first column"). F51 is the canonical behavior — remainder lands on the right column. The two-column compositor and SL-3 conform to this rule.

### F52 — `---` disambiguation: column separator vs. slide separator

A literal `---` line has two meanings, distinguished by lexical context:

| Context | Meaning |
|---------|---------|
| Inside a `proof:slide` fenced block body (and the slide is `layout=two-column` or contains a `proof:columns`) | **Column separator** — splits the body into left/right columns |
| Outside any fenced block (top-level of the source file, after front-matter) | **Slide separator** — ends the current slide, starts the next |

There is no ambiguity at parse time: the parser tracks fence depth, so the same character sequence resolves deterministically by where it appears.

### F53 — Stats remainder distribution

For a `stats` layout with `n` stat cells at total content `width`:

```
cell_w   = floor(width / n)
extra    = width % n
```

The `extra` remainder columns are distributed **left-to-right** — the first `extra` cells each receive one additional column. (This differs from F51's right-biased split for two-column layouts; stats are intentionally left-biased so the lead stat gets the extra space.)

This is the renderer rule referenced in §7 ("stats layout").

### F54 — Stat overflow clipping

If a stat's value string exceeds its allocated cell width, it is clipped via `clip_to_width()` (appending `…` on truncation). Stat values are **never wrapped** inside cells. Authors who need a longer label should use a smaller stat count or a wider slide.

This also applies to stat sublabels.

### F56 — Unknown callout style fallback

`proof:callout style=<unknown>` falls back silently to `CalloutStyle::Note` (the `◆` style). No diagnostic is emitted — callout style parsing is permissive by design.

Authors who want strict validation can lint callout styles externally; the renderer will not fail or warn on unknown values.

### F59 — `---` in YAML front-matter vs. slide separators

The slide-file parser reads YAML front-matter from the **first** `---` line to the **second** `---` line. Subsequent `---` lines in slide mode are slide separators.

- The front-matter block must be at the top of the file (line 1 begins with `---`) and is terminated by exactly one closing `---`.
- The closing `---` is **not** counted as a slide separator (per the existing parser disambiguation rules and SL-7).
- Files with no front-matter: parsing starts in slide mode at line 1.

### F61 — Speaker notes have no sidecar output

Per SL-5, `proof:notes` blocks are excluded from compiled slide output. The current implementation **silently drops notes** during normal compile — no `.notes.md` sidecar is written.

To extract notes, use `proof compile --format notes` (already specified). A future flag `--notes-output <path>` is **planned** to write a sidecar file alongside the compiled deck; until shipped, notes exist only in the source file or in `--format notes` output.

### F62 — `proof check` does not currently verify SL-5

`proof check` does **not** scan compiled `.slides.md` output for leaked `proof:notes` content. SL-5 is enforced at compile time (notes are dropped before output) but there is no post-compile verification rule.

This is a known gap. A future check rule `slide_notes_leaked` would scan compiled output for any `proof:notes` markers or content blocks and emit a diagnostic. Until that rule ships, SL-5 violations would only surface as a compile-time bug, not a check-time finding.
