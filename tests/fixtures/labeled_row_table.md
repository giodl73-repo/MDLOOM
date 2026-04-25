# Labeled Row Table — No False Positives

A table where the left column acts as a row header appearing OUTSIDE the box
delimiters. The row label appears to the left of the first `|`, so
is_content_line() (which requires starting with `|`) correctly skips these
lines and does NOT report width/column errors.

```
             Inward (self)        Outward (receiver)
         +--------------------+--------------------+
   T1    | orientation.frame  | orientation.serves |
Orient-  | "How I see the     | "Who I serve       |
  ation  |  world"            |  and why"          |
         +--------------------+--------------------+
   T2    | lens.verify        | lens.simplify      |
  Lens   | "Questions I ask"  | "Rules I apply"    |
         +--------------------+--------------------+
```

Unicode bar chart — block elements are in the safe range (U+2580-U+259F):

```
  Option A  █████████████████████████████ 78%
  Option B  ████████████████████          55%
  Option C  ███████                       19%
```
