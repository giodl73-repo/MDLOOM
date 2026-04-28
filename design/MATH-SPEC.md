# proof math — LaTeX Math Rendering for ASCII/Terminal Output

> **Status**: ✅ Implemented — `src/math/`. Inline `$...$` expansion and `proof:math` block directive live. 60+ symbols, superscripts, subscripts, fractions, integrals, matrices, cases. `proof-math` standalone crate extracted.

---

## What it is

`proof math` renders LaTeX math notation to Unicode symbols and ASCII art.
It uses the LaTeX formula syntax (`$...$` for inline, `$$...$$` for display)
because LaTeX math is the universal language for mathematical notation and
authors already know it.

The renderer does not require LaTeX to be installed — it is a pure Rust
LaTeX-subset parser that maps constructs directly to Unicode and ASCII art.

---

## Syntax

### Inline math

LaTeX inline math embedded in prose using `$...$`:

```markdown
The formula $E = mc^2$ changed physics.
For all $\epsilon > 0$, there exists $\delta > 0$ such that...
The derivative $df/dx = f'(x)$ at point $x_0$.
```

Rendered:
```
The formula E = mc² changed physics.
For all ε > 0, there exists δ > 0 such that...
The derivative df/dx = f′(x) at point x₀.
```

#### Tokenization order for `$` detection

1. Identify all `` `...` `` inline code spans on the line and mark them as non-expandable regions.
2. Within non-code regions, scan for balanced `$...$` pairs (not escaped, not inside URLs).
3. Any unmatched `$` (no closing `$` on the same line) is passed through unchanged.
4. Nested `$` inside a math span is not allowed — the first `$` after an opening `$` closes the span.

### Display (block) math

Display math requires the `proof:math` directive — a fenced code block:

> **Note**: The `$$...$$` syntax is reserved for future use. Do not use `$$` in
> prose — it will conflict with inline `$` handling. Use `` `proof:math` ``
> blocks for display math.

````markdown
```proof:math
\frac{d}{dx} e^x = e^x
```
````

````markdown
```proof:math
\int_0^{\infty} e^{-x}\, dx = 1
```
````

````markdown
```proof:math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```
````

---

## Three render tiers

### Terminal assumptions

All math rendering assumes:
- Monospace terminal with UTF-8 support
- East Asian Width (EAW) rules: Halfwidth/Neutral/Ambiguous = 1 column; Wide/Fullwidth = 2 columns; combining marks = 0 columns
- `visual_width(s)` = sum of EAW column widths for each character in `s`
- For multi-line expressions (e.g., a stacked fraction), `visual_width` = maximum visual width of any single line in that expression

### Tier 1 — Direct Unicode substitution

Single LaTeX commands map to single Unicode characters. Rendered inline,
no layout change.

