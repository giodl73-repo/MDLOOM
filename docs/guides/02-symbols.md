# proof Symbols — Named Glyph Library

The symbol system gives you named Unicode glyphs that expand anywhere in prose,
bullets, slide content, and element labels. The key insight is that symbols are
**semantic** — you write `[sym:checkmark]` and get `✓`, rather than pasting
Unicode characters that are invisible in source and fragile to copy/paste.

There are three levels: inline expansion for prose, block rendering for larger
decorative output, and the shape renderer for geometric ASCII art. Use inline
for status indicators and annotations; use blocks and shapes when the symbol
itself is the content.

---

## Inline symbol expansion

Inline expansion is the most common use. Write `[sym:name]` anywhere in prose
and proof replaces it with the Unicode glyph at compile time. This works in
paragraphs, bullet labels, callout text, and slide titles — anywhere text is
rendered.

Use inline symbols for status indicators in documentation, rating systems, and
visual emphasis that would be lost with plain ASCII. They're especially useful
in bullet lists where you want a visual left-edge that isn't just `-`.

### Status symbols

[sym:checkmark] done — [sym:cross] failed — [sym:warning] caution — [sym:info] note

### Directional arrows

[sym:arrow-right] next — [sym:arrow-left] back — [sym:arrow-up] up — [sym:arrow-down] down

### Rating stars

[sym:star] [sym:star] [sym:star] [sym:star-empty] [sym:star-empty] — 3 out of 5

### Shapes inline

[sym:circle-filled] active — [sym:square-filled] blocked — [sym:diamond] pending

---

## Symbol blocks

When the symbol needs to be large — a section header icon, a status badge in a
dashboard region, or a decorative element — use `proof:symbol`. The `size`
parameter scales the symbol from 1 (compact) to 5+ (display-sized).

<!-- proof:compiled from="proof:symbol" name="checkmark" size="3" -->
```
✓✓✓✓✓
✓✓✓✓✓
✓✓✓✓✓
✓✓✓✓✓
✓✓✓✓✓
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:symbol" name="star" size="5" -->
```
★★★★★★★★★
★★★★★★★★★
★★★★★★★★★
★★★★★★★★★
★★★★★★★★★
★★★★★★★★★
★★★★★★★★★
★★★★★★★★★
★★★★★★★★★
```
<!-- /proof:compiled -->

---

## Shape renderer

`proof:shape` generates geometric ASCII art shapes with optional labels. Use
shapes to create visual frames for content, highlight important sections, or
build visual identity in dashboards. The available shapes are banner (wide
rectangle), badge (compact), and ribbon (angled).

A labeled banner works well as a section divider in a dashboard or as a callout
header in slides.

<!-- proof:compiled from="proof:shape" name="banner" -->
```
╔══════════════════════════╗
║                          ║
╚══════════════════════════╝
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:shape" name="badge" -->
```
 ╭─────────╮
 │ COMPILE │
 ╰─────────╯
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:shape" name="ribbon" -->
```
   ╱‾‾‾‾‾‾╲
  ╱      ╲
 ╱______╲
```
<!-- /proof:compiled -->

---

## Symbol catalog

The full built-in library. All names are lowercase with hyphens. The `aliases`
column lists alternate names that resolve to the same glyph.

<!-- proof:compiled from="proof:row" uri="md://src/data/symbol-catalog.md" -->
```
checkmark            │ ✓        │ status       │ check, done, tick             
cross                │ ✗        │ status       │ x, no, fail                   
star                 │ ★        │ rating       │ star-filled                   
star-empty           │ ☆        │ rating       │ star-outline                  
bullet               │ •        │ list         │ dot                           
arrow-right          │ →        │ direction    │ right                         
arrow-left           │ ←        │ direction    │ left                          
arrow-up             │ ↑        │ direction    │ up                            
arrow-down           │ ↓        │ direction    │ down                          
warning              │ ⚠        │ status       │ warn, alert                   
info                 │ ℹ        │ status       │ information                   
note                 │ ✎        │ annotation   │ pencil, edit                  
pin                  │ 📌       │ annotation   │ pinned                        
fire                 │ 🔥       │ emphasis     │ hot                           
circle               │ ○        │ shape        │ empty-circle                  
circle-filled        │ ●        │ shape        │ filled-circle                 
square               │ □        │ shape        │ empty-square                  
square-filled        │ ■        │ shape        │ filled-square                 
diamond              │ ◇        │ shape        │ empty-diamond                 
diamond-filled       │ ◆        │ shape        │ filled-diamond                
triangle-up          │ △        │ shape        │ empty-triangle                
triangle-filled      │ ▲        │ shape        │ filled-triangle               
heart                │ ♥        │ symbol       │ love                          
flag                 │ ⚑        │ symbol       │ flagged                       
lock                 │ 🔒       │ status       │ locked, secure                
key                  │ 🔑       │ symbol       │ access                        
gear                 │ ⚙        │ symbol       │ settings, config              
bell                 │ ☆        │ notification │ notify                        
clock                │ ⏱        │ time         │ timer                         
calendar             │ 📅       │ time         │ date                          
folder               │ 📁       │ filesystem   │ dir                           
file                 │ 📄       │ filesystem   │ doc                           
link                 │ 🔗       │ navigation   │ url, href                     
tag                  │ 🏷        │ label        │ label                         
bar                  │ ▊        │ chart        │ bar-segment                   
spark                │ ▁▂▃▄▅▆▇█ │ chart        │ sparkline                     
```
<!-- /proof:compiled -->

---

## Custom symbols

Define custom symbols in `proof.toml` under `[[symbol]]`. Each symbol is a list
of lines that form the glyph. This is useful for project logos, custom icons, or
any recurring visual element that belongs in your document corpus.

```toml
[[symbol]]
name = "my-logo"
lines = [
  " ╔═╗ ",
  " ║P║ ",
  " ╚═╝ ",
]
```

After defining it, use it anywhere: `[sym:my-logo]` in prose, or
`proof:symbol name=my-logo size=1` for block rendering.

---

## Symbols in slide titles

Symbols expand in slide titles and subtitles, making it easy to add visual
markers to presentation content without special rendering:

```proof:slide layout=title
title: "[sym:star] proof"
subtitle: "Markdown quality assurance"
```

---

## Where symbols expand and where they don't

Symbols only expand in contexts where proof processes text. They are skipped
inside fenced code blocks and inline code spans to avoid corrupting literal
content.

| Context | Expands? | Notes |
|---------|----------|-------|
| Prose paragraphs | [sym:checkmark] yes | Main use case |
| Bullet labels | [sym:checkmark] yes | Use for visual left-edge |
| Slide title/subtitle | [sym:checkmark] yes | Supports any symbol |
| Callout text | [sym:checkmark] yes | |
| Fenced code blocks | [sym:cross] no | Treated as literal |
| Inline code spans | [sym:cross] no | Treated as literal |
| URLs | [sym:cross] no | Would corrupt the URL |

---

## Symbols vs math

`[sym:name]` and `$...$` both produce Unicode glyphs, but they serve different
purposes. Use symbols for decorative and semantic indicators; use math for
mathematical notation within expressions.

| Use case | Syntax | Renders as |
|----------|--------|------------|
| Star rating | `[sym:star]` | ★ |
| Star math operator | `$x \star y$` | x ⋆ y |
| Checkmark status | `[sym:checkmark]` | ✓ |
| Logical and | `$A \land B$` | A ∧ B |
