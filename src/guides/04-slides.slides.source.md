---
width: 80
height: 24
theme: minimal
---

```mdloom:slide layout=title
title: "mdloom Slides"
subtitle: "ASCII presentations with mdloom:slide"
author: "mdloom guide"
date: "2026"
```

---

```mdloom:slide layout=section
title: "Slide Layouts"
subtitle: "Seven built-in layouts"
```

---

```mdloom:slide layout=agenda
title: "Agenda"
```

---

```mdloom:slide layout=title-content
title: "title-content"
---
The most common layout. One title zone at the top,
one body zone below. The body accepts any mdloom: directives.

mdloom:bullets
- Clean separation between title and content
- Body supports mdloom:bullets, mdloom:callout, mdloom:divider
- Inline $\alpha$, $\beta$ math works in body text
- [sym:checkmark] Symbol expansion works too
```

---

```mdloom:slide layout=two-column ratio=50:50
title: "two-column"
---
LEFT COLUMN

mdloom:bullets
- Left zone content
- Use for comparisons
- Or before/after

---

RIGHT COLUMN

mdloom:bullets
- Right zone content
- Same height as left
- Ratio is configurable
```

---

```mdloom:slide layout=title-content
title: "agenda — auto-generated from sections"
---
The agenda layout scans the deck for every layout=section slide
and renders their titles as a numbered list. No body content needed:
the bullets come from the deck itself, so reordering or renaming
sections updates the agenda automatically.

mdloom:bullets
- Drop ```mdloom:slide layout=agenda``` anywhere — typically right after the title
- Title defaults to "Agenda" when the front-matter omits one
- Section slides keep their normal centered rendering
- Empty deck shows "(no section slides in this deck)"
```

---

```mdloom:slide layout=section
title: "Body Directives"
subtitle: "mdloom:bullets · mdloom:callout · mdloom:divider · mdloom:quote"
```

---

```mdloom:slide layout=title-content
title: "mdloom:bullets"
---
mdloom:bullets
- First level bullet
  - Nested level two
    - Level three nesting
- Back to level one
- [sym:checkmark] Symbols in bullets
- Math in bullets: $E = mc^2$
- Wide content wraps at slide width
```

---

```mdloom:slide layout=title-content
title: "mdloom:callout"
---
mdloom:callout style=info
This is an info callout. Use for tips, notes, and asides.
The callout box is drawn with rounded corners.

mdloom:callout style=warning
This is a warning callout. Use for cautions and gotchas.

mdloom:callout style=error
This is an error callout. Use for critical information.
```

---

```mdloom:slide layout=title-content
title: "mdloom:divider and mdloom:quote"
---
mdloom:divider style=thin

mdloom:quote attribution="Donald Knuth"
Premature optimization is the root of all evil.

mdloom:divider style=thick

mdloom:centered
Centered text is centered.
```

---

```mdloom:slide layout=stats
title: "mdloom:stats — KPI Slide"
---
mdloom:stat label="Tests" value="626" delta="+147"
mdloom:stat label="Modules" value="17" delta="+1"
mdloom:stat label="LOC" value="~8,000" delta=""
mdloom:stat label="Coverage" value="high" delta=""
```

---

```mdloom:slide layout=section
title: "Math in Slides"
subtitle: "Inline $...$ expansion in all text zones"
```

---

```mdloom:slide layout=title-content
title: "Inline Math"
---
Inline math works everywhere in slide body:

$\alpha + \beta = \gamma$ — Greek letters expand.

$x^2 + y^2 = z^2$ — Superscripts render as Unicode.

$\forall \epsilon > 0, \exists \delta > 0$ — Logic symbols.

$\nabla \times B = \mu_0 J$ — Maxwell's equation.

mdloom:divider style=thin

For multi-line math, use mdloom:math in a separate document.
```

---

```mdloom:slide layout=blank
title: ""
---
      ╔═══════════════════════════════════════════╗
      ║                                           ║
      ║   mdloom:slide layout=blank                ║
      ║                                           ║
      ║   The blank layout gives you a full       ║
      ║   canvas — no chrome, no header.          ║
      ║   Draw whatever you want.                 ║
      ║                                           ║
      ╚═══════════════════════════════════════════╝
```

---

```mdloom:slide layout=title
title: "Slide Attributes"
subtitle: "width · height · theme · show-numbers"
```

---

```mdloom:slide layout=title-content
title: "mdloom.toml for Slides"
---
Configure slide defaults in mdloom.toml:

mdloom:bullets
- width: output width in characters (default: 120)
- height: output height in lines (default: 34)
- theme: minimal | box | none
- show-numbers: true | false

Per-slide overrides go in the fence header:

```mdloom:slide layout=title width=60 height=15 theme=box
title: "Narrow slide"
```
```

---

```mdloom:slide layout=section
title: "New Directives"
subtitle: "mdloom:right · mdloom:numbered-list · mdloom:toc · word-wrap"
```

---

```mdloom:slide layout=title-content
title: "mdloom:right — Right-align text"
---
Mirror of mdloom:centered: each line is padded with leading spaces so it
ends at the slide width. Reach for it when content visually belongs at
the right margin — author bylines, dates, page numbers, citations
under a quote, or a stat that anchors the eye to the trailing edge.
Stack with mdloom:centered or left-flush prose to build a balanced
header or footer band without dropping into a two-column layout.

mdloom:right
Author: Gio Della-Libera
Date: 2026-04-28
```

---

```mdloom:slide layout=title-content
title: "mdloom:numbered-list — Ordered (numbered) list"
---
Use mdloom:numbered-list (short-form: mdloom:ol) when sequence matters —
install steps, runbook procedures, ranked priorities, anything the
reader is meant to follow in order. Indented children get decimal
sub-numbering (1.1, 1.2, 2.1) so cross-references stay stable as the
list grows. Reach for mdloom:bullets instead when the items are peers
with no implied order; switching to mdloom:numbered-list is the
cheapest way to signal "do these in this sequence."

mdloom:numbered-list
- Install mdloom
  - Clone the repo
  - Run cargo build
- Configure mdloom.toml
  - Set source_dir and output_dir
- Run mdloom compile
```

---

```mdloom:slide layout=title-content
title: "mdloom:toc — Table of Contents"
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

mdloom:bullets
- style=list: - heading bullet list
- style=tree: └── tree connectors
- style=numbered: 1. decimal numbering
- section="…": only descendants of that heading
```

---

```mdloom:slide layout=title-content
title: "Word wrap"
---
Long sentences used to fall off the right edge — the renderer now
breaks at word boundaries instead. Bullets keep a hanging indent so
wrapped text stays aligned past the marker, and prose paragraphs wrap
to the available width inside any layout zone (full body, two-column
half, callout). Write naturally; reach for explicit line breaks only
when you want them.

mdloom:bullets
- Short bullet
- This is a longer bullet that will wrap onto the next line if it exceeds the slide width, keeping the hanging indent aligned
```

---

```mdloom:slide layout=title
title: "End"
subtitle: "See also: elements.md · math.md · dashboard.md"
```
