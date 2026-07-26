# Problem Set 3 — Integration

## Problem 1

Evaluate the following integral using the substitution $u = x^2 + 1$:

<!-- mdloom:compiled from="mdloom:math" -->
```
     2     
     ⌠     
⌡ \frac  dx
     0     
```
<!-- /mdloom:compiled -->

**Solution:** Let $u = x^2 + 1$, so $du = 2x\, dx$.
When $x=0$, $u=1$; when $x=2$, $u=5$.

<!-- mdloom:compiled from="mdloom:math" -->
```
                5                
                ⌠                
⌡ \frac  du = \left15 = \sqrt - 1
                1                
```
<!-- /mdloom:compiled -->

---

## Problem 2

Use integration by parts to evaluate $\int x^2 e^x\, dx$.

**Hint:** Apply integration by parts twice.

**Result:** $x^2 e^x - 2xe^x + 2e^x + C = e^x(x^2 - 2x + 2) + C$

---

## Problem 3

Find the area enclosed by $y = \sin x$ and $y = \cos x$ on $[0, \pi]$.

The curves intersect at $x = \pi/4$. The area is:

<!-- mdloom:compiled from="mdloom:math" -->
```
                       π/4                       
                        ⌠                        
⌡ (\cos x - \sin x)  dx + ∫ (\sin x - \cos x)  dx
                        0                        
```
<!-- /mdloom:compiled -->

Evaluating: $(\sqrt{2} - 1) + (1 + \sqrt{2}) = 2\sqrt{2}$

---

## Problem 4 (Challenge)

Prove that $\int_0^{\infty} e^{-x^2}\, dx = \frac{\sqrt{\pi}}{2}$ using the
Gaussian integral result:

<!-- mdloom:compiled from="mdloom:math" -->
```
       ∞       
       ⌠       
⌡ e  dx = \sqrt
      -∞       
```
<!-- /mdloom:compiled -->

**Hint:** By symmetry, $\int_0^{\infty} e^{-x^2}\, dx = \frac{1}{2}\int_{-\infty}^{\infty} e^{-x^2}\, dx$.
