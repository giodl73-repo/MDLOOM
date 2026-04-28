---
width: 80
height: 24
theme: minimal
---

```proof:slide layout=title
title: "proof Slides"
subtitle: "ASCII presentations with proof:slide"
author: "proof guide"
date: "2026"
```

---

```proof:slide layout=section
title: "Slide Layouts"
subtitle: "Seven built-in layouts"
```

---

```proof:slide layout=agenda
title: "Agenda"
```

---

```proof:slide layout=title-content
title: "title-content"
---
The most common layout. One title zone at the top,
one body zone below. The body accepts any proof: directives.

proof:bullets
- Clean separation between title and content
- Body supports proof:bullets, proof:callout, proof:divider
- Inline $\alpha$, $\beta$ math works in body text
- [sym:checkmark] Symbol expansion works too
```

---

```proof:slide layout=two-column ratio=50:50
title: "two-column"
---
LEFT COLUMN

proof:bullets
- Left zone content
- Use for comparisons
- Or before/after

---

RIGHT COLUMN

proof:bullets
- Right zone content
- Same height as left
- Ratio is configurable
```

---

```proof:slide layout=title-content
title: "agenda — auto-generated from sections"
---
The agenda layout scans the deck for every layout=section slide
and renders their titles as a numbered list. No body content needed:
the bullets come from the deck itself, so reordering or renaming
sections updates the agenda automatically.

proof:bullets
- Drop ```proof:slide layout=agenda``` anywhere — typically right after the title
- Title defaults to "Agenda" when the front-matter omits one
- Section slides keep their normal centered rendering
- Empty deck shows "(no section slides in this deck)"
```

---

```proof:slide layout=section
title: "Body Directives"
subtitle: "proof:bullets · proof:callout · proof:divider · proof:quote"
```

---

```proof:slide layout=title-content
title: "proof:bullets"
---
proof:bullets
- First level bullet
  - Nested level two
    - Level three nesting
- Back to level one
- [sym:checkmark] Symbols in bullets
- Math in bullets: $E = mc^2$
- Wide content wraps at slide width
```

---

```proof:slide layout=title-content
title: "proof:callout"
---
proof:callout style=info
This is an info callout. Use for tips, notes, and asides.
The callout box is drawn with rounded corners.

proof:callout style=warning
This is a warning callout. Use for cautions and gotchas.

proof:callout style=error
This is an error callout. Use for critical information.
```

---

```proof:slide layout=title-content
title: "proof:divider and proof:quote"
---
proof:divider style=thin

proof:quote attribution="Donald Knuth"
Premature optimization is the root of all evil.

proof:divider style=thick

proof:centered
Centered text is centered.
```

---

```proof:slide layout=stats
title: "proof:stats — KPI Slide"
---
proof:stat label="Tests" value="626" delta="+147"
proof:stat label="Modules" value="17" delta="+1"
proof:stat label="LOC" value="~8,000" delta=""
proof:stat label="Coverage" value="high" delta=""
```

---

```proof:slide layout=section
title: "Math in Slides"
subtitle: "Inline $...$ expansion in all text zones"
```

---

```proof:slide layout=title-content
title: "Inline Math"
---
Inline math works everywhere in slide body:

$\alpha + \beta = \gamma$ — Greek letters expand.

$x^2 + y^2 = z^2$ — Superscripts render as Unicode.

$\forall \epsilon > 0, \exists \delta > 0$ — Logic symbols.

$\nabla \times B = \mu_0 J$ — Maxwell's equation.

proof:divider style=thin

For multi-line math, use proof:math in a separate document.
```

---

```proof:slide layout=blank
title: ""
---
      ╔═══════════════════════════════════════════╗
      ║                                           ║
      ║   proof:slide layout=blank                ║
      ║                                           ║
      ║   The blank layout gives you a full       ║
      ║   canvas — no chrome, no header.          ║
      ║   Draw whatever you want.                 ║
      ║                                           ║
      ╚═══════════════════════════════════════════╝
```

---

```proof:slide layout=title
title: "Slide Attributes"
subtitle: "width · height · theme · show-numbers"
```

---

```proof:slide layout=title-content
title: "proof.toml for Slides"
---
Configure slide defaults in proof.toml:

proof:bullets
- width: output width in characters (default: 120)
- height: output height in lines (default: 34)
- theme: minimal | box | none
- show-numbers: true | false

Per-slide overrides go in the fence header:

```proof:slide layout=title width=60 height=15 theme=box
title: "Narrow slide"
```
```

---

```proof:slide layout=section
title: "New Directives"
subtitle: "proof:right · proof:numbered-list · proof:toc · word-wrap"
```

---

```proof:slide layout=title-content
title: "proof:right — Right-align text"
---
Mirror of proof:centered: each line is padded with leading spaces so it
ends at the slide width. Reach for it when content visually belongs at
the right margin — author bylines, dates, page numbers, citations
under a quote, or a stat that anchors the eye to the trailing edge.
Stack with proof:centered or left-flush prose to build a balanced
header or footer band without dropping into a two-column layout.

proof:right
Author: Gio Della-Libera
Date: 2026-04-28
```

---

```proof:slide layout=title-content
title: "proof:numbered-list — Ordered (numbered) list"
---
Use proof:numbered-list (short-form: proof:ol) when sequence matters —
install steps, runbook procedures, ranked priorities, anything the
reader is meant to follow in order. Indented children get decimal
sub-numbering (1.1, 1.2, 2.1) so cross-references stay stable as the
list grows. Reach for proof:bullets instead when the items are peers
with no implied order; switching to proof:numbered-list is the
cheapest way to signal "do these in this sequence."

proof:numbered-list
- Install proof
  - Clone the repo
  - Run cargo build
- Configure proof.toml
  - Set source_dir and output_dir
- Run proof compile
```

---

```proof:slide layout=title-content
title: "proof:toc — Table of Contents"
---
Lift a navigation slide straight from the heading structure of any
markdown source — the current deck or any md:// reference. Use it as
an opening agenda, a section divider in long decks, or a recap before
Q&A. Headings stay the single source of truth: rename a section in
prose and the TOC follows, no manual sync. Pick `tree` when nesting
matters, `numbered` when you want to call out "we are here on item 3,"
and `list` (default) for a flat agenda. Use `section="API Reference"`
to scope the TOC to one subsection — only the descendants of that
heading appear, perfect for a per-section mini-TOC at the top of a
long chapter.

proof:bullets
- style=list: - heading bullet list
- style=tree: └── tree connectors
- style=numbered: 1. decimal numbering
- section="…": only descendants of that heading
```

---

```proof:slide layout=title-content
title: "Word wrap"
---
Long sentences used to fall off the right edge — the renderer now
breaks at word boundaries instead. Bullets keep a hanging indent so
wrapped text stays aligned past the marker, and prose paragraphs wrap
to the available width inside any layout zone (full body, two-column
half, callout). Write naturally; reach for explicit line breaks only
when you want them.

proof:bullets
- Short bullet
- This is a longer bullet that will wrap onto the next line if it exceeds the slide width, keeping the hanging indent aligned
```

---

```proof:slide layout=title
title: "End"
subtitle: "See also: elements.md · math.md · dashboard.md"
```
