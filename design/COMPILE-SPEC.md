# proof compile — Markdown Compilation Specification v0.1

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
````

### Directive types

| Directive | Purpose |
|-----------|---------|
| `proof:include` | Embed a single figure, table, or text element |
| `proof:layout` | Compose N figures side-by-side (see LAYOUT-SPEC.md) |
| `proof:table` | Embed a table with optional column selection and filtering |
| `proof:figure` | Explicitly mark a code block as a named figure (for figure files) |

### Figure files

A figure file is a standalone `.md` file containing ONLY figures. Each figure is
marked with a `<!-- proof:figure -->` comment that gives it an identity:

```markdown
<!-- proof:figure id="goroutine-scheduler" kind="figure.flowchart" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
│  Goroutines (G)...
└─────────────────────────────────────┘
```
```

Figure files live in a `figures/` directory (or anywhere) and are addressed by
`md://` URI: `md://figures/goroutine-scheduler.md#goroutine-scheduler:0`.

---

## CLI commands

```bash
proof compile input.md                    # compile to input.compiled.md
proof compile input.md -o output.md       # explicit output path
proof compile src/*.source.md             # batch compile
proof compile . --watch                   # watch mode: recompile on change
proof compile . --check                   # validate without writing output
proof compile . --cache-status            # show per-file cache tier hits/misses

# Cache snapshot management
proof cache snapshot save "production"
proof cache snapshot restore "production"
proof cache snapshot diff "before" "after"
proof cache snapshot list
proof cache snapshot prune --keep 5
proof cache snapshot deploy "production" --to ./dist/
```

---

## Three-Tier Cache

Each tier is content-addressed
(keyed by hash of inputs). A change at any tier cascades forward.

```
source change
    │
    ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Parse Cache  │────▶│ Resolve Cache│────▶│Compile Cache │
│  (Tier 1)   │     │  (Tier 2)    │     │  (Tier 3)    │
└──────────────┘     └──────────────┘     └──────────────┘
  source hash          parse key +          resolve key +
  + mdpath version     URI targets +        layout config +
                        mdpath version       proof version
```

### Tier 1: Parse Cache

Caches the `ParsedDocument` for each `.md` file.

```
parse_key = SHA-256(
    file_content_hash,
    mdpath_version
)
```

**Hit:** Return cached `ParsedDocument`. No re-parsing.
**Miss:** Parse file, cache result.

### Tier 2: Resolve Cache

Caches the resolved content of each `md://` URI.

```
resolve_key = SHA-256(
    parse_key,             ← from Tier 1 of the target file
    uri_string,
    mdpath_version
)
```

**Hit:** Return cached `ResolvedElement`. No re-reading the target file.
**Miss:** Resolve via mdpath, cache result.

### Tier 3: Compile Cache

Caches the full compiled output of a source document.

```
compile_key = SHA-256(
    source_parse_key,      ← Tier 1 of the source document
    sorted_resolve_keys[], ← Tier 2 of all included figures
    layout_config_hash,    ← gap, align settings
    proof_version
)
```

**Hit:** Return cached compiled document. No re-laying-out figures.
**Miss:** Resolve all directives, compose layout, write output.

### Cascading invalidation

| What changed | Parse | Resolve | Compile |
|-------------|-------|---------|---------|
| Source document content | MISS | MISS | MISS |
| Included figure content | HIT | MISS | MISS |
| Layout config (gap, align) | HIT | HIT | MISS |
| proof version | MISS | MISS | MISS |
| DaVinci invariants only | HIT | HIT | MISS (re-validate) |

The causal chain means you never manually invalidate. Content-addressed keys
automatically miss when inputs change.

### Storage layout

