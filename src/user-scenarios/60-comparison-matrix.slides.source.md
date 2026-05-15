---
slides:
  width: 80
  height: 24
  theme: minimal
---

```proof:slide layout=comparison title="Eisenhower matrix"
## axis:y Importance
## axis:x Urgency
## q:tl
URGENT + IMPORTANT
- Production incidents
- Spec breakage from upstream
## q:tr
NOT URGENT + IMPORTANT
- Architecture migrations
- Test coverage gaps
## q:bl
URGENT + NOT IMPORTANT
- "Quick" tickets that aren't quick
- Stand-up status threads
## q:br
NOT URGENT + NOT IMPORTANT
- Bikeshed discussions
- Premature tooling polish
```

---

```proof:slide layout=comparison title="Build vs buy"
## axis:y Specificity
## axis:x Cost
## q:tl
HIGH SPECIFICITY + LOW COST
Build it ourselves; this is the sweet spot.
## q:tr
HIGH SPECIFICITY + HIGH COST
Build with caution. Spike a prototype first.
## q:bl
LOW SPECIFICITY + LOW COST
Buy or use OSS. Engineering time is more valuable.
## q:br
LOW SPECIFICITY + HIGH COST
Buy. Custom-building common infrastructure is rarely the right call.
```
