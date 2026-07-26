# US-94 — Shape roster (the three currently shipped)

`mdloom:shape` currently renders three named shapes: banner, badge, ribbon.
Image-import shapes (circle, heart, octagon, ...) live behind
`mdloom figure import --shape <name>`, not the `mdloom:shape` directive.

```mdloom:shape name=banner title="Section 2 — Defense" style=double
```

```mdloom:shape name=badge label="MVP" style=rounded
```

```mdloom:shape name=ribbon text="WINNER" direction=diagonal
```
