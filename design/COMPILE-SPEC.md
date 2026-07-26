# mdloom compile — Markdown Compilation Specification v0.2

**Status:** ✅ Implemented — `src/compile.rs`. All directives wired: include, layout, table, tree, element, row, symbol, shape, region, math, toc, xref, blockquote, chart, ol/numbered-list. Watch mode, --progress, --delete-on-error, multi-target [[compile]] routing all live.

---

## What it is

`mdloom compile` is a markdown compiler that resolves **include directives** in source
markdown files and produces rendered output markdown. Source files use `mdloom:include`
and `mdloom:layout` fenced blocks to reference figures, tables, and charts by `md://`
URI. The compiler resolves each reference, applies layout composition, validates
DaVinci invariants, and writes the final compiled document.

**The mental model**: source markdown is source code. Compiled markdown is the artifact.
DaVinci invariants are types. The compiler enforces types before output ships.

---

## Why compilation, not preprocessing

The distinction matters. A preprocessor is a text substitution tool — find `!include`
and paste text. A compiler:
- **Validates** — every included figure must satisfy its DaVinci invariants or compile fails
- **Caches** — parsed documents and resolved URIs are content-addressed so unchanged
  figures are not re-resolved
- **Snapshots** — compile states can be named and restored (see Cache Snapshots below)
- **Incremental** — only recompile when inputs change (causal cache chain)
- **Addressable** — the compiled output carries `md://` addresses for each embedded figure
  so mdloom check can still validate the output

---

## Source document format

Source documents use `mdloom:` info strings on fenced blocks:

````markdown
## Concurrency Model

Intro text here.

```mdloom:include
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
```

Compare Go and Rust concurrency:

```mdloom:layout gap=4 align=top
md://languages/10-GO.md#concurrency-model:0
md://languages/09-RUST.md#ownership-model:0
```

The type system at a glance:

```mdloom:table
md://languages/10-GO.md#type-system-snapshot:table.key-value:0
?select=Axis,Value&filter=Axis contains runtime
```
<!-- filter/select syntax: see TABLE-FILTER-SPEC.md (planned) -->
````

### Directive types

| Directive | Purpose |
|-----------|---------|
| `mdloom:include` | Embed a single figure, table, or text element |
| `mdloom:layout` | Compose N figures side-by-side (see LAYOUT-SPEC.md) |
| `mdloom:table` | Embed a table with optional column selection and filtering |
| `mdloom:blockquote` | Prose-document block quote with optional attribution (see below) |

### `mdloom:blockquote` — prose-document block quote

Prose documents need a first-class block-quote primitive distinct from slides'
`mdloom:quote` (which centers content and uses curly quotes — wrong for prose).
`mdloom:blockquote` is left-aligned, indented, and emits one of two shapes:

```markdown
` ` `mdloom:blockquote attribution="Ada Lovelace" style=indent
The Analytical Engine has no pretensions whatever to originate anything.
` ` `
```

Renders to standard markdown blockquote syntax:

```
> The Analytical Engine has no pretensions whatever to originate anything.
>
> — Ada Lovelace
```

`style="boxed"` instead emits an ASCII frame (`┌─...─┐ │ ... │ └─...─┘`),
sized to the longest body line. Attribution renders right-aligned inside the
frame. Unknown style values fall back silently to `indent` (permissive).

| Attribute | Default | Notes |
|-----------|---------|-------|
| `attribution` (alias `by`) | none | Optional — emits `— Name` line if set |
| `style` | `"indent"` | `"indent"` (markdown `>`) or `"boxed"` (ASCII frame) |

Body content is the directive's fenced block body. Leading and trailing blank
lines in the body are trimmed; inner blank lines render as bare `>` lines so
the rendered markdown stays one contiguous quote (not two adjacent ones).
This directive is prose-only — it has no per-slide centering or width-fitting
logic. For slides, use `mdloom:quote`.

### Figure files

A figure file is a standalone `.md` file whose figures are marked with
`<!-- mdloom:figure -->` HTML comments immediately preceding each code fence:

```markdown
<!-- mdloom:figure id="goroutine-scheduler" kind="figure.flowchart" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
│  Goroutines (G)...
└─────────────────────────────────────┘
```
```

The HTML comment is **outside** the code fence — markdown renderers hide it, but
mdloom indexes it to give the following code block a stable named identity. A figure
file may contain multiple `<!-- mdloom:figure -->` markers, one before each code block.

Figure files live in a `figures/` directory (or anywhere) and are addressed by
`md://` URI. See [URI Resolution](#uri-resolution) for how named IDs map to selectors.

