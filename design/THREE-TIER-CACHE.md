# Three-Tier Build Cache — Parse, Resolve, Compile

> **Status**: Planned — types and APIs described here are design targets, not yet implemented.

## When you need this

Read this guide if you are:

- **Debugging cache misses** — you expected a hit but something changed upstream and you need to understand the cascade.
- **Working on compilation** — you need to understand how cache keys chain across tiers.
- **Investigating performance** — you want to know which tier is causing re-work.
- **Using `--no-cache` or `--cache-status`** — you need to understand what these flags control.

If you want to understand how invalidation propagates through the document graph, see [incremental compilation](./incremental-compilation.md). For named cache snapshots and state switching, see [cache snapshots](./cache-snapshots.md).

---

## The short version

The build pipeline has three cache tiers in a causal chain. Each tier's key includes the previous tier's key, so a change at any level cascades forward:

```
source change
    │
    ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    Parse    │────▶│   Resolve   │────▶│   Compile   │
│    Cache    │     │    Cache    │     │    Cache    │
└─────────────┘     └─────────────┘     └─────────────┘
  file content        parse key +         resolve keys +
  hash +              URI targets +       layout config +
  proof version       proof version       proof version
```

A source file change misses all three tiers. A figure file change misses resolve and compile but hits parse of the source document. A layout config change misses only compile.

## Why three tiers, not one

A single cache keyed on all inputs would work but would be wasteful. When you change a layout gap setting, you don't need to re-parse source documents or re-resolve figure URIs — you only need to re-compose the final output. The three-tier design reflects a deeper truth about the pipeline: **parsing, resolution, and compilation are independent concerns with different change frequencies.** Source documents change daily. Figure files change when diagrams are updated. Layout config changes rarely.

The causal chain (each tier's key includes the previous tier's) is what makes the system self-correcting. You never need to reason about "did I invalidate the right thing?" — a change at any level automatically cascades forward. This eliminates an entire class of staleness bugs that plague flat cache designs.

---

## Tier 1: Parse cache

The parse cache stores the `ParsedDocument` for each `.md` file.

### Cache key inputs

| Input | Source | Why it matters |
|-------|--------|---------------|
| File content hash | SHA-256 of file bytes | Any content change invalidates |
| proof version | `proof` binary version | Parser upgrade may change structure |

### Key computation

```
parse_key = SHA-256(
    file_content_hash,
    proof_version
)
```

### Storage

Content-addressed files in `.proof/cache/parse/`. Each entry is keyed by its cache key hash. Atomic writes via temp-file-then-rename prevent partial entries.

### Hit behavior

On cache hit, the `ParsedDocument` is read from disk and returned without re-parsing. The parsed document includes all extracted headings, directives, figures, tables, and code blocks.

---

## Tier 2: Resolve cache

The resolve cache stores the resolved content of each `md://` URI.

### Cache key inputs

| Input | Source | Why it matters |
|-------|--------|---------------|
| Parse key | Tier 1 output of the target file | Target file change cascades |
| URI string | The full `md://` URI | Different URI, different result |
| proof version | `proof` binary version | Resolver upgrade re-resolves |

### Key computation

```
resolve_key = SHA-256(
    parse_key,          ← from Tier 1 of the target file
    uri_string,
    proof_version
)
```

### Hit behavior

On cache hit, the `ResolvedElement` is returned without re-reading the target file or re-running the mdpath resolver. This is significant when many source documents include the same figure — each figure is resolved once and cached.

### When resolve misses but parse hits

This happens when the target file's URI has changed but the source document hasn't. Common scenario: a figure file is edited, invalidating all resolve cache entries that point to it, but source documents that reference it get their parse results from cache.

---

## Tier 3: Compile cache

The compile cache stores the full compiled output of a source document — the result of resolving all directives, composing layouts, and writing the final markdown.

### Cache key inputs

| Input | Source | Why it matters |
|-------|--------|---------------|
| Source parse key | Tier 1 of the source document | Source change invalidates |
| Resolve keys | Tier 2 of all included figures | Figure change cascades |
| Layout config hash | Hash of gap, align, width settings | Layout change re-compiles |
| proof version | `proof` binary version | Compiler upgrade re-compiles |

### Key computation

