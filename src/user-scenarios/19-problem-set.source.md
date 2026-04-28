# Problem Set 3 — Integration

## Problem 1

Evaluate the following integral using the substitution $u = x^2 + 1$:

```proof:math
\int_0^2 \frac{x}{\sqrt{x^2 + 1}}\, dx
```

**Solution:** Let $u = x^2 + 1$, so $du = 2x\, dx$.
When $x=0$, $u=1$; when $x=2$, $u=5$.

```proof:math
\int_1^5 \frac{1}{2\sqrt{u}}\, du = \left[\sqrt{u}\right]_1^5 = \sqrt{5} - 1
```

---

## Problem 2

Use integration by parts to evaluate $\int x^2 e^x\, dx$.

**Hint:** Apply integration by parts twice.

**Result:** $x^2 e^x - 2xe^x + 2e^x + C = e^x(x^2 - 2x + 2) + C$

---

## Problem 3

Find the area enclosed by $y = \sin x$ and $y = \cos x$ on $[0, \pi]$.

The curves intersect at $x = \pi/4$. The area is:

```proof:math
\int_0^{\pi/4} (\cos x - \sin x)\, dx + \int_{\pi/4}^{\pi} (\sin x - \cos x)\, dx
```

Evaluating: $(\sqrt{2} - 1) + (1 + \sqrt{2}) = 2\sqrt{2}$

---

## Problem 4 (Challenge)

Prove that $\int_0^{\infty} e^{-x^2}\, dx = \frac{\sqrt{\pi}}{2}$ using the
Gaussian integral result:

```proof:math
\int_{-\infty}^{\infty} e^{-x^2}\, dx = \sqrt{\pi}
```

**Hint:** By symmetry, $\int_0^{\infty} e^{-x^2}\, dx = \frac{1}{2}\int_{-\infty}^{\infty} e^{-x^2}\, dx$.
