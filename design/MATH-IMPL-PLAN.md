# MATH-IMPL-PLAN — LaTeX Math Rendering for proof

> **Spec**: `design/MATH-SPEC.md`
> **Status**: Ready to implement

---

## Architecture

```
src/math/
├── mod.rs          — public API: expand_inline_math, render_display_math, MathDiag
├── symbols.rs      — LATEX_SYMBOLS table (Tier 1), lookup_symbol
├── tokenizer.rs    — Token enum, tokenize()
├── superscript.rs  — ^{...} and _{...} → Unicode super/subscripts
├── tier2.rs        — \sqrt, inline \frac, prime, \lim/\max/\min
├── fraction.rs     — RenderedExpr, render_frac (stacked)
├── integral.rs     — render_int, render_sum_prod
├── matrix.rs       — render_matrix, MatrixKind, cases
└── render.rs       — render_display_math entry point, width/align padding
```

### Import DAG (no cycles)

```
tokenizer ← (no imports from math/)
symbols   ← (no imports from math/)
superscript ← tokenizer, symbols
tier2     ← tokenizer, symbols, superscript
fraction  ← tokenizer, tier2        [defines RenderedExpr]
integral  ← tokenizer, tier2, fraction
matrix    ← tokenizer, tier2, fraction
render    ← fraction, integral, matrix, tier2
mod       ← render, tier2, superscript, symbols  [public API]
```

Wired into compile pipeline in `src/compile.rs`:
- `proof:math` fenced block → `render_display_math(body, opts)`
- Inline `$...$` in `render_body_lines()`, slide text paths, bullet labels

---

## Wave 1 — Tokenizer + Symbol table + super/subscripts

**Scope**: ~300 LOC  
**Files**: `src/math/mod.rs`, `src/math/symbols.rs`, `src/math/tokenizer.rs`, `src/math/superscript.rs`

### Tokenizer (`tokenizer.rs`)

```rust
pub enum Token {
    Command(String),       // \alpha, \frac, \begin, ...
    Char(char),            // literal character
    Group(Vec<Token>),     // {content}
    Superscript,           // ^
    Subscript,             // _
    BeginEnv(String),      // \begin{pmatrix}
    EndEnv(String),        // \end{pmatrix}
    Dollar,                // literal $
}

pub fn tokenize(src: &str) -> (Vec<Token>, Vec<MathDiag>);
```

Rules:
- `\command` = backslash + one or more ASCII letters (or a single non-letter for `\,`, `\!`, etc.)
- `{...}` groups are recursively parsed into `Token::Group`
- Unmatched `{` or `}` emits MATH-006
- `\begin{env}` and `\end{env}` emit `BeginEnv`/`EndEnv` with the environment name
- All other characters emit `Token::Char`

### Symbol table (`symbols.rs`)

```rust
pub static LATEX_SYMBOLS: &[(&str, &str)] = &[
    ("alpha", "α"), ("beta", "β"), ...
    // All 60+ mappings from MATH-SPEC.md Tier 1 table
    // Font commands: ("mathbf", ""), ("mathrm", ""), ... → strip silently
    // \text → strip command, pass through content
    // \left, \right → MATH-001 (include in UNSUPPORTED list)
];

pub fn lookup_symbol(cmd: &str) -> Option<&'static str>;
pub static UNSUPPORTED: &[&str] = &["left", "right", "color", "includegraphics", ...];
```

### Superscript/subscript renderer (`superscript.rs`)

```rust
const SUPERSCRIPT_CHARS: &[(char, char)] = &[
    ('0', '⁰'), ('1', '¹'), ..., ('+', '⁺'), ('-', '⁻'), ('n', 'ⁿ'),
];
const SUBSCRIPT_CHARS: &[(char, char)] = &[
    ('0', '₀'), ..., ('a', 'ₐ'), ('e', 'ₑ'), ('i', 'ᵢ'),
];

// Try to map every char in s to a Unicode super/subscript.
// If all chars map: return the mapped string.
// If any char fails: return "^(s)" or "_(s)" bracket notation.
pub fn to_superscript(s: &str) -> String;
pub fn to_subscript(s: &str) -> String;
```

The `^` and `_` operators consume exactly 1 token (char or Group). `x^ab` → `x^a` then `b` as separate tokens. `{...}` braces always produce a single `Token::Group`, so `x^{ab}` is `x` + Superscript + Group(["a","b"]) — 1 token consumed.

Nested environments are **not** supported in Wave 3. A `\begin{pmatrix}` inside another environment emits MATH-001 and is treated as an unknown command.