```
compile_key = SHA-256(
    source_parse_key,       ← Tier 1 of the source document
    sorted_resolve_keys[],  ← Tier 2 of all included figures
    layout_config_hash,     ← gap, align settings
    proof_version
)
```

### Storage

Content-addressed files in `.proof/cache/compile/`. Each entry stores the full compiled markdown output for a source document.

### Hit behavior

On cache hit, the compiled markdown is read from disk and written to the output path without re-running the compilation pipeline. DaVinci validation results are also cached — if it compiled, it passed.

---

## Cascading invalidation

The causal chain means changes cascade forward through the tiers:

| What changed | Parse | Resolve | Compile |
|-------------|-------|---------|---------|
| Source document content | MISS | MISS | MISS |
| Figure file content | HIT (source) | MISS | MISS |
| Layout config (gap, align) | HIT | HIT | MISS |
| proof version | MISS | MISS | MISS |
| DaVinci invariants only | HIT | HIT | MISS (re-validate) |

The key insight: you never need to explicitly delete cache entries. Content-addressed keys naturally miss when inputs change.

---

## CLI flags

### `--no-cache`

Bypass all cache tiers. Forces full parse, resolve, and compile regardless of cached state.

The `--no-cache` flag applies to the command it is passed to. `proof compile --no-cache` bypasses all three tiers for that compile invocation. It does not affect subsequent runs.

```bash
proof compile source.md --no-cache        # bypass all cache tiers for this compile
proof compile . --no-cache                # bypass for all source files
```

### `--cache-status`

Report cache tier hits and misses without changing behavior. Shows per-file, per-tier status.

```bash
proof compile . --cache-status
# Output:
# languages/10-GO.source.md:   parse HIT  | resolve HIT  | compile MISS (layout changed)
# languages/09-RUST.source.md: parse HIT  | resolve HIT  | compile HIT
# overview.source.md:          parse MISS | resolve MISS | compile MISS (source changed)
```

---

## Content-addressed storage

All three tiers use the same storage model:

1. **Key computation**: deterministic hash of all inputs.
2. **Lookup**: check if a file named `{key}.json` exists in the tier's cache directory.
3. **Read**: deserialize the cached entry (JSON + schema validation).
4. **Write**: serialize to temp file, compute SHA-256, rename atomically to final path.

Atomic writes (temp then rename) prevent partial entries from corrupting the cache. If a process crashes mid-write, the temp file is orphaned — never visible as a cache entry.

### Directory layout

```
.proof/cache/
  parse/           ← Tier 1: ParsedDocument per .md file
    {key}.json
  resolve/         ← Tier 2: ResolvedElement per md:// URI
    {key}.json
  compile/         ← Tier 3: compiled output per source document
    {key}.json
  snapshots/       ← Named snapshots (see cache-snapshots.md)
    production/
    canary/
```

---

## Three-tier cache keys type

The `TieredCacheKeys` type represents all three tiers for a single source document:

```rust
struct TieredCacheKeys {
    parse: CacheKey,            // always present
    resolve: Vec<CacheKey>,     // one per included md:// URI
    compile: Option<CacheKey>,  // present only if compile was run
}
```

`CacheKey` is a newtype over `String` in proof, preventing accidental use of arbitrary strings as cache keys.

This type appears in snapshot manifests and cache status reports.

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| COMPILE-004 | error | Cache snapshot integrity hash mismatch |
| COMPILE-005 | error | Cache snapshot restore rejected: compile in progress |
| COMPILE-006 | warning | Snapshot missing some files present in current state |

---

## Key files

| File | Purpose |
|------|---------|
| `src/cache/parse_cache.rs` | Tier 1: ParsedDocument cache |
| `src/cache/resolve_cache.rs` | Tier 2: ResolvedElement cache (planned) |
| `src/cache/compile_cache.rs` | Tier 3: compiled output cache (planned) |
| `src/cache/snapshot.rs` | Named snapshot manager (planned) |

---

## See also

- [Cache Snapshots](./cache-snapshots.md) — named snapshots for state switching
- [Compile Spec](./compile-spec.md) — the compilation pipeline that uses these caches
- [Layout Spec](./layout-spec.md) — layout engine whose config is part of the compile key
