# US-55 — Heatmap: weekly availability grid

Body syntax for heatmap cells is `row|col: value`. Rows and columns appear
in first-seen order. Five shading levels (` ░▒▓█`) bucket the values.

<!-- mdloom:compiled from="mdloom:chart" -->
```
                 Availability by day × time
    9am  11am 1pm  3pm
Mon ▒▒▒▒ ████ ▓▓▓▓ ░░░░
Tue ▓▓▓▓ ████ ▓▓▓▓ ▒▒▒▒
Wed ░░░░ ▒▒▒▒ ████ ▒▒▒▒
```
<!-- /mdloom:compiled -->

Higher values produce darker shading; the busiest cell (Tue 11am: 9) renders
at the maximum shading level.
