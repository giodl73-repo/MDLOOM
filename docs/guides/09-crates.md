# proof Crates — Standalone Libraries

proof ships two standalone library crates alongside the main CLI. Both live
in `crates/` inside the proof repo and are workspace members. They can be
used independently of the CLI — import them in any Rust project.

---

## proof-canvas — Fixed-width ASCII character grid

`proof-canvas` is a minimal 2D character grid for terminal and TUI composition.
Every cell is a `char`. Content is placed by position. The grid renders to a
fixed-width string with no color, no escape codes, no event handling — just
characters at exact column/row coordinates.

This is the layer proof's dashboard compositor is built on. It's useful anywhere
you need to compose ASCII content into a fixed-width frame.

### Add to your project

```toml
[dependencies]
proof-canvas = { git = "https://github.com/giodl73-repo/PROOF" }
```

### Core API

```rust
use proof_canvas::Canvas;

// Create a 80×24 canvas filled with spaces
let mut canvas = Canvas::new(80, 24);

// Paste content at exact positions
canvas.paste(0, 0, &["╔══════════════════════════════╗"]);
canvas.paste(0, 1, &["║  Status Board                ║"]);
canvas.paste(0, 2, &["╚══════════════════════════════╝"]);

// Side-by-side panels — no bleed between them
canvas.paste(0,  4, &["Panel A", "value: 42"]);
canvas.paste(40, 4, &["Panel B", "value: 99"]);

// Render to a newline-terminated string
let output = canvas.render();
print!("{}", output);
```

### Wide character handling

`paste()` correctly handles wide Unicode characters (CJK, emoji) — they occupy
two columns. The second column is filled with a space to prevent the next
character drifting. Box-drawing characters are always treated as 1 column
regardless of terminal font.

### Invariant D-6

`render()` guarantees every row is exactly `width` characters wide, including
trailing spaces. This makes the output predictable for downstream tooling.

---

## proof-math — LaTeX → terminal renderer

`proof-math` renders LaTeX math notation to Unicode symbols and ASCII art.
No LaTeX installation required — pure Rust, zero external dependencies beyond
`unicode-width`.

### Add to your project

```toml
[dependencies]
proof-math = { git = "https://github.com/giodl73-repo/PROOF" }
```

### Inline expansion

Expands `$...$` spans in a string — Greek letters, operators, super/subscripts:

```rust
use proof_math::expand_inline_math;

let (result, warnings) = expand_inline_math("The formula $E = mc^2$ is famous.");
assert_eq!(result, "The formula E = mc² is famous.");

let (result, _) = expand_inline_math("For $\\alpha + \\beta = \\gamma$:");
assert_eq!(result, "For α + β = γ:");
```

Warnings are typed `MathDiag` — check `d.code` for `"MATH-001"` through `"MATH-006"`.

### Display math

Renders multi-line constructs — fractions, integrals, matrices:

```rust
use proof_math::{render_display_math, MathAlign};

let (lines, warnings) = render_display_math(r"\frac{n(n+1)}{2}", 0, MathAlign::Left);
// lines: ["n(n+1)", "──────", "  2"]

let (lines, _) = render_display_math(
    r"\sum_{i=1}^{n} i",
    40,
    MathAlign::Center,
);
// lines: 3 lines, each exactly 40 columns wide
```

### Three render tiers

<!-- proof:compiled from="proof:tree kind=taxonomy" uri="" -->
```taxonomy
Render tiers
├── Tier 1: Unicode substitution (instant, no layout)
├── Greek: α β γ δ ε ζ η θ λ μ ν π ρ σ τ φ χ ψ ω
├── Operators: × ÷ ± · ≤ ≥ ≠ ≈ ≡ ∈ ⊂ ∪ ∩ ∀ ∃ ∧ ∨
├── Arrows: → ← ⇒ ⇐ ⟺ ⟹ ↦
├── Tier 2: Single-line ASCII art
├── Superscripts: x² xⁿ x^(n+1)
├── Subscripts: x₀ xᵢ x_(n+1)
├── Square roots: √x √(x+1) ³√x
├── Primes: f′ f″ f‴
├── Tier 3: Multi-line display (proof:math blocks only)
├── Stacked fractions
├── Integrals with limits
├── Sum and product with bounds
├── Matrices (pmatrix bmatrix vmatrix Vmatrix)
└── Cases environments
```
<!-- /proof:compiled -->

### Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `MATH-001` | warning | Unknown `\command` — passed through |
| `MATH-003` | error | Unmatched `\begin{env}` / `\end{env}` |
| `MATH-004` | warning | Display math exceeds declared width — clipped |
| `MATH-005` | warning | Tier 3 construct in inline context — simplified |
| `MATH-006` | warning | Unmatched braces |

---

## Using both crates together

proof's dashboard compiler uses both: `proof-math` renders math expressions
into strings, then `proof-canvas` places those strings at exact coordinates:

```rust
use proof_canvas::Canvas;
use proof_math::{render_display_math, MathAlign};

let mut canvas = Canvas::new(60, 10);

// Render a formula
let (lines, _) = render_display_math(r"\sum_{i=1}^{n} i = \frac{n(n+1)}{2}", 0, MathAlign::Left);

// Paste it into the canvas at row 2, col 5
let refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
canvas.paste(5, 2, &refs);

println!("{}", canvas.render());
```

---

## Workspace structure

Both crates are members of the proof Cargo workspace:

<!-- proof:compiled from="proof:tree kind=dirtree" uri="" -->
```dirtree
crates/
├── proof-canvas/
│   ├── src/
│   │   └── lib.rs
│   └── Cargo.toml
└── proof-math/
    ├── src/
    │   ├── fraction.rs
    │   ├── integral.rs
    │   ├── lib.rs
    │   ├── matrix.rs
    │   ├── render.rs
    │   ├── superscript.rs
    │   ├── symbols.rs
    │   ├── tier2.rs
    │   └── tokenizer.rs
    └── Cargo.toml
```
<!-- /proof:compiled -->

Clone the proof repo — both crates are immediately available. No separate
repository or separate install step.