| LaTeX | Unicode | Description |
|-------|---------|-------------|
| `\alpha` | α | alpha |
| `\beta` | β | beta |
| `\gamma` | γ | gamma |
| `\delta` | δ | delta |
| `\epsilon` | ε | epsilon |
| `\zeta` | ζ | zeta |
| `\eta` | η | eta |
| `\theta` | θ | theta |
| `\lambda` | λ | lambda |
| `\mu` | μ | mu |
| `\nu` | ν | nu |
| `\pi` | π | pi |
| `\rho` | ρ | rho |
| `\sigma` | σ | sigma |
| `\tau` | τ | tau |
| `\phi` | φ | phi |
| `\chi` | χ | chi |
| `\psi` | ψ | psi |
| `\omega` | ω | omega |
| `\Gamma` | Γ | capital gamma |
| `\Delta` | Δ | capital delta |
| `\Theta` | Θ | capital theta |
| `\Lambda` | Λ | capital lambda |
| `\Pi` | Π | capital pi |
| `\Sigma` | Σ | capital sigma |
| `\Phi` | Φ | capital phi |
| `\Psi` | Ψ | capital psi |
| `\Omega` | Ω | capital omega |
| `\infty` | ∞ | infinity |
| `\partial` | ∂ | partial derivative |
| `\nabla` | ∇ | nabla/del |
| `\sum` | ∑ | summation |
| `\prod` | ∏ | product |
| `\int` | ∫ | integral |
| `\oint` | ∮ | contour integral |
| `\times` | × | multiplication |
| `\div` | ÷ | division |
| `\pm` | ± | plus-minus |
| `\mp` | ∓ | minus-plus |
| `\cdot` | · | dot product |
| `\circ` | ∘ | composition |
| `\leq` | ≤ | less or equal |
| `\geq` | ≥ | greater or equal |
| `\neq` | ≠ | not equal |
| `\approx` | ≈ | approximately |
| `\equiv` | ≡ | equivalent |
| `\sim` | ∼ | similar |
| `\propto` | ∝ | proportional |
| `\in` | ∈ | element of |
| `\notin` | ∉ | not element of |
| `\subset` | ⊂ | subset |
| `\subseteq` | ⊆ | subset or equal |
| `\cup` | ∪ | union |
| `\cap` | ∩ | intersection |
| `\emptyset` | ∅ | empty set |
| `\forall` | ∀ | for all |
| `\exists` | ∃ | there exists |
| `\neg` | ¬ | negation |
| `\land` | ∧ | logical and |
| `\lor` | ∨ | logical or |
| `\to` | → | maps to / implies |
| `\leftarrow` | ← | left arrow |
| `\Rightarrow` | ⇒ | implies |
| `\Leftarrow` | ⇐ | implied by |
| `\Leftrightarrow` | ⟺ | if and only if |
| `\iff` | ⟺ | if and only if |
| `\implies` | ⟹ | implies |
| `\therefore` | ∴ | therefore |
| `\because` | ∵ | because |
| `\prime` | ′ | prime |
| `\degree` | ° | degree |
| `\langle` | ⟨ | left angle bracket |
| `\rangle` | ⟩ | right angle bracket |
| `\vert` | \| | vertical bar |
| `\Vert` | ‖ | double vertical bar |

### Tier 2 — Simple ASCII art (single-line constructs)

Constructs that fit on one line but require more than a single Unicode char.

#### Super/subscripts

The `^` and `_` operators consume exactly one token: a single character, or a `{...}` group.
`x^ab` parses as `(x^a)b` — two tokens, not one compound superscript. To group multiple
characters, use braces: `x^{ab}`.

Superscripts map to Unicode combining chars or raised notation:

| Input | Rendered | Notes |
|-------|----------|-------|
| `x^2` | `x²` | Unicode superscript digit |
| `x^3` | `x³` | Unicode superscript digit |
| `x^n` | `xⁿ` | Unicode superscript letter |
| `x^{n+1}` | `x^(n+1)` | Grouped: bracket notation |
| `x^{ab}` | `x^(ab)` | Grouped: bracket notation |
| `x_i` | `xᵢ` | Unicode subscript letter |
| `x_0` | `x₀` | Unicode subscript digit |
| `x_{n+1}` | `x_(n+1)` | Grouped: bracket notation |

Unicode superscripts available: ⁰¹²³⁴⁵⁶⁷⁸⁹ ⁺⁻⁼⁽⁾ ⁿ
Unicode subscripts available: ₀₁₂₃₄₅₆₇₈₉ ₊₋₌₍₎ ₐₑᵢₒᵤₓ

#### Square root

| Input | Rendered |
|-------|----------|
| `\sqrt{x}` | `√x` |
| `\sqrt{x+1}` | `√(x+1)` |
| `\sqrt[3]{x}` | `³√x` |

#### Absolute value / norm

| Input | Rendered |
|-------|----------|
| `|x|` | `|x|` (pass-through) |
| `\|x\|` | `‖x‖` |
| `\lvert x \rvert` | `|x|` |

#### Prime notation

| Input | Rendered |
|-------|----------|
| `f'` | `f′` |
| `f''` | `f″` |
| `f^{\prime}` | `f′` |

### Tier 3 — Multi-line ASCII art (display constructs)

These require multiple lines and are only valid in display (block) math.

#### Fraction `\frac{a}{b}`

```
  a
─────
  b
```

Rules:
- Bar width = `max(visual_width(numerator), visual_width(denominator))`
- Numerator is left-padded by `(bar_width - visual_width(numerator)) // 2` spaces; remaining right-padded. Odd-width asymmetries favor right-padding.
- Denominator same centering rule.
- Nested fractions: render inner fraction first; its `visual_width` = width of its bar line. Outer bar spans full inner width.

