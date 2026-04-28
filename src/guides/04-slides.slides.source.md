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
subtitle: "Six built-in layouts"
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
subtitle: "proof:right · proof:ol · proof:toc · word-wrap"
```

---

```proof:slide layout=title-content
title: "proof:right — Right-align text"
---
Right-aligned text works like centered text, but pushes to the right edge.
Use it for dates, authors, page numbers, or visual balance.

proof:right
Author: Gio Della-Libera
Date: 2026-04-28
```

---

```proof:slide layout=title-content
title: "proof:ol — Ordered (numbered) list"
---
Numbered lists use decimal sub-numbering automatically.

proof:ol
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
Generates a TOC from headings in the current file or any md:// source.
Styles: list (default), tree, numbered.

proof:bullets
- style=list: - heading bullet list
- style=tree: └── tree connectors
- style=numbered: 1. decimal numbering
```

---

```proof:slide layout=title-content
title: "Word wrap"
---
Prose lines in slide bodies now wrap automatically at the slide width
rather than being clipped. Bullet text wraps with a hanging indent —
continuation lines align past the bullet character so the visual
structure stays clean even on long descriptions.

proof:bullets
- Short bullet
- This is a longer bullet that will wrap onto the next line if it exceeds the slide width, keeping the hanging indent aligned
```

---

```proof:slide layout=title
title: "End"
subtitle: "See also: elements.md · math.md · dashboard.md"
```
