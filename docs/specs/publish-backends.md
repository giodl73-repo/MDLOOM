# Publish Backends Spec

PROOF's compile graph resolves `.source.md` into a canonical compiled Markdown
document first. Publish backends consume that resolved document plus compile
metadata; they do not re-parse source directives or invent separate document
semantics.

## Current supported targets

| Target | Status | Purpose | Contract |
|---|---:|---|---|
| `md` | supported | Canonical terminal-first compiled document. | Resolves directives and writes Markdown. |
| `html` | supported | Standalone human-readable web document. | Resolves through Markdown, escapes raw HTML, emits common Markdown blocks with a small stylesheet. |
| `pebble` | supported | Agent/retrieval context transfer. | Emits `pebble.v1` JSON with source path, title, refs, stable section IDs, heading paths, line numbers, and resolved Markdown text. |
| `json-report` | supported | Machine-readable compile/report bundle. | Emits `proof.publish.json_report.v1` JSON with artifact summary, resolved Markdown, sections, source metadata, dependency refs, diagnostics, and compile counts. |

`html`, `pebble`, and `json-report` are fully supported within those scopes.
They are not claims of full site generation, PDF layout fidelity,
office-document styling, or slide deck generation.

## Planned targets

| Target | Primary user | Backend role | First useful claim |
|---|---|---|---|
| `site` | Docs publishers | Multi-page static documentation site. | Compile a source tree to HTML pages, index/navigation, assets, and a site manifest. |
| `pdf` | Report readers | Portable human-readable artifact. | Render the HTML publish output to PDF with deterministic metadata and manifest entries. |
| `docx` | Business/document workflows | Editable Word-processing document. | Convert resolved Markdown into headings, paragraphs, lists, tables, code blocks, and document metadata. |
| `pptx` | Presentations/executive updates | Slide deck artifact. | Convert explicit slide-oriented source into a basic deck with title/content slides, speaker notes metadata where available, and stable manifest records. |

LaTeX is intentionally deferred. It remains attractive for academic/technical
publishing, but it adds a separate typesetting contract and should not block the
publish backends above.

## Backend invariants

- Every backend starts from the same resolved compile output used by `md`.
- Source-only frontmatter stays source-only unless a backend explicitly maps safe
  metadata fields into its output.
- `.proof/artifacts.json` records target, source, output path, status,
  diagnostics, cache use, and resolved directive counts for every non-watch
  compile.
- Backends may add target-specific sidecar metadata, but they must not replace
  the artifact manifest.
- A backend failure must surface as compile diagnostics/status, not silently
  falling back to Markdown while claiming the target was written.
- Watch mode remains Markdown-only until target-aware watch invalidation is
  explicitly designed.

## Target-specific boundaries

### JSON report bundle

The JSON bundle serializes information PROOF already owns: resolved Markdown
text, sections, dependencies, diagnostics, source metadata, and compile stats. It
is stable enough for CI and agents, but not a replacement for Pebble's compact
retrieval schema.

### Static site

The site backend builds on HTML. It owns page layout, navigation, index files,
asset copying, and site manifests. It does not own CROP graph cuts, search
ranking, hosting, deployment, authentication, or browser pixel equivalence.

### PDF

PDF should initially be rendered from the existing HTML output to avoid a second
Markdown layout implementation. The contract is a portable artifact with
reasonable typography and metadata, not exact cross-engine visual equivalence.

### DOCX

DOCX should be an editable document target. The first version should support
document title, headings, paragraphs, lists, tables, fenced code blocks, links,
and basic metadata. Advanced Word features such as tracked changes, comments,
complex section breaks, custom templates, and corporate styles belong in later
pulses.

### PPTX

PPTX must be a native Office Open XML deck backend, not a screenshot, image
export, or HTML-in-slide wrapper. PowerPoint is hard because slides are structured
presentation objects: placeholders, text runs, bullet levels, notes, dimensions,
themes, relationships, and content types all have to line up for the file to be
editable and reliable.

PROOF already has slide source concepts, but a deck backend should require either
`.slides.source.md` or clear slide sections rather than guessing a deck from
arbitrary prose. First support should focus on a small native model:

- title slides and title/content slides;
- real PowerPoint text boxes/placeholders, not rasterized text;
- native bullets and numbered lists with bounded nesting;
- fenced code as monospace text runs;
- speaker notes when source notes are available;
- deterministic slide order, relationships, and manifest records;
- ZIP/XML validation that inspects `ppt/slides/slide*.xml`,
  `ppt/notesSlides/notesSlide*.xml`, relationships, and content types.

PPTX should have staged fidelity gates before being called supported:

1. **Package gate**: the `.pptx` opens as a valid OOXML package with expected
   parts and relationships.
2. **Structure gate**: slide titles, bullet levels, and notes are represented as
   native editable XML.
3. **Presentation gate**: STAGE review confirms default density and hierarchy are
   usable for a real audience.

Rich layout, animations, transitions, themes, charts, embedded media, and brand
templates come later.

## Done definition for a publish backend

A backend is "supported" only when it has:

- a public `proof compile --target <target>` path or documented command surface;
- at least one integration test proving output shape and manifest target;
- README and spec coverage;
- deterministic output for unchanged inputs where feasible;
- clear non-goals and deferred capabilities.