Example: `\frac{d}{dx}`
```
d
──
dx
```

Example: `\frac{n(n+1)}{2}`
```
n(n+1)
──────
  2
```

Example: `\frac{\frac{a}{b}}{c}` (nested fraction)
```
a
─
b
─────
  c
```

#### Integral with limits `\int_a^b`

```
b
⌠
⌡ f(x) dx
a
```

Layout:
- Upper limit: rendered 1 line above `⌠`, left-aligned at the column of `⌠`
- Lower limit: rendered 1 line below `⌡`, left-aligned at the column of `⌡`
- Integrand: follows `⌡` on the middle line with 1 space padding
- Both limits are optional: `\int f(x) dx` renders without limit lines
- Limit order in source is flexible: `\int_0^n` and `\int^n_0` are equivalent
- Limits are single-line expressions; if they exceed `width`, emit MATH-004

Example: `\int_0^{\infty} e^{-x} dx`
```
∞
⌠
⌡ e^(-x) dx
0
```

Example: `\int f(x) dx` (no limits)
```
⌠
⌡ f(x) dx
```

#### Sum with limits `\sum_{i=0}^{n}`

```
  n
  ∑  f(i)
 i=0
```

Layout:
- Upper limit: 1 line above `∑`, centered over `∑`
- Lower limit: 1 line below `∑`, centered under `∑`
- Summand: follows `∑` on the middle line with 1 space padding
- Both limits are optional; if only one is present it goes below (for `\sum`/`\prod`) or renders inline (for `\int`)
- If an upper/lower limit is itself multi-line, its bounding box is preserved and spacing adjusts

Example: `\sum_{i=1}^{n} i = \frac{n(n+1)}{2}`
```
 n          n(n+1)
 ∑  i  =  ──────
i=1           2
```

#### Product `\prod_{i=1}^{n}`

Same structure as sum but with ∏.

#### Limit operators `\lim`, `\max`, `\min`

These render as single-line Tier 2 constructs with subscript in bracket notation:

| Input | Rendered |
|-------|----------|
| `\lim_{x \to 0}` | `lim_(x→0)` |
| `\max_{x \in S}` | `max_(x∈S)` |
| `\min_{i=1}^n` | `min_(i=1)^n` |

They do not receive multi-line treatment even in display math blocks.

#### Matrix `\begin{pmatrix}...\end{pmatrix}`

```
⎛ a  b ⎞
⎜      ⎟
⎝ c  d ⎠
```

Supported environments:
- `pmatrix` — round parens `⎛⎜⎝ ⎞⎟⎠`
- `bmatrix` — square brackets `⎡⎢⎣ ⎤⎥⎦`
- `matrix` — no delimiters
- `vmatrix` — single vertical bars `|`
- `Vmatrix` — double vertical bars `‖`

Elements separated by `&`, rows by `\\`. The trailing `\\` on the last row is optional.

Column alignment: numeric cells (digits and operators only) are right-aligned; text cells are left-aligned. All cells in the same column use the same justification rule (determined by majority).

Column width: `visual_width` of the widest cell in that column. All cells in a column padded to that width. Minimum 1 space between columns.

Ragged rows: if a row has fewer columns than the maximum, pad with empty cells on the right. If a row has more columns than the first row, emit MATH-001 warning.

Example: `\begin{pmatrix} a & b \\ c & d \end{pmatrix}`
```
⎛ a  b ⎞
⎝ c  d ⎠
```

#### Cases `\begin{cases}...\end{cases}`

```
⎧ n+1  if n is odd
⎨
⎩ n/2  if n is even
```

Rows are separated by `\\`. The trailing `\\` on the last row is optional. Minimum 1 case required. Each row is rendered as: left brace glyph + expression + optional `&` condition (everything after `&` is treated as plain text).

---

## Grouping, text mode, and unsupported commands

### Grouping with `{...}`

Curly braces `{...}` group content as a single unit for operators like `^` and `_`. Unmatched braces emit MATH-006. Nested braces are allowed: `x^{a^{b}}` parses correctly. Braces not following an operator are silent wrappers: `{x}` renders as `x`.

### `\text{...}` — embed plain text

`\text{...}` passes content through unexpanded as plain text. Use it for words inside formulas:

