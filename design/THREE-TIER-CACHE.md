# Three-Tier Build Cache — Parse, Resolve, Compile

> **Status**: ✅ All three tiers implemented — `src/cache.rs`.
> - **Tier 1** (parse key): `PathIndex` + `get_or_compute_parse_key()` — content hash cached to disk, avoids re-hashing unchanged files.
> - **Tier 2** (resolve): `ResolveCacheEntry`, `try_resolve_cache_hit()`, `store_resolve_cache()` — resolved figure content cached by `(figure_content_hash, uri)`. The main compile loop (`proof:include`, `proof:layout`, `proof:table`) uses `resolve_uri_cached()`. 5 Tier-2 tests.
> - **Tier 3** (compile): `CompileCacheEntry`, `try_compile_cache_hit()`, `store_compile_cache()` — full compiled output keyed by source + all referenced figure content hashes.

## When you need this

Read this guide if you are:

- **Debugging cache misses** — you expected a hit but something changed upstream and you need to understand the cascade.
- **Working on compilation** — you need to understand how cache keys chain across tiers.
- **Investigating performance** — you want to know which tier is causing re-work.
- **Using `--no-cache` or `--cache-status`** — you need to understand what these flags control.

For named snapshots and state switching, see [Cache Snapshots](./cache-snapshots.md).

---

## Current state of the codebase

| Component | Status | Notes |
|-----------|--------|-------|
| `CompileResult.from_cache` | exists, always `false` | `src/compile.rs:22` — wired but unset |
| `mdpath::resolver::BatchResolver` | implemented | per-file `ParsedDocument` reuse, in-process only, no disk persistence |
| `mdpath::parse_document` | implemented | re-runs on every `BatchResolver::new` — no parse cache between commands |
| `.proof/cache/` directory | does not exist | not created on any current command |
| `--no-cache` flag | not implemented | not on `proof compile` |
| `--cache-status` flag | not implemented | not on `proof compile` |
| `proof cache snapshot ...` | not implemented | no `cache` subcommand exists |
| `src/cache/` module | does not exist | greenfield |

What this means: this spec is greenfield design. There is no migration concern, no compatibility constraint, no behavior to preserve. The only existing primitive that survives is `BatchResolver`, which acts as the **in-memory** layer underneath Tier 1 (a single command resolving N URIs from one figure file parses that file once, regardless of disk-cache hit/miss).

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
  file content        parse_key of       source parse_key +
  hash +              target file +      sorted resolve_keys +
  proof version       URI string +       layout config hash +
                      proof version      proof version
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
| proof version | `proof` binary version (`env!("CARGO_PKG_VERSION")`) | Parser upgrade may change structure |

### Key computation

```
parse_key = hex(SHA-256(
    file_content_hash,    ← 32 bytes, raw
    proof_version_bytes,  ← UTF-8 bytes, length-prefixed
))
```

Length-prefixing the version bytes prevents a `proof_version + content` boundary collision (e.g. version `"1.0"` + content `"foo"` vs. version `"1.0f"` + content `"oo"`).

### Storage

Content-addressed files in `.proof/cache/parse/`. Each entry is keyed by its `parse_key` hex string. The on-disk format is JSON:

```rust
struct ParseCacheEntry {
    parse_key: String,
    source_path: String,    // relative to proof.toml root, for diagnostic display only — NOT a key input
    content_hash: String,   // hex SHA-256 of original file bytes
    parsed: ParsedDocument, // serde-serialized mdpath::document::ParsedDocument
    proof_version: String,
    created_at: u64,        // epoch ms
}
```

`source_path` is informational; the cache is content-addressed, so two files with identical content share one entry. Two paths pointing to the same parse_key is normal and correct.

Atomic writes via temp-file-then-rename prevent partial entries.

### Path index — Tier 1's reverse lookup (resolves F17)

The parse cache is content-addressed (keys are hashes of content), so there is no built-in way to ask "what's the current parse_key for `figures/foo.md`?" Tier 2 needs exactly this answer to compute `resolve_key`.

A **path index** is maintained alongside the cache:

```
.proof/cache/parse-index.json
{
  "figures/foo.md":     { "parse_key": "abc123...", "mtime": 1714200000123, "size": 4321 },
  "languages/10-GO.md": { "parse_key": "def456...", "mtime": 1714200001456, "size": 8910 },
  ...
}
```