### Public API (`mod.rs`)

```rust
pub struct MathDiag {
    pub code: &'static str,   // "MATH-001", "MATH-005", etc.
    pub severity: Severity,
    pub col: usize,
    pub message: String,
}

/// Expand all $...$ spans in a single line of prose.
/// Returns (expanded_line, diagnostics).
pub fn expand_inline_math(line: &str) -> (String, Vec<MathDiag>);
```

Tokenization order:
1. Scan for `` `...` `` spans on this line; mark them as opaque regions.
2. Within non-opaque regions, find balanced `$...$` pairs.
3. For each span: tokenize, expand (Tier 1 + Tier 2 only). Tier 3 → downgrade + MATH-005.
4. Unmatched `$` passes through unchanged.

### Tests (Wave 1) — 35+ tests

L0 unit tests in `src/math/`:
- Every Tier 1 symbol renders correctly (sample 20 from table)
- `expand_inline_math("$\\alpha + \\beta = \\gamma$")` → `"α + β = γ"`
- Superscript: `x^2` → `x²`, `x^n` → `xⁿ`, `x^{ab}` → `x^(ab)`
- Subscript: `x_0` → `x₀`, `x_i` → `xᵢ`, `x_{n+1}` → `x_(n+1)`
- Unknown command passthrough + MATH-001
- Unmatched `$` passes through
- `$` inside backtick span not expanded
- MATH-005 triggered for `$\frac{a}{b}$` in inline context

---

## Wave 2 — Tier 2 ASCII constructs

**Scope**: ~250 LOC  
**Files**: `src/math/tier2.rs`

### Single-line constructs

```rust
pub fn render_sqrt(arg_tokens: &[Token]) -> String;
// \sqrt{x} → "√x", \sqrt{x+1} → "√(x+1)", \sqrt[3]{x} → "³√x"

pub fn render_frac_inline(num: &[Token], den: &[Token]) -> String;
// \frac{a}{b} in inline context → "a/b"

pub fn render_prime(count: u32) -> &'static str;
// 1 → "′", 2 → "″", 3 → "‴"

pub fn render_lim_op(cmd: &str, sub: Option<&[Token]>, sup: Option<&[Token]>) -> String;
// \lim_{x \to 0} → "lim_(x→0)"
// \max_{x \in S} → "max_(x∈S)"
```

### Display math entry point

```rust
/// Entry point for block math rendering.
/// Returns lines of the rendered expression.
pub fn render_display_math(expr: &str, width: usize, align: Align) -> (Vec<String>, Vec<MathDiag>);
```

For Wave 2 this delegates to inline rendering (no multi-line constructs yet), padded to `width`. Width=0 means auto: use max line width of rendered output, capped at 200 columns.

### Tests (Wave 2) — 20+ tests

- `\sqrt{x}` → `"√x"`
- `\sqrt{x+1}` → `"√(x+1)"`
- `\sqrt[3]{x}` → `"³√x"`
- Inline fraction `\frac{a}{b}` → `"a/b"` (in inline context)
- Prime: `f'` → `f′`, `f''` → `f″`
- `render_lim_op("lim", Some(&to_subscript), None)` → `lim_(x→0)`
- `display_math` with `width=40 align=center` pads correctly
- MATH-004 when auto-width exceeds declared width

---

## Wave 3 — Tier 3 multi-line constructs

**Scope**: ~400 LOC  
**Files**: `src/math/fraction.rs`, `src/math/integral.rs`, `src/math/matrix.rs`

### RenderedExpr — shared type

```rust
pub struct RenderedExpr {
    pub lines: Vec<String>,
    pub width: usize,      // max visual_width of any single line
    pub baseline: usize,   // index of the primary alignment line
}
```

Baseline semantics by construct:
- Leaf (single char/symbol): `baseline = 0`
- Fraction: `baseline = index of bar line` (= `num_lines.len()`)
- Integral: `baseline = index of ⌡ line` (= `1` if upper limit present, `0` otherwise)
- Sum/product: `baseline = index of ∑/∏ line` (middle line)
- Matrix: `baseline = lines.len() / 2` (middle row)

### Stacked fraction (`fraction.rs`)

```rust
pub fn render_frac(num: RenderedExpr, den: RenderedExpr) -> RenderedExpr;
```

