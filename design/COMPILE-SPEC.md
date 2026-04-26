# proof compile — Markdown Compilation Specification v0.2

**Status:** Design — not yet implemented.

---

## What it is

`proof compile` is a markdown compiler that resolves **include directives** in source
markdown files and produces rendered output markdown. Source files use `proof:include`
and `proof:layout` fenced blocks to reference figures, tables, and charts by `md://`
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
  so proof check can still validate the output

---

## Source document format

Source documents use `proof:` info strings on fenced blocks:

````markdown
## Concurrency Model

Intro text here.

```proof:include
md://languages/10-GO.md#concurrency-model:figure.flowchart:goroutine-scheduler
```

Compare Go and Rust concurrency:

```proof:layout gap=4 align=top
md://languages/10-GO.md#concurrency-model:0
md://languages/09-RUST.md#ownership-model:0
```

The type system at a glance:

```proof:table
md://languages/10-GO.md#type-system-snapshot:table.key-value:0
?select=Axis,Value&filter=Axis contains runtime
```
<!-- filter/select syntax: see TABLE-FILTER-SPEC.md (planned) -->
````

### Directive types

| Directive | Purpose |
|-----------|---------|
| `proof:include` | Embed a single figure, table, or text element |
| `proof:layout` | Compose N figures side-by-side (see LAYOUT-SPEC.md) |
| `proof:table` | Embed a table with optional column selection and filtering |

### Figure files

A figure file is a standalone `.md` file whose figures are marked with
`<!-- proof:figure -->` HTML comments immediately preceding each code fence:

```markdown
<!-- proof:figure id="goroutine-scheduler" kind="figure.flowchart" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
│  Goroutines (G)...
└─────────────────────────────────────┘
```
```

The HTML comment is **outside** the code fence — markdown renderers hide it, but
proof indexes it to give the following code block a stable named identity. A figure
file may contain multiple `<!-- proof:figure -->` markers, one before each code block.

Figure files live in a `figures/` directory (or anywhere) and are addressed by
`md://` URI. See [URI Resolution](#uri-resolution) for how named IDs map to selectors.

---

## URI resolution

### Base path

All `md://` URIs are resolved relative to the **proof.toml root directory** — the
directory containing the `proof.toml` file, or the current working directory if no
`proof.toml` is found. The source file's own directory is NOT the base.

```
proof.toml root: /project/
source file:     /project/languages/10-GO.source.md
URI:             md://figures/goroutine-scheduler.md#goroutine-scheduler:0
resolves to:     /project/figures/goroutine-scheduler.md  ← from project root
```

### Named figure ID selector

The URI fragment `#goroutine-scheduler:0` is a named figure selector. It resolves to
the code block immediately following the `<!-- proof:figure id="goroutine-scheduler" -->` marker in the target file. The `:0` ordinal selects the first (and usually only) code block following that marker.

Proof indexes all `proof:figure id=` markers in a file during the parse step. The
parse cache stores this index as part of the `ParsedDocument`. The mdpath URI scheme's
`#heading:kind:ordinal` form also works — named figure IDs are an additional selector
shortcut layered on top.

### What `proof:include` embeds

`proof:include` embeds the **content inside the code fence**, not the raw fence
delimiter lines. Given this figure:

```
<!-- proof:figure id="foo" kind="figure.flowchart" -->
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
proof compile input.source.md             # compile to input.md (drops .source.)
proof compile input.source.md -o out.md   # explicit output path
proof compile src/*.source.md             # batch compile all source files
proof compile . --watch                   # watch mode: recompile on change
proof compile . --check                   # validate without writing output
proof compile . --cache-status            # show per-file cache tier hits/misses
proof compile . --no-cache                # bypass all cache tiers

# Cache snapshot management
proof cache snapshot save "production"
proof cache snapshot restore "production"
proof cache snapshot diff "before" "after"
proof cache snapshot diff "before" --vs-current
proof cache snapshot list
proof cache snapshot prune --keep 5
proof cache snapshot deploy "production" --to ./dist/
```

### Output path convention

When compiling `foo.source.md`, the default output is `foo.md` (dropping `.source.`),
in the same directory as the source file. All paths are relative to the proof.toml
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
  proof version         proof version       proof version