Lookup protocol:

1. Stat the file. Compare `(mtime, size)` to the index entry.
2. If both match, the cached `parse_key` is current — return it without re-hashing.
3. If either differs, re-hash file content, compute new `parse_key`, update the index entry, return.
4. If the path is absent from the index, cold path: hash, compute, insert.

The index is loaded once at command startup and rewritten atomically at command exit. Stale entries (paths whose files no longer exist) are pruned on the rewrite.

> **Note (F17):** Without this path index, every Tier 2 lookup would have to re-hash the target file, which would make the parse cache approximately useless for Tier 2's purposes. The index is what makes the causal chain efficient.

### Hit behavior

On cache hit, the `ParsedDocument` is read from disk and returned without re-parsing. The cached document includes all extracted headings, directives, figure markers, tables, and code blocks — everything needed to extract directive lists for the compile pipeline (F16: directive extraction reads the cached `ParsedDocument`, never re-parses).

### Interaction with `BatchResolver`

`BatchResolver::new()` currently always parses from disk. After Tier 1 ships, `BatchResolver` should accept an optional shared parse cache and consult it before parsing — turning `BatchResolver` into the in-memory layer above the on-disk parse cache. This is a follow-on change tracked separately; the two layers are conceptually distinct (in-process N-URI batch reuse vs. cross-command persistence).

---

## Tier 2: Resolve cache

The resolve cache stores the resolved content of each `md://` URI.

### Cache key inputs

| Input | Source | Why it matters |
|-------|--------|---------------|
| Target file `parse_key` | Tier 1 of the file the URI points to (looked up via path index) | Target file change cascades |
| URI string | The full `md://` URI as written in the source | Different URI, different result |
| proof version | `proof` binary version | Resolver upgrade re-resolves |

### Key computation

```
resolve_key = hex(SHA-256(
    target_parse_key,      ← from Tier 1 of the target file (via path index)
    uri_bytes,             ← UTF-8 bytes, length-prefixed
    proof_version_bytes,   ← UTF-8 bytes, length-prefixed
))
```

### Storage

`.proof/cache/resolve/{resolve_key}.json`:

```rust
struct ResolveCacheEntry {
    resolve_key: String,
    uri: String,                    // for diagnostic display
    target_path: String,            // relative path of resolved file
    target_parse_key: String,       // for chain-debugging
    resolved: ResolvedElement,      // serde-serialized mdpath::ResolvedElement
    proof_version: String,
    created_at: u64,
}
```

### Hit behavior

On cache hit, the `ResolvedElement` is returned without re-reading the target file or re-running the mdpath resolver. This is significant when many source documents include the same figure — each figure is resolved once and cached.

Same URI appearing N times in a single source document → 1 actual resolution + N Tier 2 cache hits, N embedded copies in compiled output (F21).

### When resolve misses but parse hits

This happens when the target file changed but the source document didn't. Common scenario: a figure file is edited, invalidating all resolve cache entries that point to it (their `target_parse_key` no longer matches), but source documents that reference it get their parse results from cache (their content hasn't changed).

---

## Tier 3: Compile cache

The compile cache stores the full compiled output of a source document — the result of resolving all directives, composing layouts, and writing the final markdown.

### Cache key inputs

| Input | Source | Why it matters |
|-------|--------|---------------|
| Source `parse_key` | Tier 1 of the source document | Source change invalidates |
| Resolve keys | Tier 2 of all included URIs, in **source order**, **NOT deduplicated** | Figure change cascades |
| Layout config hash | SHA-256 of normalized directive attributes | Layout / label / attr change re-compiles |
| proof version | `proof` binary version | Compiler upgrade re-compiles |

### Key computation

```
compile_key = hex(SHA-256(
    source_parse_key,
    resolve_keys_concatenated,  ← in source order, NOT deduplicated, length-prefixed
    layout_config_hash,
    proof_version_bytes,
))
```

#### Why resolve_keys is not deduplicated (F20 — critical)

If `figures/A.md` is included three times in a source document, its `resolve_key` appears three times in the input list. **Deduplication would be a silent correctness bug**: a document with one include and a document with three identical includes would produce the same `compile_key`, and the cache would hand back the wrong compiled output.

Order is also significant. `[A, B, A]` and `[A, A, B]` are different keys.

#### `layout_config_hash` — what's in it (F02, F07, F10)

