# proof — Tutorial

Zero to first scan in five minutes. By the end you will have:

- installed `proof`
- run a scan against a small docs directory and read the output
- written a real `proof.toml` with a section schema
- generated a draft fix plan, reviewed it, and applied the auto-fixable parts
- pinned a figure with DaVinci invariants and verified `proof check --daVinci` catches drift

The example docs directory is small (10 files) and contains the kinds of drift `proof` is built to catch: a misaligned ASCII box, a table missing a required row, a broken internal link, a wide Unicode character.

---

## 0. Prerequisites

You need:

- A Rust toolchain (`cargo`) — install from <https://rustup.rs> if missing.
- A directory of markdown you want to validate. (We will scaffold one in step 2.)

That's it. `proof` is a single static binary; no Node, no Python, no runtime.

---

## 1. Install (60 seconds)

```bash
git clone https://github.com/giodl73-repo/PROOF
cd proof
cargo build --release
./target/release/proof --version
```

Move the binary onto your `PATH` if you want to call it from anywhere:

```bash
# Linux/macOS
sudo cp target/release/proof /usr/local/bin/

# Windows (PowerShell, run as admin if needed)
Copy-Item target\release\proof.exe C:\Windows\System32\
```

Verify:

```bash
proof --version
# proof 0.5.0
```

---

## 2. Scaffold a sample docs directory (60 seconds)

We'll build a tiny, realistic docs tree:

```
mydocs/
├── README.md
├── architecture/
│   ├── 00-OVERVIEW.md
│   ├── 01-COMPONENTS.md      ← contains a misaligned ASCII box
│   └── 02-DATAFLOW.md
├── reference/
│   ├── 00-OVERVIEW.md
│   ├── 01-API.md             ← Type System Snapshot table missing a row
│   └── 02-CLI.md             ← contains a wide Unicode em-dash
├── guides/
│   ├── 00-OVERVIEW.md
│   └── 01-GETTING-STARTED.md ← link to a deleted file
└── CHANGELOG.md              ← we'll exclude this from checks
```

Just create the directory; copy this minimal `01-COMPONENTS.md` to give us something to lint:

```markdown
# Components

## The Big Picture

```
+---------------+     +----------------+
| Frontend      | --> | API Gateway    |
+---------------+     +----------------+
                              |
                              v
                      +----------------+
                      | Service Layer  |
                      +-----------------+   ← width mismatch
```

## Decision Cheat Sheet

| Component | When to use |
|-----------|-------------|
| Frontend  | UI rendering |

```

Note the bottom border of the third box is one `-` longer than the top — exactly the silent drift `proof` exists to catch.

---

## 3. First scan (30 seconds)

From inside `mydocs/`:

```bash
proof check .
```

You'll see something like:

```
mydocs/architecture/01-COMPONENTS.md:11:1: error [ascii_box_width]: bottom border 21 chars, top border 20
mydocs/reference/02-CLI.md:8:14: error [char_wide]: U+2014 EM DASH (2 cols) breaks alignment
mydocs/guides/01-GETTING-STARTED.md:34:1: warning [md_missing_section]: required H2 "Decision Cheat Sheet" absent

FAIL — 10 files checked, 2 errors, 1 warning
```

### Reading the output

Each line is `file:line:col: severity [code]: message`.

| Element | Meaning |
|---------|---------|
| `file:line:col` | Exact location — your editor can usually jump straight to it |
| `severity` | `error` (blocks CI), `warning` (advisory), `info` |
| `[code]` | Short identifier (`ascii_box_width`, `char_wide`, etc.) — useful for grep and silencing |
| message | What's wrong, in one line |

The trailing summary tells you whether the run failed (`FAIL`) or passed (`OK`), how many files were scanned, and the error/warning counts. Exit code is non-zero if there were errors (override with `--no-fail` for advisory runs).

### Useful flags right away