Rules:
- `bar_width = max(num.width, den.width)`
- Numerator centered: left_pad = `(bar_width - num.width) / 2`, right_pad fills remaining
- Same for denominator
- Bar line: `"─".repeat(bar_width)`
- Result baseline = num.lines.len() (the bar line)
- Nested fractions: `render_frac(render_frac(a, b), c)` — inner RenderedExpr.width feeds outer

### Integral/sum/product (`integral.rs`)

```rust
pub fn render_int(
    lower: Option<RenderedExpr>,
    upper: Option<RenderedExpr>,
    integrand: RenderedExpr,
) -> RenderedExpr;

pub fn render_sum_prod(
    op: char,   // '∑' or '∏'
    lower: Option<RenderedExpr>,
    upper: Option<RenderedExpr>,
    body: RenderedExpr,
) -> RenderedExpr;
```

Integral layout:
- Lines: [upper_limit_line, "⌠", "⌡ " + integrand_line, lower_limit_line]
- If no upper: skip that line. If no lower: skip.
- All lines left-aligned to column 0.

Sum/product layout:
- Lines: [upper_line_centered_over_op, op_line + " " + body, lower_line_centered_under_op]
- Width = max(visual_width(upper), visual_width(op), visual_width(lower)) + 1 + body.width

### Matrix/cases (`matrix.rs`)

```rust
pub fn render_matrix(
    kind: MatrixKind,   // Pmatrix, Bmatrix, Matrix, Vmatrix, Vmatrix2, Cases
    rows: Vec<Vec<RenderedExpr>>,
) -> RenderedExpr;
```

Column width: max visual_width of all cells in that column. Padding: right-align numeric, left-align text. Ragged rows: pad with empty cells (M-7). Emit MATH-001 if any row exceeds the first row's column count.

Delimiter glyphs:
- `pmatrix`: top `⎛`, mid `⎜`, bot `⎝` | top `⎞`, mid `⎟`, bot `⎠`
- `bmatrix`: top `⎡`, mid `⎢`, bot `⎣` | top `⎤`, mid `⎥`, bot `⎦`
- `vmatrix`: single `|` on each side
- `Vmatrix`: `‖` on each side
- `matrix`: no delimiters
- `cases`: left brace `⎧`/`⎨`/`⎩` only; no right delimiter

For 2-row matrices: use top/bot glyphs only (no mid). For 3+ rows: top + (n-2)×mid + bot.

Environment parsing:
- `\begin{env}` ... `\end{env}` where `env` ∈ {pmatrix, bmatrix, matrix, vmatrix, Vmatrix, cases}
- Unknown env → MATH-001
- Missing `\end{}` → MATH-003
- Mismatched `\begin{a}...\end{b}` → MATH-003

### Tests (Wave 3) — 25+ tests

All L0 tests live inline in `src/math/fraction.rs`, `integral.rs`, `matrix.rs`.

- `render_frac`: 1-char/1-char, 6-char numerator over 2-char denominator (centering), 2-char over 6-char
- `render_frac` baseline: assert `result.baseline == num_lines.len()`
- Nested fraction `\frac{\frac{a}{b}}{c}`: assert inner width == 1 (bar of "─"), outer bar spans that
- Odd-width centering: 3-char over 4-char → right padding is 1 more than left
- `render_int` with upper+lower, upper-only, lower-only, no limits
- `render_int` baseline: assert baseline == 1 (⌡ line) when upper limit present
- `render_sum_prod '∑'` with both limits, one limit only
- `render_matrix` pmatrix 2×2 and 3×3: check delimiters and column widths
- Ragged matrix: row 1 has 2 cols, row 2 has 1 col → row 2 padded to 2 cols
- `cases` 2-row and 3-row: check ⎧/⎨/⎩ glyphs
- MATH-003 on mismatched begin/end: `\begin{pmatrix}...\end{bmatrix}`
- MATH-003 on unclosed environment

---

## Wave 4 — proof:math directive + inline expansion wiring

**Scope**: ~200 LOC  
**Files**: `src/compile.rs` (new match arm), `src/slide/layout.rs` (inline expansion), `src/math/mod.rs` (finalization)

### `proof:math` compile directive

In `compile.rs`, add match arm for `"proof:math"` info string:

```rust
"proof:math" => {
    let attrs = parse_directive_attrs(info);
    let width = attrs.get("width").map(|s| s.parse().ok()).flatten().unwrap_or(0);
    let align = attrs.get("align").map(Align::from_str).unwrap_or(Align::Center);
    let no_chrome = attrs.get("no-chrome").map(|s| s == "true").unwrap_or(false);
    let (lines, diags) = render_display_math(body.trim(), width, align);
    for d in &diags { emit_diag(d, source_line); }
    if no_chrome {
        out.extend(lines);
    } else {
        out.push(format!("<!-- proof:math -->"));
        out.extend(lines);
        out.push(format!("<!-- /proof:math -->"));
    }
}
```

