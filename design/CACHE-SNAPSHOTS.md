# Cache Snapshots — Named Compile States for Safe Experimentation

> **Status**: ✅ Implemented — `src/cache.rs` (snapshot_save / restore / list / diff / prune / deploy). CLI surface lives at `proof cache snapshot {save|restore|list|diff|prune|deploy}`. Integrity hash covers manifest + per-file tier keys; tampered snapshots are rejected with `COMPILE-004`. Snapshots live under `.proof/cache/snapshots/<name>/` and capture parse + resolve + compile tiers.

## When you need this

Read this guide if you are:

- **Editing source documents** and want instant rollback to a known-good compiled state.
- **Comparing two compile states** — you need to know which source files differ between "before" and "after".
- **Deploying from cache** — you want to materialize compiled artifacts to a directory layout without recompiling.
- **Debugging snapshot corruption** — you got `COMPILE-004` and need to understand integrity verification.

If you want to understand the three cache tiers themselves, see [three-tier cache](./three-tier-cache.md).

---

## The short version

Named cache snapshots are like git refs pointing at sealed compile states. `save` is a commit, `restore` is a checkout, `diff` shows which files changed between two snapshots, `prune` is garbage collection, and `deploy` is an archive export. Each snapshot captures all three cache tiers (parse, resolve, compile) with an integrity hash that is verified before restore.

```
save "production"           ← snapshot current three-tier cache state
save "canary"               ← snapshot after experimental edits
diff "production" "canary"  ← which source files changed?
restore "production"        ← instant rollback to known-good state
deploy "production" --to ./dist/  ← materialize compiled output without recompile
```

## Why this matters

Cache snapshots solve a problem that every long-lived build system eventually hits: **how do you reason about compile state across time?** Without snapshots, the cache is a black box — you know it speeds things up, but you can't name a state, compare two states, or go back to a previous one. With snapshots, the cache becomes a versioned store with the same mental model as git.

This matters for proof because editing large figure libraries is risky. When an author rewires figures that dozens of source documents include, a bad edit breaks compiled output across the whole library. Without snapshots, rollback means recompiling everything from scratch. With snapshots, rollback is a restore operation that completes in seconds.

The integrity hash prevents corruption (the obvious purpose), but it also makes snapshots **portable**. Because the hash covers the manifest and all cache keys, a snapshot that passes verification is guaranteed to be complete and self-consistent, regardless of how it arrived on disk.

---

## Snapshot structure

A snapshot is a named copy of cache entries stored under `.proof/cache/snapshots/{name}/`:

```
.proof/cache/snapshots/
  production/
    manifest.json       ← SnapshotManifest
    parse/              ← copied parse cache entries
      {key}.json
    resolve/            ← copied resolve cache entries
      {key}.json
    compile/            ← copied compile cache entries
      {key}.json
  canary/
    manifest.json
    parse/
    resolve/
    compile/
```

### SnapshotManifest

```rust
struct SnapshotManifest {
    name: String,
    created_at: u64,                              // epoch ms
    proof_version: String,
    files: Vec<SourceFileRef>,                    // which source documents are captured
    tiers: HashMap<String, TieredCacheKeys>,      // per-file, three-tier keys
    total_size: u64,                              // bytes
    integrity_hash: String,                       // SHA-256 over manifest + all cache entry keys
}
```

The `tiers` field maps source file paths (as strings) to their three-tier cache keys. File paths are stored as plain strings because the manifest must be serializable to JSON and back — newtype wrappers don't survive serialization. Code that reads the manifest re-wraps after validation.

```rust
struct TieredCacheKeys {
    parse: CacheKey,
    resolve: Vec<CacheKey>,       // one per included md:// URI
    compile: Option<CacheKey>,    // present only if compile was run
}
```

---

## Save — capturing a snapshot

`proof cache snapshot save "production"` copies current cache entries to a named snapshot directory.

```
save_snapshot("production", cache_dir)
    │
    ├── Read current cache state for all files WITH cache entries
    │       (files that have never been compiled have no entries — they are not included)
    ├── Copy cache entries to temp directory
    │       → parse entries for each source file
    │       → resolve entries (if present)
    │       → compile entries (if present)
    │
    ├── Build SnapshotManifest
    │       → source file list
    │       → per-file TieredCacheKeys
    │       → total size
    │
    ├── Compute integrity hash
    │       → SHA-256 over manifest + all cache entry keys
    │
    └── Atomic rename: temp dir → .proof/cache/snapshots/{name}/
```

### Crash safety

Save uses an atomic temp-then-rename protocol. If the process crashes mid-save, only the temp directory is left — the named snapshot either exists completely or not at all. No partial snapshots.

---

## Restore — switching to a snapshot

`proof cache snapshot restore "production"` restores cached compiled artifacts to a
snapshot's state. **This is a cache operation, not a file system rollback.** Working
files (source documents and figure files) are not touched. After restore, files that
were edited since the snapshot was saved will naturally miss the restored cache and
recompile on the next `proof compile .`.

