# proof Cache Snapshots — Named Compile States

Cache snapshots let you name a compile state and come back to it. Think of
each snapshot as a git ref pointing at a sealed copy of the three-tier cache
(parse + resolve + compile). `save` is a commit, `restore` is a checkout,
`diff` shows which files changed between two states, `prune` cleans up old
snapshots, `deploy` materializes compiled output without recompiling.

This is the right tool for any of these:

- **Editing risky figures.** Save before, edit, save after. If the edits
  break things you didn't intend, restore is instant — much faster than
  recompiling everything.
- **Comparing two states.** Diff between named snapshots shows exactly which
  source files changed; useful when reviewing a wave of edits.
- **Build-once, deploy-many.** CI compiles the library and saves a
  snapshot; downstream deploy targets materialize from the same cached
  state without recompiling.

---

## Save — capture the current state

```text
proof cache snapshot save production
```

Copies all current cache entries (parse, resolve, compile) into
`.proof/cache/snapshots/production/`. Writes a manifest carrying the
per-file three-tier keys plus an integrity hash over everything.

The save is atomic: a temp directory is built up, then renamed into place.
If the process crashes mid-save, the temp is left behind (you can delete it)
but the named snapshot either fully exists or doesn't.

Output:

```text
Saved snapshot "production" (12 files, 84231 bytes, integrity 4b8a3f0c1e92...)
```

---

## Restore — switch back to a snapshot

```text
proof cache snapshot restore production
```

Verifies the snapshot's integrity hash, then copies its cache entries back
into the active cache directories. **Working files (source documents and
figures on disk) are not touched.** After restore, files you've edited
since the snapshot will naturally miss the restored cache and recompile on
the next `proof compile .`.

If the integrity hash doesn't match what's stored, restore is rejected with
`COMPILE-004` and the active cache stays untouched. This catches partial
saves (crash mid-write), manual tampering (someone edited cache files
directly), and silent disk corruption.

---

## List — see what's saved

```text
proof cache snapshot list
```

Shows all snapshots, newest first, with file count and total bytes.

```text
name                          files      bytes
production                       12      84231
canary                            8      52104
before-redesign                   5      31002
```

---

## Diff — compare two snapshots

```text
proof cache snapshot diff before-redesign production
```

Per-file comparison of the three tier keys. Each file falls into one of
four buckets:

```text
Only in before-redesign: (none)
Only in production: languages/12-OCAML.source.md
Changed: overview.source.md, languages/10-GO.source.md
Identical: 9 files
```

A file is **changed** if any of its three tier keys differ between the two
snapshots — a different parse key means the source changed, a different
resolve key means a referenced figure changed, a different compile key
means the directive output would differ.

---

## Prune — remove old snapshots

```text
proof cache snapshot prune --keep 3
```

Keeps the N most recent snapshots (ordered by save time), removes everything
older. Returns the list of deleted names.

```text
Removed: backup-20260401, test-20260330
```

---

## Deploy — materialize compiled output

```text
proof cache snapshot deploy production --to ./dist/
```

Verifies the snapshot, then writes each cached compile entry's
`compiled_text` to `./dist/{relative_output_path}`. No recompilation
happens — the deployed files are byte-identical to what was in the cache
when the snapshot was saved.

This enables CI flows like: build once, snapshot, then deploy the same
artifacts to staging and production without re-running compile.

---

## Typical workflow

A risky redesign of shared figures:

```text
# 1. Baseline before any edits
proof cache snapshot save before-redesign

# 2. Make changes to figures, source documents
$EDITOR src/figures/team-org.md

# 3. Recompile to produce new artifacts
proof compile .

# 4. See what changed
proof cache snapshot save after-redesign
proof cache snapshot diff before-redesign after-redesign

# 5a. Looks good — keep "after" as the new baseline
proof cache snapshot save production
proof cache snapshot prune --keep 3

# 5b. Looks broken — instant rollback
proof cache snapshot restore before-redesign
# Next `proof compile .` returns to the pre-edit state.
```

---

## Where snapshots live

```text
.proof/cache/snapshots/
  production/
    manifest.json      ← name, created_at, files, tier keys, integrity_hash
    parse/             ← copied parse-tier entries
      <key>.json
    resolve/           ← copied resolve-tier entries
      <key>.json
    compile/           ← copied compile-tier entries
      <key>.json
  canary/
    ...
```

Snapshot directories are independent — wiping `.proof/cache/` doesn't touch
`.proof/cache/snapshots/`. To clean snapshots specifically, use `prune` or
delete a single named directory.

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| COMPILE-004 | error | Snapshot integrity hash mismatch — restore rejected |
| COMPILE-005 | error | (Server mode only) Restore rejected: compile session active |
| COMPILE-006 | warning | Snapshot missing some files present in current state |