The `layout_config_hash` covers **every directive attribute that affects rendering** across the whole source document, not just `proof:layout` blocks. Specifically:

- All `proof:layout` attributes: `gap`, `align`, `direction`, `cols`, `width`, `border`, **`labels`** (F07: labels are pure presentation, included here, not in resolve keys).
- All `proof:include`, `proof:table`, `proof:tree`, `proof:element`, `proof:row`, `proof:symbol`, `proof:shape`, `proof:region` attributes that affect output.
- The proof.toml `[ascii_box]`, `[markdown]`, and any other render-affecting config sections.

**Normalization rule (F10):** before hashing, every attribute is normalized to a canonical form:

1. Default values are filled in. `gap` omitted ≡ `gap=3` (the default) — both produce the same hash.
2. Attribute order within a directive is sorted alphabetically. `gap=3 align=top` and `align=top gap=3` hash identically.
3. Whitespace inside attribute values is preserved verbatim (it can be semantically meaningful).
4. The serialized form is a stable JSON representation: `[{directive: "layout", attrs: {align: "top", gap: 3, ...}}, ...]`.

**No-layout case (F02):** When a source document has no render-affecting directives, `layout_config_hash = SHA-256("[]")` — the hash of an empty JSON array. This is distinct from `SHA-256("")` and from any populated config, so "no directives" never collides with any specific configuration.

### Storage

`.proof/cache/compile/{compile_key}.json`:

```rust
struct CompileCacheEntry {
    compile_key: String,
    source_path: String,            // relative to proof.toml root
    output_path: String,            // relative to proof.toml root
    compiled_text: String,          // the full compiled markdown, ready to write
    resolved_uris: Vec<String>,     // all md:// URIs embedded, in source order, not deduplicated
    davinci_violations: Vec<CompileViolation>,  // cached check results — see F06
    proof_version: String,
    created_at: u64,
}
```

The `davinci_violations` field caches DaVinci invariant check results so a Tier 3 hit doesn't have to re-run them (F06). If `compiled_text` was produced by a successful compile, the violation list is whatever was emitted at that compile (including warnings). On hit, the violations are re-emitted to the user — they're part of the compile result, not the output file.

### Hit behavior

On cache hit:

1. Read `compiled_text` from the cache entry.
2. Compare against current contents of `output_path`. If identical, **skip the write entirely** — preserves output file mtime, avoids spurious watch mode recompiles in downstream tools (F18, F19).
3. Otherwise, write `compiled_text` to `output_path` atomically (temp + rename).
4. Re-emit `davinci_violations` to the user.
5. Return `CompileResult { from_cache: true, written: <bool>, violations, ... }`.

---

## Cascading invalidation

The causal chain means changes cascade forward through the tiers:

| What changed | Parse | Resolve | Compile | Notes |
|-------------|-------|---------|---------|-------|
| Source document content | MISS | (no Tier 2 entries are keyed by source) | MISS | Source's parse_key changes |
| Figure file content | HIT (source) MISS (figure) | MISS | MISS | Cascade through resolve_keys |
| Layout / directive attribute | HIT | HIT | MISS | Only layout_config_hash differs |
| `labels=` on proof:layout | HIT | HIT | MISS | Labels are presentation, in layout hash (F07) |
| proof.toml render config | HIT | HIT | MISS | Folded into layout_config_hash |
| proof binary upgrade | MISS | MISS | MISS | Version is in every tier's key |
| DaVinci invariants only (proof.toml `[[davinci]]`) | HIT | HIT | MISS | Treated as render config (re-validate cheaply) |

The key insight: **you never need to explicitly delete cache entries.** Content-addressed keys naturally miss when inputs change. Watch mode does NOT call any "invalidate" function (F27); it simply re-runs compile, which produces new keys for the changed files and naturally misses.

---

## CLI flags

### `--no-cache`

Bypass all cache tiers. Forces full parse, resolve, and compile regardless of cached state. Reads and writes are skipped — the run does not pollute or warm any tier.

The flag applies to the command it is passed to. `proof compile --no-cache` bypasses all three tiers for that compile invocation. It does not affect subsequent runs.

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