```
.proof/cache/
  parse/           ← Tier 1: ParsedDocument per .md file
    {key}.json
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

Directly adapted from craftworks cache-snapshots.md. Named snapshots let you
save and restore compile states — useful before risky diagram edits.

```bash
proof cache snapshot save "before-redesign"    # capture current state
proof cache snapshot restore "before-redesign" # instant rollback
proof cache snapshot diff "v1" "v2"           # which files changed between states?
proof cache snapshot deploy "production" --to ./dist/  # materialize without recompile
```

### Snapshot structure

```
.proof/cache/snapshots/{name}/
  manifest.json    ← SnapshotManifest
  parse/           ← copy of Tier 1 entries
  resolve/         ← copy of Tier 2 entries
  compile/         ← copy of Tier 3 entries
```

```toml
# SnapshotManifest (stored as JSON)
[manifest]
name = "production"
created_at = 1745000000000   # epoch ms
proof_version = "0.2.0"
integrity_hash = "sha256:..."  # over manifest + all cache entry keys

[[files]]
source = "languages/10-GO.source.md"
output = "languages/10-GO.md"
parse_key = "abc123"
resolve_keys = ["def456", "ghi789"]
compile_key = "jkl012"
```

### Integrity verification

Every restore verifies the snapshot's integrity hash before applying.
A mismatch means the snapshot is corrupt or tampered — restore is rejected.

### Crash safety

Save uses atomic temp-then-rename. A crashed save leaves only a temp directory —
the named snapshot either exists completely or not at all.

---

## Compilation pipeline

```
proof compile source.md
    │
    ├── 1. Parse source.md → ParsedDocument (Tier 1 cache check)
    │
    ├── 2. Find all proof: directives (include, layout, table)
    │
    ├── 3. For each directive:
    │       ├── Resolve md:// URI via mdpath (Tier 2 cache check)
    │       ├── Validate DaVinci invariants (FAIL if violated)
    │       └── Stage resolved content
    │
    ├── 4. For proof:layout directives:
    │       └── Apply layout engine (see LAYOUT-SPEC.md)
    │
    ├── 5. Check Tier 3 compile cache
    │       ├── HIT → write cached output, done
    │       └── MISS → continue
    │
    ├── 6. Compose final document:
    │       ├── Replace each directive with resolved/composed content
    │       └── Embed proof:source comments for traceability
    │
    ├── 7. Write output file (atomic)
    │
    └── 8. Update Tier 3 cache
```

### DaVinci validation during compile

The compile step validates ALL included figures against their registered DaVinci
invariants BEFORE composing output. This is the key quality gate:

```
DaVinci violation during compile:
  → compile fails with error [fig_invariant_violated]
  → output file is NOT written
  → error shows which figure, which invariant, which URI

The compiled output is ALWAYS invariant-clean. If it compiled, it passed.
```

---

## Watch mode

```bash
proof compile . --watch
```

Watches all source `.source.md` files and re-compiles on change. Uses the
three-tier cache so only changed figures trigger re-resolution.

```
[proof] watching . for source changes...
[proof] detected change: figures/goroutine-scheduler.md
  → invalidated resolve cache for 2 URIs
  → recompiling: languages/10-GO.source.md
  → ✓ compiled in 43ms
[proof] watching...
```

---

## Traceability in compiled output

Compiled output embeds HTML comments that trace each embedded figure back to
its source URI. This lets `proof check` on the compiled output emit errors
with stable `md://` addresses rather than just file:line:col.

```markdown
<!-- proof:compiled from="md://figures/goroutine-scheduler.md#:0" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌─────────────────────────────────────┐
...
└─────────────────────────────────────┘
```
<!-- /proof:compiled -->
```

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `COMPILE-001` | error | DaVinci invariant violated in included figure — compile aborted |
| `COMPILE-002` | error | md:// URI failed to resolve — target does not exist |
| `COMPILE-003` | warning | Included figure has no DaVinci pin — unprotected |
| `COMPILE-004` | error | Cache snapshot integrity hash mismatch — restore rejected |
| `COMPILE-005` | error | Cache snapshot restore rejected: compile in progress |
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
