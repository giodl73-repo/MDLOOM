# US-94 — Shape roster (the three currently shipped)

`proof:shape` currently renders three named shapes: banner, badge, ribbon.
Image-import shapes (circle, heart, octagon, ...) live behind
`proof figure import --shape <name>`, not the `proof:shape` directive.

```proof:shape name=banner title="Section 2 — Defense" style=double
```

```proof:shape name=badge label="MVP" style=rounded
```

```proof:shape name=ribbon text="WINNER" direction=diagonal
```
