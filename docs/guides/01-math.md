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

<!-- proof:compiled from="proof:math" -->
```
d         
── eˣ = eˣ
dx        
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:math" -->
```
n(n+1)
──────
  2   
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:math" -->
```
a
─
b
─
c
```
<!-- /proof:compiled -->

### Integrals with limits

<!-- proof:compiled from="proof:math" -->
```
    ∞     
    ⌠     
⌡ e dx = 1
    0     
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:math" -->
```
    b    
    ⌠    
⌡ f(x) dx
    a    
```
<!-- /proof:compiled -->

### Sums and products

<!-- proof:compiled from="proof:math" -->
```
 n           
 ∑  i = \frac
i=1          
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:math" -->
```
 n        
 ∏  k = n!
k=1       
```
<!-- /proof:compiled -->

### Matrices

<!-- proof:compiled from="proof:math" -->
```
⎛ a  b ⎞
⎝ c  d ⎠
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:math" -->
```
⎡ 1  0 ⎤
⎣ 0  1 ⎦
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:math" -->
```
| a  b |
| c  d |
```
<!-- /proof:compiled -->

### Cases

<!-- proof:compiled from="proof:math" -->
```
⎧ n+1  \textif n is odd 
⎩ n/2  \textif n is even
```
<!-- /proof:compiled -->

---

## Display attributes

Control width and alignment:

<!-- proof:compiled from="proof:math" -->
```
                        n                                   
                        ∑  i2 = \frac                       
                       i=1                                  
```
<!-- /proof:compiled -->

<!-- proof:compiled from="proof:math" -->
```
α + β = γ                               
```
<!-- /proof:compiled -->

```
π ≈ 3.14159
```

---

## Real examples

### Pythagorean theorem

$a^2 + b^2 = c^2$

### Einstein field equations (simplified)

$G_{\mu\nu} + \Lambda g_{\mu\nu} = \frac{8\pi G}{c^4} T_{\mu\nu}$

### Normal distribution

<!-- proof:compiled from="proof:math" -->
```
  1                         
────── e^(-\frac(x-μ)^22σ^2)
σ√(2π)                      
```
<!-- /proof:compiled -->

### Euler's identity

$e^{i\pi} + 1 = 0$

### Derivative definition

<!-- proof:compiled from="proof:math" -->
```
            f(x+h) - f(x)
lim_(h → 0) ─────────────
                  h      
```
<!-- /proof:compiled -->

### Bayes' theorem

<!-- proof:compiled from="proof:math" -->
```
P(A ∩ B)
────────
  P(B)  
```
<!-- /proof:compiled -->

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
