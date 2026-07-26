---
width: 80
height: 22
theme: minimal
---

```mdloom:slide layout=title
title: "Calculus II"
subtitle: "Integration Techniques"
author: "Prof. Rivera"
date: "Spring 2026"
```

---

```mdloom:slide layout=section
title: "The Fundamental Theorem"
subtitle: "Connecting differentiation and integration"
```

---

```mdloom:slide layout=title-content
title: "FTC Part 1"
---
If $f$ is continuous on $[a, b]$ and $F(x) = \int_a^x f(t)\, dt$, then:

$F'(x) = f(x)$

That is, $F$ is an antiderivative of $f$. In other words, differentiation
and integration are inverse operations.

mdloom:callout style=key
The derivative of the area function equals the original function.
```

---

```mdloom:slide layout=title-content
title: "Integration by Parts"
---
For differentiable functions $u$ and $v$:

$\int u\, dv = uv - \int v\, du$

mdloom:divider style=thin

mdloom:bullets
- Choose $u$ = function that simplifies when differentiated
- Choose $dv$ = function that is easy to integrate
- LIATE rule: Logarithmic → Inverse trig → Algebraic → Trig → Exponential
```

---

```mdloom:slide layout=title-content
title: "Example — $\int x e^x dx$"
---
Let $u = x$ and $dv = e^x\, dx$.

Then $du = dx$ and $v = e^x$.

By parts: $\int x e^x\, dx = x e^x - \int e^x\, dx = x e^x - e^x + C$

mdloom:callout style=info
Check by differentiation: $\frac{d}{dx}(xe^x - e^x) = e^x + xe^x - e^x = xe^x$ [sym:checkmark]
```

---

```mdloom:slide layout=title
title: "Practice"
subtitle: "Try: $\int \ln x\, dx$ using integration by parts"
```
