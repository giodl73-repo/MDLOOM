# Publication AST and Theme Spec

MDLOOM now has several publish backends. The next quality jump is a shared
publication model: one target-neutral document tree plus one target-neutral theme
token set that every backend can map into its own native output.

## Mission

Turn resolved MDLOOM content into a structured publication AST before rendering to
HTML, site pages, PDF, DOCX, PPTX, JSON report, or Pebble. The AST owns document
semantics; themes own visual intent. Backends own target-specific serialization.

## Current problem

The first backend wave proved the targets can be generated. It also created
backend-local mappings for headings, paragraphs, lists, tables, code, links,
notes, slide text, and metadata. That is acceptable for MVP support, but it will
not scale to professional output because every backend would otherwise reinvent:

- heading hierarchy and section metadata;
- inline spans and links;
- list nesting and numbering;
- table structure;
- code block language/style hints;
- figures/media and captions;
- notes, speaker notes, and sidebars;
- document metadata;
- typography, color, spacing, and layout tokens.

## Architecture

```text
.source.md / .slides.source.md
        │
        ▼
resolved compile output / slide source model
        │
        ▼
Publication AST + Theme tokens
        │
        ├── html/site renderer
        ├── json-report/pebble serializers
        ├── pdf renderer
        ├── docx OOXML renderer
        └── pptx OOXML renderer
```

## Publication AST v1

The first AST version should be intentionally small and stable.

```text
PublicationDocument
  schema = "mdloom.publication_ast.v1"
  kind = document | deck
  title
  metadata
  theme
  blocks[]

Block
  Heading { level, text, id }
  Paragraph { inlines[] }
  List { ordered, items[] }
  CodeBlock { language, text }
  Table { headers[], rows[][] }
  Figure { source, alt, caption }
  Note { kind, blocks[] }
  Slide { title, subtitle, blocks[], notes[] }

Inline
  Text
  Emphasis
  Strong
  Code
  Link { href, children[] }
```

### Invariants

- The AST must be generated after source directive resolution, not from raw
  directive syntax.
- The AST must preserve stable section IDs and heading paths where available.
- The AST must distinguish semantic intent from target serialization details.
- Backends must not silently invent structure that is absent from the AST.
- Unknown or unsupported AST nodes must fail loudly or degrade with an explicit
  diagnostic/manifest note, not disappear.

## Theme tokens v1

Themes should be target-neutral, then mapped into CSS, PDF text settings, DOCX
styles, and PPTX theme/layout defaults.

```text
PublicationTheme
  name
  fonts
    body
    heading
    monospace
  colors
    text
    muted
    background
    accent
    code_background
    border
  spacing
    page_margin
    block_gap
    list_indent
  typography
    body_size
    heading_scale
    line_height
  slide
    aspect_ratio
    title_size
    body_size
    max_bullets
    bullet_indent
```

Initial built-in themes:

| Theme | Purpose |
|---|---|
| `plain` | Conservative readable default for all targets. |
| `professional` | Better typography, spacing, and accent color for reports/decks. |
| `dense` | Compact technical output for docs with high information density. |

## Backend mapping requirements

| Backend | AST use | Theme use |
|---|---|---|
| HTML/site | Render semantic blocks/inlines to HTML. | CSS variables and stylesheet. |
| JSON report | Serialize AST summary and theme name alongside existing report fields. | Metadata only initially. |
| Pebble | Continue compact chunks; optionally derive chunks from AST sections. | None initially. |
| PDF | Render AST text with page/typography tokens. | Font sizes, margins, line height, colors where supported. |
| DOCX | Map headings/lists/tables/code to styles and numbering. | `word/styles.xml`, numbering, theme colors/fonts. |
| PPTX | Map `Slide` nodes to native text boxes/placeholders. | Theme, sizes, colors, bullet indentation, slide dimensions. |

## Configuration surface

Theme selection should be usable from both CLI and config:

```powershell
mdloom compile deck.slides.source.md --target pptx --theme professional -o deck.pptx
```

```toml
[publish]
theme = "professional"

[publish.theme.professional]
font_body = "Aptos"
font_heading = "Aptos Display"
font_monospace = "Cascadia Mono"
accent = "#2563eb"
```

The first pulse may add built-in themes only. User-defined themes should be
introduced after the AST and renderer adoption path is stable.

## Done definition

The publication model is ready when:

- it has a public Rust module with typed AST/theme structures;
- Markdown-family publish backends can build from the AST without behavior
  regressions;
- PPTX and DOCX map theme tokens to native OOXML styles/theme parts;
- tests prove AST extraction, theme selection, and backend-specific mapping;
- docs identify supported nodes, unsupported nodes, and backend mapping limits.
