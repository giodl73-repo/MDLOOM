# NumericsLib API Reference

## `compute_gradient`

Computes the gradient $\nabla f(x)$ at point $x \in \mathbb{R}^n$ using
finite differences with step size $h$.

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `f` | `Fn(&[f64]) -> f64` | yes | The function to differentiate |
| `x` | `&[f64]` | yes | Point at which to evaluate the gradient |
| `h` | `f64` | no | Step size (default: $10^{-5}$) |

**Returns:** `Vec<f64>` — the gradient vector $\nabla f(x)$

**Formula:**

```proof:math
\frac{\partial f}{\partial x_i} \approx \frac{f(x + h e_i) - f(x - h e_i)}{2h}
```

Where $e_i$ is the $i$-th standard basis vector.

---

## `newton_step`

Computes one step of Newton's method: $x_{k+1} = x_k - H^{-1} \nabla f(x_k)$

**Formula:**

```proof:math
x_{k+1} = x_k - \left[H_f(x_k)\right]^{-1} \nabla f(x_k)
```

**Convergence:** Quadratic near the optimum — error at step $k+1$ satisfies
$\|x_{k+1} - x^*\| \leq C \|x_k - x^*\|^2$ for some constant $C > 0$.

---

## `line_search`

Implements the Armijo condition: finds $\alpha > 0$ such that

```proof:math
f(x_k + \alpha d_k) \leq f(x_k) + c_1 \alpha \nabla f(x_k)^T d_k
```

where $c_1 \in (0, 1)$ (typically $10^{-4}$) and $d_k$ is the search direction.