The "miss reason" annotation derives from which input differed (compared against the most recent cache entry for that source file's path). Implementation may store a small `last_keys.json` per source path to support this — the annotation is best-effort, not a correctness primitive.

---

## Content-addressed storage

All three tiers use the same storage model:

1. **Key computation**: deterministic hash of all inputs.
2. **Lookup**: check if a file named `{key}.json` exists in the tier's cache directory.
3. **Read**: deserialize the cached entry (JSON + schema validation; on validation failure, treat as miss and overwrite).
4. **Write**: serialize to temp file in the same directory, then `rename()` atomically to final path.

Atomic writes (temp then rename) prevent partial entries from corrupting the cache. If a process crashes mid-write, the temp file is orphaned — never visible as a cache entry. A startup sweep deletes orphan `*.tmp` files older than 1 hour.

### Directory layout

```
.proof/cache/
  parse/                ← Tier 1: ParsedDocument per .md file
    {key}.json
  parse-index.json      ← path → (parse_key, mtime, size)
  resolve/              ← Tier 2: ResolvedElement per md:// URI
    {key}.json
  compile/              ← Tier 3: compiled output per source document
    {key}.json
  snapshots/            ← Named snapshots (see CACHE-SNAPSHOTS.md)
    production/
    canary/
```

`.proof/` should be added to `.gitignore` by `proof init` (it is local build state).

### Garbage collection

Cache entries are not automatically pruned. A separate command (out of scope for this spec) will prune entries not referenced by the current path index or any active snapshot. For now, the cache grows monotonically; users can `rm -rf .proof/cache/` at any time to start fresh.

---

## Public types

### `CacheKey`

```rust
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct CacheKey(String);  // hex-encoded SHA-256, 64 chars
```

Newtype to prevent accidental use of arbitrary strings as cache keys.

### `TieredCacheKeys`

The three tiers for a single source document:

```rust
pub struct TieredCacheKeys {
    pub parse: CacheKey,                 // always present
    pub resolve: Vec<CacheKey>,          // one per included md:// URI, in source order, not deduplicated
    pub compile: Option<CacheKey>,       // present only if compile was run
}
```

Used in snapshot manifests and `--cache-status` reporting.

### `CompileResult` integration

`CompileResult.from_cache` (already present at `src/compile.rs:22`, currently always `false`) becomes meaningful:

- `from_cache: true` ⟺ Tier 3 hit produced the output (no actual compilation work happened beyond key computation, file read, and optional write).
- `written: true` ⟺ the output file was actually written (i.e. its prior contents differed from `compiled_text`, or it didn't exist).

The two are independent: a Tier 3 hit may or may not write (depends on whether the output file already matches).

---

## Diagnostic codes

| Code | Severity | Meaning | Defined here? |
|------|----------|---------|---------------|
| COMPILE-004 | error | Cache snapshot integrity hash mismatch | See [CACHE-SNAPSHOTS.md](./cache-snapshots.md) |
| COMPILE-005 | error | Cache snapshot restore rejected: compile in progress | See [CACHE-SNAPSHOTS.md](./cache-snapshots.md) |
| COMPILE-006 | warning | Snapshot missing some files present in current state | See [CACHE-SNAPSHOTS.md](./cache-snapshots.md) |

Tier 1/2/3 themselves emit no diagnostics — a corrupt cache entry is treated as a miss and silently overwritten on the next write. Read failures are diagnostic-free for the same reason.

---

## Key files (planned)

| File | Purpose |
|------|---------|
| `src/cache/mod.rs` | Module root, shared types (`CacheKey`, `TieredCacheKeys`) |
| `src/cache/parse_cache.rs` | Tier 1: `ParsedDocument` cache + path index |
| `src/cache/resolve_cache.rs` | Tier 2: `ResolvedElement` cache |
| `src/cache/compile_cache.rs` | Tier 3: compiled output cache |
| `src/cache/storage.rs` | Atomic write helpers, JSON read/validate, orphan cleanup |
| `src/cache/snapshot.rs` | Named snapshot manager (see CACHE-SNAPSHOTS.md) |

`src/compile.rs` integration points (already in place, unset):

- `CompileResult.from_cache` — set to `true` on Tier 3 hit
- `CompileResult.written` — already tracks the actual write decision

CLI surface in `src/main.rs`:

- `compile.no_cache: bool` — new flag
- `compile.cache_status: bool` — new flag
- `Command::Cache { ... }` — new subcommand for snapshot operations (see CACHE-SNAPSHOTS.md)

---

## Implementation order

A staged rollout that lets each tier be validated before the next is layered on top:

1. **Storage + key types** (`storage.rs`, `mod.rs`) — atomic write, JSON read, `CacheKey` newtype, `TieredCacheKeys`. No tier wired in yet.
2. **Tier 1 + path index** (`parse_cache.rs`) — add to `compile.rs` as a read-through wrapper around current parsing. Wire `BatchResolver` to consult it (in-process layer above on-disk Tier 1).
3. **Tier 2** (`resolve_cache.rs`) — add to the resolve path. At this point, `--cache-status` can show parse/resolve hits.
4. **Tier 3** (`compile_cache.rs`) — add to the compile path. `from_cache` becomes meaningful. `--no-cache` becomes meaningful.
5. **Snapshots** (`snapshot.rs`) — see [CACHE-SNAPSHOTS.md](./cache-snapshots.md).

Each step is independently shippable; correctness of a downstream tier is not blocked on the upstream tier landing first (a missing upstream just means more cache misses, not wrong results).

---

## Spec Clarifications (from scenario findings)

These clarifications resolve cache-related ambiguities surfaced during scenario testing. They are normative — implementations must conform.

### F02 — `layout_config_hash` when no directives present

`layout_config_hash = SHA-256("[]")` (the hash of an empty JSON array) when the source document contains no render-affecting directives. This is distinct from `SHA-256("")` and from any populated config, ensuring "no directives" never collides with any specific configuration.

### F06 — Compile cache entry schema

`CompileCacheEntry` includes `davinci_violations: Vec<CompileViolation>` so Tier 3 hits can re-emit DaVinci check results without re-running them. The full schema is documented in §"Tier 3 Storage" above.

### F07 — `labels=` is part of `layout_config_hash`

Labels are pure presentation. Changing a label re-composes the layout but does not re-resolve URIs. They are part of `layout_config_hash`, NOT part of any `resolve_key`.

### F10 — Attribute normalization before hashing

Default values are filled in, attribute keys are sorted alphabetically per directive, and the serialized form is a stable JSON representation. Two attribute orderings that mean the same thing produce the same hash.

### F16 — Directive extraction reads the cached `ParsedDocument`

On Tier 1 hit, the compile pipeline extracts directives from the cached `ParsedDocument` directly. It does NOT re-parse to find directives. Without this, Tier 1 would be nearly useless for the compile pipeline.

### F17 — Path index for "parse_key for this file path"

The parse cache is content-addressed, but Tier 2 needs path → `parse_key` lookup. The path index at `.proof/cache/parse-index.json` provides this mapping, validated by `(mtime, size)` on access. See §"Path index" above.

### F18 / F19 — Tier 3 hit avoids spurious file writes

On Tier 3 hit, the compiler reads the cached `compiled_text` and compares it against the current contents of `output_path`. If they are identical, the write is skipped to preserve the file's mtime — preventing spurious watch-mode recompiles in downstream tools that watch the output.

### F20 — `resolve_keys` is NOT deduplicated (CRITICAL)

If the same URI appears N times in a source document, its `resolve_key` appears N times in the `compile_key` input — in source order. Deduplication would cause documents with different include counts to share a `compile_key` and hand back wrong cached output. This is enforced as an invariant in `compile_cache.rs` and tested explicitly.

### F21 — Same URI N times → 1 resolution + N hits

The first include resolves and caches. The remaining N−1 includes hit Tier 2. The compiled output embeds N copies. This is the expected behavior — it falls out of content-addressed Tier 2 + non-deduplicated Tier 3 keys naturally.

### F27 — Watch mode does NOT "invalidate" the cache

Content-addressed caches do not need explicit invalidation. When a watched file changes, watch mode simply re-runs compile. The new file content produces a new content-hash, which produces a new `parse_key`, which naturally misses Tier 1, which cascades. There is no `invalidate()` function in the cache API — it would be wrong to delete entries that may still be reachable from snapshots.

---

## See also

- [Cache Snapshots](./cache-snapshots.md) — named snapshots for state switching, built on top of these three tiers
- [Compile Spec](./compile-spec.md) — the compilation pipeline that uses these caches
- [Layout Spec](./layout-spec.md) — layout engine whose config is part of the compile key
- [Scenarios](./scenarios.md) — full scenario traces, including the cache-correctness scenarios that produced findings F02–F31
