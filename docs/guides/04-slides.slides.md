<!-- mdloom:compiled from="mdloom:slides" count=34 -->
```slides
SLIDE 1 ─────────────────────────────────────────────────────────────────────── 1/34









                                  mdloom Slides
                      ASCII presentations with mdloom:slide

                                  mdloom guide
                                      2026










SLIDE 2 ─────────────────────────────────────────────────────────────────────── 2/34










                              ── Slide Layouts ──

                              Six built-in layouts











SLIDE 3 ─────────────────────────────────────────────────────────────────────── 3/34
title-content


────────────────────────────────────────────────────────────────────────────────




















SLIDE 4 ─────────────────────────────────────────────────────────────────────── 4/34



────────────────────────────────────────────────────────────────────────────────
The most common layout. One title zone at the top,
one body zone below. The body accepts any mdloom: directives.

● Clean separation between title and content
● Body supports mdloom:bullets, mdloom:callout, mdloom:divider
● Inline $\alpha$, $\beta$ math works in body text
● [sym:checkmark] Symbol expansion works too
● ```












SLIDE 5 ─────────────────────────────────────────────────────────────────────── 5/34
two-column
────────────────────────────────────────────────────────────────────────────────






















SLIDE 6 ─────────────────────────────────────────────────────────────────────── 6/34



────────────────────────────────────────────────────────────────────────────────
LEFT COLUMN

● Left zone content
● Use for comparisons
● Or before/after















SLIDE 7 ─────────────────────────────────────────────────────────────────────── 7/34



────────────────────────────────────────────────────────────────────────────────
RIGHT COLUMN

● Right zone content
● Same height as left
● Ratio is configurable
● ```














SLIDE 8 ─────────────────────────────────────────────────────────────────────── 8/34










                             ── Body Directives ──

          mdloom:bullets · mdloom:callout · mdloom:divider · mdloom:quote











SLIDE 9 ─────────────────────────────────────────────────────────────────────── 9/34
mdloom:bullets


────────────────────────────────────────────────────────────────────────────────




















SLIDE 10 ────────────────────────────────────────────────────────────────────── 10/34



────────────────────────────────────────────────────────────────────────────────
● First level bullet
  ◦ Nested level two
    ▸ Level three nesting
● Back to level one
● [sym:checkmark] Symbols in bullets
● Math in bullets: $E = mc^2$
● Wide content wraps at slide width
● ```












SLIDE 11 ────────────────────────────────────────────────────────────────────── 11/34
mdloom:callout


────────────────────────────────────────────────────────────────────────────────




















SLIDE 12 ────────────────────────────────────────────────────────────────────── 12/34



────────────────────────────────────────────────────────────────────────────────
ℹ This is an info callout. Use for tips, notes, and asides.
  The callout box is drawn with rounded corners.

⚠ This is a warning callout. Use for cautions and gotchas.

◆ This is an error callout. Use for critical information.
  ```













SLIDE 13 ────────────────────────────────────────────────────────────────────── 13/34
mdloom:divider and mdloom:quote


────────────────────────────────────────────────────────────────────────────────




















SLIDE 14 ────────────────────────────────────────────────────────────────────── 14/34



────────────────────────────────────────────────────────────────────────────────
────────────────────────────────────────────────────────────────────────────────

               “Premature optimization is the root of all evil.”
                                 — Donald Knuth

────────────────────────────────────────────────────────────────────────────────

                           Centered text is centered.
                                      ```











SLIDE 15 ────────────────────────────────────────────────────────────────────── 15/34
























SLIDE 16 ────────────────────────────────────────────────────────────────────── 16/34