```
restore_snapshot("production", cache_dir)
    │
    ├── Read manifest from .proof/cache/snapshots/production/manifest.json
    │
    ├── Verify integrity hash
    │       → recompute SHA-256 over manifest + cache entry keys
    │       → compare against stored integrity_hash
    │       → COMPILE-004 on mismatch (corrupted/tampered snapshot)
    │
    ├── Check compile state (server/session mode only)
    │       → COMPILE-005 if a long-running compile session is active
    │       → In CLI (one-shot) mode, this guard never fires
    │
    ├── Compare snapshot files against current source documents
    │       → COMPILE-006 warning on uncovered files (files compiled after snapshot was saved)
    │
    └── Copy snapshot entries to active cache directories
            → next compile run hits for all files in the snapshot
            → files edited since snapshot was saved naturally miss and recompile
```

### Verify-before-restore

The integrity hash is always verified before applying a restore. This prevents:

- **Partial snapshots**: crash during save left incomplete data.
- **Manual tampering**: someone edited cache files directly.
- **Disk corruption**: silent bit-flip in stored entries.

If verification fails, the restore is rejected with `COMPILE-004` and the active cache is unchanged.

---

## Deploy — materializing artifacts

`proof cache snapshot deploy "production" --to ./dist/` writes compiled artifacts to a target directory without recompiling.

```
deploy_snapshot("production", target_dir)
    │
    ├── Read and verify snapshot (same integrity check as restore)
    │
    └── For each source file in snapshot:
            └── Write compiled output to {target_dir}/{relative_path}
                    → uses cached compile artifacts
```

This enables build-once-deploy-many patterns. CI compiles the library, saves a snapshot, and multiple deploy targets materialize from the same cached state.

---

## Diff — comparing two snapshots

`proof cache snapshot diff "before" "after"` shows which source files differ between
two named snapshots. To diff against the current live cache without creating a snapshot,
use `--vs-current`:

```bash
proof cache snapshot diff "before-redesign" "after-redesign"  # two named snapshots
proof cache snapshot diff "before-redesign" --vs-current      # named vs. live cache
```

```rust
struct SnapshotDiff {
    only_in_a: Vec<String>,    // files only in first snapshot
    only_in_b: Vec<String>,    // files only in second snapshot
    changed: Vec<String>,      // same file, different cache keys
    identical: Vec<String>,    // same file, same cache keys
}
```

The comparison is per-file, per-tier. A file is "changed" if any of its three tier keys differ.

```bash
proof cache snapshot diff "before-redesign" "after-redesign"
# Output:
# Only in before: (none)
# Only in after: (none)
# Changed: languages/10-GO.source.md, overview.source.md
# Identical: languages/09-RUST.source.md, languages/08-TYPESCRIPT.source.md
```

---

## Prune — cleaning up old snapshots

`proof cache snapshot prune --keep 3` removes all but the N most recent snapshots, ordered by `created_at`.

```bash
proof cache snapshot prune --keep 3
# Removed: backup-20260401, test-20260330
# Kept: production, canary, staging
```

Returns the list of deleted snapshot names.

---

## Typical workflow

```
1. Before risky edits — save a baseline
       proof cache snapshot save "before-redesign"

2. Edit figures, source documents
       # ... make changes ...

3. Recompile
       proof compile .

4. Check what changed
       proof cache snapshot diff "before-redesign" --vs-current
       # See which source documents were affected

5a. Changes look good — save as new baseline
       proof cache snapshot save "production"

5b. Changes broke things — instant rollback
       proof cache snapshot restore "before-redesign"
       # Next proof compile . returns to pre-edit state
```

---

## CLI commands

```bash
proof cache snapshot save <name>                     # capture current state
proof cache snapshot restore <name>                  # switch to snapshot
proof cache snapshot deploy <name> --to <dir>        # materialize compiled output
proof cache snapshot list                            # show snapshots with dates + sizes
proof cache snapshot diff <name-a> <name-b>          # compare two named snapshots
proof cache snapshot diff <name-a> --vs-current      # compare snapshot vs. live cache
proof cache snapshot prune --keep <n>                # remove old snapshots
```

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| COMPILE-004 | error | Snapshot integrity hash mismatch (corrupted or partially written) |
| COMPILE-005 | error | Snapshot restore rejected: compilation in progress |
| COMPILE-006 | warning | Snapshot missing some source files present in current state |

---

## Key files

| File | Purpose |
|------|---------|
| `src/cache/snapshot.rs` | Snapshot manager (save, restore, diff, prune) (planned) |
| `src/cache/deploy.rs` | Deploy layout materialization (planned) |
| `src/commands/cache.rs` | CLI surface for snapshot commands (planned) |

---

## See also

- [Three-Tier Cache](./three-tier-cache.md) — the cache tiers that snapshots capture
- [Compile Spec](./compile-spec.md) — the compilation pipeline
