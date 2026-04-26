# proof layout — ASCII Art Collage Composer v0.1

**Status:** Design — implementation in progress.

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

- N source figures (each a list of content lines)
- `gap`: spaces between frames (default: 3)
- `align`: `top` | `center` | `bottom` (default: `top`)
- `labels`: optional text labels above each frame
- `width`: max output width in columns (default: 120)
- `cols`: number of columns per row (default: N, wraps if > cols)

### Step 1: Fetch figures

For each source (URI or file):
1. Resolve via mdpath → `ResolvedElement.content`
2. Split into lines
3. Measure visual width of each line (using unicode-width — handles box-drawing chars)

### Step 2: Normalize frames

For each figure:
1. **Frame width** = max visual width across all lines in that figure
2. **Pad lines** to frame width (right-pad with spaces so all lines are equal width)
3. **Frame height** = number of lines

### Step 3: Equalize heights

All figures in a row must have the same number of lines (so their frames align):
- `max_height` = max(all frame heights in the row)
- Short frames are padded with blank lines according to `align`:
  - `top`: blank lines appended at bottom
  - `bottom`: blank lines prepended at top
  - `center`: blank lines split top and bottom

### Step 4: Add labels

If `--labels` is provided, prepend one line per frame with the label text,
centered over the frame width.

### Step 5: Compose rows

For each row of frames (wrapping at `--cols`):
- For each line number 0..max_height:
  - Join `frames[0].lines[i]` + `" " * gap` + `frames[1].lines[i]` + ... 
- Collect all rows, separated by a blank line

### Step 6: Emit as fenced code block

Wrap the composition in a ` ``` ` fence with optional `proof:layout` info string
(for compile-mode) or plain fence (for standalone use).

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
directive block with the composed output.

### Directive attributes

| Attribute | Type | Default | Description |
|-----------|------|---------|-------------|
| `gap` | integer | 3 | Spaces between frames |
| `align` | top\|center\|bottom | top | Vertical alignment for unequal-height frames |
| `labels` | comma-separated | (none) | Labels above each frame |
| `cols` | integer | N | Frames per row before wrapping |
| `width` | integer | 120 | Max output width in columns |
| `direction` | h\|v | h | Horizontal or vertical composition |
| `border` | bool | false | Add a thin border around each frame |

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

If figures don't fit in `--width`, the layout wraps to multiple rows:

```bash
proof layout *.fig.md --width 200 --gap 3 --cols 3
# Row 1: fig1 fig2 fig3
# Row 2: fig4 fig5 fig6  
# Row 3: fig7 (alone)
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
1. Resolves each URI (with Tier 2 cache)
2. Calls the layout engine with the resolved content
3. Embeds the composed output
4. Caches the result (Tier 3 cache key includes layout config hash)

Changes to any figure in a layout → Tier 2 cache miss → Tier 3 cache miss →
layout recomputed on next compile.

---

## Invariants

| Invariant | Claim |
|-----------|-------|
| L-1 | Output visual width ≤ input `--width` for all rows |
| L-2 | All frames in a row have equal height after alignment padding |
| L-3 | All lines in each frame have equal visual width |
| L-4 | Gap between frames is exactly `gap` spaces (measured in visual columns) |
| L-5 | Unicode box-drawing characters measured at 1 column (not 2) |
| L-6 | An empty figure (no content) renders as a single blank line frame |
| L-7 | Label, when provided, is centered over the frame width |

---

## New roles

**COMPOSE** — layout and visual composition specialist.

Lens questions:
- Is the output visually correct for every combination of figure sizes?
- Does the frame padding correctly handle unicode box-drawing chars?
- Does the gap measurement use visual column width (not byte count)?
- Does vertical alignment (top/center/bottom) work for single-frame layouts?
- Does wrapping at `--cols` produce clean row separations?

Pulls against: PARSE (composition speed vs. correctness of unicode handling).
