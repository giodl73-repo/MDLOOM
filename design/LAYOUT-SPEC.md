# proof layout — ASCII Art Collage Composer v0.2

**Status:** ✅ Implemented — `src/layout.rs`. `proof layout` CLI command and `proof:layout` compile directive both live. Side-by-side, stacked, and grid arrangements with configurable gap, alignment, direction, and labels.

---

## What it is

`proof layout` takes N figures (via `md://` URIs or inline content) and arranges
them side-by-side in a single fenced code block — like a wall of picture frames.
The output is a clean, aligned ASCII art composition that fits naturally in wide
markdown files, documentation, and presentations.

---

## Why this matters

ASCII art figures are written and maintained as individual diagrams. But documentation
and presentations need them composed together — a comparison of two architectures,
a row of language type-system snapshots, a progression of states. Without a layout
engine, authors copy-paste figures and manually align them, which:
- Creates duplicates that drift from the source
- Requires painstaking space counting to align rows
- Breaks when a source figure changes width

`proof layout` solves this by fetching figures by stable `md://` address and composing
them programmatically with correct alignment.

---

## CLI

```bash
# Compose 3 figures side-by-side, 4-space gap
proof layout \
    "md://languages/10-GO.md#concurrency-model:0" \
    "md://languages/09-RUST.md#ownership-model:0" \
    "md://languages/05-CSHARP.md#async-model:0" \
    --gap 4

# From file paths (no md:// needed)
proof layout fig1.md fig2.md fig3.md --gap 3

# With labels above each frame
proof layout \
    "md://fig1.md#:0" \
    "md://fig2.md#:0" \
    --gap 4 \
    --labels "Go Concurrency" "Rust Ownership"

# Output to file
proof layout fig1.md fig2.md --gap 3 -o layout.md

# In a presentation: 200-column wide layout, 3 columns
proof layout *.fig.md --gap 4 --cols 3 --width 200

# Vertical stacking (default is horizontal)
proof layout fig1.md fig2.md --direction vertical --gap 2
```

---

## The layout algorithm

### Inputs

- N source figures (each a list of content lines — fence delimiters stripped)
- `gap`: spaces between frames (default: 3)
- `align`: `top` | `center` | `bottom` (default: `top`)
- `labels`: optional text labels above each frame
- `width`: max output width in columns (default: 120)
- `cols`: number of columns per row (default: N, wraps if > cols)
- `direction`: `horizontal` | `vertical` (default: `horizontal`)
- `border`: bool (default: false)

### Step 1: Fetch figures

For each source (URI or file):
1. Resolve via mdpath → `ResolvedElement.content`
2. **Strip fence delimiters**: if the resolved content is a fenced code block, take only
   the lines between the opening and closing ` ``` `. The layout engine operates on
   figure content lines, never on raw fence delimiter lines. Wrapping the composed
   output in a new fence is the layout engine's job (step 6).
3. Split into lines
4. Measure visual width of each line (using unicode-width — handles box-drawing chars,
   CJK at 2 columns, box-drawing at 1 column per L-5)

### Step 2: Normalize frames

For each figure:
1. **Frame width** = max visual width across all content lines in that figure
2. **Pad lines** to frame width (right-pad with spaces so all lines are equal width)
3. **Frame height** = number of content lines
4. **Empty figure**: if a figure has 0 content lines, it produces a single blank line
   padded to a minimum frame width of 1 (L-6).

### Step 3: Equalize heights

All figures in a row must have the same number of lines (so rows align across frames):
- `max_height` = max(all frame heights in the row)
- Short frames are padded with blank lines (spaces × frame_width) according to `align`:
  - `top`: blank lines appended at bottom
  - `bottom`: blank lines prepended at top
  - `center`: `floor((max_height - height) / 2)` blank lines at top, remainder at bottom

Blank padding lines contain `frame_width` spaces — NOT zero-length. This is required
for correct visual alignment when frames are joined with the gap. However, to avoid
trailing-whitespace issues in the composed output, the final emit step (step 6) strips
trailing spaces from every output line.

### Step 4: Add labels

If `--labels` is provided, prepend one label line per frame before the content lines.
The label is centered over the frame width:

```
left_pad  = floor((frame_width - label_len) / 2)
right_pad = frame_width - label_len - left_pad
```

When the label is longer than the frame width, it is truncated to `frame_width`
characters. When centering produces a half-space (odd label width in even frame or
vice versa), the extra space goes on the **right** side.

### Step 5: Compose rows

For each row of frames (wrapping at `--cols`):
- For each line index 0..max_height:
  - Join `frames[0].lines[i]` + `" " * gap` + `frames[1].lines[i]` + ...
- Rows are separated by exactly **one blank line** in the output.
- The final emit step strips trailing whitespace from every output line (including
  blank lines that were all-spaces from height equalization).

### Step 6: Emit as fenced code block

Wrap the composition in a ` ``` ` fence:
- Standalone CLI: plain ` ``` ` fence
- Inside a source document after compile: the directive block is replaced with the
  composed content (no extra outer fence — the compiled output is inline content)

---

## Example

**Input:** three figures from Go, Rust, C# guides

**Command:**
```bash
proof layout \
    "md://languages/10-GO.md#type-system-snapshot:table:0" \
    "md://languages/09-RUST.md#type-system-snapshot:table:0" \
    "md://languages/05-CSHARP.md#type-system-snapshot:table:0" \
    --gap 4 \
    --labels "Go" "Rust" "C#"
```

**Output:**
````
```
       Go                        Rust                      C#