Width=0 means auto: use maximum line width of rendered output.

### Inline expansion wiring

In `render_body_lines()` (slide/layout.rs), after symbol expansion:

```rust
let line = expand_symbols(&line);
let (line, math_diags) = expand_inline_math(&line);
diags.extend(math_diags.into_iter().map(Into::into));
```

Same in slide title/subtitle path (already a string — just call `expand_inline_math`).

Bullet labels: `render_bullets()` calls `expand_inline_math()` on each label string before rendering.

### Tests (Wave 4) — 15+ L1 tests

L1 integration tests in `tests/math_integration.rs` (new file, separate from `integration_tests.rs`):
- `proof:math` block with `\frac{n(n+1)}{2}` → 3-line stacked output
- `proof:math` block with `\sum_{i=1}^{n}` → lines with upper/lower bounds
- `proof:math` with `width=40 align=center` → each output line is exactly 40 visual columns
- `proof:math` with `no-chrome=true` → no comment wrapper lines
- Inline `$\alpha + \beta$` in slide body → `α + β`
- Inline `$x^2 + y^2 = z^2$` → `x² + y² = z²`
- MATH-005 on inline `$\frac{a}{b}$` → downgraded to `a/b`, warning emitted
- Inline math not expanded inside `` `code span` ``
- Symbol and math both expand in same line: `[sym:checkmark] $\alpha$` → `✓ α`
- Unmatched `$` passes through unchanged
- `proof:math` block with ragged matrix → padded output, no panic
- MATH-001 on unknown command in block math

---

## File map

| File | Wave | LOC est. | Role |
|------|------|----------|------|
| `src/math/mod.rs` | 1 | 80 | Public API, `expand_inline_math`, `MathDiag` |
| `src/math/symbols.rs` | 1 | 100 | `LATEX_SYMBOLS` table, `lookup_symbol` |
| `src/math/tokenizer.rs` | 1 | 120 | `Token` enum, `tokenize()` |
| `src/math/superscript.rs` | 1 | 80 | Unicode super/subscript mapping |
| `src/math/tier2.rs` | 2 | 150 | `\sqrt`, inline `\frac`, primes, lim/max/min |
| `src/math/render.rs` | 2 | 100 | `render_display_math` entry point, width/align |
| `src/math/fraction.rs` | 3 | 120 | Stacked fraction, `RenderedExpr` |
| `src/math/integral.rs` | 3 | 140 | Integral/sum/product with bounds |
| `src/math/matrix.rs` | 3 | 180 | Matrix/cases environments |
| `src/compile.rs` | 4 | +40 | `proof:math` directive match arm |
| `src/slide/layout.rs` | 4 | +20 | Inline expansion in `render_body_lines` |

**Total**: ~1,130 LOC new code + ~60 LOC modifications

---

## Invariant coverage by wave

| Invariant | Wave |
|-----------|------|
| M-1 (display width) | Wave 2 (`render.rs` padding/clipping) |
| M-2 (inline = single line) | Wave 1 (`expand_inline_math`) |
| M-3 (frac bar width) | Wave 3 (`fraction.rs`) |
| M-4 (limit placement) | Wave 3 (`integral.rs`) |
| M-5 (matrix column widths) | Wave 3 (`matrix.rs`) |
| M-6 (MATH-001 passthrough) | Wave 1 (`symbols.rs`) |
| M-7 (ragged matrix padding) | Wave 3 (`matrix.rs`) |

---

## Test count targets

| Wave | L0 unit | L1 integration |
|------|---------|----------------|
| Wave 1 | 35 | 0 |
| Wave 2 | 20 | 0 |
| Wave 3 | 25 | 0 |
| Wave 4 | 0 | 15 |
| **Total** | **80** | **15** |

---

## Exit criteria

- `cargo test` passes with all 95 new tests green
- `$\alpha + \beta = \gamma$` in a slide body renders as `α + β = γ`
- `proof:math` block with `\frac{n(n+1)}{2}` renders the 3-line stacked form
- `proof:math` block with `\sum_{i=1}^{n}` renders with upper/lower bounds
- Inline `$\frac{a}{b}$` triggers MATH-005 and renders as `a/b`
- Unknown `\command` triggers MATH-001 and passes through
