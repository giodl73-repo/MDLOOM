# proof Math — LaTeX Rendering for ASCII/Terminal

proof renders LaTeX math notation to Unicode symbols and ASCII art — no LaTeX
installation, no MathJax, no external tools. The renderer is a pure Rust
subset parser that maps LaTeX constructs directly to terminal output.

There are two entry points. **Inline** math uses `$...$` and expands anywhere
in prose — in bullet labels, slide titles, callout text. **Display** math uses
the `proof:math` fenced block and renders multi-line constructs like stacked
fractions, integrals with limits, and matrices.

The renderer has three tiers. Tier 1 maps single LaTeX commands to Unicode
characters directly (instant, no layout). Tier 2 handles single-line constructs
like superscripts, square roots, and inline fractions. Tier 3 handles multi-line
display math and is only available in `proof:math` blocks — if you use a Tier 3
construct inside `$...$`, proof downgrades it to Tier 2 and emits a MATH-005
warning.

---

## Tier 1 — Unicode substitution

Tier 1 is the simplest: single LaTeX commands map directly to single Unicode
characters. These work everywhere — inline, in bullets, in slide titles. There
is no rendering overhead; proof just does a table lookup at compile time.
Use these freely; they expand to well-supported Unicode characters that render
correctly in any modern terminal.

### Greek alphabet

Inline: $\alpha$, $\beta$, $\gamma$, $\delta$, $\epsilon$, $\theta$, $\lambda$, $\mu$, $\pi$, $\sigma$, $\omega$

Uppercase: $\Gamma$, $\Delta$, $\Lambda$, $\Pi$, $\Sigma$, $\Omega$

### Operators and relations

$\times$ $\div$ $\pm$ $\cdot$ $\circ$ $\leq$ $\geq$ $\neq$ $\approx$ $\equiv$ $\propto$ $\infty$

### Set theory and logic

$\in$ $\notin$ $\subset$ $\subseteq$ $\cup$ $\cap$ $\emptyset$ $\forall$ $\exists$ $\neg$ $\land$ $\lor$

### Arrows

$\to$ $\leftarrow$ $\Rightarrow$ $\Leftarrow$ $\Leftrightarrow$ $\implies$ $\iff$ $\mapsto$

### Calculus symbols

$\partial$ $\nabla$ $\int$ $\oint$ $\sum$ $\prod$

---

## Tier 2 — Single-line ASCII art

Tier 2 constructs produce more than one character but fit on a single line.
They work in both inline `$...$` and `proof:math` blocks. Superscripts and
subscripts use Unicode combining characters when possible (so `x²` is a real
superscript, not `x^2`). Complex arguments fall back to bracket notation.
Square roots, primes, and limit operators all render inline.

### Superscripts and subscripts

Single character: $x^2$, $x^3$, $x^n$, $x_0$, $x_i$

Multi-character (bracket notation): $x^{n+1}$, $x_{n+1}$

Unicode superscripts: ⁰¹²³⁴⁵⁶⁷⁸⁹ⁿ
Unicode subscripts: ₀₁₂₃₄₅₆₇₈₉ₐₑᵢₒᵤₓ

### Square roots

$\sqrt{x}$ — $\sqrt{x+1}$ — $\sqrt[3]{x}$

### Primes

$f'$ — $f''$ — $f'''$

### Limit operators

$\lim_{x \to 0} f(x)$ — $\max_{x \in S}$ — $\min_{i=1}^n$

### Inline fractions (Tier 2 downgrade)

In inline context, `\frac` renders as `a/b`:
$\frac{df}{dx}$ — $\frac{n+1}{2}$

---

## Tier 3 — Multi-line display math

Tier 3 constructs require multiple output lines and are only valid in
`proof:math` blocks. If you try to use them inline in `$...$`, proof simplifies
them to a single-line form and emits MATH-005. The main Tier 3 constructs are
stacked fractions, integrals and sums with limits, matrices, and cases.

### Stacked fractions

```proof:math
\frac{d}{dx} e^x = e^x
```

```proof:math
\frac{n(n+1)}{2}
```

```proof:math
\frac{\frac{a}{b}}{c}
```

### Integrals with limits

```proof:math
\int_0^{\infty} e^{-x} dx = 1
```

```proof:math
\int_a^b f(x) dx
```

### Sums and products

```proof:math
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
```

```proof:math
\prod_{k=1}^{n} k = n!
```

### Matrices

```proof:math
\begin{pmatrix} a & b \\ c & d \end{pmatrix}
```

```proof:math
\begin{bmatrix} 1 & 0 \\ 0 & 1 \end{bmatrix}
```

```proof:math
\begin{vmatrix} a & b \\ c & d \end{vmatrix}
```

### Cases

```proof:math
\begin{cases} n+1 & \text{if n is odd} \\ n/2 & \text{if n is even} \end{cases}
```

---

## Display attributes

Control width and alignment:

```proof:math width=60 align=center
\sum_{i=1}^{n} i^2 = \frac{n(n+1)(2n+1)}{6}
```

```proof:math width=40 align=left
\alpha + \beta = \gamma
```

```proof:math no-chrome=true
\pi \approx 3.14159
```

---

## Real examples

### Pythagorean theorem

$a^2 + b^2 = c^2$

### Einstein field equations (simplified)

$G_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}$

### Normal distribution

```proof:math
\frac{1}{\sigma\sqrt{2\pi}} e^{-\frac{(x-\mu)^2}{2\sigma^2}}
```

### Euler's identity

$e^{i\pi} + 1 = 0$

### Derivative definition

```proof:math
\lim_{h \to 0} \frac{f(x+h) - f(x)}{h}
```

### Bayes' theorem

```proof:math
\frac{P(A \cap B)}{P(B)}
```

---

## Diagnostic codes

| Code | Meaning |
|------|---------|
| `MATH-001` | Unknown `\command` — passed through |
| `MATH-002` | Inline math spans multiple lines |
| `MATH-003` | Unmatched `\begin{env}` / `\end{env}` |
| `MATH-004` | Display math exceeds width — clipped |
| `MATH-005` | Tier 3 construct in inline context — simplified |
| `MATH-006` | Unmatched braces or malformed syntax |