Axis         | Value        Axis         | Value       Axis         | Value
-------------|----------    -------------|----------   -------------|----------
Binding      | Late         Binding      | Compile     Binding      | Late
Typing       | Static       Typing       | Static      Typing       | Static
Strength     | Strong       Strength     | Strong      Strength     | Strong
Type system  | Structural   Type system  | Affine      Type system  | Nominal
```
````

---

## The `proof:layout` directive (compile mode)

When used inside a source document, the layout directive is a fenced block:

````markdown
```proof:layout gap=4 align=top labels="Go,Rust,C#"
md://languages/10-GO.md#type-system-snapshot:table:0
md://languages/09-RUST.md#type-system-snapshot:table:0
md://languages/05-CSHARP.md#type-system-snapshot:table:0
```
````

The compiler resolves each URI, applies the layout algorithm, and replaces the
directive block with the composed output inline (no wrapping code fence is added —
the composed content is already ready to embed).

### Directive attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `gap` | integer | 3 | Spaces between frames |
| `align` | top\|center\|bottom | top | Vertical alignment for unequal-height frames |
| `labels` | comma-separated string | (none) | Labels above each frame |
| `cols` | integer | N | Frames per row before wrapping |
| `width` | integer | 120 | Max output width in columns |
| `direction` | horizontal\|vertical | horizontal | Composition direction (CLI also accepts `h`/`v`) |
| `border` | bool | false | Add a thin border around each frame |

### Cache key for layout config

When a source document uses `proof:layout`, the compile cache key includes a
`layout_config_hash`. This hash is computed from the **normalized** attribute set —
all defaults are filled in before hashing. This guarantees that `gap=3` (explicit)
and `gap` (omitted, using the default of 3) produce the same hash.

`labels` is part of the layout config hash. Changing a label string ("Go" → "Golang")
misses the compile cache but hits the resolve cache — the layout is recomputed with
the new label, but figures are not re-resolved.

---

## Frame border option

With `--border`, each frame gets an explicit border box:

```
┌──────────────────────────────┐   ┌──────────────────────────────┐
│ GOROUTINE SCHEDULER          │   │ RUST OWNERSHIP MODEL         │
│ ┌─────────────────────────┐  │   │ Stack:  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓│  │
│ │  M:N Goroutines         │  │   │ Heap:   │░░░░░░░░░░░░░░░░░│  │
│ └─────────────────────────┘  │   │ Borrow: │▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒│  │
└──────────────────────────────┘   └──────────────────────────────┘
```

---

## Width management for presentations

For 200-column wide presentations, the layout engine can fill the available space
intelligently:

```bash
proof layout fig1.md fig2.md fig3.md fig4.md \
    --width 200 \
    --gap 4 \
    --cols 4      # all 4 side-by-side
```

If figures don't fit in `--width`, the layout wraps to multiple rows. Lone figures in
the last row are left-aligned (not stretched to fill the row width):

```bash
proof layout *.fig.md --width 200 --gap 3 --cols 3
# Row 1: fig1 fig2 fig3
# Row 2: fig4 fig5 fig6
# Row 3: fig7            ← left-aligned, not stretched
```

---

## Figure libraries

A figure library is a directory of standalone figure files, each containing one
or more named figures. Figures are addressed by `md://` and can be included in
any document:

```
figures/
  concurrency/
    goroutine-scheduler.md       ← md://figures/concurrency/goroutine-scheduler.md#:0
    rust-ownership.md            ← md://figures/concurrency/rust-ownership.md#:0
  type-systems/
    go-types.md
    rust-types.md
    csharp-types.md
```

**`proof figures .`** — list all figure files, their DaVinci status, and which
documents include them:

```
figures/concurrency/goroutine-scheduler.md#:0
  label: GOROUTINE SCHEDULER — M:N multiplexing
  kind:  figure.flowchart
  pinned: yes (goroutine-scheduler, protection=error)
  included by: languages/10-GO.source.md:34, presentations/go-deep.source.md:12

figures/type-systems/go-types.md#:0
  label: Go Type System
  kind:  table.key-value
  pinned: no
  included by: (none)
```

---

## Integration with proof compile

The layout engine is the core primitive that `proof compile` uses. When a
source document contains a `proof:layout` directive, the compile step:
1. Resolves each URI (with Tier 2 cache) → gets fence content (delimiters stripped)
2. Calls the layout engine with the resolved content lines
3. Replaces the `proof:layout` directive block with the composed output inline
4. Caches the result (Tier 3 cache key includes layout config hash)

Changes to any figure in a layout → Tier 2 cache miss → Tier 3 cache miss →
layout recomputed on next compile.

---

## Invariants

| Invariant | Claim |
|-----------|-------|
| L-1 | Output visual width ≤ input `--width` for all rows |
| L-2 | All frames in a row have equal height after alignment padding |
| L-3 | All content lines in each frame have equal visual width before emit |
| L-4 | Gap between frames is exactly `gap` spaces (measured in visual columns) |
| L-5 | Unicode box-drawing characters measured at 1 column (not 2) |
| L-6 | An empty figure (no content) renders as a 1-line frame of width ≥ 1 |
| L-7 | Label is centered over frame width; tie-break: extra space on right |
| L-8 | Row separator between `--cols` wrapping rows is exactly 1 blank line |
| L-9 | All output lines have trailing spaces stripped on emit |

---

## New roles

**COMPOSE** — layout and visual composition specialist.

Lens questions:
- Is the output visually correct for every combination of figure sizes?
- Does the frame padding correctly handle unicode box-drawing chars?
- Does the gap measurement use visual column width (not byte count)?
- Does vertical alignment (top/center/bottom) work for single-frame layouts?
- Does wrapping at `--cols` produce clean row separations?
- Are trailing spaces stripped from every output line (including blank pads)?

Pulls against: PARSE (composition speed vs. correctness of unicode handling).