| Input | Rendered |
|-------|----------|
| `f(x) = 1 \text{ if } x > 0` | `f(x) = 1 if x > 0` |

### Font styling commands

Font commands are silently accepted (content passes through, command stripped):

| Command | Behavior |
|---------|----------|
| `\mathbf{A}` | renders as `A` |
| `\mathrm{sin}` | renders as `sin` |
| `\mathit{x}` | renders as `x` |
| `\mathtt{n}` | renders as `n` |

### Scalable delimiters `\left` / `\right`

Not supported in this release. Use fixed-width equivalents: `\langle` / `\rangle`, `\vert` / `\Vert`, or literal `(` / `)`. If encountered, emit MATH-001.

### Limitations (non-goals)

This renderer does not support:
- Graphics: `\includegraphics`, `\tikz`, `\draw`
- Color and font sizing: `\color{}`, `\fontsize{}`
- Page layout: `\newpage`, `\phantom`, `\hspace`
- Advanced `amsmath` environments: `\align`, `\split`, `\gather`
- User macros: `\newcommand`, `\def`

All unsupported commands emit MATH-001 and are passed through unchanged.

---

## Integration with other proof directives

### In proof:bullets

Inline math works in bullet labels (Tier 1 and Tier 2 constructs only — Tier 3 in inline context triggers MATH-005 downgrade):

```
proof:bullets
- For $\epsilon > 0$, choose $\delta = \epsilon / M$
- Energy: $E = mc^2$
```

### In proof:element kind=value

```
proof:element kind=value field=formula format="$\nabla \cdot E = \rho/\epsilon_0$"
```

### In slide title/subtitle

```
```proof:slide layout=title
title: "$\nabla \times B = \mu_0 J$"
subtitle: "One of Maxwell's equations"
```
```

### In dashboard region content

Math in regions renders as literal text. Display math uses the block renderer.

---

## The `proof:math` directive (compile mode)

````markdown
```proof:math
\frac{d}{dx} e^x = e^x
```
````

````markdown
```proof:math width=40 align=center
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```
````

### Directive attributes

| Attribute | Default | Description |
|-----------|---------|-------------|
| `width` | auto | Width of rendered output |
| `align` | center | `left`, `center`, `right` |
| `no-chrome` | false | Omit fence and traceability comment |

`width=auto` computes the width as the maximum visual line width of the rendered expression. The `align` attribute pads within `width` using spaces. `align=left` means no left-padding; `align=center` splits padding evenly; `align=right` pushes left.

---

## Inline `$...$` expansion scope

Inline math is expanded in the same contexts as `[sym:name]`:
- Prose paragraphs, bullet labels, callout text, slide titles/subtitles
- NOT inside fenced code blocks, inline code spans `` `...` ``, or URLs

Tokenization order: code spans are masked first, then `$...$` is scanned within non-code regions. Any unmatched `$` (no closing `$` on the same line) is passed through unchanged.

### MATH-005 downgrade for Tier 3 in inline context

When a Tier 3 construct (`\frac`, `\int` with limits, matrix, cases) appears inside inline `$...$`, it is simplified to a Tier 2 single-line form:

| Original | Inline rendering |
|----------|-----------------|
| `$\frac{a}{b}$` | `a/b` |
| `$\int_0^n f(x) dx$` | `∫_(0)^(n) f(x) dx` |
| `$\sum_{i=1}^n i$` | `∑_(i=1)^(n) i` |

MATH-005 is emitted as a warning with suggestion: "Move to a `proof:math` block for multi-line rendering."

---

## LaTeX command mapping reference

### Combining `proof:math` with `proof:symbol`

`[sym:name]` is for semantic symbols in non-math contexts (decorations, status icons, ratings).
`$...$` is for mathematical notation within formulas.

| Use case | Syntax |
|----------|--------|
| Star rating icon | `[sym:star]` → `★` |
| Star as math operator | `$x \star y$` → `x ⋆ y` |
| Checkmark status | `[sym:checkmark]` → `✓` |
| Logical and | `$A \land B$` → `A ∧ B` |

Both work; they address different needs. Use `[sym:]` when the symbol is decorative or semantic; use `$...$` when the symbol is part of a mathematical expression.

---

