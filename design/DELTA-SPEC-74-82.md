# Delta Spec — Tasks #74–#82

> Role-review findings from PRESS, PANEL, BOOK, STAGE (Round 2).  
> Covers features not yet specced or only partially specced in existing files.

---

## #74 — proof:chart compile directive

**Status in existing spec:** CHART-SPEC.md (744 lines) and CHART-IMPL-PLAN.md (789 lines)  
cover the full chart system in detail. No delta needed — read those.

**Subset to implement first (MVP for #74):**
- `kind=bar` — horizontal bar chart from a markdown table
- `kind=line` — simple line chart from x/y table
- `kind=sparkline` — promoted from inline element to standalone directive

Defer: scatter, heatmap, stacked-bar, waterfall, gantt, area, candlestick, sankey.

---

## #75 — proof check --unused

### What it does

Scans the corpus for figures (`.md` files in source directories) that are never
referenced by any `.source.md` file via `proof:include` or `proof:layout`.

Reports as a warning per unused file:

```
docs/figures/old-arch.md:1:1  warning  md_unused_figure  Figure never referenced by any source document
```

### Scope

"Figure" means any `.md` file that:
- Lives under a `source_dir` declared in `[[compile]]`
- OR ends with a figure-like pattern (configurable via `figure_dirs` in proof.toml)

Compiled `.md` output files are **not** checked — only the resolved figures that
source documents would include.

### Invocation

```bash
proof check . --unused           # includes unused figure warnings in normal check
proof check . --unused-only      # only report unused figures (suppress all other checks)
```

`--unused` is off by default (it requires a full corpus walk and is slow on large repos).

### Algorithm

1. Walk all `.source.md` files in the corpus
2. Collect every `md://` URI found in `proof:include` and `proof:layout` directives
3. Resolve each URI to an absolute file path
4. Walk all `.md` files that could be figures (source dirs, or explicit figure_dirs)
5. Report any that appear in step 4 but not in step 3

### Diagnostic code

`md_unused_figure` — severity: warning

---

## #76 — proof status

### What it does

Single-screen corpus health summary. No deep analysis — fast stat collection.

```bash
proof status [dir]
```

Output format:

```
proof status — C:\src\maxim

  Sources       2,703 files
  Compiled      2,703 files
  Stale           12 files  (source newer than output)
  Errors           0        (last proof check)
  Warnings        47        (last proof check)
  Last compile    2026-04-28 14:23 (3 hours ago)
  Config          proof.toml (root=true, 4 schemas, 2 compile targets)
```

### Implementation

- "Sources" = count of `.source.md` files under configured `source_dir` paths
- "Compiled" = count of corresponding `.md` output files that exist
- "Stale" = sources where `source.mtime > output.mtime` (or output missing)
- "Errors/Warnings" = read from a `.proof/last-check.json` cache file written by `proof check`
- "Last compile" = mtime of the newest compiled output file
- "Config" = summary of active proof.toml settings

### Cache file `.proof/last-check.json`

`proof check` writes this after every run:

```json
{
  "timestamp": "2026-04-28T14:23:00Z",
  "files_checked": 2703,
  "errors": 0,
  "warnings": 47
}
```

`proof status` reads this if present, shows "(cached)" label; omits error/warning
counts if the file doesn't exist.

---

## #77 — Slide footer

### Spec addition to SLIDE-SPEC.md

Add to front-matter:

```yaml
---
slides:
  width: 120
  height: 34
  footer: true              # show auto footer (author · date)
  footer-text: "Custom footer text"   # overrides auto content
  show-numbers: true        # already specced — slide N / M in separator header
---
```

### Footer rendering

When `footer: true`, the **bottom row** (row index `height - 1`) of every slide
canvas is reserved for footer content. The content area becomes `height - 1` rows.

Footer row format (auto): `{author}  ·  {date}  ·  {title}`
- `author` and `date` come from the deck front-matter
- `title` is the deck title (not the slide title)
- Content is left-aligned; right side shows slide number if `show-numbers: true`

When `footer-text:` is set, use that literal string instead of auto-format.

Footer row is separated from body by a thin `─` rule on the second-to-last row
when theme is `minimal` or `box`. With theme `none`, footer is just the last row.

### Invariant

**SL-8**: When `footer: true`, the slide content area is `height - 1` rows (the
footer row is not counted toward SL-1's `width × height` content requirement —
total canvas height is still `height`).

### Diagnostic

`SLIDE-008` warning: footer row overflows canvas width (footer text truncated).

---

## #78 — proof:slide layout=agenda

### What it does

Auto-generates an agenda slide from all `layout=section` slides in the deck.
The agenda lists each section title as a bullet.

```markdown
```proof:slide layout=agenda
title: "Agenda"
```
```

### Rendering

1. Parser scans all slides in the deck for `layout=section`
2. Collects their `title` values in order
3. Renders as a `title-content` layout with `proof:bullets` body listing each title

```
┌────────────────────────────────────────────────────────────────────────────────┐
│ Agenda                                                                         │
├────────────────────────────────────────────────────────────────────────────────┤
│ ● Introduction                                                                 │
│ ● Key Findings                                                                 │
│ ● Recommendations                                                              │
│ ● Next Steps                                                                   │
└────────────────────────────────────────────────────────────────────────────────┘
```

### Position matters

The agenda slide lists section slides that appear **after** it in the deck (not
before). An agenda at position 2 in a 10-slide deck lists sections from slides
3–10. Sections before the agenda are not listed.

### Diagnostic

`SLIDE-009` warning: agenda slide has no section slides after it (empty agenda).

---

## #79 — forbidden_h2

### Spec addition to SPEC.md / section_schemas

`forbidden_h2` is the complement to `required_h2_all`: it specifies H2 sections
that **must not** appear in matching files. Primary use: keeping authoring scaffolds
(`## Draft`, `## TODO`, `## WIP`) out of production guides.

```toml
[[section_schemas]]
paths = ["**/*.md"]
forbidden_h2 = ["Draft", "TODO", "WIP", "Placeholder"]
```

### Behavior

For each heading in `forbidden_h2`, if that H2 heading is found in the file
(outside code fences), emit:

```
guide.md:12:1  warning  md_forbidden_section  Section "Draft" is not allowed in production guides
```

### Config fields (already added to config.rs)

`MarkdownConfig.forbidden_h2: Vec<String>` — enforced by `MarkdownCheck`  
`SectionSchema.forbidden_h2: Vec<String>` — propagated by `apply_section_schema`

### Merge behavior

`forbidden_h2` lists are unioned across parent + child configs (additive, like
`required_h2_all`). Once a heading is forbidden at any level of the config
cascade, it is forbidden for all files that inherit that level.

---

## #80 — proof:blockquote

### What it does

A prose document block quote — the document-context counterpart to `proof:quote`
(which is slide-only and centered). `proof:blockquote` renders as a visually
indented block with a left-margin bar, suitable for inline document prose.

```markdown
```proof:blockquote attribution="Donald Knuth"
Premature optimization is the root of all evil.
```
```

### Rendered output

```
│ Premature optimization is the root of all evil.
│ — Donald Knuth
```

- Left margin: `│ ` (box-drawing vertical bar + space)
- Attribution line (if present): `│ — {attribution}`
- Multi-line text: each line prefixed with `│ `
- No centering (document context, not presentation context)

### Attributes

| Attribute | Default | Meaning |
|-----------|---------|---------|
| `attribution=` | none | Attribution text, rendered as `— {text}` on final line |
| `style=` | `bar` | `bar` (│), `indented` (4-space indent, no bar), `double` (║) |
| `width=` | inherited | Override line width for wrapping |

### Diagnostic

None. All valid inputs render.

### Distinction from proof:quote

| | `proof:quote` | `proof:blockquote` |
|-|---------------|--------------------|
| Context | Slides only | Prose documents |
| Alignment | Centered with curly quotes | Left-aligned with bar margin |
| Attribution | `— Name` centered | `│ — Name` left-aligned |

---

## #81 — Slide progress indicator

### Spec addition to SLIDE-SPEC.md

`show-numbers: true` is already in the front-matter spec. The current rendered
output includes `N/M` in the separator header between slide canvases. This is
visible in the compiled output but not in terminal slide display.

### Progress bar option

Add `progress-bar: true` to front-matter:

```yaml
---
slides:
  progress-bar: true    # renders a thin progress row after the slide separator
---
```

When enabled, a progress bar row is inserted **between the separator header and
the slide canvas content** (not inside the canvas — SL-1 still holds):

```
SLIDE 3 ─────────────────────── 3/8
████████████████████░░░░░░░░░░░  3/8
```

Bar character: `█` for completed (proportion = N/M), `░` for remaining.
Bar width = canvas `width - 5` (leaving room for ` N/M` label on the right).

### What's already there

The `N/M` label is already in the separator line. The progress bar is additive
— it does not change any existing behavior when `progress-bar` is omitted or false.

---

## #82 — two-column default ratio 60:40

### Change to SLIDE-SPEC.md

Section 3 (`two-column`) currently states:

> Body split into two columns. Configurable ratio (default 50:50).

**Delta:** Change default to `60:40`.

**Rationale:** Presentation best practice (PowerPoint, Keynote, Reveal.js defaults)
uses asymmetric splits. 50:50 signals "equal importance" which is rarely the
design intent. 60:40 gives the primary content column more weight by default.

**Backward compatibility:** `ratio=50:50` in existing source files continues to
work. Only the **default** (when no ratio is specified) changes.

---

## Summary table

| Task | Spec file | Delta type |
|------|-----------|------------|
| #74 chart | CHART-SPEC.md + CHART-IMPL-PLAN.md | Read existing — no delta needed |
| #75 --unused | This file (new) | New feature |
| #76 proof status | This file (new) | New command |
| #77 slide footer | SLIDE-SPEC.md | Additive section |
| #78 agenda slide | SLIDE-SPEC.md | New layout |
| #79 forbidden_h2 | SPEC.md | Additive config field |
| #80 proof:blockquote | This file (new) | New directive |
| #81 progress indicator | SLIDE-SPEC.md | Additive front-matter option |
| #82 two-column ratio | SLIDE-SPEC.md | Default value change |
