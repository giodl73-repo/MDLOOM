# proof slide — ASCII Presentation Composer

> **Status**: Design — not yet implemented.

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

Body split into two columns. Configurable ratio (default 50:50).

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

```proof:slide layout=two-column title="McDavid vs Kucherov" ratio=50:50
# Left
proof:stat field=pts_82 format="{:.1}" label="Pts/82" source=md://stats.md#mcdavid[row=0]
proof:mini-bar field=pts_82 max=200 width=30
# Right
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

Max recommended bullets per slide: 6 (configurable via `max-bullets` in slide front-matter).

### `proof:quote`

Centered block quote with attribution.

```
proof:quote attribution="Connor McDavid"
I want to win. Everything else is secondary.
```

Rendered with `"` and `"` (curly quotes) and a `—` attribution line, centered in the content area.

### `proof:columns`

Splits the content area into N columns. Column bodies are written under `# Column N` headings.

```
```proof:slide layout=blank title="Comparison"
proof:columns cols=2 ratio=60:40 divider=true
# Column 1
proof:bullets
- Strengths
- More strengths
# Column 2
proof:tree kind=org source=md://team.md#:table:0
```
```

`ratio=60:40` — first column gets 60% of width, second gets 40%.
`divider=true` — draws a `│` separator between columns.

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
```

### `proof:notes`

Speaker notes — rendered in a separate `notes:` section, excluded from slide output.

```
proof:notes
Talk about the contract situation here. Mention that his agent is Pat Brisson.
The comparison to Gretzky is the key talking point — use it.
```

---

## Compilation

```bash
proof compile deck.slides.source.md
# → deck.slides.md  (all slides in one file, separated by ─── dividers)

proof compile deck.slides.source.md --slide 3
# → render only slide 3

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