---

## URI resolution

### Base path

All `md://` URIs are resolved relative to the **mdloom.toml root directory** — the
directory containing the `mdloom.toml` file, or the current working directory if no
`mdloom.toml` is found. The source file's own directory is NOT the base.

```
mdloom.toml root: /project/
source file:     /project/languages/10-GO.source.md
URI:             md://figures/goroutine-scheduler.md#goroutine-scheduler:0
resolves to:     /project/figures/goroutine-scheduler.md  ← from project root
```

### Named figure ID selector

The URI fragment `#goroutine-scheduler:0` is a named figure selector. It resolves to
the code block immediately following the `<!-- mdloom:figure id="goroutine-scheduler" -->` marker in the target file. The `:0` ordinal selects the first (and usually only) code block following that marker.

Mdloom indexes all `mdloom:figure id=` markers in a file during the parse step. The
parse cache stores this index as part of the `ParsedDocument`. The mdpath URI scheme's
`#heading:kind:ordinal` form also works — named figure IDs are an additional selector
shortcut layered on top.

### What `mdloom:include` embeds

`mdloom:include` embeds the **content inside the code fence**, not the raw fence
delimiter lines. Given this figure:

```
<!-- mdloom:figure id="foo" kind="figure.flowchart" -->
` ` `
BOX CONTENT
└────────┘
` ` `
```

The embedded content is:
```
BOX CONTENT
└────────┘
```

