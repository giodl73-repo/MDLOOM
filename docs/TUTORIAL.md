# proof — Tutorial

Zero to first scan in five minutes. Assumes you know markdown and have a Rust toolchain.

---

## 1. Install

`proof` depends on `mdpath` (the `md://` URI resolver). Both must live as siblings under the same parent.

```bash
cd C:\src
git clone https://github.com/giodl73-repo/MDPATH mdpath
git clone https://github.com/giodl73-repo/PROOF proof

cd proof
cargo build --release
./target/release/proof --version    # proof 0.5.0
```

Put the binary on `PATH`:

```bash
# Windows
copy target\release\proof.exe %USERPROFILE%\bin\

# Linux/macOS
sudo cp target/release/proof /usr/local/bin/
```

---

## 2. First scan

Point `proof check` at any directory of markdown:

```bash
cd /path/to/your-docs
proof check .
```

You'll see something like:

```
docs/architecture/01-COMPONENTS.md:11:1: error [ascii_box_width]: bottom border 21 chars, top border 20
docs/reference/02-CLI.md:8:14: error [char_wide]: U+2014 EM DASH (2 cols) breaks alignment
docs/guides/01-GETTING-STARTED.md:34:1: warning [md_missing_section]: required H2 absent

FAIL — 47 files checked, 2 errors, 1 warning
```

No config required — `proof` runs sensible defaults out of the box.

---

## 3. Read the output

Each line is `file:line:col: severity [code]: message`.

| Field | Meaning |
|-------|---------|
| `file:line:col` | Exact location — most editors jump straight to it |
| `severity` | `error` (non-zero exit), `warning` (advisory), `info` |
| `[code]` | Short identifier — `ascii_box_width`, `char_wide`, `md_missing_section`, etc. |
| message | One-line description of the defect |

Common codes you'll see early:

| Code | What it catches |
|------|-----------------|
| `ascii_box_width` | Top and bottom borders of an ASCII box don't match |
| `ascii_box_col` | Internal column drifts across rows |
| `char_wide` | Em-dash, CJK glyph, or other wide char inside what should be monospace |
| `md_missing_section` | Required H2 absent from a guide |
| `link_broken_target` | Markdown link points at a file that doesn't exist |
| `table_missing_row` | GFM table missing a required row by key |

Useful flags:

```bash
proof check . --errors-only           # hide warnings (CI mode)
proof check . --deduplicate           # corpus scale: "42x SLIDE-001 in docs/slides/*.md"
proof check . --unused                # find figures no source file references
proof check . --by-code               # count per diagnostic code
proof check . -f json -o out.json     # machine-readable
proof check . -f github               # GitHub Actions annotations
```

Get a one-screen corpus health summary:

```bash
proof status .
```

```
proof status — C:\src\maxim

  Sources         2,703 files
  Compiled        2,703 files
  Stale               0 files
  Last compile    3 hours ago
  Config          proof.toml (root=true, 4 schemas, 2 compile targets)
```

---

## 4. Minimal `proof.toml`

Drop a `proof.toml` at the repo root to set rules and mark this as the cascade root:

```toml
# proof.toml
[files]
root = true
include = ["**/*.md"]
exclude = ["CHANGELOG.md", "_archive/**"]

[ascii_box]
enabled = true
tolerance = 0          # exact width match required

[ascii_char]
enabled = true         # catch wide chars

[markdown]
enabled = true
max_h1 = 1
required_h2_all = ["Decision Cheat Sheet"]
```

Re-run `proof check .` — every guide now must have a `## Decision Cheat Sheet`.

To inspect the effective config that applies to one file (after cascade):

```bash
proof config docs/architecture/01-COMPONENTS.md
```

Section-specific rules go in `[[section_schemas]]`:

```toml
[[section_schemas]]
paths = ["reference/**"]
paths_exclude = ["reference/00-OVERVIEW.md"]
required_h2_all = ["API Reference", "Type System Snapshot"]
```

Full field reference in `docs/SCHEMA-REFERENCE.md`.

---

## 5. Auto-fix what's safe

`proof` separates detection from repair. The fix pipeline produces a structured plan, then applies only the parts marked safe.

```bash
proof draft . -o plan.json                        # generate plan
proof fix --plan plan.json --min-confidence high  # apply safe fixes only
```

What you get:

- `--min-confidence high` — only deterministic fixes apply (e.g. trim a border to match width). Medium and low groups are skipped.
- **Signal-loss guard** — any fix that removes non-whitespace content is rejected unless you pass `--no-signal-check`.
- **Stale-anchor detection** — if `old_string` no longer matches the file, the fix is skipped and logged, not silently corrupted.
- **Auto re-check** — `proof check` runs again after applying. `0 errors remaining` means you're done.

To preview without writing:

```bash
proof fix --plan plan.json --dry-run
```

