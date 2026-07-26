# MDCROP Index and TOC Integration for MDLOOM

## Goal

Make MDLOOM depend on MDCROP for corpus index and TOC primitives while MDLOOM keeps
ownership of Markdown, HTML, ASCII-art, and source-to-output compilation.

## Motivation

MDLOOM already generates Markdown from `.source.md`, now also HTML, and validates
links, tables, ASCII figures, directives, and source references. MDCROP has the
right local corpus view layer: filtered roots, named views, inspection, extension
profiles, source samples, and Markdown index generation. MDLOOM should reuse that
library surface instead of rebuilding corpus discovery and source-table logic.

## Dependency

Add a dependency on `mdcrop-core`.

Local development:

```toml
mdcrop-core = { path = "../MDCROP/crates/mdcrop-core" }
```

Published/Git dependency once stable:

```toml
mdcrop-core = { git = "https://github.com/giodl73-repo/MDCROP.git", package = "mdcrop-core", branch = "main" }
```

## MDCROP APIs to consume

- `markdown_index(root, title, options) -> Result<String, MdcropError>`
- `markdown_index_for_view_json(json, base_dir) -> Result<String, MdcropError>`
- `markdown_index_report(root, title, options) -> Result<MarkdownIndex, MdcropError>`
- `inspect_view_json(json, base_dir) -> Result<MdcropViewInspect, MdcropError>`
- `inspect_view_store(store) -> Result<MdcropViewStoreInspect, MdcropError>`
- `IngestOptions { include_extensions, exclude_dirs }`

## MDLOOM commands

Add non-destructive commands first:

```powershell
mdloom index --root . --extension md --extension html --exclude-dir target
mdloom index --view .mdloom\views\docs.json --output INDEX.md
mdloom toc --root docs --extension md --output TOC.md
mdloom inspect-views --dir .mdloom\views --strict
```

`mdloom index` should render a README-style source table. `mdloom toc` can start as
an alias or narrower rendering of the same MDCROP `MarkdownIndex` report, then
later grow heading-depth options. `mdloom inspect-views` should surface MDCROP view
inspection for CI before MDLOOM compiles or publishes a large corpus.

## MDLOOM directives

After the command surface lands, wire generated indexes into source compilation:

```markdown
mdloom:index root="docs" extensions="md,html" exclude="target" title="Documentation Index"
mdloom:toc root="docs/guides" extensions="md" depth=2
mdloom:view-index file=".mdloom/views/docs.json"
```

The directive renderer should call MDCROP APIs and insert Markdown tables into the
compiled `.md` and `.html` outputs.

## Output contract

Initial Markdown index table:

| Path | Title | Type | Directory | Links |
|------|-------|------|-----------|------:|
| `docs/README.md` | Documentation | `md` | `docs` | 4 |

Also include:

- root path
- total source count
- extension profile table
- stable sorting by path

## Implementation notes

- Do not overwrite existing `README.md`, `INDEX.md`, or `TOC.md` unless the user
  passes `--output` explicitly.
- Honor MDLOOM's existing include/exclude configuration where possible, translating
  it into MDCROP `IngestOptions`.
- Keep generated artifacts deterministic for stable diffs.
- For HTML output, MDLOOM should render the Markdown table through its existing
  HTML target rather than asking MDCROP to emit HTML.
- Treat MDCROP errors as MDLOOM diagnostics with file/path context.

## Validation

```powershell
cargo fmt
cargo test
mdloom index --root . --extension md --extension html --exclude-dir target
mdloom inspect-views --dir .mdloom\views --strict
```

Add tests for:

- root-based index generation
- view-based index generation
- output file writing
- bad view/root diagnostics
- directive expansion in Markdown and HTML compile targets
