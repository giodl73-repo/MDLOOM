# CROP Index and TOC Integration for PROOF

## Goal

Make PROOF depend on CROP for corpus index and TOC primitives while PROOF keeps
ownership of Markdown, HTML, ASCII-art, and source-to-output compilation.

## Motivation

PROOF already generates Markdown from `.source.md`, now also HTML, and validates
links, tables, ASCII figures, directives, and source references. CROP has the
right local corpus view layer: filtered roots, named views, inspection, extension
profiles, source samples, and Markdown index generation. PROOF should reuse that
library surface instead of rebuilding corpus discovery and source-table logic.

## Dependency

Add a dependency on `crop-core`.

Local development:

```toml
crop-core = { path = "../CROP/crates/crop-core" }
```

Published/Git dependency once stable:

```toml
crop-core = { git = "https://github.com/giodl73-repo/CROP.git", package = "crop-core", branch = "main" }
```

## CROP APIs to consume

- `markdown_index(root, title, options) -> Result<String, CropError>`
- `markdown_index_for_view_json(json, base_dir) -> Result<String, CropError>`
- `markdown_index_report(root, title, options) -> Result<MarkdownIndex, CropError>`
- `inspect_view_json(json, base_dir) -> Result<CropViewInspect, CropError>`
- `inspect_view_store(store) -> Result<CropViewStoreInspect, CropError>`
- `IngestOptions { include_extensions, exclude_dirs }`

## PROOF commands

Add non-destructive commands first:

```powershell
proof index --root . --extension md --extension html --exclude-dir target
proof index --view .proof\views\docs.json --output INDEX.md
proof toc --root docs --extension md --output TOC.md
proof inspect-views --dir .proof\views --strict
```

`proof index` should render a README-style source table. `proof toc` can start as
an alias or narrower rendering of the same CROP `MarkdownIndex` report, then
later grow heading-depth options. `proof inspect-views` should surface CROP view
inspection for CI before PROOF compiles or publishes a large corpus.

## PROOF directives

After the command surface lands, wire generated indexes into source compilation:

```markdown
proof:index root="docs" extensions="md,html" exclude="target" title="Documentation Index"
proof:toc root="docs/guides" extensions="md" depth=2
proof:view-index file=".proof/views/docs.json"
```

The directive renderer should call CROP APIs and insert Markdown tables into the
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
- Honor PROOF's existing include/exclude configuration where possible, translating
  it into CROP `IngestOptions`.
- Keep generated artifacts deterministic for stable diffs.
- For HTML output, PROOF should render the Markdown table through its existing
  HTML target rather than asking CROP to emit HTML.
- Treat CROP errors as PROOF diagnostics with file/path context.

## Validation

```powershell
cargo fmt
cargo test
proof index --root . --extension md --extension html --exclude-dir target
proof inspect-views --dir .proof\views --strict
```

Add tests for:

- root-based index generation
- view-based index generation
- output file writing
- bad view/root diagnostics
- directive expansion in Markdown and HTML compile targets
