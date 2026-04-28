# proof compile + layout — Spec Validation Scenarios

Hand-simulations of the compile and layout specs against concrete inputs.
Each scenario traces through the spec step-by-step; **Findings** are spec gaps
or ambiguities discovered during the trace. Target: 5-15 findings per scenario.

Related specs: [COMPILE-SPEC.md](./compile-spec.md) · [LAYOUT-SPEC.md](./layout-spec.md) · [THREE-TIER-CACHE.md](./three-tier-cache.md)

---

## Scenario 01 — Basic single include

**Tests:** Simplest compile path. One source document, one `proof:include`, one figure file.

### Input

`figures/goroutine-scheduler.md`:
````markdown
<!-- proof:figure id="goroutine-scheduler" kind="figure.flowchart" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌──────────────────────────────────────┐
│  G  G  G  G  ← goroutines           │
│  │  │  │  │                          │
│  └──┴──┴──┘                          │
│      M:N                             │
│  ┌──┬──┬──┐                          │
│  P  P  P  P  ← OS threads           │
└──────────────────────────────────────┘
```
<!-- /proof:figure -->
````

`languages/10-GO.source.md`:
````markdown
## Concurrency Model

Go uses M:N multiplexing — goroutines run on OS threads managed by the runtime.

```proof:include
md://figures/goroutine-scheduler.md#goroutine-scheduler:0
```

The scheduler is cooperative: goroutines yield at blocking calls.
````

### Expected output

`languages/10-GO.md`:
````markdown
## Concurrency Model

Go uses M:N multiplexing — goroutines run on OS threads managed by the runtime.

<!-- proof:compiled from="md://figures/goroutine-scheduler.md#goroutine-scheduler:0" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌──────────────────────────────────────┐
│  G  G  G  G  ← goroutines           │
│  │  │  │  │                          │
│  └──┴──┴──┘                          │
│      M:N                             │
│  ┌──┬──┬──┐                          │
│  P  P  P  P  ← OS threads           │
└──────────────────────────────────────┘
```
<!-- /proof:compiled -->

The scheduler is cooperative: goroutines yield at blocking calls.
````

### Trace

**Step 1 — Parse `languages/10-GO.source.md`**
- Hash file content → compute parse_key → check Tier 1 cache (miss, first run)
- Result: `ParsedDocument` with one `proof:include` directive at line 5

**Step 2 — Find directives**
- One `proof:include` block: URI = `md://figures/goroutine-scheduler.md#goroutine-scheduler:0`

**Step 3 — Compute resolve_keys**
- Target file: `figures/goroutine-scheduler.md`
- Hash target file content → parse_key_of_target → `resolve_key = SHA-256(parse_key_of_target, uri_string, proof_version)`
- **Finding F01**: The spec says "hash target file content → parse_key". But the parse_key of the TARGET file is `SHA-256(file_content_hash, proof_version)`. So step 3 requires hashing the target file. This is documented correctly but the spec doesn't say WHERE the target file is searched from. Is `figures/goroutine-scheduler.md` relative to the source file's directory? The proof.toml root? The current working directory? **The md:// root resolution base is unspecified in the compile spec.** (COMPILE-SPEC.md says "Resolve via mdpath" but doesn't specify the base path for resolution.)

**Step 4 — Compute compile_key and check Tier 3**
- `compile_key = SHA-256(source_parse_key, [resolve_key], layout_config_hash=none, proof_version)`
- **Finding F02**: `layout_config_hash` is undefined when there are no layout directives. The spec says this is an input to compile_key but doesn't specify the value when no layout is present. Zero? Empty hash? Omit from hash input? Any inconsistency here creates key collisions between "no layout" and "layout with some specific config."

**Step 5 — Fetch resolved content**
- Resolve `md://figures/goroutine-scheduler.md#goroutine-scheduler:0` via mdpath
- **Finding F03**: The `#goroutine-scheduler:0` selector uses `id="goroutine-scheduler"` — the figure marker. But the `proof:figure` marker is an HTML comment OUTSIDE the code block (before it). The COMPILE-SPEC shows the marker before the ` ``` ` opening. The mdpath resolver (MDPATH spec) resolves by heading path and ordinal. How does it handle a `proof:figure id=` marker? Is this a SEPARATE selector mechanism from the standard `#heading:kind:ordinal` mdpath syntax? The compile spec uses a named figure ID in the URI fragment but the mdpath spec doesn't define how figure IDs are indexed.

**Step 6 — No layout directives, skip**