```bash
proof check . --errors-only           # hide warnings — useful for CI
proof check . -f json -o diags.json   # machine-readable output
proof check . -f github               # GitHub Actions annotations
proof check . -f rich                 # context-rich output (used by `draft`)
proof stats . --by-code               # how many of each diagnostic code
```

---

## 4. Write your first proof.toml (90 seconds)

`proof check` worked with zero config — it ran sensible defaults. To enforce *your* rules, drop a `proof.toml` at the root of `mydocs/`.

The fastest way:

```bash
cd mydocs
proof init
```

Then open it and edit. A real-world starter looks like this — every line earns its keep:

```toml
# proof.toml — root schema for mydocs

[meta]
name = "mydocs"
description = "Internal documentation for the Foo platform"

[files]
include = ["**/*.md"]
exclude = ["CHANGELOG.md", "_archive/**"]
root = true                          # stop cascade here

# ─── ASCII art ────────────────────────────────
[ascii_box]
enabled = true
tolerance = 0                        # exact alignment required
code_blocks_only = true              # don't false-positive on prose

[ascii_flow]
enabled = true

[ascii_char]
enabled = true                       # catch wide chars that break alignment

# ─── Markdown structure ────────────────────────
[markdown]
enabled = true
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet"]   # every guide ends with this

[[markdown.required_patterns]]
pattern = "```"
description = "must contain at least one code block"
severity = "warning"
```

Re-run:

```bash
proof check .
```

You'll now see the warning about `01-GETTING-STARTED.md` upgraded to a hard error in any guide that's missing the Decision Cheat Sheet H2 — and the wide-character check from `[ascii_char]` is now active.

### Excluding files mid-tree

`exclude` patterns are matched against the relative path under the config's directory. Common patterns:

```toml
exclude = [
    "TRACKER.md",                 # specific file at any depth that matches the literal name
    "CHANGELOG.md",
    "_archive/**",                # entire subtree
    "*/00-OVERVIEW.md",           # every directory's overview file
]
```

---

## 5. Add a section schema (60 seconds)

Different parts of your docs have different rules. The `reference/` directory should have API tables with a fixed shape; the `guides/` directory should have a "Getting Started" H2 in every file. Add per-section rules with `[[section_schemas]]`:

```toml
# Continue editing mydocs/proof.toml

# Reference section — every file must have an API table with these columns
[[section_schemas]]
paths = ["reference/**"]
paths_exclude = ["reference/00-OVERVIEW.md"]
required_h2_all = ["API Reference", "Type System Snapshot"]

# Guides section — every file must have a Getting Started section
[[section_schemas]]
paths = ["guides/**"]
paths_exclude = ["guides/00-OVERVIEW.md"]
required_h2_all = ["Getting Started", "Common Pitfalls"]

# Architecture section — every file must have a "Big Picture" diagram
[[section_schemas]]
paths = ["architecture/**"]
paths_exclude = ["architecture/00-OVERVIEW.md"]
required_h2_all = ["The Big Picture"]
```

For tables with a *known shape* (required columns, required rows), use `[[markdown_table.table_schemas]]`:

```toml
# Type System Snapshot must have these exact rows in the key column
[[markdown_table.table_schemas]]
heading = "Type System Snapshot"
required_columns = ["Axis"]
required_row_keys = ["Binding", "Typing", "Memory model"]
min_body_rows = 3
```

Re-run:

```bash
proof check .
```

Now `reference/01-API.md` will fail loudly with `table_missing_row` if any of those required rows is absent — no more silently incomplete reference tables.

> **Cascade tip.** You can put a `proof.toml` in `reference/` itself with reference-specific rules. It inherits the root `proof.toml` automatically. Schemas in directory-level configs use simple paths (`paths = ["*.md"]`) — `proof` auto-prefixes them with the directory name. See `SCHEMA-REFERENCE.md` for the full merge semantics.

To inspect the *effective* config for any file:

```bash
proof config reference/01-API.md
```

---

## 6. The fix pipeline (90 seconds)

This is where `proof` differs from a normal linter. Instead of just reporting errors, it produces a **draft plan** that an AI fills in with reasoning, then applies with safety guards.

### Step 1: generate a draft plan

```bash
proof draft . -o draft-plan.json
```

You'll see:

```
draft — 2 errors, 1 warnings across 3 groups (1 auto-fixable, 2 need review)
Draft plan written to draft-plan.json