Lower-confidence groups in `plan.json` carry a rich context block (surrounding lines, expected vs. actual widths, diagnosis) so an AI can fill in `decision` and `edit.new_string`. Hand the file to Claude or Cursor with: *"Fill in `decision` and `edit` for every `needs_review` group."*

Then re-apply:

```bash
proof fix --plan plan.json --min-confidence medium
```

---

## 6. Compile a `.source.md` file

`proof compile` is the source-document pipeline. A `.source.md` file authors a document by referencing figures and math; `proof` resolves the directives and writes the rendered `.md`.

Create `docs/example.source.md`:

```markdown
# Example

The quadratic formula:

```proof:math
x = (-b ± sqrt(b^2 - 4ac)) / (2a)
```

That's it. The math directive renders as ASCII.
```

Compile:

```bash
proof compile docs/example.source.md
```

This writes `docs/example.md` next to the source. The math block expands to a centered ASCII fraction with a real square-root sign and superscript exponents.

Other directives you can drop in:

| Directive | What it does |
|-----------|--------------|
| `proof:math` | Render an equation to ASCII (fractions, integrals, matrices, sub/super) |
| `proof:include` | Embed a figure addressed by `md://` URI |
| `proof:include pin=id` | Embed + declare expected DaVinci invariant pin |
| `proof:layout` | Compose multiple figures side-by-side |
| `proof:tree` | Render a directory or hierarchy tree (dirtree, org, taxonomy, dependency, outline) |
| `proof:chart` | Bar or line chart from a markdown table source |
| `proof:element` | Fixed-width data cell (value, delta, sparkline, mini-bar, label, badge) |
| `proof:row` | Row of elements from a data table, one row per source record |
| `proof:toc` | Auto-generated table of contents with optional `section=` scoping |
| `proof:xref` | Cross-reference that resolves the target heading at compile time |
| `proof:blockquote` | Document-context block quote with left margin bar |
| `[sym:name]` | Inline Unicode symbol expansion (`[sym:checkmark]` → ✓) |
| `proof:symbol` | Symbol block at a given size |
| `proof:slide` | One slide in a `.slides.source.md` deck |

When a heading in a referenced file is renamed, `proof:xref` and `proof:toc` auto-update on recompile. `proof:include` with `pin=` warns you if the figure isn't locked.

Use `proof depends md://path#heading` to see every source file that references a URI before you rename it.

---

## 7. Watch mode + multi-target compile

For a docs site with several source directories, declare them all in `proof.toml`:

```toml
[[compile]]
source_dir = "src/guides"
output_dir = "docs/guides"

[[compile]]
source_dir = "src/presentations"
output_dir = "docs/presentations"
```

Then run a single watch loop:

```bash
proof compile --watch
```

Every `.source.md` save under either `source_dir` triggers a recompile to its paired `output_dir`. Edits to a referenced figure also retrigger compile of every dependent file (the cache tracks figure → consumer edges).

One-shot variants:

```bash
proof compile                          # use [[compile]] targets from proof.toml
proof compile docs/example.source.md   # one file, output next to source
proof compile src/ -o-dir build/       # one CLI run, override output dir
proof compile --check                  # verify outputs are up-to-date; exit non-zero if stale
```

---

## 8. Author a slide deck

Presentation files use the `.slides.source.md` extension. The front-matter sets canvas dimensions; `---` separators divide slides.

```markdown
---
slides:
  width: 80
  height: 20
  footer: true
  progress-bar: true
---

```proof:slide layout=title
title: "My Deck"
author: "Your Name"
date: "2026"
```
---
```proof:slide layout=section
title: "Part One"
```
---
```proof:slide layout=title-content
title: "Key Points"
---
proof:bullets
- First point always visible
[2] - Appears on step 2
[3] - Appears on step 3
```
---
```proof:slide layout=agenda
title: "Agenda"
```
```

Six layouts: `title` · `title-content` · `two-column` · `section` · `stats` · `blank` · `agenda`

Compile:

```bash
proof compile deck.slides.source.md
```

The output contains one canvas block per slide (plus one per reveal step for `[N]`-marked bullets). The `agenda` layout auto-generates a bullet list of all `layout=section` slide titles in the deck.

---

## What's next

| Want to | Read |
|---------|------|
| Every `proof.toml` field documented | `docs/SCHEMA-REFERENCE.md` |
| Author math, trees, charts, slides | `docs/guides/*.md` |
| Pin canonical figures with invariants | `design/COMPILE-SPEC.md` (DaVinci section) |
| Understand the compile pipeline | `design/COMPILE-SPEC.md` |
| Every proof.toml field documented | `docs/SCHEMA-REFERENCE.md` |
| Symbol library and shapes | `docs/guides/02-symbols.md` |
| Slide layouts and reveal | `docs/guides/04-slides.slides.md` |
| Dashboard canvas regions | `docs/guides/06-dashboard.md` |
| See real-world config | the `proof.toml` files in this repo, or the MAXIM library |

When in doubt: `proof config <file>` shows what rules apply. Trust the resolved config, not your memory of three layers of `proof.toml`.