The output wraps this content in a new code fence and traceability comments (see
[Traceability](#traceability-in-compiled-output)). Fence delimiter lines from figure
files are never copied verbatim into compiled output.

---

## CLI commands

```bash
mdloom compile input.source.md             # compile to input.md (drops .source.)
mdloom compile input.source.md -o out.md   # explicit output path
mdloom compile src/*.source.md             # batch compile all source files
mdloom compile . --watch                   # watch mode: recompile on change
mdloom compile . --check                   # validate without writing output
mdloom compile . --cache-status            # show per-file cache tier hits/misses
mdloom compile . --no-cache                # bypass all cache tiers

# Cache snapshot management
mdloom cache snapshot save "production"
mdloom cache snapshot restore "production"
mdloom cache snapshot diff "before" "after"
mdloom cache snapshot diff "before" --vs-current
mdloom cache snapshot list
mdloom cache snapshot prune --keep 5
mdloom cache snapshot deploy "production" --to ./dist/
```

### Output path convention

When compiling `foo.source.md`, the default output is `foo.md` (dropping `.source.`),
in the same directory as the source file. All paths are relative to the mdloom.toml
root:

```
source:  languages/10-GO.source.md
output:  languages/10-GO.md           ← drops .source. in-place
```

If the source filename does not contain `.source.`, use `-o` to specify the output
path explicitly.

---

## Three-Tier Cache

Each tier is content-addressed (keyed by hash of inputs). A change at any tier
cascades forward.

```
source change
    │
    ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Parse Cache  │────▶│ Resolve Cache│────▶│Compile Cache │
│  (Tier 1)   │     │  (Tier 2)    │     │  (Tier 3)    │
└──────────────┘     └──────────────┘     └──────────────┘
  file content          parse key +         resolve keys[] +
  hash +                URI string +        layout config hash +
  mdloom version         mdloom version       mdloom version
```

### Tier 1: Parse Cache

Caches the `ParsedDocument` for each `.md` file.

```
parse_key = SHA-256(
    file_content_hash,
    mdloom_version
)
```

**Hit:** Return cached `ParsedDocument` directly — no re-parsing, no re-scanning for
directives. The cached `ParsedDocument` includes all directives, figure markers, and
heading structure.

**Miss:** Parse file, cache result, also update the path index (see below).

#### Parse cache path index

The parse cache is content-addressed (keys are hashes), so there is no built-in way
to look up "what is the current parse_key for `/project/figures/foo.md`?" without
re-hashing the file. To support efficient compile_key computation (step 3 of the
pipeline), the compiler maintains a **path index** alongside the cache:

```
.mdloom/cache/parse-index.json   ← maps file path → (parse_key, mtime)
```

On startup, any entry whose stored mtime differs from the actual file mtime is treated
as stale and re-hashed. This lets the compiler find the current parse_key for any file
path without re-reading or re-parsing the file content.

### Tier 2: Resolve Cache

Caches the resolved content of each `md://` URI.

```
resolve_key = SHA-256(
    parse_key,      ← Tier 1 of the TARGET file (not the source document)
    uri_string,
    mdloom_version
)
```

**Hit:** Return cached `ResolvedElement` (the content inside the code fence).
**Miss:** Resolve via mdpath, extract fence content, cache result.

### Tier 3: Compile Cache

Caches the full compiled output of a source document.

```
compile_key = SHA-256(
    source_parse_key,       ← Tier 1 of the source document
    sorted_resolve_keys[],  ← Tier 2 of ALL included figures, in order, NOT deduplicated
    layout_config_hash,     ← SHA-256 of normalized layout config (see note)
    mdloom_version
)
```

**`sorted_resolve_keys[]` must NOT be deduplicated.** If the same URI appears twice
in a source document, its resolve_key appears twice in this list. Deduplication would
cause two documents with different include counts to share the same compile_key, which
is a silent correctness bug.

**`layout_config_hash`**: When a source document has no `mdloom:layout` directives,
use `SHA-256("")` (hash of empty string) as the sentinel value. When layout directives
are present, hash the normalized attribute set — with all defaults filled in before
hashing, so that `gap=3` (explicit) and `gap` (omitted, default 3) produce the
same hash. Labels are part of the layout config hash.

**Hit:** Read compiled text from cache, write to output path (skip write if output file
already has identical content to avoid spurious mtime updates that would confuse watch
mode). Done.

**Miss:** Continue pipeline.

### Cascading invalidation

| What changed | Parse | Resolve | Compile |
|-------------|-------|---------|---------|
| Source document content | MISS | MISS | MISS |
| Included figure content | HIT | MISS | MISS |
| Layout config (gap, align, labels) | HIT | HIT | MISS |
| mdloom version | MISS | MISS | MISS |
| DaVinci invariants only | HIT | HIT | MISS (re-validate) |

The causal chain means you never manually invalidate. Content-addressed keys
automatically miss when inputs change.

### Compile cache entry schema

Each `.mdloom/cache/compile/{key}.json` stores:

```rust
struct CompileCacheEntry {
    compile_key: String,
    source_path: String,        // relative to mdloom.toml root
    output_path: String,        // relative to mdloom.toml root
    compiled_text: String,      // the full compiled markdown
    resolved_uris: Vec<String>, // all md:// URIs that were embedded
    mdloom_version: String,
    created_at: u64,            // epoch ms
}
```

### Storage layout

```
.mdloom/cache/
  parse/           ← Tier 1: ParsedDocument per .md file
    {key}.json
  parse-index.json ← path → (parse_key, mtime) for efficient lookup
  resolve/         ← Tier 2: ResolvedElement per md:// URI
    {key}.json
  compile/         ← Tier 3: compiled output per source document
    {key}.json
  snapshots/       ← Named snapshots (see below)
    production/
      manifest.json
      parse/
      resolve/
      compile/
```

---

## Named Cache Snapshots

Named snapshots let you save and restore compile states — useful before risky diagram
edits. See [CACHE-SNAPSHOTS.md](./cache-snapshots.md) for the full spec.

```bash
mdloom cache snapshot save "before-redesign"          # capture current state
mdloom cache snapshot restore "before-redesign"        # restore cache state (not files)
mdloom cache snapshot diff "v1" "v2"                  # which files changed?
mdloom cache snapshot diff "before-redesign" --vs-current   # diff vs live cache
mdloom cache snapshot deploy "production" --to ./dist/ # materialize without recompile
```

**Important:** `restore` is a cache operation — it restores cached compiled artifacts,
not the working files. After restore, source documents and figure files remain in their
current edited state on disk. Files that were edited since the snapshot was saved will
naturally miss the restored cache and recompile on the next `mdloom compile .`.

---

## Compilation pipeline

```
mdloom compile source.md
    │
    ├── 1. Parse source.md → ParsedDocument (Tier 1 cache check via path index)
    │       └── HIT: use cached ParsedDocument for all directive extraction below
    │
    ├── 2. Extract all mdloom: directives from ParsedDocument
    │       (mdloom:include, mdloom:layout, mdloom:table)
    │
    ├── 3. Compute resolve_keys for all directive URIs
    │       ├── Look up parse_key of each target file via path index
    │       ├── resolve_key = SHA-256(target_parse_key, uri_string, mdloom_version)
    │       └── Collect as sorted_resolve_keys[] — preserve duplicates, do not deduplicate
    │
    ├── 4. Compute compile_key, check Tier 3 cache
    │       ├── compile_key = SHA-256(source_parse_key, sorted_resolve_keys[],
    │       │       layout_config_hash, mdloom_version)
    │       ├── HIT → write cached output (skip if identical), done
    │       └── MISS → continue
    │
    ├── 5. For each directive — fetch resolved content and validate
    │       ├── Fetch resolved content (Tier 2 cache read or full resolve via mdpath)
    │       │     Same URI appearing N times → N Tier 2 cache hits, 1 actual resolution
    │       ├── Validate DaVinci invariants for all fetched figures
    │       │     Collect ALL violations before aborting (fail-all, not fail-fast)
    │       └── If any violation at protection=error → COMPILE-001, abort (see below)
    │
    ├── 6. For mdloom:layout directives:
    │       └── Apply layout engine with resolved content (see LAYOUT-SPEC.md)
    │
    ├── 7. Compose final document:
    │       ├── Replace each directive with resolved/composed content
    │       └── Embed traceability comments (see Traceability section)
    │
    ├── 8. Write output file atomically (temp-then-rename)
    │
    └── 9. Update Tier 3 cache and path index
```

### DaVinci validation during compile

All included figures are validated against their registered DaVinci invariants BEFORE
composing output. Violations are collected across ALL directives before aborting — the
error report shows every violation, not just the first.

```
DaVinci violations during compile:
  → ALL violations reported (fail-all, not fail-fast)
  → compile aborts with COMPILE-001 for each error-level violation
  → output file is NOT written (existing compiled output is preserved unchanged)
  → COMPILE-003 warning emitted for each warn-level violation; compile continues

Error format per violation:
  error[COMPILE-001]: DaVinci invariant violated
    figure:    goroutine-scheduler
    file:      figures/goroutine-scheduler.md:8
    uri:       md://figures/goroutine-scheduler.md#goroutine-scheduler:0
    invariant: box-count min=2
    found:     1 box (expected ≥ 2)
    included by: languages/10-GO.source.md:34

The compiled output is ALWAYS invariant-clean at error level.
protection=warn figures are embedded with a warning — compile still succeeds.
```

---

## Watch mode

```bash
mdloom compile . --watch
```

Watch mode watches **both** source documents (`.source.md`) and all figure files
referenced by any `md://` URI in any source document. When a figure file changes, only
the source documents that include it are recompiled.

```
[mdloom] watching . for changes (12 source files, 8 figure files)...
[mdloom] detected change: figures/goroutine-scheduler.md
  → recompiling: languages/10-GO.source.md (includes this figure)
  → ✓ compiled in 43ms
[mdloom] watching...
```

Content-addressed keys naturally miss for changed files — watch mode does **not**
explicitly invalidate cache entries. The recompile simply produces a new hash that
misses the cache.

### Watch inverse index

To efficiently determine which source documents include a changed figure, watch mode
builds an inverse index on startup:

```
.mdloom/cache/watch-index.json  ← maps figure_path → [source_paths that include it]
```

This index is rebuilt when source documents are compiled or recompiled. A new source
document that has never been compiled is not yet in the index and will not be
automatically recompiled when its figures change — run `mdloom compile .` once to
build the index, then watch mode is fully responsive.

---

## Traceability in compiled output

Compiled output embeds HTML comments that trace each embedded element back to its
source URI. This lets `mdloom check` on the compiled output emit errors with stable
`md://` addresses rather than just file:line:col.

```markdown
<!-- mdloom:compiled from="md://figures/goroutine-scheduler.md#goroutine-scheduler:0" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
...
└─────────────────────────────────────┘
```
<!-- /mdloom:compiled -->
```

For `mdloom:layout`, the traceability comment lists all composed URIs:

```markdown
<!-- mdloom:compiled from="mdloom:layout"
     uris="md://figures/go-types.md#go-type-system:0,md://figures/rust-types.md#rust-type-system:0" -->
```
      Go                         Rust
Axis         | Value        Axis         | Value
...
```
<!-- /mdloom:compiled -->
```

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `COMPILE-001` | error | DaVinci invariant violated in included figure — compile aborted |
| `COMPILE-002` | error | md:// URI failed to resolve — target does not exist |
| `COMPILE-003` | warning | Included figure has a warn-level DaVinci violation — compile continues |
| `COMPILE-004` | error | Cache snapshot integrity hash mismatch — restore rejected |
| `COMPILE-005` | error | Snapshot restore rejected: compilation in progress (server/session mode only) |
| `COMPILE-006` | warning | Snapshot missing some files present in current state |
| `COMPILE-007` | warning | Included figure has lint errors — embedded output may be misaligned |

---

## New roles needed

This compilation system expands mdloom's scope significantly. Three new roles:

**SOURCE** — source/output document model expert. Understands the include system,
directive syntax, traceability comments, and the relationship between `.source.md`
and compiled `.md` files.

**COMPOSE** — layout and visual composition specialist. Understands the layout
engine, figure framing, alignment, padding, gap management, and how compositions
render in different column widths.

**CACHE** — cache correctness specialist. Understands cache key computation,
cascading invalidation, snapshot integrity, and crash safety. Pulls against
SOURCE and COMPOSE (more correctness, less throughput).

---

## Spec Clarifications (from scenario findings)

The following clarifications resolve ambiguities surfaced during scenario walkthroughs
of `mdloom compile`, `mdloom check`, watch mode, and the math/canvas helpers. Each item
is keyed to its finding ID for traceability.

### Compile errors and stale output

**F93 — Stale output on error.** When `has_errors=true`, `compile_file` returns
`written=false` and does NOT modify the output file. Any existing stale output file
is left in place unchanged. Authors can verify staleness by checking file timestamps
(the source `.source.md` mtime will be newer than the output `.md` mtime when the
last compile failed). A future `--delete-on-error` flag may change this.

**F119 — Stale output policy rationale.** Leaving stale output on disk is the
current behavior. The reasoning: better a stale correct file than a fresh broken
file. Mid-edit failures should not blow away the last-known-good artifact. Authors
who want clean failure semantics in CI should use `mdloom compile --check`, which
validates without writing and returns a non-zero exit code on error.

**F120 — Error summary on stderr.** When compile has errors, stderr prints the
count of files not written. Format:

```
FAIL — N compiled, M errors
```

Where `M > 0` means some files were not updated. `N` is the count of source files
that compiled and wrote output successfully; `M` is the count that failed and left
stale output in place.

### TOC scanning

**F102 — TOC scan start.** The `mdloom:toc` heading scan starts from the beginning
of the current file. The `mdloom:toc` fence itself is inside a fenced code block and
is therefore skipped by the heading scanner — headings inside fences are not real
headings. This means a TOC directive placed anywhere in the document produces the
same output, and the directive's own location does not appear as a heading entry.

### Watch mode

**F107 — Watch set includes md:// deps.** In `mdloom compile --watch`, the initial
watch set covers all `source_dir` paths from active `[[compile]]` targets. Future
improvement: after the initial compile pass, also watch all `md://` files resolved
during compilation so that figure file edits trigger recompile of the source
documents that include them. The watch inverse index (see Watch mode section above)
already enables this — the missing piece is registering the figure paths with the
file watcher.

**F108 — Watch with initial errors.** If the initial compile pass has errors,
`--watch` continues into the watch loop. Authors can fix files and save — the
watcher recompiles without needing to restart. This is the correct behavior: watch
is a development tool, not a CI gate. Errors during watch never terminate the
process; they only suppress writes for the failing files (per F93).

**F112 — Watch + output-dir.** When `mdloom compile --watch --output-dir path` is
used, the `--output-dir` override applies for the entire watch session including
all subsequent recompiles triggered by file changes. The override is captured at
session start and is not re-read from CLI on each recompile.

### Compile target configuration

**F109 — Overlapping targets.** If two `[[compile]]` targets have overlapping
`source_dir` paths, files in the overlap are compiled once per matching target,
to each target's `output_dir`. This may produce two output files with the same
derived name in different output directories — that is intentional. However, two
targets producing the same output path (same directory + same filename) is a
configuration error and should emit COMPILE-002.

**F110 — `source_dir` base.** `source_dir` in `[[compile]]` is relative to the
directory containing `mdloom.toml` (the mdloom root). This is the same base as all
other relative paths in `mdloom.toml`. Source paths are not relative to the
`[[compile]]` block's position in the file or to any parent table.

**F111 — Filename collision across targets.** If two source files in different
target `source_dir`s have the same filename, they produce files with the same
name in their respective output directories. There is no collision when output
directories differ. Collision within a single output directory (which can occur
when `--output-dir` flattens multiple targets to one directory) is a configuration
error — last write wins with a warning. Authors should avoid this by keeping
output directories distinct.

### Check command

**F113 — `mdloom check` exit code.** `mdloom check` exits non-zero if any
`error`-severity diagnostics are found and `--fail-on-error` is set. Warnings
alone do not trigger non-zero exit. `md_broken_uri` is severity `error` — it
triggers non-zero exit under `--fail-on-error`. The default mode (without the
flag) always exits zero so that `mdloom check` can be used informationally without
breaking pipelines that haven't opted in to strict mode.

### Diagnostic codes

**F115 — COMPILE-002 disambiguation.** COMPILE-002 currently covers both "source
file not found" and "directive has no source attribute". The message text
distinguishes them at runtime, but the code is shared. Future: may split into
COMPILE-002 (file not found) and COMPILE-005 (directive missing required
attribute) to allow distinct CI suppression rules.

**F122 — `source=` required for `mdloom:row`.** `source=md://...` is a required
attribute for `mdloom:row`. If absent, the directive is collected with an empty
source URI and fails at compile time with COMPILE-002 ("directive has no source").
Future: `mdloom check` should catch this at lint time via a `source_links` check
extension, so authors get the error before invoking compile.

### Math rendering

**F124 — MATH-004 severity.** MATH-004 (display math overflow) is currently a
warning — `written=true`. Consider promoting to error for display math blocks
where clipping produces meaningless output (e.g., a multi-line aligned equation
where the right-hand side is cut). For now, authors must set appropriate `width`
to avoid clipping. The current warning-level treatment matches the general
philosophy that overflow is a presentation issue, not a correctness issue.

**F129 — Mixed subscript fallback.** If a subscript argument contains a mix of
characters (some with Unicode subscript equivalents, some without), the entire
argument uses bracket notation `_{...}`. There is no partial Unicode expansion
within a single subscript argument. Example: `x_{2a}` renders as `x_{2a}` (literal
braces in output) rather than as `x₂a` — because `a` has no subscript form, the
whole group falls back. Same rule for superscripts.

**F130 — `expand_inline_math` API success semantics.** `expand_inline_math` always
returns `Ok((string, Vec<MathDiag>))`. Partial expansion (unknown commands,
MATH-005 downgrades) is represented as warnings in the diagnostic list, not as
`Err`. The function only returns `Err` for internal panics, which should not occur
in normal use. Callers should not pattern-match on `Err` for user-facing errors —
they should iterate the diagnostic vector and check severity.

**F131 — `render_display_math` return type.** Signature:

```rust
render_display_math(expr: &str, width: usize, align: Align) -> (Vec<String>, Vec<MathDiag>)
```

Returns one `String` per output line. Single-line expressions return a `Vec` with
one element. Empty expressions return an empty `Vec`. The function does not return
`Result` — internal failures are panics, and partial-render warnings are in the
diagnostic vector.

**F132 — `render_display_math` overflow check.** `render_display_math` applies
MATH-004 when any output line exceeds the declared `width`. With `width=0` (auto),
no overflow is possible — auto-width expands to fit the widest line. Authors who
want strict width enforcement must pass an explicit non-zero `width`.

### Canvas and clipping

**F123 — Unicode-safe clip.** `clip_to_width` must not split wide Unicode
characters. If a 2-column CJK character (or other East Asian Wide character)
straddles the clip boundary, the clip occurs before it — the character is excluded
entirely, not truncated. The resulting line may be one column shorter than `width`
because of this. Splitting a wide character produces broken output that downstream
terminal tools cannot render correctly.

**F125 — `Canvas::paste` writes only line width.** `Canvas::paste` writes only the
characters present in each pasted line. Cells to the right of the line content
retain their existing value (typically space from canvas initialization). Paste
does NOT zero-fill or space-fill the remainder of the row. This allows layered
pastes to compose without erasing prior content in the unpainted region.

**F126 — `render()` trailing spaces.** `render()` produces rows of exactly `width`
characters including trailing spaces. Trailing spaces are NOT trimmed. This
preserves the fixed-width guarantee (D-6) and ensures downstream tools see
consistent row lengths. Authors who need trimmed output should post-process
`render()`'s result; the canvas itself does not trim.

**F128 — Wide char at last column.** If a wide character (2 columns) is positioned
at the last available column, only the first cell is written. The second cell
(which would be column `width`) is out-of-bounds and is silently skipped. The
character appears truncated — one column of a two-column glyph. To avoid this,
authors should size their canvas with the wide-char alignment in mind, or use
`clip_to_width` (per F123) which handles the boundary correctly by excluding the
character entirely.