Next steps:
  1. Open draft-plan.json — AI fills in `decision` and `new_string` for non-auto groups
  2. proof fix --plan draft-plan.json --dry-run
  3. proof fix --plan draft-plan.json
```

A draft plan looks like this:

```json
{
  "schema_version": "1",
  "summary": { "total_groups": 3, "auto_fixable": 1, "needs_review": 2 },
  "groups": [
    {
      "id": "g-001",
      "kind": "auto",
      "code": "ascii_box_width",
      "file": "architecture/01-COMPONENTS.md",
      "line": 11,
      "context": "+----------------+\n| Service Layer  |\n+-----------------+",
      "diagnosis": "bottom border is one char longer than top",
      "decision": "trim trailing - to match top width",
      "edit": {
        "old_string": "+-----------------+",
        "new_string": "+----------------+"
      },
      "confidence": "high"
    },
    {
      "id": "g-002",
      "kind": "needs_review",
      "code": "char_wide",
      "file": "reference/02-CLI.md",
      "line": 8,
      "context": "Use proof check — fast and exact.",
      "diagnosis": "U+2014 EM DASH at column 14; replaces with `--` would preserve narrow alignment",
      "decision": null,
      "edit": null,
      "confidence": null
    }
  ]
}
```

The `auto` groups are filled in by `proof draft` itself — these are deterministic fixes (e.g. trim-to-match-width). The `needs_review` groups have rich context attached (`context`, `diagnosis`) so an AI can fill in `decision` and `edit.new_string` with reasoning.

### Step 2: hand the plan to an AI (or fill it in yourself)

Open `draft-plan.json` in Claude / Cursor / your AI of choice and ask: "Fill in `decision` and `edit` for every `needs_review` group. Set `confidence` to `high`, `medium`, or `low`."

The AI sees the full surrounding context for each diagnostic, so it can make informed calls.

### Step 3: preview the plan

Before applying anything, dry-run:

```bash
proof fix --plan draft-plan.json --dry-run
```

Output shows every fix that *would* apply, none of them written. Review carefully.

### Step 4: apply

```bash
proof fix --plan draft-plan.json
# Applies high-confidence fixes by default; adjust with --min-confidence
```

What you get for free:

| Guard | What it prevents |
|-------|------------------|
| `--min-confidence high` (default) | Low/medium confidence fixes are skipped — safest path |
| **Signal-loss check** | A fix that removes non-whitespace content is rejected unless you pass `--no-signal-check` |
| **Bottom-up application** | Fixes apply highest-line-first, so earlier line numbers stay valid |
| **Stale-anchor detection** | If `old_string` no longer matches current file content, the fix is **skipped and logged**, not silently corrupted |
| **Auto re-check** | After applying, `proof check` runs again to verify zero errors remain (suppress with `--no-verify`) |

If `proof fix` finishes with `DONE — N fixes applied, 0 skipped` and the verify pass shows `zero errors remaining`, you're done. Commit.

```bash
proof fix --plan draft-plan.json --min-confidence medium  # apply more aggressive fixes
proof fix --plan draft-plan.json --dry-run                # see what *would* happen
```

---

## 7. Pinning canonical figures (DaVinci)

The pipeline above catches drift inside individual files. **DaVinci pinning** protects figures across time — a diagram you pin carries invariants that must hold on every future run. If the diagram changes in a way that violates a rule, `proof check --daVinci` fails before the change can ship.

### Register a figure with `proof pin`

```bash
proof pin "md://architecture/01-COMPONENTS.md#the-big-picture:0" --id service-layer
```

This resolves the URI, verifies the figure exists, and appends to `proof.toml`:

```toml
[[davinci]]
id = "service-layer"
uri = "md://architecture/01-COMPONENTS.md#the-big-picture:0"
description = "Canonical 3-tier architecture diagram"
protection = "warn"
```

### Add invariants

Edit the `[[davinci]]` block to declare what must always be true:

```toml
[[davinci]]
id = "service-layer"
uri = "md://architecture/01-COMPONENTS.md#the-big-picture:0"
protection = "error"

  [[davinci.invariant]]
  rule = "box-count"
  min = 3

  [[davinci.invariant]]
  rule = "contains-text"
  value = "Service Layer"