**Step 7 — Compose output**
- Replace `proof:include` block with resolved content wrapped in `<!-- proof:compiled ... -->` comments
- **Finding F04**: The spec shows the compiled figure wrapped in a code fence (` ``` `), but the figure's content is ALREADY a code fence in `goroutine-scheduler.md`. So the output would be a code fence inside HTML comments. The spec doesn't say whether the outer code fence is preserved, stripped, or re-wrapped. Does `proof:include` embed the raw code block (fence and all) or just the content inside the fence?

**Step 8 — Write output atomically to `languages/10-GO.md`**
- **Finding F05**: The output path drops `.source.` from the filename. But `proof compile languages/10-GO.source.md` — is the output relative to the source file's directory? Or the proof.toml root? If source is `src/languages/10-GO.source.md`, is output `src/languages/10-GO.md` or `languages/10-GO.md`?

**Step 9 — Update Tier 3 cache**
- Write compile cache entry to `.proof/cache/compile/{compile_key}.json`
- **Finding F06**: The compile cache stores "compiled markdown" — but what is the JSON schema? The THREE-TIER-CACHE spec shows `.json` files but doesn't specify the structure. Parse cache stores `ParsedDocument`, resolve cache stores `ResolvedElement`. What does compile cache store? Just the compiled text? Or metadata too (which URIs were resolved, which DaVinci checks passed)?

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F01 | High | md:// URI base path for resolution is unspecified — relative to source file, proof.toml, or cwd? |
| F02 | Medium | `layout_config_hash` when no layout directives present — undefined value in compile_key |
| F03 | High | Named figure IDs (`#goroutine-scheduler:0`) — spec doesn't define how `proof:figure id=` markers integrate with the mdpath URI scheme |
| F04 | High | Does `proof:include` embed the code fence or just the code fence content? Outer fences in figure files create fence-in-fence ambiguity |
| F05 | Medium | Output path resolution when compiling from a subdirectory — relative to source dir, project root, or cwd? |
| F06 | Low | Compile cache entry JSON schema is unspecified — what fields besides the compiled text? |

---

## Scenario 02 — Layout: two figures side-by-side

**Tests:** `proof:layout` directive in compile mode, two-figure horizontal composition.

### Input

`figures/go-types.md`:
````markdown
<!-- proof:figure id="go-type-system" kind="table.key-value" -->
```
Axis         | Value
-------------|----------
Binding      | Late
Typing       | Static
Strength     | Strong
Type system  | Structural
```
<!-- /proof:figure -->
````

`figures/rust-types.md`:
````markdown
<!-- proof:figure id="rust-type-system" kind="table.key-value" -->
```
Axis         | Value
-------------|----------
Binding      | Compile
Typing       | Static
Strength     | Strong
Type system  | Affine
```
<!-- /proof:figure -->
````

`comparison.source.md`:
````markdown
## Type System Comparison

```proof:layout gap=4 align=top labels="Go,Rust"
md://figures/go-types.md#go-type-system:0
md://figures/rust-types.md#rust-type-system:0
```
````

### Expected output

`comparison.md`:
````markdown
## Type System Comparison

<!-- proof:compiled from="proof:layout" -->
```
      Go                         Rust
Axis         | Value        Axis         | Value
-------------|----------    -------------|----------
Binding      | Late         Binding      | Compile
Typing       | Static       Typing       | Static
Strength     | Strong       Strength     | Strong
Type system  | Structural   Type system  | Affine
```
<!-- /proof:compiled -->
````

### Trace

**Step 3 — Compute resolve_keys**
- Two URIs → two resolve_keys

**Step 4 — Compile key includes both resolve_keys + layout_config_hash**
- `layout_config_hash = SHA-256(gap=4, align=top, labels="Go,Rust")`
- **Finding F07**: The `labels` attribute is part of the layout config hash. If you change a label ("Go" → "Golang"), this invalidates the compile cache but NOT the resolve cache. That's correct — the label is pure presentation, no re-resolution needed. But the spec doesn't confirm that labels are part of `layout_config_hash`. It should.

**Step 5 — Fetch both resolved figures**
- Resolve each URI → get the table content from each figure file

**Step 6 — Apply layout engine**
- Step 1 (Fetch): content lines of each figure
- Step 2 (Normalize frames): go-types frame is 28 wide, rust-types frame is 28 wide. Pad all lines to 28.
- **Finding F08**: Both figures are code fences. Does the layout engine compose the raw code fence (including ` ``` `) or just the content inside? If it composes the raw fence, each figure's frame starts with ` ``` ` and ends with ` ``` ` which is not renderable side-by-side. The spec doesn't address this. The layout engine almost certainly operates on the content INSIDE the fence, then wraps the whole composition in a new fence. But this is not stated.
- Step 3 (Equalize heights): go-types = 6 lines, rust-types = 6 lines. Equal, no padding needed.
- Step 4 (Labels): "Go" centered over 28 chars = 13 spaces + "Go" + 13 spaces. "Rust" centered over 28 chars = 12 spaces + "Rust" + 12 spaces.
- **Finding F09**: Label centering spec: `centered over the frame width`. For odd-length strings in even-width frames (or vice versa), is the extra space on the left or right? The spec says "centered" but doesn't specify tie-breaking. Different implementations could produce different whitespace, which would be a cache key issue if labels are compared.
- Step 5 (Compose rows): join label lines with gap=4 → join content lines with gap=4
- **Finding F10**: The gap is specified in the `proof:layout` directive. But the compile cache key uses `layout_config_hash`. If the directive is `gap=4` and the CLI uses `--gap 4`, are these identical? Yes. But what about default values? If `gap` is omitted from the directive (using the default of 3), the hash input should be `gap=3` (normalized), not "omitted". The spec doesn't say whether defaults are normalized before hashing or whether the raw attribute string is hashed. If the raw string is hashed, `gap=3` (explicit) and `gap` (omitted default) would produce different cache keys for identical output — a cache correctness bug.

**Step 7 — Compose final document**
- **Finding F11**: When a `proof:layout` is compiled, the traceability comment says `from="proof:layout"`. But this loses the specific URIs that were composed. If `proof check` runs on the compiled output, it can't verify which figures were embedded. The traceability comment should include the resolved URIs, e.g. `from="proof:layout md://figures/go-types.md#go-type-system:0 md://figures/rust-types.md#rust-type-system:0"`.

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F07 | Low | Spec should confirm labels are part of layout_config_hash |
| F08 | High | Layout engine input ambiguity: does it compose raw code fence content or the fence lines including delimiters? |
| F09 | Low | Label centering tie-breaking (odd label width in even frame) not specified |
| F10 | Medium | Default attribute values must be normalized before hashing — raw attribute string vs. resolved value |
| F11 | Medium | Traceability comment for `proof:layout` loses the composed URIs — compiled output can't be re-validated against sources |

---

## Scenario 03 — DaVinci violation blocks compile

**Tests:** An included figure violates a pinned invariant. Compile must abort without writing output.

### Input

`proof.toml`:
```toml
[[davinci]]
id = "goroutine-scheduler"
file = "figures/goroutine-scheduler.md"
protection = "error"

  [[davinci.invariant]]
  rule = "contains-text"
  value = "M:N multiplexing"

  [[davinci.invariant]]
  rule = "box-count"
  min = 2
```

`figures/goroutine-scheduler.md` (MODIFIED — someone removed the inner box):
````markdown
<!-- proof:figure id="goroutine-scheduler" kind="figure.flowchart" -->
```
GOROUTINE SCHEDULER — M:N multiplexing
┌──────────────────────────────────────┐
│  goroutines → OS threads             │
└──────────────────────────────────────┘
```
<!-- /proof:figure -->
````

`languages/10-GO.source.md`:
````markdown
## Concurrency Model

```proof:include
md://figures/goroutine-scheduler.md#goroutine-scheduler:0
```
````

### Expected behavior

- Compile aborts with `COMPILE-001`
- `languages/10-GO.md` is NOT written (or not modified if it already exists)
- Error output identifies the figure, violated invariant, and URI

### Trace

**Step 5 — Fetch resolved content and validate DaVinci**
- Resolve figure → check `box-count min=2`: only 1 box present → invariant violated
- `protection = "error"` → emit `COMPILE-001` and abort

**Finding F12**: The spec says "output file is NOT written." But what if `languages/10-GO.md` ALREADY EXISTS from a previous successful compile? The spec doesn't say whether the existing file is preserved, deleted, or left stale. If it's left stale, the compiled output is now out-of-date with the source. If it's deleted, the author loses the last good version. The correct behavior (preserve the last good compile, emit an error) is not stated.

**Finding F13**: The spec says the error "shows which figure, which invariant, which URI." But does it show the figure's current content so the author can see what changed? Does it show the invariant as written in proof.toml (`box-count min=2`) or in a human-readable form ("expected at least 2 boxes, found 1")? Error message format is unspecified.

**Finding F14**: What if multiple figures in a source document violate invariants? Does compile report ALL violations before aborting, or abort on the first? The spec says "compile fails" but not whether it's fail-fast or fail-all.

**Finding F15**: The `protection = "error"` level is for pinned figures. What does `protection = "warn"` do during compile? Does the compile continue and write output? The spec defines protection tiers but doesn't explicitly say how `warn` interacts with compile — does compile succeed with warnings?

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F12 | High | Behavior when existing compiled output is present and compile fails — preserve, delete, or leave stale? |
| F13 | Medium | Error message format for COMPILE-001 — current figure content? Human-readable invariant description? |
| F14 | Medium | Fail-fast vs fail-all on DaVinci violations — spec is silent |
| F15 | High | `protection = "warn"` during compile — does compile succeed and write output? Spec doesn't say |

---

## Scenario 04 — Cache hit on second compile

**Tests:** Second compile run with no file changes should be a full Tier 3 hit — no resolution, no composition.

### Input

Same as Scenario 01, after one successful compile.

### Trace

**Step 1 — Parse source file**
- File unchanged → `file_content_hash` unchanged → `parse_key` unchanged → Tier 1 HIT

**Step 2 — Find directives**
- (Still must scan even on Tier 1 hit, because the cache stores `ParsedDocument`, not directive list)
- **Finding F16**: On a Tier 1 parse cache hit, does the compiler use the cached `ParsedDocument` to extract directives (avoiding re-parsing)? Or does it re-parse to find directives? If re-parsing, Tier 1 cache is nearly useless for the directive extraction step. The spec should say: Tier 1 hit → use cached `ParsedDocument` → extract directives from cached doc.

**Step 3 — Compute resolve_keys**
- Target file unchanged → same `parse_key_of_target` → same `resolve_key`
- **Finding F17**: To compute `resolve_key`, the spec says "hash target file content → parse_key_of_target." On the second run, the target file is in the Tier 1 parse cache. Can `parse_key_of_target` be read from the parse cache (keyed by file path → cached parse_key) instead of re-hashing the target file? The spec doesn't say parse cache entries are indexed by file path — they're content-addressed by key. How does the compiler efficiently find "what's the parse_key for this file path?"

**Step 4 — Compile key**
- All inputs identical → compile_key identical → Tier 3 HIT

**Finding F18**: On Tier 3 hit, the spec says "write cached output, done." But writing the output file (even from cache) updates its mtime. Watch mode detects mtime changes. Could watch mode trigger a spurious recompile of a file that consumed the compiled output? The spec doesn't address interaction between Tier 3 cache writes and watch mode file watching.

**Finding F19**: On Tier 3 hit, the compile cache entry must be read and the output file written. The spec doesn't say what happens if the output file already has identical content — does it still write (updating mtime) or skip (no-op)? For watch mode and downstream tools, mtime stability matters.

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F16 | Medium | On Tier 1 parse cache hit, directives must be extracted from cached ParsedDocument — spec should make this explicit |
| F17 | High | Parse cache is content-addressed (by key), but step 3 needs "parse_key for this file path" — no reverse index specified |
| F18 | Medium | Tier 3 cache write updates output file mtime — watch mode may trigger spurious recompile |
| F19 | Low | Cache hit write: skip if output already identical, or always write? |

---

## Scenario 05 — Multiple includes of the same figure

**Tests:** One figure included twice in the same source document.

### Input

`comparison.source.md`:
````markdown
## Go Scheduler

```proof:include
md://figures/goroutine-scheduler.md#goroutine-scheduler:0
```

See also:

```proof:include
md://figures/goroutine-scheduler.md#goroutine-scheduler:0
```
````

### Trace

**Step 3 — Compute resolve_keys**
- Same URI appears twice. Two resolve_keys are computed — both identical.
- **Finding F20**: The compile_key formula uses `sorted_resolve_keys[]`. If the same URI appears twice, are there two identical resolve_key entries in the sorted list? Or is the list deduplicated? If deduplicated, a document with one include and a document with two identical includes would have the SAME compile_key — a cache correctness bug, since their outputs differ. If not deduplicated, order matters — but the list is sorted. `SHA-256([k, k])` ≠ `SHA-256([k])`, so deduplication would cause a bug. The spec must say: include duplicates (do NOT deduplicate).

**Step 5 — Fetch resolved content**
- Same figure resolved twice.
- **Finding F21**: Is the figure resolved once (Tier 2 cache hit on second occurrence) or twice? The spec implies Tier 2 is per-URI, so the second occurrence is a cache hit. But the resolved content is fetched/used twice in the output. The spec should confirm: N directives for the same URI → N Tier 2 cache hits (1 actual resolution), N embedded copies in output.

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F20 | Critical | `sorted_resolve_keys[]` must NOT deduplicate — deduplication causes cache key collision between documents with different include counts |
| F21 | Low | Same URI appearing N times → 1 resolve + N cache hits (worth confirming, not just implied) |

---

## Scenario 06 — Layout wrapping when `--cols` < N figures

**Tests:** 4 figures with `--cols 2` — should produce 2 rows of 2 figures each.

### Input (command line)

```bash
proof layout \
    "md://fig/a.md#:0" \
    "md://fig/b.md#:0" \
    "md://fig/c.md#:0" \
    "md://fig/d.md#:0" \
    --cols 2 --gap 4
```

Figures A and B are 20 lines tall. C is 12 lines tall. D is 20 lines tall.

### Trace

**Step 3 — Equalize heights per row**
- Row 1: A (20 lines) + B (20 lines) → `max_height = 20`, no padding needed
- Row 2: C (12 lines) + D (20 lines) → `max_height = 20`, C needs 8 blank lines appended (align=top default)
- **Finding F22**: The spec says rows are "separated by a blank line." But the algorithm description doesn't specify the separator. Is it exactly 1 blank line? 2? Same as gap? The layout spec says "Collect all rows, separated by a blank line" — singular, so 1. But 1 blank line between two composed rows looks cramped in a 200-column wide presentation. Should the row separator be configurable? Spec doesn't define it.

**Step 5 — Compose rows**
- Row 1: `A_line[0] + " " * 4 + B_line[0]` for each of 20 lines
- Row 2: `C_line[0] + " " * 4 + D_line[0]` for each of 20 lines (C is blank-padded for lines 12-19)

**Finding F23**: When C is blank-padded on the right with `align=top`, lines 12-19 of C are all spaces padded to C's frame_width. Combined with the gap (4 spaces), the right side of the row separator has `frame_width_C + 4 + frame_width_D` characters. If the blank pad for C is spaces-only, proof's ASCII checker would likely flag these as trailing-space errors in the compiled output. The spec doesn't address trailing space in blank padding.

**Finding F24**: The spec says each frame's lines are padded to `frame_width` (max line width of that figure). Then blank lines for height equalization are also padded to `frame_width`. But a "blank line" padded to `frame_width` is all spaces — this is N trailing spaces in the output. Should blank padding lines be empty (no spaces) or padded? If empty, the gap alignment in subsequent rows would be wrong (lines from different frames would be different lengths causing visual misalignment in some editors).

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F22 | Low | Row separator width (between `--cols` wrapping) not specified — "a blank line" is ambiguous |
| F23 | Medium | Blank-padded lines produce trailing spaces in compiled output — proof would flag its own output |
| F24 | Medium | Blank height-equalization lines: all-spaces (for alignment) vs empty (avoids trailing spaces) — spec is silent, the choice has visual correctness implications |

---

## Scenario 07 — Watch mode: figure edit triggers recompile

**Tests:** Watch mode only recompiles documents that include the changed figure.

### Setup

Two source documents:
- `go.source.md` — includes `goroutine-scheduler.md`
- `rust.source.md` — includes `rust-ownership.md`

Both compiled successfully. Now `goroutine-scheduler.md` is edited.

### Trace

**Watch event fired for `goroutine-scheduler.md`**

- **Finding F25**: The spec says watch mode "Watches all source `.source.md` files." But a figure file (`goroutine-scheduler.md`) is NOT a `.source.md` file — it's a plain `.md` file. Does watch mode watch figure files too? If not, editing a figure file doesn't trigger any recompile, which defeats the purpose. The spec must specify that watch mode watches both source documents AND all referenced figure files (all URIs that appear in any source directive).

- **Finding F26**: When a figure file changes, watch mode must know which source documents include it to trigger targeted recompile. This requires an **inverse index**: figure file → list of source documents that include it. The spec doesn't describe this index — where it's stored (memory? disk?), how it's built (on startup? incrementally?), and what happens when a new source document is compiled (does it update the index?). Without this index, watch mode must re-scan all source documents on any file change, which is O(N).

**Finding F27**: The watch mode output shows "→ invalidated resolve cache for 2 URIs." This implies the watch mode ACTIVELY invalidates cache entries when a file changes — rather than relying on content-addressed keys to naturally miss. But content-addressed caches don't need invalidation: the new file content produces a new hash, which produces a new cache key, which naturally misses. Actively deleting old entries would be WRONG (it removes valid cached versions). The spec output is misleading — watch mode should just recompile (which will naturally miss the cache), not invalidate.

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F25 | Critical | Watch mode spec says "watches .source.md files" but figure files (.md) must also be watched — a figure edit won't trigger recompile |
| F26 | High | Inverse index (figure → source documents) required for targeted recompile — not specified in watch spec |
| F27 | Medium | Watch mode output says "invalidated resolve cache" — misleading; content-addressed caches don't need explicit invalidation |

---

## Scenario 08 — Snapshot save, edit, diff, restore

**Tests:** Snapshot workflow — save before risky edit, verify diff, restore on failure.

### Trace

**`proof cache snapshot save "before-edit"`**

- **Finding F28**: The snapshot save spec says "Read current cache state for all source documents." But "all source documents" is undefined — does it mean all `.source.md` files under the proof.toml root? What if some have never been compiled and have no cache entries? The snapshot should capture only files that HAVE cache entries, but the spec doesn't say so explicitly.

**Edit `goroutine-scheduler.md`, recompile `go.source.md`**

**`proof cache snapshot diff "before-edit" "current"`**

- **Finding F29**: The diff command compares two named snapshots. But "current" is not a named snapshot — it's the current cache state. The CLI in CACHE-SNAPSHOTS.md shows `diff <name-a> <name-b>` — both must be named snapshots. To diff against current state, the author would need to `save "current"` first. But then they have a snapshot called "current" that may be confusing. Should there be a special `current` keyword? Or `proof cache snapshot diff "before-edit" --vs-current`?

**`proof cache snapshot restore "before-edit"`**

- Verify integrity hash → copy snapshot entries to active cache
- **Finding F30**: After restore, the active cache is in the "before-edit" state. But the WORKING FILES (`goroutine-scheduler.md`, `go.source.md`) are still in their edited state. The next `proof compile .` will hit the restored cache keys for UNEDITED files, but MISS for `go.source.md` (whose parse_key has changed since the edit). So restore doesn't actually prevent recompile of edited files — it only restores cache entries for UNCHANGED files. The spec doesn't clarify that restore is a cache operation, not a file system rollback. Authors may expect it to work like `git checkout`.

- **Finding F31**: The CACHE-SNAPSHOTS spec says restore is rejected with `COMPILE-005` if "compilation is in progress." But proof compile is a one-shot command (not a daemon). How does the tool know if "compilation is in progress"? This guard makes sense for a long-lived session/server mode, but for CLI invocation, it's unclear when this guard would ever fire.

### Findings

| # | Severity | Finding |
|---|----------|---------|
| F28 | Medium | "All source documents" in snapshot save — undefined scope; should be "all files with cache entries" |
| F29 | High | `diff` requires two named snapshots — no way to diff against current state without creating a "current" snapshot; spec should address `--vs-current` or equivalent |
| F30 | High | Restore is a cache operation, not file system rollback — compiled output may still be stale for edited files after restore; spec should clarify |
| F31 | Medium | `COMPILE-005` "compilation in progress" guard — undefined for CLI (one-shot) mode; only meaningful in server/session mode |

---

## Finding Summary

### By Severity

| Severity | Count | Findings |
|----------|-------|---------|
| Critical | 2 | F20, F25 |
| High | 8 | F01, F03, F04, F08, F12, F15, F17, F29, F30 |
| Medium | 11 | F02, F05, F10, F13, F14, F16, F23, F24, F26, F28, F31 |
| Low | 10 | F06, F07, F09, F11, F18, F19, F21, F22, F27, F32 |

### By Area

| Area | Findings |
|------|---------|
| URI resolution / md:// base path | F01, F03 |
| Cache key correctness | F02, F07, F10, F17, F20 |
| Figure embed format (fence vs. content) | F04, F08 |
| Output path resolution | F05 |
| DaVinci + compile interaction | F12, F13, F14, F15 |
| Parse cache efficiency | F16, F17 |
| Watch mode | F18, F19, F25, F26, F27 |
| Layout algorithm | F09, F22, F23, F24 |
| Traceability | F11 |
| Snapshot system | F28, F29, F30, F31 |

### Critical + High — must resolve before implementation

- **F20**: `sorted_resolve_keys[]` must NOT deduplicate — deduplication is a silent cache correctness bug
- **F25**: Watch mode must watch figure files, not just `.source.md` files
- **F01**: md:// base path for resolution — relative to source file, proof.toml root, or cwd?
- **F03**: Named figure ID selector (`#id:0`) integration with mdpath URI scheme — undefined
- **F04**: Does `proof:include` embed raw code fence or just the content inside?
- **F08**: Layout engine input — operates on fence content or raw fence including delimiters?
- **F12**: Behavior when existing compiled output is present and compile fails
- **F15**: `protection = "warn"` during compile — does compile succeed and write output?
- **F17**: Parse cache reverse index — how to look up "parse_key for this file path"?
- **F29**: `diff` requires two named snapshots — no vs-current mode
- **F30**: Restore is cache-only — spec must clarify it doesn't roll back working files

---

## Scenario 09 — Inline `$...$` math expansion

**Tests:** Inline math tokenization and symbol lookup for a simple expression in prose.

### Input

`computing/01-CALC.source.md`:
```markdown
The fundamental identity is $\alpha + \beta = \gamma$ for all values.
```

### Expected output

```markdown
The fundamental identity is α + β = γ for all values.
```

### Trace

**Step 1 — Tokenize line**
- Scanner sees `$` at column 28, enters inline-math mode
- Reads until closing `$` → raw token: `\alpha + \beta = \gamma`

**Step 2 — Symbol lookup**
- `expand_inline_math` splits token stream
- `\alpha` → `SymbolTable["alpha"]` → `α` (U+03B1); `\beta` → `β`; `\gamma` → `γ`
- `+` and `=` are operator chars → passed through as literals

**Step 3 — Reconstruct and splice**
- Expanded string: `α + β = γ`
- Replace `$...$` span in original line with expanded string

### Findings

- **F32**: Spaces around commands are ambiguous — `\alpha+\beta` (no space) vs `\alpha + \beta` (space). Tokenizer must decide whether spaces are significant or collapsed. Spec does not state the whitespace handling rule inside `$...$`.
- **F33**: Operator characters (`+`, `=`, `-`) appear as literal pass-through. Spec should enumerate which characters are always literals vs which might be command prefixes in a future extension.

---

## Scenario 10 — Inline math in slide title

**Tests:** `$...$` in YAML front-matter title field passes through slide parser and expands correctly.

### Input

`slides/energy.source.md`:
```markdown
---
title: "$E = mc^2$"
layout: title
---
```

### Expected output

Slide title rendered as: `E = mc²`

### Trace

**Step 1 — Parse front-matter**
- YAML parser reads `title` value as string: `"$E = mc^2$"`
- Stored verbatim in `SlideAttrs.title`

**Step 2 — render_title calls render_body_lines**
- `render_body_lines([title_string])` calls `expand_inline` on each line

**Step 3 — expand_inline → expand_inline_math**
- `$E = mc^2$` matched; raw expr: `E = mc^2`
- `E`, `m`, `c` → bare letter literals; `^2` → superscript lookup → `²` (U+00B2)
- Result: `E = mc²`

**Step 4 — Center in width**
- `center_in_width("E = mc²", slide_width=80)` → pad with spaces; SL-1 invariant satisfied

### Findings

- **F34**: `^2` in inline context produces `²` but `^{10}` requires multi-char superscript. Spec must define which superscript values have Unicode equivalents and what the fallback is for `^{10}` (render as `^10`? emit MATH-005?).
- **F35**: YAML double-quoting strips the outer `"`. If title is `'$E = mc^2$'` (single-quoted YAML), the `$` delimiters are preserved. Confirm YAML parsing does not alter `$` characters inside string values.

---

## Scenario 11 — `proof:math` display block — fraction

**Tests:** Display-math block renders `\frac{n(n+1)}{2}` as a 3-line stacked output.

### Input

````markdown
```proof:math
\frac{n(n+1)}{2}
```
````

### Expected output

```
 n(n+1)
────────
   2
```

### Trace

**Step 1 — Tokenize**
- `tokenize_math_expr` sees `\frac` → consume two brace groups: num=`n(n+1)`, den=`2`
- Returns `[Frac { num: "n(n+1)", den: "2" }]`

**Step 2 — render_frac**
- `num_width = visual_width("n(n+1)") = 6`; `den_width = 1`; `bar_width = max(6,1) = 6`
- Line 1: `n(n+1)` (no center padding needed at width 6)
- Line 2: `──────` (bar_width × U+2500)
- Line 3: center `2` in 6 → `  2   `

**Step 3 — width=auto**
- No explicit `width` attr → block width = bar_width = 6; emit 3-line block

### Findings

- **F36**: Centering denominator `2` in width 6 → (6-1)/2 = 2.5 → floor=2 left, 3 right. Spec must state rounding rule: remainder goes left or right.
- **F37**: `n(n+1)` contains `(` and `)` — these are literal chars in numerator context, not grouping operators. Spec should clarify that brace-group parsing stops at `}` only, not at `)`.

---

## Scenario 12 — `proof:math` integral with limits

**Tests:** `\int_0^{\infty} e^{-x} dx` produces a 4-line integral display.

### Input

````markdown
```proof:math
\int_0^{\infty} e^{-x} dx
```
````

### Expected output

```
∞
⌠
⌡ e^{-x} dx
0
```

### Trace

**Step 1 — consume_limit_args**
- `\int` recognized as large-operator; `_0` → lower=`0`; `^{\infty}` → upper=`∞`

**Step 2 — render_int, 4-line layout**
- Line 1: upper limit `∞`
- Line 2: `⌠` (U+2320)
- Line 3: `⌡ e^{-x} dx` — body with `^{-x}` kept as-is (no Unicode superscript for `-x`)
- Line 4: lower limit `0`

**Step 3 — emit block**
- 4 lines output; block width = max visual_width across all lines

### Findings

- **F38**: `e^{-x}` in the body — `-x` has no single Unicode superscript. Spec must state the fallback for multi-char superscripts in body text (keep as `^{-x}` literal, or use combining chars).
- **F39**: Lower limit `0` aligns at column 0 (same as `⌠`). Spec should specify the horizontal offset rule — is the lower limit always at column 0, or indented to align under the `⌡` glyph body?

---

## Scenario 13 — `proof:math` pmatrix environment

**Tests:** `\begin{pmatrix} a & b \\ c & d \end{pmatrix}` renders as a parenthesized matrix.

### Input

````markdown
```proof:math
\begin{pmatrix} a & b \\ c & d \end{pmatrix}
```
````

### Expected output

```
⎛ a  b ⎞
⎜      ⎟
⎝ c  d ⎠
```

### Trace

**Step 1 — parse_matrix_body**
- `\begin{pmatrix}` → matrix mode, delimiter=paren
- Split on `\\` → rows: [`a & b`, `c & d`]; split on `&` → cells per row

**Step 2 — compute column widths**
- col0: max(1,1)=1; col1: max(1,1)=1; inner width = 1+2+1 = 4

**Step 3 — render_matrix with delimiter glyphs**
- 2-row matrix: top `⎛`, spacer `⎜`, bottom `⎝` on left; `⎞`, `⎟`, `⎠` on right
- Row 0: `⎛ a  b ⎞`; spacer: `⎜      ⎟`; row 1: `⎝ c  d ⎠`

### Findings

- **F40**: For a 2-row matrix the spacer row may be omitted in some renderers. Spec must define whether spacer rows are emitted between every row pair or only for matrices with 3+ rows.
- **F41**: `&` is the cell separator inside `\begin{pmatrix}`. The parser must recognize raw `&` as column separator only inside matrix environments, not as an HTML entity start.

---

## Scenario 14 — MATH-005 downgrade: inline fraction

**Tests:** `$\frac{a}{b}$` in inline context cannot render as display block — downgrades to `a/b` and emits MATH-005.

### Input

```markdown
The ratio is $\frac{a}{b}$ of the total.
```

### Expected output

```markdown
The ratio is a/b of the total.
```

Plus diagnostic: `MATH-005 [WARN]: inline \frac downgraded to a/b (line 1, col 14)`

### Trace

**Step 1 — expand_inline_math detects `\frac`**
- Context = inline (single `$...$`); `\frac` requires multi-line rendering → incompatible

**Step 2 — apply downgrade rule**
- `downgrade_frac(num="a", den="b")` → `"a/b"`; substitute into inline output

**Step 3 — emit MATH-005**
- Diagnostic logged; `written=true` (warning does not block output)

### Findings

- **F42**: Downgrade rule for `\frac` is `num/den`. For `\frac{x+y}{z}` the result is `x+y/z` — ambiguous precedence. Spec should require parentheses: `(x+y)/z`.
- **F43**: MATH-005 is a warning, not an error — `written=true`. Spec must confirm this for all MATH-005 occurrences, and that `proof check` reports them as warnings not failures.

---

## Scenario 15 — proof-math standalone public API

**Tests:** External caller uses `proof_math::expand_inline_math("$\\pi r^2$")` directly without invoking the compiler.

### Input

```rust
let result = proof_math::expand_inline_math("$\\pi r^2$");
```

### Expected output

```
Ok("πr²")
```

### Trace

**Step 1 — public API entry**
- `expand_inline_math` takes `&str`, returns `Result<String, MathError>`
- Input: one `$...$` span

**Step 2 — tokenize**
- Raw expr: `\pi r^2`; `\pi` → `π` (U+03C0); `r` → literal; `^2` → `²` (U+00B2)

**Step 3 — reconstruct**
- Tokens: `π`, `r`, `²` joined → `"πr²"`; return `Ok("πr²")`

### Findings

- **F44**: Does the public API accept `"$\\pi r^2$"` (with delimiters) or `"\\pi r^2"` (without)? Spec must define whether the caller passes the full delimited string or just the inner expression.
- **F45**: `r` is a bare letter with no backslash — passed through as literal `r`. Spec should confirm bare ASCII letters are always literals in both inline and display contexts.

---

## Scenario 16 — Math + symbol coexistence on one line

**Tests:** `[sym:checkmark] $\alpha$` on one line → `✓ α`. Symbol expansion runs before math expansion.

### Input

```markdown
Status: [sym:checkmark] $\alpha$ confirmed.
```

### Expected output

```markdown
Status: ✓ α confirmed.
```

### Trace

**Step 1 — expand_symbols pass**
- `[sym:checkmark]` → `SymbolLibrary::lookup("checkmark")` → `✓` (U+2713)
- Intermediate: `Status: ✓ $\alpha$ confirmed.`

**Step 2 — expand_inline_math pass**
- `$\alpha$` → `\alpha` → `α`
- Result: `Status: ✓ α confirmed.`

**Step 3 — write**
- No further transformation; line written to output

### Findings

- **F46**: Order of expansion (symbols first, then math) must be fixed and documented. If math ran first, `[sym:checkmark]` could contain `$` characters that confuse the math scanner. The spec must state the canonical pass order.

---

## Scenario 17 — Slide title layout at width=80

**Tests:** `layout: title` slide renders title, subtitle, author, date centered in an 80-column frame satisfying SL-1.

### Input

```markdown
---
title: "Concurrency Patterns"
subtitle: "Go vs Rust"
author: "J. Smith"
date: "2026-04-27"
layout: title
width: 80
---
```

### Expected output

```
                        Concurrency Patterns
                             Go vs Rust
                               J. Smith
                             2026-04-27
```

### Trace

**Step 1 — render_title called with SlideAttrs**
- Fields: title, subtitle, author, date

**Step 2 — center_in_width for each field**
- `center_in_width("Concurrency Patterns", 80)` → 30 left-pad spaces
- `center_in_width("Go vs Rust", 80)` → 35 left-pad

**Step 3 — SL-1 invariant**
- Each output line is exactly 80 chars (text + right-pad spaces)
- Blank separating lines also padded to width=80

### Findings

- **F47**: SL-1 requires every line to be exactly `width` chars. Title-only slides have no body — remaining rows below the fields must be blank lines padded to 80. Spec must say how many blank-padded lines fill the remainder (to total a fixed slide height).
- **F48**: If title string is longer than `width`, centering is impossible. Spec must define truncation vs. wrapping behavior for overlong slide titles.

---

## Scenario 18 — title-content slide with 3-level bullets

**Tests:** `proof:bullets` with three indent levels renders ●/◦/▸ with correct hanging indent for wrapped lines.

### Input

````markdown
---
layout: title-content
width: 60
---
# Ownership Rules

```proof:bullets
- Memory is owned by exactly one variable
  - Owner goes out of scope → value dropped
    - Destructor called automatically
```
````

### Expected output

```
● Memory is owned by exactly one variable
  ◦ Owner goes out of scope → value dropped
    ▸ Destructor called automatically
```

### Trace

**Step 1 — parse bullets, detect level**
- `- ` at col 0 → level 1 → `●`; `  - ` at col 2 → level 2 → `◦`; `    - ` at col 4 → level 3 → `▸`

**Step 2 — BulletConfig**
- `BulletConfig { level_chars: ['●','◦','▸'], indent_per_level: 2 }`

**Step 3 — hanging indent for wrapped lines**
- If level-1 text wraps, continuation indented 2 spaces (marker `● ` = 2 chars wide)

### Findings

- **F49**: Indent-per-level is 2 spaces in the example but spec should state whether this is configurable (slide attrs? proof.toml?) or fixed.
- **F50**: Level detection is by leading-space count. Tab characters break level detection. Spec must state that tabs in bullet source are an error or normalized to spaces.

---

## Scenario 19 — Two-column layout ratio=60:40

**Tests:** `proof:two-column ratio=60:40` at width=80 allocates 48-col left and 32-col right body with height equalization.

### Input

````markdown
---
layout: title-content
width: 80
---
```proof:two-column ratio=60:40
LEFT_BODY
---
RIGHT_BODY
```
````

### Trace

**Step 1 — split_two_column**
- Separator `---` splits body into left and right text
- `left_width = floor(80 * 0.60) = 48`; `right_width = 80 - 48 = 32`

**Step 2 — render each column independently**
- Left lines wrapped at 48; right lines wrapped at 32
- Produces `N_L` and `N_R` lines respectively

**Step 3 — height equalization**
- `height = max(N_L, N_R)`; shorter column padded with blank lines (each padded to column width)

### Findings

- **F51**: `floor(80 * 0.60) = 48` — for ratio=1:1 at odd width (e.g. 79): left=39, right=40. Spec must document rounding and which column gets the remainder.
- **F52**: The column separator `---` conflicts with slide separator `---`. Spec must clarify disambiguation: `---` inside a fenced block is a column separator, not a slide boundary.

---

## Scenario 20 — Stats layout with 4 proof:stat cells

**Tests:** Four `proof:stat` cells render as a horizontally tiled row with centered values and labels.

### Input

````markdown
---
layout: stats
width: 80
---
```proof:stat
value: 4.2ms
label: P99 Latency
---
value: 99.9%
label: Uptime
---
value: 1.2M
label: RPS
---
value: 3
label: Regions
```
````

### Expected output

```
  4.2ms       99.9%        1.2M          3
P99 Latency   Uptime        RPS      Regions
```

### Trace

**Step 1 — parse_stat lines**
- Split on `---` → 4 stat blocks; each has `value:` and `label:` keys

**Step 2 — render_stat centering**
- Cell width = `floor(80 / 4) = 20`; center value in 20, center label in 20

**Step 3 — horizontal layout**
- Four cells concatenated side-by-side → line 1 all values, line 2 all labels

### Findings

- **F53**: For 3 stats, `floor(80/3) = 26` with 2 chars left over. Spec must state how remainder columns are distributed across cells.
- **F54**: If `value` text is wider than cell width, stat cell overflows. Spec must define truncation or wrapping behavior inside stat cells.

---

## Scenario 21 — proof:callout styles

**Tests:** `proof:callout style=warning` produces distinct border chars and label; style enum validated.

### Input

````markdown
```proof:callout style=warning
Check your configuration before proceeding.
```
````

### Expected output

```
┌─ WARNING ──────────────────────────────────┐
│ Check your configuration before proceeding.│
└────────────────────────────────────────────┘
```

### Trace

**Step 1 — CalloutStyle::parse**
- `style=warning` → `CalloutStyle::Warning`; label=`WARNING`; border chars `┌─┐│└┘`

**Step 2 — top border**
- `┌─ WARNING ` + `─` repeated to fill slide_width + `┐`

**Step 3 — text wrapping inside callout**
- Inner width = slide_width - 2; text wrapped; each line: `│ {text}{pad}│`

### Findings

- **F55**: Label placement in top border — how many dashes before and after label text? Spec must define label padding rule (e.g., one dash each side, or symmetric fill).
- **F56**: Three styles (`info`, `warning`, `error`) must map to distinct labels. Spec should enumerate all valid styles; unknown styles should emit a diagnostic.

---

## Scenario 22 — proof:ol numbered list with sub-items

**Tests:** `proof:ol` renders a numbered outline with decimal sub-counters.

### Input

````markdown
```proof:ol
- Top item one
  - Sub item A
  - Sub item B
- Top item two
```
````

### Expected output

```
1. Top item one
   1.1. Sub item A
   1.2. Sub item B
2. Top item two
```

### Trace

**Step 1 — render_ol, counter stack**
- `stack = [0]`; `- Top item one` → stack[0]++ = 1 → emit `1.`

**Step 2 — sub-item push**
- `  - Sub item A` → push: stack=[1,0]; stack[1]++ = 1 → emit `1.1.`
- `  - Sub item B` → stack[1]++ = 2 → emit `1.2.`

**Step 3 — pop back to level 0**
- Dedent → pop: stack=[1]; `- Top item two` → stack[0]++ = 2 → emit `2.`

### Findings

- **F57**: Counter does not reset on pop — `stack[0]` goes 1→2 correctly. Spec must state that popping to a higher level resumes the existing counter without reset.
- **F58**: Sub-item output indent: `   1.1.` has 3 leading spaces. Rule: indent = parent counter display width + 1. For `1.` (width 2) → 3 spaces. Spec should codify this formula.

---

## Scenario 23 — Multi-slide deck with 3 slides

**Tests:** Three slides separated by `---` produce a compiled output with SLIDE N headers and a final count= sentinel.

### Input

```markdown
---
title: "Slide One"
layout: title
---

---
---
title: "Slide Two"
layout: title-content
---
Content here.

---
---
title: "Slide Three"
layout: title
---
```

### Expected output

```
<!-- SLIDE 1 -->
...slide one content...
<!-- SLIDE 2 -->
...slide two content...
<!-- SLIDE 3 -->
...slide three content...
<!-- count=3 -->
```

### Trace

**Step 1 — parse_slide_doc**
- Split on bare `---` lines between front-matter blocks → 3 slide segments

**Step 2 — render each slide**
- Slide 1: `render_title`; Slide 2: `render_title_content`; Slide 3: `render_title`

**Step 3 — SLIDE N headers and final count**
- Wrap each in `<!-- SLIDE N -->` comment; append `<!-- count=3 -->` at end

### Findings

- **F59**: Front-matter uses `---` as YAML delimiter AND `---` separates slides. The parser must distinguish "this `---` ends a YAML front-matter block" from "this `---` separates two slides." Spec must state this rule explicitly.
- **F60**: `<!-- count=3 -->` sentinel — spec should define what downstream tools use this count for, and whether its absence indicates a compile error.

---

## Scenario 24 — proof:notes excluded from compiled output

**Tests:** A `proof:notes` block present in source does not appear in compiled output (SL-5 invariant).

### Input

```markdown
---
layout: title-content
---
Main content here.

```proof:notes
Speaker notes: emphasize the third point.
```
```

### Expected output

```markdown
Main content here.
```

(No notes block in output.)

### Trace

**Step 1 — parse slide body**
- Compiler identifies `proof:notes` fenced block in body

**Step 2 — SL-5 invariant**
- `proof:notes` content is excluded from compiled output; block silently dropped (no diagnostic)

**Step 3 — exact match guard**
- `assert!(!output.contains("proof:notes"))` passes

### Findings

- **F61**: SL-5 says notes are excluded. But should notes content be written to a sidecar `.notes.md` file for presenter tools? If silently dropped, notes are lost. Spec should address sidecar output.
- **F62**: `proof check` should verify SL-5 — if notes accidentally leak into compiled output, that is a bug. Spec should include a check mode rule for SL-5 verification.

---

## Scenario 25 — Two-region dashboard

**Tests:** A dashboard with a header region (y=0) and body region (y=3) composes correctly on a Canvas.

### Input

```markdown
---
kind: dashboard
width: 80
height: 24
---
regions:
  - id: header
    x: 0
    y: 0
    width: 80
    height: 3
    body: "SYSTEM STATUS"
  - id: body
    x: 0
    y: 3
    width: 80
    height: 21
    body: "All systems nominal."
```

### Trace

**Step 1 — parse front-matter**
- `kind=dashboard`, `width=80`, `height=24`; two regions parsed into `Vec<Region>`

**Step 2 — Canvas::new**
- `Canvas::new(80, 24)` → 24 rows × 80 cols initialized to spaces

**Step 3 — paste each region**
- `paste(0, 0, ["SYSTEM STATUS"])` at row 0; `paste(0, 3, ["All systems nominal."])` at row 3

**Step 4 — render**
- `canvas.render()` joins all 24 rows with `\n`; header text visible at row 0, body at row 3

### Findings

- **F63**: Region body text placed at row 3 with height=21. If body content is fewer than 21 lines, remaining rows stay as spaces. Spec must confirm this is correct (no bottom border or padding required).
- **F64**: `Canvas::new` initializes all cells to space (U+0020). Spec must confirm the default fill character to ensure ASCII-clean output.

---

## Scenario 26 — DASHBOARD-003 overlap error

**Tests:** Two dashboard regions with overlapping rectangles trigger DASHBOARD-003 and suppress output.

### Input

```markdown
regions:
  - id: left
    x: 0
    y: 0
    width: 50
    height: 10
  - id: right
    x: 40
    y: 0
    width: 40
    height: 10
```

### Expected output

Diagnostic only — no compiled file written:
```
DASHBOARD-003: regions "left" and "right" overlap at x=40..49, y=0..9
```

### Trace

**Step 1 — AABB intersection check**
- `left` rect: x=[0..49], y=[0..9]; `right` rect: x=[40..79], y=[0..9]
- x-intersection: [40..49] non-empty; y-intersection: [0..9] non-empty → overlap

**Step 2 — emit DASHBOARD-003**
- Error logged with region IDs and overlap rect; `has_errors = true`

**Step 3 — written=false**
- Output file not touched; `proof compile` exits non-zero

### Findings

- **F65**: AABB check is O(N²) over all region pairs. Spec should note acceptable complexity or recommend a spatial index for large dashboards.
- **F66**: Overlap uses exclusive upper bound (`width=50` → cols 0–49). Spec must clarify whether adjacent regions (left ends at x=49, right starts at x=50) are considered overlapping or not.

---

## Scenario 27 — proof:element in dashboard region body

**Tests:** A `proof:element kind=value` directive inside a dashboard region body renders within the region's column bounds.

### Input

Region body (region width=30):
````markdown
```proof:element kind=value
value: 99.9%
label: Uptime
width: 20
```
````

### Expected output

Within region:
```
       99.9%
       Uptime
```

### Trace

**Step 1 — render_region_body dispatches proof:element**
- `proof:element` fenced block found; dispatched to `compile_element`

**Step 2 — compile_element for kind=value**
- `ElementAttrs { kind: Value, value: "99.9%", label: "Uptime", width: 20 }`
- Center `99.9%` in 20; center `Uptime` in 20

**Step 3 — clip to region width**
- Region width=30 > element width=20; element fits; pasted at region x offset

### Findings

- **F67**: Element `width=20` is narrower than region `width=30`. Spec should state horizontal placement within the region: left-aligned? centered? Controlled by an `align` attribute?
- **F68**: If element `width` exceeds region width, clip to region boundary. Spec must define the clip behavior (hard truncate at region edge).

---

## Scenario 28 — Dashboard with proof:tree region

**Tests:** A `proof:tree kind=org` directive inside a dashboard region renders and is pasted at the correct canvas position.

### Input

Region (x=0, y=5, width=40, height=10) body:
````markdown
```proof:tree kind=org
root: Engineering
- Platform
- Product
```
````

### Expected output

Pasted at row 5, cols 0–39:
```
Engineering
├── Platform
└── Product
```

### Trace

**Step 1 — render_region_body dispatches proof:tree**
- `proof:tree kind=org` fenced block → `generate_tree_block(attrs, body)`

**Step 2 — generate_tree_block**
- Root=`Engineering`; children=[`Platform`, `Product`]; `render_inline_tree` produces 3 lines

**Step 3 — paste into canvas at region position**
- Lines clipped to width=40; pasted at x=0, y=5

### Findings

- **F69**: Clipping to region width: should happen in `generate_tree_block` (clip to `attrs.width`) or in the paste step? Spec should clarify where clipping responsibility lies.
- **F70**: If tree is taller than region height (height=10), excess lines are silently clipped by canvas paste. Spec must state whether this triggers a diagnostic or is silently truncated.

---

## Scenario 29 — All 6 element kinds in compile_element dispatch

**Tests:** `compile_element` correctly dispatches all six element kinds: value, delta, sparkline, mini-bar, label, badge.

### Input

Six separate `proof:element` directives with kinds: value, delta, sparkline, mini-bar, label, badge.

### Trace

**Step 1 — compile_element dispatch**
- Match on `ElementKind`: `Value` → `render_value_element`; `Delta` → `render_delta_element`; etc.

**Step 2 — per-kind rendering**
- `value`: center numeric string in width
- `delta`: prepend `+`/`-` sign
- `sparkline`: series → block chars `▁▂▃▄▅▆▇█`
- `mini-bar`: fill fraction of width with `█` chars
- `label`: text, no decoration
- `badge`: text surrounded by `[` `]` or box chars

**Step 3 — ELEMENT-003 warning for sparkline**
- Series length < 3 → ELEMENT-003 emitted

### Findings

- **F71**: `delta` kind in ASCII output — no color. Spec must state the text-only representation (sign + value, or `▼3.2%` for negative).
- **F72**: `badge` border chars — `[PASS]` vs `╔PASS╗`? Spec must enumerate badge rendering style per output mode (plain ASCII vs box-drawing).

---

## Scenario 30 — proof:row from md:// data table

**Tests:** `proof:row` with `source=md://data.md#table:0` iterates over 5 rows and applies R-1 column pinning.

### Input

`data.md` — 5-row table with columns Name, Score, Status.

````markdown
```proof:row source=md://data.md#table:0 separator=" │ "
- kind=label col=Name
- kind=value col=Score
- kind=badge col=Status
```
````

### Expected output

```
Alice │ 95 │ Pass
Bob   │ 82 │ Pass
Carol │ 71 │ Warn
Dave  │ 60 │ Fail
Eve   │ 88 │ Pass
```

### Trace

**Step 1 — parse_foreach, resolve_source**
- URI `md://data.md#table:0` → mdpath resolution → 5-row table

**Step 2 — render_row_foreach**
- For each row: call `compile_element` per column definition; separator ` │ ` inserted between columns

**Step 3 — R-1 column pinning**
- Column widths fixed by max across all rows; `Name` col: max(5,3,5,4,3)=5; pad all to 5

### Findings

- **F73**: R-1 column pinning requires two-pass render (first pass: max widths; second pass: render with pinned widths). Spec must confirm two-pass approach or specify streaming with buffering.
- **F74**: `separator=" │ "` contains Unicode box-drawing char; `visual_width(" │ ")` = 3. Spec must confirm separator width is included in total row width for R-1.

---

## Scenario 31 — proof:element kind=sparkline width=14

**Tests:** Sparkline with 10 data points normalized and rendered as block characters; ELEMENT-003 if too few points.

### Input

````markdown
```proof:element kind=sparkline width=14
series: 1,3,2,8,5,9,4,7,6,10
```
````

### Expected output

10 block chars padded to width 14: `▁▃▂▇▄█▄▆▅█    `

### Trace

**Step 1 — series normalization**
- Parse `1,3,2,8,5,9,4,7,6,10`; min=1, max=10
- Normalize: `(v-min)/(max-min)` → values in [0.0, 1.0]

**Step 2 — block char selection**
- 8 chars `▁▂▃▄▅▆▇█`; index = `floor(normalized * 7)`

**Step 3 — width=14, 10 points**
- 10 points < width 14 → pad right with spaces to fill width; ELEMENT-003 not triggered (10 ≥ 3)

### Findings

- **F75**: `width=14` with 10 data points — stretch each point to fill width, or pad the end with spaces? Spec must define behavior when point count < width.
- **F76**: If all series values are equal (min=max), normalization divides by zero. Spec must define the constant-series case (all mid-block `▄`? all `▁`?).

---

## Scenario 32 — proof:row with empty data table

**Tests:** `proof:row` source table has header but zero data rows — COMPILE-004 emitted, no rows in output.

### Input

`empty.md` — header-only table (no data rows).

### Expected output

`COMPILE-004 [WARN]: proof:row source table has 0 data rows`; output file written with empty block.

### Trace

**Step 1 — resolve_source**
- Parses `empty.md` table → header row found, 0 data rows

**Step 2 — rows.is_empty() guard**
- `if rows.is_empty() { emit COMPILE-004; return Ok(vec![]); }`

**Step 3 — written=true with empty output**
- COMPILE-004 is a warning; output file still written with empty block where rows would be

### Findings

- **F77**: Empty `proof:row` — leave a blank fence in output or remove entirely? Spec must state the empty-block representation.
- **F78**: An empty table may be intentional during development. Spec should note whether COMPILE-004 can be suppressed with `warn=false` or similar attribute.

---

## Scenario 33 — proof:element value with formatted string "1,024"

**Tests:** `value: "1,024"` — comma-stripped parse succeeds; original display string preserved.

### Input

````markdown
```proof:element kind=value
value: "1,024"
label: Records
```
````

### Expected output

```
  1,024
 Records
```

### Trace

**Step 1 — parse value field**
- Raw string: `"1,024"`

**Step 2 — cleaned.parse attempt**
- `cleaned = "1,024".replace(',', "") = "1024"` → `"1024".parse::<f64>()` = `Ok(1024.0)`
- Parse succeeds via cleaned form; store as `ElementData::Numeric(1024.0, display="1,024")`

**Step 3 — render**
- Center original display string `"1,024"` in element width

### Findings

- **F79**: Comma-stripping makes `"1,024"` parseable — the `ElementData::Text` fallback fires only if parsing fails even after cleaning. Spec must document exactly what cleaning is applied before parse attempt.
- **F80**: Display string preservation: when user writes `"1,024"`, display must show `1,024` not `1024` or `1024.0`. Spec must state that the original display string is preserved when numeric parse succeeds via cleaned form.

---

## Scenario 34 — proof:row separator=" │ " column alignment

**Tests:** Three-column row with separator ` │ ` satisfies R-1 — all rows same total visual width.

### Input

````markdown
```proof:row source=md://scores.md#table:0 separator=" │ "
- kind=label col=Name   width=8
- kind=value col=Score  width=5
- kind=badge col=Grade  width=4
```
````

### Expected output

```
Alice    │ 95    │ A   
Bob      │ 82    │ B   
```

### Trace

**Step 1 — R-1 invariant setup**
- Widths: Name=8, Score=5, Grade=4; separator `visual_width(" │ ")` = 3
- Total row width = 8 + 3 + 5 + 3 + 4 = 23

**Step 2 — render each element**
- Each element padded to its column width; separator inserted between columns

**Step 3 — verify visual_width per element**
- `visual_width(element_string) == column_width` for every element every row

### Findings

- **F81**: R-1 requires `visual_width` not byte length — Unicode multi-byte chars in names cause misalignment if byte length is used. Spec must mandate `visual_width` throughout row rendering.
- **F82**: Badge `"N/A"` (3 chars) in `width=4` must right-pad to 4. Spec must confirm badges are left-aligned and right-padded to `width`.

---

## Scenario 35 — proof:tree kind=dirtree

**Tests:** `proof:tree kind=dirtree root=src max_depth=2 exclude=target` scans filesystem and produces a tree.

### Input

````markdown
```proof:tree kind=dirtree root=src max_depth=2 exclude=target
```
````

### Expected output

```
src
├── lib.rs
├── main.rs
└── utils
    ├── mod.rs
    └── parse.rs
```

### Trace

**Step 1 — DirtreeOptions**
- `{ root: "src", max_depth: 2, exclude: ["target"] }`

**Step 2 — WalkDir with exclusion**
- Walk `src/` to depth 2; skip entries matching exclude patterns; sort alphabetically

**Step 3 — render_tree_lines**
- Apply `├──`/`└──` connectors based on sibling position at each depth

### Findings

- **F83**: `exclude=target` — basename match or path glob? Spec must define: a name match skips any directory named `target` at any depth.
- **F84**: `max_depth=2` — is depth counted from root (root=depth 0) or from root's children (children=depth 1)? Spec must define the depth counting origin.

---

## Scenario 36 — proof:tree kind=org inline body

**Tests:** `proof:tree kind=org` with inline body renders with `├──`/`└──` connector chars.

### Input

````markdown
```proof:tree kind=org
root: Engineering
- Platform
- Product
  - Frontend
  - Backend
```
````

### Expected output

```
Engineering
├── Platform
└── Product
    ├── Frontend
    └── Backend
```

### Trace

**Step 1 — render_inline_tree**
- Root node: `Engineering`; children at indent 0: Platform, Product; grandchildren at indent 2: Frontend, Backend

**Step 2 — connector chars**
- Platform has sibling → `├──`; Product is last → `└──`; Frontend has sibling → `├──`; Backend is last → `└──`

**Step 3 — depth from leading spaces**
- `depth = leading_spaces / indent_unit`; indent_unit detected from first child

### Findings

- **F85**: Mixed indentation (2-space and 4-space in same tree) is ambiguous. Spec must define whether indent_unit is auto-detected from first child or fixed at 2.
- **F86**: If `root:` key is absent from inline body, what is the root node? Error? First non-indented line? Spec must define the root detection rule.

---

## Scenario 37 — proof:tree kind=taxonomy from md:// source

**Tests:** Flat table with a `Category` column builds synthetic parent nodes in a taxonomy tree.

### Input

`taxonomy.md` — table with columns Category, Item; 3 rows (Mammals/Dog, Mammals/Cat, Reptiles/Lizard).

````markdown
```proof:tree kind=taxonomy source=md://taxonomy.md#table:0 parent-col=Category child-col=Item
```
````

### Expected output

```
(root)
├── Mammals
│   ├── Dog
│   └── Cat
└── Reptiles
    └── Lizard
```

### Trace

**Step 1 — resolve_source**
- 3 data rows; columns `Category` and `Item`

**Step 2 — build_dfs_tree**
- Group by `Category`: `{ "Mammals": ["Dog","Cat"], "Reptiles": ["Lizard"] }`
- Synthetic root `(root)` with categories as children

**Step 3 — render with connector chars**
- DFS traversal with `├──`/`└──` per node

### Findings

- **F87**: Synthetic root label `(root)` — hardcoded or configurable via `root-label=Animals`? Spec should allow override.
- **F88**: `build_dfs_tree` assumes 2 levels. For multi-level taxonomy (3+ columns), spec must define recursive grouping or explicitly limit to 2-level taxonomy.

---

## Scenario 38 — proof:tree kind=dependency inline body

**Tests:** `proof:tree kind=dependency` renders identically to `kind=org` — same connectors, different semantic label only.

### Input

````markdown
```proof:tree kind=dependency
root: proof-compile
- proof-canvas
- proof-math
  - unicode-width
```
````

### Expected output

```
proof-compile
├── proof-canvas
└── proof-math
    └── unicode-width
```

### Trace

**Step 1 — parse kind=dependency**
- `TreeKind::Dependency` → delegates to `render_inline_tree` (same as org)

**Step 2 — connector chars**
- Same `├──`/`└──` logic as org tree

**Step 3 — no rendering difference from kind=org**
- Kind label is purely documentary; output is identical to org tree

### Findings

- **F89**: `kind=dependency` and `kind=org` produce identical output. Spec should document why both exist (semantic intent) or consolidate them with a `label=` attribute.
- **F90**: Dependency trees can have cycles (A depends on B which depends on A). Spec must state how cycles are detected and reported (COMPILE-002? infinite-loop guard?).

---

## Scenario 39 — proof:tree kind=outline passthrough

**Tests:** `proof:tree kind=outline` renders a numbered hierarchy by preserving author-provided number prefixes.

### Input

````markdown
```proof:tree kind=outline
1. Introduction
   1. Background
   2. Motivation
2. Methods
```
````

### Expected output

```
1. Introduction
   1. Background
   2. Motivation
2. Methods
```

### Trace

**Step 1 — render_inline_outline**
- Input lines carry numeric prefixes; `kind=outline` passes them through verbatim

**Step 2 — passthrough format**
- No connector chars; indentation preserved from input

**Step 3 — no synthetic numbering**
- Unlike `proof:ol`, `proof:tree kind=outline` does NOT auto-number

### Findings

- **F91**: `proof:tree kind=outline` preserves author numbers; `proof:ol` auto-generates. Spec must make this distinction clear with a guidance note on when to use each.
- **F92**: If author's outline numbers are inconsistent (e.g., `1.` then `3.`), renderer passes them through silently. Spec should state whether sequential validation is performed.

---

## Scenario 40 — proof:tree broken source

**Tests:** `proof:tree source=md://missing.md` — resolution fails, COMPILE-002 emitted, output NOT written.

### Input

````markdown
```proof:tree kind=taxonomy source=md://missing.md#table:0 parent-col=Category child-col=Item
```
````

### Expected output

Diagnostic only — file not written:
```
COMPILE-002 [ERROR]: cannot resolve md://missing.md — file not found
```

### Trace

**Step 1 — resolve_source fails**
- `md://missing.md` → file does not exist; returns `Err(ResolveError::FileNotFound)`

**Step 2 — COMPILE-002 emitted**
- Error logged; `has_errors = true`

**Step 3 — written=false**
- Output file not written; existing compiled output (if any) left unchanged; exit non-zero

### Findings

- **F93**: When `written=false`, an existing stale output file is left on disk. Spec must state whether stale files are deleted, left in place, or replaced with an error marker.
- **F94**: COMPILE-002 fires for missing file. If file exists but `#table:0` not found, should that be a distinct sub-code? Spec should enumerate resolution failure sub-cases.

---

## Scenario 41 — [sym:name] expansion in prose, bullet, and title

**Tests:** `[sym:checkmark]` expands consistently in three contexts: prose, bullet label, slide title.

### Input

```markdown
Status: [sym:checkmark] All tests pass.

- [sym:checkmark] Unit tests
- [sym:warning] Integration tests

---
title: "[sym:checkmark] Build Passing"
```

### Expected output

```
Status: ✓ All tests pass.
- ✓ Unit tests
- ⚠ Integration tests
title: "✓ Build Passing"
```

### Trace

**Step 1 — expand_symbols called on every output line**
- Pattern `[sym:\w+]` matched globally; each match replaced via `SymbolLibrary::lookup`

**Step 2 — SymbolLibrary lookup**
- `checkmark` → `✓` (U+2713); `warning` → `⚠` (U+26A0)

**Step 3 — contexts: prose, bullet, YAML title**
- Symbol expansion runs before bullet rendering and before render_title

### Findings

- **F95**: `[sym:name]` inside YAML front-matter — YAML parsers may treat `[` as array start. Values containing `[sym:...]` must be quoted strings. Spec must warn authors.
- **F96**: Symbol lookup is case-sensitive by default. `[sym:CheckMark]` would fail. Spec must define the case rule (all lowercase, or case-folded lookup).

---

## Scenario 42 — proof:symbol block with size=3

**Tests:** `proof:symbol name=checkmark size=3` renders a 3×3 grid of the glyph.

### Input

````markdown
```proof:symbol name=checkmark size=3
```
````

### Expected output

```
✓✓✓
✓✓✓
✓✓✓
```

### Trace

**Step 1 — SymbolLibrary::new, resolve**
- `SymbolLibrary::lookup("checkmark")` → `✓`

**Step 2 — render_symbol_block**
- `size=3` → 3 rows × 3 columns of the glyph; each row: `✓✓✓`

**Step 3 — size scaling**
- `size=1` → 1×1; `size=N` → N×N glyph grid

### Findings

- **F97**: For wide symbols (visual_width=2), a 3-wide row = 6 display columns, not 3. Spec must define `size` in display columns or glyph-count units.
- **F98**: Scaling by repetition is only meaningful for decorative symbols. Spec should note that `size` is primarily intended for box/border/fill symbols, not complex glyphs.

---

## Scenario 43 — proof:shape banner

**Tests:** `proof:shape name=banner width=24 height=3 label="COMPILE"` renders a filled banner with centered label.

### Input

````markdown
```proof:shape name=banner width=24 height=3 label="COMPILE"
```
````

### Expected output

```
████████████████████████
█       COMPILE        █
████████████████████████
```

### Trace

**Step 1 — ShapeAttrs::parse**
- `name=banner`, `width=24`, `height=3`, `label="COMPILE"`

**Step 2 — render_shape**
- Top/bottom rows: `█` × 24; middle rows (height-2=1): `█` + centered label + `█`

**Step 3 — label placement**
- Inner width = 24 - 2 = 22; `center_in_width("COMPILE", 22)` → 7 left, 8 right spaces
- Middle row: `█       COMPILE        █`

### Findings

- **F99**: `center_in_width("COMPILE", 22)` — (22-7)/2 = 7.5 → floor=7 left, 8 right. Spec must state the rounding rule (same as F36).
- **F100**: `height=1` banner has no space for a middle row. Spec must handle `height=1` as a special case (single solid-fill row, or label embedded in the sole row?).

---

## Scenario 44 — Symbol + math coexistence, same line (confirmed)

**Tests:** `[sym:checkmark] $\alpha = 0.05$` — symbol first, math second, both expand correctly.

### Input

```markdown
Result: [sym:checkmark] $\alpha = 0.05$
```

### Expected output

```markdown
Result: ✓ α = 0.05
```

### Trace

**Step 1 — expand_symbols**
- `[sym:checkmark]` → `✓`; intermediate: `Result: ✓ $\alpha = 0.05$`

**Step 2 — expand_inline_math**
- `$\alpha = 0.05$` → `\alpha` → `α`; `0.05` literal; result: `Result: ✓ α = 0.05`

**Step 3 — write**
- Line stable; no further transformation

### Findings

- **F46**: (Confirmed from Scenario 16.) Pass order — symbols before math — must be documented. This scenario is the canonical positive-case confirmation.
- **F101**: `[sym:...]` cannot appear inside `$...$` delimiters. Spec must state that symbol tags inside math expressions are not expanded — the math tokenizer treats `[` as a literal or error character.

---

## Scenario 45 — proof:toc generation

**Tests:** `proof:toc max-depth=2 style=list` scans headings, skips headings inside code blocks, emits list TOC.

### Input

````markdown
```proof:toc max-depth=2 style=list
```

## Introduction

## Methods

### Subsection A

### Subsection B

## Results

```code
## Not a heading
```

## Conclusion
````

### Expected output

```
- Introduction
- Methods
  - Subsection A
  - Subsection B
- Results
- Conclusion
```

### Trace

**Step 1 — generate_toc, heading scan**
- Scan top-to-bottom tracking fence open/close state; skip lines inside fences

**Step 2 — skip code blocks**
- `` ```code `` fence encloses `## Not a heading` → excluded from TOC

**Step 3 — depth filter and list output**
- `max-depth=2` → include H2 (`##`) and H3 (`###`); H2 → top-level; H3 → 2-space indent

### Findings

- **F102**: TOC directive appears before the headings it references. Heading scan must skip the `proof:toc` fence itself. Spec must clarify the scan start point.
- **F103**: `style=list` produces a bullet list. Spec should enumerate other style values (e.g., `style=numbered`, `style=links`) and their output formats.

---

## Scenario 46 — proof:ol counter stack

**Tests:** `proof:ol` with two-level nesting produces correct `1.` / `1.1.` / `1.2.` prefixes.

### Input

````markdown
```proof:ol
- First
  - Sub one
  - Sub two
- Second
```
````

### Expected output

```
1. First
   1.1. Sub one
   1.2. Sub two
2. Second
```

### Trace

**Step 1 — render_ol, initial counter stack**
- `stack = [0]`; `- First` → stack[0]++ = 1 → emit `1.`

**Step 2 — sub-items push new counter**
- `  - Sub one` → push: stack=[1,0]; stack[1]++ = 1 → emit `1.1.`
- `  - Sub two` → stack[1]++ = 2 → emit `1.2.`

**Step 3 — pop back to level 0**
- Dedent → pop: stack=[1]; `- Second` → stack[0]++ = 2 → emit `2.`

### Findings

- **F57**: (Confirmed from Scenario 22.) Counter does not reset on pop — parent counter increments when the parent item is encountered, not when exiting children.
- **F104**: Sub-item indent formula: `indent = parent_counter_display_width + 1`. For `1.` (width=2) → 3 spaces. Spec should codify this formula.

---

## Scenario 47 — proof:right text alignment

**Tests:** `proof:right` in a 40-column slide right-aligns each line by left-padding with spaces.

### Input

````markdown
```proof:right width=40
Aligned right.
Short.
```
````

### Expected output

```
                          Aligned right.
                                 Short.
```

### Trace

**Step 1 — render_right**
- `width=40`; for each line: `padding = width - visual_width(line)`

**Step 2 — padding computation**
- `"Aligned right."` → visual_width=14; padding=26 spaces
- `"Short."` → visual_width=6; padding=34 spaces

**Step 3 — emit**
- Each line: `" ".repeat(padding) + line`

### Findings

- **F105**: If a line is wider than `width`, padding is negative. Spec must define behavior: truncate, emit warning, or leave as-is.
- **F106**: `width` attr on `proof:right` vs inherited slide width — precedence rule needed. If in a two-column region, use region width or slide width?

---

## Scenario 48 — proof compile --watch initial pass

**Tests:** `--watch` mode performs an initial compile of all targets, then enters the file-watch loop.

### Input

`proof.toml` with two `[[compile]]` targets. Run: `proof compile --watch`

### Trace

**Step 1 — compile_watch_pass: initial file collection**
- For each target: glob `source_dir/**/*.source.md` → collect source files

**Step 2 — initial compile**
- Compile all collected files; report: `Compiled N files across 2 targets`

**Step 3 — enter watch loop**
- Register filesystem watchers on all `source_dir` paths and all referenced figure files
- Block on watcher events; recompile affected files on change

### Findings

- **F107**: After initial compile, watch must also watch figure files referenced by source documents (cross-reference F25). Spec must state that the watch set includes all `md://` URIs found during initial compile.
- **F108**: If initial compile has errors, does watch still enter the watch loop or exit? Spec must state watch continues even after initial errors so authors can fix files without restarting.

---

## Scenario 49 — [[compile]] multi-target routing

**Tests:** Two `[[compile]]` targets in `proof.toml` route different source directories to different output directories.

### Input

`proof.toml`:
```toml
[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
```

### Trace

**Step 1 — source_dir_pairs construction**
- Parse `proof.toml` → `Vec<CompileTarget>`; two targets with distinct source and output dirs

**Step 2 — per-target output_dir**
- `src/guides/01-intro.source.md` → strip `src/guides/` → `01-intro.md` → prepend `docs/guides/` → `docs/guides/01-intro.md`
- `src/presentations/keynote.source.md` → `docs/presentations/keynote.md`

**Step 3 — compile and write independently**
- Each target compiled and written to its respective output dir

### Findings

- **F109**: A source file matching two `source_dir` globs would be compiled twice to different output dirs. Spec must state whether overlapping targets are an error or handled (last target wins).
- **F110**: `source_dir` paths in `proof.toml` are relative to the `proof.toml` file location. Spec must confirm this base path rule.

---

## Scenario 50 — proof compile --output-dir override

**Tests:** `--output-dir docs/out` on the CLI overrides all `[[compile]]` target `output_dir` values.

### Input

`proof.toml` has two targets: `docs/guides` and `docs/presentations`.
Run: `proof compile --output-dir docs/out`

### Expected output

Both targets write to `docs/out/`:
- `docs/out/01-intro.md`
- `docs/out/keynote.md`

### Trace

**Step 1 — output_dir_override takes precedence**
- CLI flag sets `output_dir_override = Some("docs/out")`
- During path computation: if override is set, use it instead of per-target `output_dir`

**Step 2 — all files routed to single dir**
- Both source files → flat output in `docs/out/`

**Step 3 — directory created if absent**
- `docs/out/` created if it does not exist before first write

### Findings

- **F111**: Flat output loses `guides/` vs `presentations/` hierarchy. If two sources have the same filename in different targets, they collide in `docs/out/`. Spec must define collision behavior (error? last-write-wins?).
- **F112**: `--output-dir` interacts with `--watch`. In watch mode with override, newly created source files must also route to the override dir. Spec must confirm override persists for the entire watch session.

---

## Scenario 51 — Broken md:// caught by proof check

**Tests:** `proof check` with `SourceLinkCheck` finds a `md://missing.md` reference and emits `md_broken_uri` diagnostic.

### Input

`guide.source.md` containing:
````markdown
```proof:include
md://figures/missing-figure.md#fig:0
```
````

`figures/missing-figure.md` does not exist.

### Trace

**Step 1 — fence detection**
- `proof check` scans `.source.md` files for fenced blocks; `proof:include` found

**Step 2 — URI extraction**
- Body: `md://figures/missing-figure.md#fig:0`; path: `figures/missing-figure.md`

**Step 3 — file existence check**
- `Path::exists("figures/missing-figure.md")` → false
- Emit `md_broken_uri` diagnostic: `CHECK: broken md:// URI at guide.source.md:2 — target not found`

### Findings

- **F113**: `proof check` is a lint pass, not a compile pass — it does not write output files. Spec must confirm exit code: non-zero if any `md_broken_uri` found?
- **F114**: URI extraction must handle multi-line directive bodies (URI on second line, comments on others). Spec should define the URI extraction rule (first non-blank line? all lines matching `md://` prefix?).

---

## Scenario 52 — proof:tree empty body and no source

**Tests:** `proof:tree` with empty body and no `source=` attribute emits COMPILE-002 and does not write output.

### Input

````markdown
```proof:tree kind=org
```
````

(No body text, no `source=` attribute.)

### Trace

**Step 1 — body.trim().is_empty() guard**
- After parsing attrs, body content extracted: `body.trim() == ""`

**Step 2 — no source attr**
- `attrs.source == None`; both body and source absent

**Step 3 — COMPILE-002 emitted, written=false**
- `COMPILE-002 [ERROR]: proof:tree has no body and no source attribute`
- `has_errors = true`; output file not written

### Findings

- **F115**: COMPILE-002 is reused for "missing source file" (Scenario 40) and "missing body+source" (here). Spec should either use distinct sub-codes or provide enough context in the message to distinguish the two failure modes.
- **F116**: An empty `proof:tree` body might be a work-in-progress stub. Spec should note whether a `stub=true` attribute suppresses the error for intentional stubs during authoring.

---

## Scenario 53 — proof:math MATH-003 mismatched environment

**Tests:** `\begin{pmatrix}...\end{bmatrix}` emits MATH-003 and renders partial output.

### Input

````markdown
```proof:math
\begin{pmatrix} a & b \\ c & d \end{bmatrix}
```
````

### Trace

**Step 1 — parse \begin{pmatrix}**
- Enter pmatrix environment; expect closing `\end{pmatrix}`

**Step 2 — encounter \end{bmatrix}**
- Mismatch: expected `pmatrix`, found `bmatrix`
- Emit `MATH-003 [ERROR]`: mismatched environment delimiters

**Step 3 — partial render**
- Best-effort: use opening delimiter (`pmatrix` → `⎛⎝`) for partial output
- `has_errors = true`

### Findings

- **F117**: MATH-003 is an error — does it set `written=false` or write partial output? For math errors, writing corrupted math may be worse than writing nothing. Spec must state the written policy for MATH-003.
- **F118**: Best-effort render of mismatched environments is ambiguous. Spec should define either: (a) always render with opening delimiter, or (b) refuse to render and emit only the error.

---

## Scenario 54 — proof compile with errors: output file untouched

**Tests:** When `has_errors=true`, the output file is not written — existing compiled output left unchanged.

### Input

`guide.source.md` has a `proof:tree` with missing source (COMPILE-002). Previous compile produced `guide.md`.

### Trace

**Step 1 — compile triggers COMPILE-002**
- `has_errors = true`

**Step 2 — has_errors check before write**
- `if has_errors { return Err(CompileError::HasErrors); }`; write step skipped

**Step 3 — sentinel file test**
- `guide.md` on disk is the previously compiled version; not deleted, truncated, or replaced
- Exit code: non-zero

### Findings

- **F119**: Leaving stale output on disk can mislead tooling that serves `guide.md`. Spec should consider a `--keep-stale` vs `--delete-on-error` policy flag, or at minimum document stale-output behavior.
- **F120**: `proof compile` should report "1 file had errors, 0 files written" in stdout even when `has_errors` — the operator needs to know which files were not updated.

---

## Scenario 55 — proof:row missing source URI

**Tests:** `proof:row` with empty `source_uri` emits COMPILE-002.

### Input

````markdown
```proof:row
- kind=label col=Name
```
````

(No `source=` attribute — `source_uri` is empty string.)

### Trace

**Step 1 — compile_row called with empty URI**
- `RowAttrs { source_uri: "", columns: [...] }`

**Step 2 — resolve_source_for_compile fails**
- `if source_uri.is_empty() { return Err(CompileError::MissingSource); }`

**Step 3 — COMPILE-002 emitted**
- `COMPILE-002 [ERROR]: proof:row missing required source= attribute`
- `has_errors = true`; `written = false`

### Findings

- **F121**: `compile_row` with empty URI vs URI that resolves to missing file — both emit COMPILE-002 but with different messages. Spec should provide distinct error text to help authors diagnose quickly.
- **F122**: `source=` is a required attribute for `proof:row`. Spec should list it as required in the directive reference, and `proof check` should catch its absence before compile time.

---

## Scenario 56 — proof:math overflow — width=5 for wide expression

**Tests:** `proof:math width=5` for a wide fraction triggers MATH-004 warning and clips output.

### Input

````markdown
```proof:math width=5
\frac{alpha+beta}{gamma}
```
````

Expression natural width = 11 (`alpha+beta`). Width constraint = 5.

### Trace

**Step 1 — render display math**
- `render_frac` computes natural bar_width = 11; configured `width=5` < 11

**Step 2 — MATH-004 warning**
- `MATH-004 [WARN]: display math exceeds width=5 (natural width=11); output clipped`

**Step 3 — clip_to_width applied**
- Each line clipped to 5 display columns; `written=true` (warning, not error)

### Findings

- **F123**: Clipping at 5 columns may split multi-byte Unicode chars (a 2-wide CJK char at columns 4–5 would be split). Spec must define `clip_to_width` for Unicode: always clip at safe boundary (no split of wide chars).
- **F124**: MATH-004 is a warning (`written=true`). Clipped display math may be misleading. Spec should consider MATH-004 as an error for display math overflow.

---

## Scenario 57 — proof-canvas TUI integration: three regions

**Tests:** Three regions pasted at specific positions produce a clean 80×24 render with no bleed.

### Input

```rust
let mut canvas = Canvas::new(80, 24);
canvas.paste(0, 0, &["Header line 1", "Header line 2"]);
canvas.paste(0, 5, &["Body content"]);
canvas.paste(60, 0, &["Sidebar line 1", "Sidebar line 2"]);
let output = canvas.render();
```

### Trace

**Step 1 — Canvas::new**
- 24 rows × 80 cols initialized to space

**Step 2 — three paste() calls**
- `paste(0,0,...)` writes rows 0–1, cols 0–12
- `paste(60,0,...)` writes rows 0–1, cols 60–73 (no overlap)
- `paste(0,5,...)` writes row 5, cols 0–11

**Step 3 — render() → final string**
- Each row joined as 80-char string; rows joined with `\n`; no bleed between regions

### Findings

- **F125**: `Canvas::paste` must only overwrite cells covered by the line, not the full region width. Spec must confirm: paste writes only as many cells as the line is wide, leaving remaining canvas cells unchanged.
- **F126**: `render()` returns rows with trailing spaces (80 chars each). Spec must state whether trailing spaces are trimmed per-row or preserved — trimming affects visual alignment assertions.

---

## Scenario 58 — proof-canvas wide char at column boundary

**Tests:** A CJK character (visual_width=2) at column 78 of an 80-wide canvas is placed correctly; second column filled with placeholder space.

### Input

```rust
let mut canvas = Canvas::new(80, 1);
canvas.paste(78, 0, &["界"]); // U+754C, visual_width=2
```

### Expected output

Row 0: 78 spaces + `界` + placeholder space (total: 80 display cols)

### Trace

**Step 1 — char_width returns 2**
- `unicode_width::UnicodeWidthChar::width('界')` → `Some(2)`

**Step 2 — place glyph at col 78**
- `cell[0][78] = '界'`; `cell[0][79] = ' '` (placeholder for wide char's second column)

**Step 3 — next char at col+2**
- Any subsequent char would start at col 80 — out of bounds, silently ignored

### Findings

- **F127**: Placeholder space at col 79 ensures correct rendering. Spec must define the placeholder convention (regular space, zero-width space, sentinel value?) and confirm `render()` outputs glyph + placeholder correctly.
- **F128**: Wide char placed at last column (col 79 in width=80): only one column available. Spec must define whether char is placed (truncated visually) or rejected (canvas silently skips it).

---

## Scenario 59 — proof-math standalone: summation with limits

**Tests:** `proof_math::expand_inline_math("$\\sum_{i=1}^n i$")` produces best-effort Unicode expansion via the public API.

### Input

```rust
let result = proof_math::expand_inline_math("$\\sum_{i=1}^n i$");
```

### Expected output

`Ok("Σ_{i=1}ⁿ i")` (partial Unicode: `n` → `ⁿ`; `i=1` kept as `_{i=1}` because `=` has no Unicode subscript)

### Trace

**Step 1 — public API, tokenize**
- `\sum` → `Σ`; `_{i=1}` → subscript group `i=1`; `^n` → superscript `n`

**Step 2 — Subscript/Superscript handling**
- `n` superscript: `ⁿ` available (U+207F)
- `i=1` subscript: `i` → `ᵢ` but `=` has no Unicode subscript → fallback: keep as `_{i=1}`

**Step 3 — lim operator inline**
- Result: `Σ_{i=1}ⁿ i` — partial expansion, `Ok` returned with best-effort string

### Findings

- **F129**: Mixed subscripts (some chars have Unicode equivalents, some do not) — spec must define the fallback policy: render all as Unicode (skip missing) or render all as plain `_{...}` text.
- **F130**: `expand_inline_math` returns `Result<String, MathError>`. For partial expansion, should it return `Ok(partial_string)` or `Err(MathError::PartialExpansion)`? Spec must define success/error semantics.

---

## Scenario 60 — proof-math render_display_math: centered π at width=40

**Tests:** `render_display_math("\\pi", width=40, align=center)` produces `π` centered in 40 columns.

### Input

```rust
let block = proof_math::render_display_math(
    "\\pi",
    DisplayOpts { width: 40, align: MathAlign::Center }
);
```

### Expected output

```
                   π                    
```
(19 leading spaces + `π` + 20 trailing spaces = 40 cols)

### Trace

**Step 1 — tokenize `\pi`**
- `\pi` → `π` (U+03C0); natural width = 1

**Step 2 — auto-width=1, center alignment**
- `left_pad = floor((40 - 1) / 2) = 19`; `right_pad = 40 - 1 - 19 = 20`

**Step 3 — MathAlign::Center output**
- `" ".repeat(19) + "π" + " ".repeat(20)`; single-line block; total visual_width = 40

### Findings

- **F131**: `render_display_math` return type — `Vec<String>` (one per line) or `String`? For multi-line expressions, must be `Vec<String>`. Spec must define the return type.
- **F132**: `width=40` is a hard constraint. If expression natural width > 40, MATH-004 should fire. Spec must confirm `render_display_math` applies the same overflow check as the directive renderer.

---

## Finding Index — Scenarios 09-60

| Finding | Scenario | Topic |
|---------|----------|-------|
| F32 | 09 | Inline math whitespace handling around commands |
| F33 | 09 | Operator passthrough enumeration |
| F34 | 10 | Superscript multi-char fallback (`^{10}`) |
| F35 | 10 | YAML quoting of `$...$` in front-matter |
| F36 | 11 | Fraction centering rounding rule |
| F37 | 11 | Brace-group vs parenthesis parsing in numerator |
| F38 | 12 | Multi-char superscript fallback in integral body |
| F39 | 12 | Lower limit horizontal alignment under `⌡` |
| F40 | 13 | Spacer rows for 2-row vs 3+-row matrices |
| F41 | 13 | `&` as cell separator vs HTML entity |
| F42 | 14 | Complex numerator downgrade needs parentheses |
| F43 | 14 | MATH-005 warning: `written=true` confirmation |
| F44 | 15 | Public API delimiter contract (`$...$` vs inner expr) |
| F45 | 15 | Bare ASCII letters are always literals |
| F46 | 16 | Symbol-before-math pass order must be documented |
| F47 | 17 | Blank-padded remainder lines in title-only slide |
| F48 | 17 | Overlong title: truncate vs. wrap |
| F49 | 18 | Indent-per-level: configurable or fixed |
| F50 | 18 | Tab normalization in bullet source |
| F51 | 19 | Two-column odd-width rounding (which column gets remainder) |
| F52 | 19 | `---` column separator vs slide separator disambiguation |
| F53 | 20 | Stat cell remainder columns for non-divisible widths |
| F54 | 20 | Stat cell overflow for wide value strings |
| F55 | 21 | Callout label padding in top border |
| F56 | 21 | Unknown callout style handling |
| F57 | 22 | Counter does not reset on pop (cross-ref Sc46) |
| F58 | 22 | Sub-item indent formula |
| F59 | 23 | `---` front-matter vs slide separator parsing |
| F60 | 23 | `<!-- count=N -->` sentinel purpose and absence handling |
| F61 | 24 | Notes sidecar output for presenter tools |
| F62 | 24 | `proof check` SL-5 verification rule |
| F63 | 25 | Region under-fill: remaining rows stay as spaces |
| F64 | 25 | Canvas default fill character is U+0020 |
| F65 | 26 | AABB overlap check complexity |
| F66 | 26 | Adjacent (touching) regions: overlap or not |
| F67 | 27 | Element horizontal placement within region |
| F68 | 27 | Element wider than region: clip behavior |
| F69 | 28 | Clip happens in generate_tree_block or paste step |
| F70 | 28 | Tree taller than region: silent clip or diagnostic |
| F71 | 29 | Delta text-only representation (sign + value) |
| F72 | 29 | Badge border chars per output mode |
| F73 | 30 | Two-pass render for R-1 column pinning |
| F74 | 30 | Separator width included in total row width |
| F75 | 31 | Sparkline width > point count: stretch vs pad |
| F76 | 31 | Constant-series normalization (division by zero) |
| F77 | 32 | Empty proof:row block representation in output |
| F78 | 32 | COMPILE-004 suppression mechanism |
| F79 | 33 | Cleaning steps applied before numeric parse |
| F80 | 33 | Original display string preservation |
| F81 | 34 | visual_width mandatory throughout row rendering |
| F82 | 34 | Badge left-aligned, right-padded to width |
| F83 | 35 | Exclusion pattern: basename vs path glob |
| F84 | 35 | max_depth counting origin (root=0 or root's children=1) |
| F85 | 36 | indent_unit: auto-detect or fixed |
| F86 | 36 | Root detection when `root:` key absent |
| F87 | 37 | Synthetic root label configurability |
| F88 | 37 | Multi-level taxonomy (3+ columns) |
| F89 | 38 | kind=dependency vs kind=org rendering difference |
| F90 | 38 | Cycle detection in dependency trees |
| F91 | 39 | proof:tree kind=outline vs proof:ol distinction |
| F92 | 39 | Outline number validation |
| F93 | 40 | Stale output file handling when written=false |
| F94 | 40 | COMPILE-002 sub-cases (file missing vs section missing) |
| F95 | 41 | `[sym:...]` in YAML must be quoted |
| F96 | 41 | Symbol lookup case sensitivity |
| F97 | 42 | size= in display columns vs glyph-count for wide chars |
| F98 | 42 | size= for complex glyphs |
| F99 | 43 | Banner label centering rounding (same as F36) |
| F100 | 43 | height=1 banner special case |
| F101 | 44 | `[sym:...]` inside `$...$` not expanded |
| F102 | 45 | TOC heading scan start (skip TOC fence itself) |
| F103 | 45 | TOC style enumeration |
| F104 | 46 | Sub-item indent formula for proof:ol |
| F105 | 47 | proof:right overflow (line wider than width) |
| F106 | 47 | proof:right width precedence (attr vs region vs slide) |
| F107 | 48 | Watch set includes md:// URIs from initial compile |
| F108 | 48 | Watch mode continues after initial compile errors |
| F109 | 49 | Overlapping source_dir targets between compile entries |
| F110 | 49 | source_dir relative to proof.toml location |
| F111 | 50 | Filename collision when --output-dir flattens hierarchy |
| F112 | 50 | --output-dir persists for entire watch session |
| F113 | 51 | proof check exit code for broken URIs |
| F114 | 51 | URI extraction rule from multi-line directive bodies |
| F115 | 52 | COMPILE-002 reuse for distinct failure modes |
| F116 | 52 | stub=true suppression for empty proof:tree |
| F117 | 53 | MATH-003 written policy (partial output vs suppress) |
| F118 | 53 | Best-effort render for mismatched environments |
| F119 | 54 | Stale output policy flag (--keep-stale) |
| F120 | 54 | Compile report counts (files with errors vs written) |
| F121 | 55 | COMPILE-002 messages: missing attr vs file not found |
| F122 | 55 | source= listed as required in directive reference |
| F123 | 56 | clip_to_width Unicode safety (no split wide chars) |
| F124 | 56 | MATH-004 severity for display math overflow |
| F125 | 57 | Canvas::paste writes only line-covered cells |
| F126 | 57 | render() trailing space policy per row |
| F127 | 58 | Wide char placeholder convention |
| F128 | 58 | Wide char at last column: place or reject |
| F129 | 59 | Mixed subscript Unicode fallback policy |
| F130 | 59 | expand_inline_math success/error for partial expansion |
| F131 | 60 | render_display_math return type (Vec<String> vs String) |
| F132 | 60 | Public API overflow check (same as directive renderer) |
