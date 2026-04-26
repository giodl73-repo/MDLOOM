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