## Invariants

| Invariant | Claim |
|-----------|-------|
| M-1 | For display math: rendered output is exactly `width` visual columns (padded/clipped). For inline math: no width constraint; output is a single-line string. |
| M-2 | Inline `$...$` expansion produces a single-line string (no newlines in inline math) |
| M-3 | `\frac{}{}` bar width = `max(visual_width(numerator), visual_width(denominator))`; `visual_width` for a multi-line sub-expression = max visual width of any single line in it |
| M-4 | Integral/sum/product limits: upper limit 1 line above the operator, lower 1 line below; both optional |
| M-5 | Matrix: all cells in a column have the same visual width (widest cell determines column width); all rows padded to the same column count |
| M-6 | Unrecognized `\command` emits `MATH-001` warning and passes through unchanged |
| M-7 | Ragged matrix rows (fewer columns than max) are padded with empty cells on the right |

---

## Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `MATH-001` | warning | Unknown `\command` or unknown environment — passed through unexpanded |
| `MATH-002` | warning | Inline math spans multiple lines — only first line rendered |
| `MATH-003` | error | Unmatched `\begin{env}` / `\end{env}` or mismatched environment names |
| `MATH-004` | warning | Display math expression exceeds declared `width` — clipped |
| `MATH-005` | warning | Tier 3 construct (`\frac`, matrix) in inline context — simplified to Tier 2 form |
| `MATH-006` | warning | Invalid syntax — unmatched braces or malformed operator argument |

### Diagnostic output format

Each diagnostic includes: code, severity, line:column, problematic expression, and a suggestion. Example:

```
WARNING MATH-005 at line 3, col 12:
  Inline math with `\frac`: $\frac{a}{b}$
  Suggestion: move to a `proof:math` block for multi-line rendering.

WARNING MATH-001 at line 7, col 5:
  Unknown command `\alph` — did you mean `\alpha`?
```

---

## Implementation plan

### Wave 1 — Symbol table + super/subscripts (~300 LOC)
- `src/math/mod.rs`: `MathRenderer` struct
- `src/math/symbols.rs`: `LATEX_SYMBOLS` table (all Tier 1 mappings)
- `src/math/superscript.rs`: `^{...}` and `_{...}` → Unicode super/subscripts
- Inline expansion: `expand_inline_math(text: &str) -> String`
- 30+ tests: each symbol, super/subscript single/complex, unknown command passthrough

### Wave 2 — Tier 2 ASCII constructs (~250 LOC)
- `src/math/tier2.rs`: `\sqrt{}`, `\frac{}{}` single-line form, prime notation
- Inline fraction: `a/b` single-line form for simple fractions
- `display_math(expr: &str, width: usize) -> Vec<String>` entry point
- 20+ tests: fractions, roots, primes

### Wave 3 — Tier 3 multi-line constructs (~400 LOC)
- `src/math/fraction.rs`: stacked fraction renderer
- `src/math/integral.rs`: integral/sum/product with bounds
- `src/math/matrix.rs`: matrix/pmatrix/bmatrix/cases environments
- 25+ tests: fractions with nested expressions, integrals with limits, 2×2/3×3 matrices

### Wave 4 — proof:math directive + inline expansion (~200 LOC)
- `proof:math` directive in compile.rs
- Inline `$...$` expansion in render_body_lines() and slide text paths
- `MATH-001` through `MATH-005` diagnostic codes
- L1 integration tests: compile .source.md with inline and block math
- Exit criterion: `$\alpha + \beta = \gamma$` in a slide body renders correctly

---

## Examples

### Calculus
```
d         
── sin(x) = cos(x)
dx        
```

### Statistics
```
     1       -(x-μ)²
f(x) = ────── e ────────
       σ√(2π)   2σ²
```

### Einstein field equations (simplified)
```
Gμν + Λgμν = (8πG/c⁴) Tμν
```

### Pythagorean theorem
```
a² + b² = c²
```

---

## See also

- [Symbol Spec](./symbol-spec.md) — decorative symbols `[sym:name]` (non-math)
- [Element Spec](./element-spec.md) — inline math in element labels
- [Slide Spec](./slide-spec.md) — math in slide titles and body
- [Compile Spec](./compile-spec.md) — proof:math directive

---

