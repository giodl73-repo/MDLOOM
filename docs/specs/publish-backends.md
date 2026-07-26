# Publish Backends Spec

MDLOOM's compile graph resolves `.source.md` into a canonical compiled Markdown
document first. Publish backends consume that resolved document plus compile
metadata; they do not re-parse source directives or invent separate document
semantics.

## Current supported targets

| Target | Status | Purpose | Contract |
|---|---:|---|---|
| `md` | supported | Canonical terminal-first compiled document. | Resolves directives and writes Markdown. |
| `html` | supported | Standalone human-readable web document. | Resolves through Markdown, escapes raw HTML, emits common Markdown blocks with a small stylesheet. |
| `pebble` | supported | Agent/retrieval context transfer. | Emits `pebble.v1` JSON with source path, title, refs, stable section IDs, heading paths, line numbers, and resolved Markdown text. |
| `json-report` | supported | Machine-readable compile/report bundle. | Emits `mdloom.publish.json_report.v1` JSON with artifact summary, resolved Markdown, sections, source metadata, dependency refs, diagnostics, and compile counts. |
| `site` | supported | Local static documentation site. | Emits HTML pages, navigation `index.html`, and `mdloom-site.json` page manifest from a source tree. |
| `pdf` | supported | Portable human-readable artifact. | Renders the resolved HTML publish output into a deterministic PDF with basic text and metadata. |
| `docx` | supported | Editable Word-processing document. | Emits a native OOXML package with editable headings, paragraphs, lists, tables, code text, links, and metadata. |
| `pptx` | supported | Editable PowerPoint deck. | Emits a native OOXML package from explicit `.slides.source.md` inputs with editable slide text, native bullets/numbering, code text, notes, relationships, and manifest records. |

`html`, `pebble`, `json-report`, `site`, `pdf`, `docx`, and `pptx` are fully
supported within those scopes. They are not claims of hosting/deployment, PDF
layout fidelity, full Word styling, or rich PowerPoint production.

## Planned targets

No additional publish targets are planned in this wave.

LaTeX is intentionally deferred. It remains attractive for academic/technical
publishing, but it adds a separate typesetting contract and should not block the
publish backends above.

## Backend invariants

- Markdown-family backends start from the same resolved compile output used by
  `md`; `pptx` starts from the explicit slide source model after the slide
  compiler validates that `.slides.source.md` input.
- Source-only frontmatter stays source-only unless a backend explicitly maps safe
  metadata fields into its output.
- `.mdloom/artifacts.json` records target, source, output path, status,
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

The JSON bundle serializes information MDLOOM already owns: resolved Markdown
text, sections, dependencies, diagnostics, source metadata, and compile stats. It
is stable enough for CI and agents, but not a replacement for Pebble's compact
retrieval schema.

### Static site

The site backend builds on HTML. It owns page layout, navigation index files and
site manifests. It does not own CROP graph cuts, search ranking, hosting,
deployment, authentication, or browser pixel equivalence.

### PDF

PDF renders from the existing HTML output to avoid a second Markdown layout
implementation. The contract is a portable artifact with reasonable text output
and metadata, not exact cross-engine visual equivalence.

### DOCX

DOCX is an editable document target. The first version writes a native Office
Open XML package with document title metadata, headings, paragraphs, native
bullet and numbered lists, tables, fenced code blocks, and link text. Advanced
Word features such as tracked changes, comments, complex section breaks, custom
templates, and corporate styles belong in later pulses.

### PPTX

PPTX is a native Office Open XML deck backend, not a screenshot, image export, or
HTML-in-slide wrapper. PowerPoint is hard because slides are structured
presentation objects: text boxes, text runs, bullet levels, notes, dimensions,
themes, relationships, and content types all have to line up for the file to be
editable and reliable.

The backend requires `.slides.source.md` so MDLOOM does not guess a deck from
arbitrary prose. First support focuses on a small native model:

- title slides and title/content slides;
- real PowerPoint text boxes/placeholders, not rasterized text;
- native bullets and numbered lists with bounded nesting;
- fenced code as monospace text runs;
- speaker notes when source notes are available;
- deterministic slide order, relationships, and manifest records;
- ZIP/XML validation that inspects `ppt/slides/slide*.xml`,
  `ppt/notesSlides/notesSlide*.xml`, relationships, and content types.

PPTX has staged fidelity gates:

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

- a public `mdloom compile --target <target>` path or documented command surface;
- at least one integration test proving output shape and manifest target;
- README and spec coverage;
- deterministic output for unchanged inputs where feasible;
- clear non-goals and deferred capabilities.