────────────────────────────────────────────────────────────────────────────────
mdloom:stat label="Tests" value="626" delta="+147"
mdloom:stat label="Modules" value="17" delta="+1"
mdloom:stat label="LOC" value="~8,000" delta=""
mdloom:stat label="Coverage" value="high" delta=""
```















SLIDE 17 ────────────────────────────────────────────────────────────────────── 17/34










                              ── Math in Slides ──

                    Inline $...$ expansion in all text zones











SLIDE 18 ────────────────────────────────────────────────────────────────────── 18/34
Inline Math


────────────────────────────────────────────────────────────────────────────────




















SLIDE 19 ────────────────────────────────────────────────────────────────────── 19/34



────────────────────────────────────────────────────────────────────────────────
Inline math works everywhere in slide body:

α + β = γ — Greek letters expand.

x² + y² = z² — Superscripts render as Unicode.

∀ ε > 0, ∃ δ > 0 — Logic symbols.

∇ × B = μ₀ J — Maxwell's equation.

────────────────────────────────────────────────────────────────────────────────

For multi-line math, use mdloom:math in a separate document.
```






SLIDE 20 ────────────────────────────────────────────────────────────────────── 20/34
























SLIDE 21 ────────────────────────────────────────────────────────────────────── 21/34



────────────────────────────────────────────────────────────────────────────────
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










SLIDE 22 ────────────────────────────────────────────────────────────────────── 22/34











                                Slide Attributes
                     width · height · theme · show-numbers











SLIDE 23 ────────────────────────────────────────────────────────────────────── 23/34
mdloom.toml for Slides


────────────────────────────────────────────────────────────────────────────────




















SLIDE 24 ────────────────────────────────────────────────────────────────────── 24/34



────────────────────────────────────────────────────────────────────────────────
Configure slide defaults in mdloom.toml:

● width: output width in characters (default: 120)
● height: output height in lines (default: 34)
● theme: minimal | box | none
● show-numbers: true | false

Per-slide overrides go in the fence header:

```mdloom:slide layout=title width=60 height=15 theme=box
title: "Narrow slide"
```
```







SLIDE 25 ────────────────────────────────────────────────────────────────────── 25/34










                              ── New Directives ──

                 mdloom:right · mdloom:ol · mdloom:toc · word-wrap











SLIDE 26 ────────────────────────────────────────────────────────────────────── 26/34
mdloom:right — Right-align text


────────────────────────────────────────────────────────────────────────────────




















SLIDE 27 ────────────────────────────────────────────────────────────────────── 27/34



────────────────────────────────────────────────────────────────────────────────
Right-aligned text works like centered text, but pushes to the right edge.
Use it for dates, authors, page numbers, or visual balance.

                                                        Author: Gio Della-Libera
                                                                Date: 2026-04-28
                                                                             ```














SLIDE 28 ────────────────────────────────────────────────────────────────────── 28/34
mdloom:ol — Ordered (numbered) list


────────────────────────────────────────────────────────────────────────────────




















SLIDE 29 ────────────────────────────────────────────────────────────────────── 29/34



────────────────────────────────────────────────────────────────────────────────
Numbered lists use decimal sub-numbering automatically.

1. Install mdloom
  1.1. Clone the repo
  1.2. Run cargo build
2. Configure mdloom.toml
  2.1. Set source_dir and output_dir
3. Run mdloom compile
4. ```











SLIDE 30 ────────────────────────────────────────────────────────────────────── 30/34
mdloom:toc — Table of Contents


────────────────────────────────────────────────────────────────────────────────




















SLIDE 31 ────────────────────────────────────────────────────────────────────── 31/34



────────────────────────────────────────────────────────────────────────────────
Generates a TOC from headings in the current file or any md:// source.
Styles: list (default), tree, numbered.

● style=list: - heading bullet list
● style=tree: └── tree connectors
● style=numbered: 1. decimal numbering
● ```













SLIDE 32 ────────────────────────────────────────────────────────────────────── 32/34
Word wrap


────────────────────────────────────────────────────────────────────────────────




















SLIDE 33 ────────────────────────────────────────────────────────────────────── 33/34



────────────────────────────────────────────────────────────────────────────────
Prose lines in slide bodies now wrap automatically at the slide width
rather than being clipped. Bullet text wraps with a hanging indent —
continuation lines align past the bullet character so the visual
structure stays clean even on long descriptions.

● Short bullet
● This is a longer bullet that will wrap onto the next line if it exceeds the
  slide width, keeping the hanging indent aligned
● ```











SLIDE 34 ────────────────────────────────────────────────────────────────────── 34/34











                                      End
                 See also: elements.md · math.md · dashboard.md











```
<!-- /mdloom:compiled -->