## Spec Clarifications (from scenario findings)

These clarifications resolve ambiguities surfaced during scenario testing. They are normative — implementations must conform.

### F32 — Whitespace inside `$...$` is significant

Spaces between tokens inside `$...$` are preserved verbatim. The tokenizer does NOT strip whitespace after commands.

| Input | Rendered |
|-------|----------|
| `$\alpha+\beta$` | `α+β` (no spaces) |
| `$\alpha + \beta$` | `α + β` (spaces around `+`) |

Authors control inter-token spacing by writing the spaces they want. There is no implicit space normalization.

### F34 — Superscript/subscript multi-char fallback rule

For grouped super/subscript arguments (e.g. `^{10}`, `_{ab}`), the renderer applies this rule:

- **If every character in the group has a Unicode superscript/subscript equivalent**, emit the contiguous Unicode form (e.g. `x^{10}` → `x¹⁰`).
- **Otherwise**, emit bracket notation (e.g. `x^{n+1}` → `x^(n+1)`).

The "all-or-nothing" rule keeps mixed-mapping output from looking ragged. This is already implemented; the spec previously did not state it explicitly.

### F36 — Fraction centering rounding

When centering numerator or denominator over a bar, total padding is `bar_width - visual_width(content)`. Distribution of odd-width padding:

```
left_pad  = total_pad / 2          (integer floor division)
right_pad = total_pad - left_pad   (extra space goes to the right)
```

This is consistent with M-3 ("odd-width asymmetries favor right-padding").

### F37 — Parentheses are literal inside groups

`(` and `)` are literal characters inside numerator and denominator groups. They are NOT grouping operators in this renderer. Only `{` and `}` group tokens.

`\frac{(a+b)}{c}` renders with literal parentheses around `a+b`.

### F38 — Multi-char superscript with no Unicode mapping

`^{-x}` has no Unicode equivalent for `-x` as a superscript run (no superscript `x`). Per F34, this falls back to bracket notation: `^(-x)`.

This is **not** a MATH-005 condition — MATH-005 fires only on Tier 3 constructs in inline context. F38 is correct Tier 2 behavior for a multi-char superscript argument that fails the all-or-nothing Unicode test.

### F42 — Inline `\frac` downgrade with complex arguments

When `\frac{n}{d}` appears in inline `$...$` context (MATH-005), the simplified form depends on the numerator's complexity:

| Source | Inline rendering |
|--------|------------------|
| `$\frac{a}{b}$` | `a/b` (single token — no parens) |
| `$\frac{x+y}{z}$` | `(x+y)/z` (operators present — parens added) |

A "simple single-token numerator" is a single character or single Tier 1 command. Anything containing operators (`+`, `-`, `*`, etc.) gets parenthesized to preserve precedence. Same rule applies to denominators.

### F43 — MATH-005 is a non-blocking warning

MATH-005 (Tier 3 in inline context) is a **warning**. Documents emitting MATH-005 still compile and write successfully:

- `written = true` after compile
- `proof check` reports MATH-005 but does NOT block (non-blocking)
- The downgraded Tier 2 rendering (per F42) is what appears in output

### F44 — Public API delimiter conventions

| Function | Input format |
|----------|--------------|
| `expand_inline_math(line: &str) -> String` | Full prose line **including** `$` delimiters. Function locates `$...$` spans and expands them in place. |
| `render_display_math(expr: &str, width, align) -> Vec<String>` | Inner expression **without** any `$` delimiters. Caller has already stripped fence/delimiters. |

Mixing these up — passing bare expr to `expand_inline_math` or wrapped `$...$` to `render_display_math` — is a caller bug, not a renderer condition.

### F46 — Pass order and `[sym:...]` inside `$...$`

The expansion pipeline is fixed and runs in this order:

1. **Symbol expansion** — `[sym:name]` → Unicode (per SYMBOL-SPEC)
2. **Math expansion** — `$...$` → rendered (per this spec)

Consequence: `[sym:...]` tags **inside** `$...$` are NOT expanded. The math tokenizer treats `[` as a literal character; once math expansion runs, the symbol pass has already completed. To use a decorative symbol in a math expression, write it outside the `$...$` span or use the math-native LaTeX command (e.g. `\star` rather than `[sym:star]`).