```

### Tier 1: Parse Cache

Caches the `ParsedDocument` for each `.md` file.

```
parse_key = SHA-256(
    file_content_hash,
    proof_version
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
.proof/cache/parse-index.json   ← maps file path → (parse_key, mtime)
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
    proof_version
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
    proof_version
)
```

**`sorted_resolve_keys[]` must NOT be deduplicated.** If the same URI appears twice
in a source document, its resolve_key appears twice in this list. Deduplication would
cause two documents with different include counts to share the same compile_key, which
is a silent correctness bug.

**`layout_config_hash`**: When a source document has no `proof:layout` directives,
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
| proof version | MISS | MISS | MISS |
| DaVinci invariants only | HIT | HIT | MISS (re-validate) |

The causal chain means you never manually invalidate. Content-addressed keys
automatically miss when inputs change.

### Compile cache entry schema

Each `.proof/cache/compile/{key}.json` stores:

```rust
struct CompileCacheEntry {
    compile_key: String,
    source_path: String,        // relative to proof.toml root
    output_path: String,        // relative to proof.toml root
    compiled_text: String,      // the full compiled markdown
    resolved_uris: Vec<String>, // all md:// URIs that were embedded
    proof_version: String,
    created_at: u64,            // epoch ms
}
```

### Storage layout

```
.proof/cache/
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
proof cache snapshot save "before-redesign"          # capture current state
proof cache snapshot restore "before-redesign"        # restore cache state (not files)
proof cache snapshot diff "v1" "v2"                  # which files changed?
proof cache snapshot diff "before-redesign" --vs-current   # diff vs live cache
proof cache snapshot deploy "production" --to ./dist/ # materialize without recompile
```

**Important:** `restore` is a cache operation — it restores cached compiled artifacts,
not the working files. After restore, source documents and figure files remain in their
current edited state on disk. Files that were edited since the snapshot was saved will
naturally miss the restored cache and recompile on the next `proof compile .`.

---

## Compilation pipeline

```
proof compile source.md
    │
    ├── 1. Parse source.md → ParsedDocument (Tier 1 cache check via path index)
    │       └── HIT: use cached ParsedDocument for all directive extraction below
    │
    ├── 2. Extract all proof: directives from ParsedDocument
    │       (proof:include, proof:layout, proof:table)
    │
    ├── 3. Compute resolve_keys for all directive URIs
    │       ├── Look up parse_key of each target file via path index
    │       ├── resolve_key = SHA-256(target_parse_key, uri_string, proof_version)
    │       └── Collect as sorted_resolve_keys[] — preserve duplicates, do not deduplicate
    │
    ├── 4. Compute compile_key, check Tier 3 cache
    │       ├── compile_key = SHA-256(source_parse_key, sorted_resolve_keys[],
    │       │       layout_config_hash, proof_version)
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
    ├── 6. For proof:layout directives:
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
proof compile . --watch
```

Watch mode watches **both** source documents (`.source.md`) and all figure files
referenced by any `md://` URI in any source document. When a figure file changes, only
the source documents that include it are recompiled.

```
[proof] watching . for changes (12 source files, 8 figure files)...
[proof] detected change: figures/goroutine-scheduler.md
  → recompiling: languages/10-GO.source.md (includes this figure)
  → ✓ compiled in 43ms
[proof] watching...
```

Content-addressed keys naturally miss for changed files — watch mode does **not**
explicitly invalidate cache entries. The recompile simply produces a new hash that
misses the cache.

### Watch inverse index

To efficiently determine which source documents include a changed figure, watch mode
builds an inverse index on startup:

```
.proof/cache/watch-index.json  ← maps figure_path → [source_paths that include it]
```

This index is rebuilt when source documents are compiled or recompiled. A new source
document that has never been compiled is not yet in the index and will not be
automatically recompiled when its figures change — run `proof compile .` once to
build the index, then watch mode is fully responsive.

---

## Traceability in compiled output

Compiled output embeds HTML comments that trace each embedded element back to its
source URI. This lets `proof check` on the compiled output emit errors with stable
`md://` addresses rather than just file:line:col.

```markdown
<!-- proof:compiled from="md://figures/goroutine-scheduler.md#goroutine-scheduler:0" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
...
└─────────────────────────────────────┘
```
<!-- /proof:compiled -->
```

For `proof:layout`, the traceability comment lists all composed URIs:

```markdown
<!-- proof:compiled from="proof:layout"
     uris="md://figures/go-types.md#go-type-system:0,md://figures/rust-types.md#rust-type-system:0" -->
```
      Go                         Rust
Axis         | Value        Axis         | Value
...
```
<!-- /proof:compiled -->
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

---

## New roles needed

This compilation system expands proof's scope significantly. Three new roles:

**SOURCE** — source/output document model expert. Understands the include system,
directive syntax, traceability comments, and the relationship between `.source.md`
and compiled `.md` files.

**COMPOSE** — layout and visual composition specialist. Understands the layout
engine, figure framing, alignment, padding, gap management, and how compositions
render in different column widths.

**CACHE** — cache correctness specialist. Understands cache key computation,
cascading invalidation, snapshot integrity, and crash safety. Pulls against
SOURCE and COMPOSE (more correctness, less throughput).