```

### Verify pins

```bash
proof check --daVinci .
# ✓ all 1 DaVinci invariants satisfied
```

If the diagram is later edited and the "Service Layer" label is removed, `proof check --daVinci` emits `fig_invariant_violated` before the change merges.

### Protection levels

| Level | Effect |
|-------|--------|
| `warn` | Violation reported as warning — compile and check continue |
| `error` | Violation reported as error — `proof compile` aborts |
| `lock` | Same as `error`; reserved for future `proof fix` hard-block |

### List all pins

```bash
proof pin-list
# 1 DaVinci entries:
#   service-layer [error] md://architecture/01-COMPONENTS.md#the-big-picture:0 — 2 invariants
```

---

## 8. Wire it into CI (30 seconds)

A minimal GitHub Actions step:

```yaml
- name: Build proof
  run: cargo install --git https://github.com/giodl73-repo/PROOF --branch master

- name: Lint docs
  run: proof check . -f github
```

`-f github` emits `::error file=...,line=...,col=...::message` lines — GitHub renders them as inline annotations on the PR.

For a non-failing advisory run (warnings without breaking the build):

```yaml
- name: Lint docs (advisory)
  run: proof check . --no-fail -f github
```

---

## 9. What you've learned

| You can now | Command |
|-------------|---------|
| Lint a directory | `proof check .` |
| Filter to errors | `proof check . --errors-only` |
| See machine-readable output | `proof check . -f json` |
| Get summary stats | `proof stats . --by-code` |
| Write a starter config | `proof init` |
| Inspect effective config | `proof config path/to/file.md` |
| Generate a draft plan | `proof draft . -o draft-plan.json` |
| Preview fixes | `proof fix --plan draft-plan.json --dry-run` |
| Apply fixes safely | `proof fix --plan draft-plan.json` |
| Apply more aggressive fixes | `proof fix --plan draft-plan.json --min-confidence medium` |
| Pin a figure with invariants | `proof pin "md://path/file.md#section:0" --id name` |
| Verify pinned figures | `proof check --daVinci .` |
| List pins | `proof pin-list` |
| Resolve a URI | `proof resolve "md://path/file.md#section:0"` |
| Compile source documents | `proof compile .` |
| Compose figures side-by-side | `proof layout fig1.md fig2.md --gap 4 --labels "A,B"` |

---

## 10. Where to go next

- **`docs/SCHEMA-REFERENCE.md`** — every `proof.toml` field documented with type, default, and what it catches.
- **`design/COMPILE-SPEC.md`** — `proof compile` pipeline, directive syntax, three-tier cache.
- **`design/LAYOUT-SPEC.md`** — `proof layout` algorithm and invariants L-1 through L-9.
- **`design/SCENARIOS.md`** — 8 hand-simulated scenarios with 31 spec findings; useful before extending the compiler.
- **`schemas/reference.toml`** — the real-world schema used by the MAXIM library (2,170 files); a useful starting point for your own.
- **`design/STYLE-GUIDE.md`** — style rules baked into the linter (S-01 wide-char policy, etc.).

When in doubt: `proof config <file>` shows what rules apply to that exact file. Trust the resolved config, not your memory of three layers of `proof.toml`.
