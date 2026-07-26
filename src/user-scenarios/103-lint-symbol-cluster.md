# US-103 — Lint cluster: typo + missing pin

Multiple lint signals in one file:

- `[sym:checkmrk]` typo → SYMBOL-001 with did-you-mean
- mdloom:include pin reference that doesn't match a `[[davinci]]` entry

Status: [sym:checkmrk] all green.

<!-- mdloom:compiled from="md://src/data/features.md#:table:0" -->
```
name | category | status | directive | output
------ | ---------- | -------- | ----------- | --------
LaTeX math inline | math | stable | $...$ | inline Unicode
LaTeX math display | math | stable | mdloom:math | multi-line ASCII art
Symbol expansion | symbols | stable | [sym:name] | Unicode glyph
Symbol block | symbols | stable | mdloom:symbol | ASCII art block
Shape renderer | symbols | stable | mdloom:shape | ASCII art shape
Element value | elements | stable | mdloom:element kind=value | numeric cell
Element delta | elements | stable | mdloom:element kind=delta | delta with arrow
Element sparkline | elements | stable | mdloom:element kind=sparkline | ASCII sparkline
Element mini-bar | elements | stable | mdloom:element kind=mini-bar | ASCII bar chart
Element label | elements | stable | mdloom:element kind=label | text label
Element badge | elements | stable | mdloom:element kind=badge | bracketed badge
Row compositor | elements | stable | mdloom:row | column-pinned row
Slide title | slides | stable | mdloom:slide layout=title | title card
Slide title-content | slides | stable | mdloom:slide layout=title-content | two-zone slide
Slide two-column | slides | stable | mdloom:slide layout=two-column | split layout
Slide section | slides | stable | mdloom:slide layout=section | section divider
Slide stats | slides | stable | mdloom:slide layout=stats | stat row
Slide blank | slides | stable | mdloom:slide layout=blank | empty canvas
Slide bullets | slides | stable | mdloom:bullets | bullet list
Slide callout | slides | stable | mdloom:callout | callout box
Slide divider | slides | stable | mdloom:divider | horizontal rule
Slide quote | slides | stable | mdloom:quote | attributed quote
Slide centered | slides | stable | mdloom:centered | centered text
Dashboard canvas | dashboard | stable | mdloom:region | canvas grid
Tree dirtree | trees | stable | mdloom:tree kind=dirtree | filesystem tree
Tree org | trees | stable | mdloom:tree kind=org | org chart
Tree taxonomy | trees | stable | mdloom:tree kind=taxonomy | taxonomy tree
Tree dependency | trees | stable | mdloom:tree kind=dependency | dependency graph
Tree outline | trees | stable | mdloom:tree kind=outline | numbered outline
Figure import | figures | beta | mdloom:include kind=figure | ASCII image
DaVinci pin | figures | beta | mdloom pin | invariant storage
Lint check | linting | stable | mdloom check | diagnostic report
Auto-fix | linting | stable | mdloom fix | patched files
Compile pipeline | compile | stable | mdloom compile | resolved output
```
<!-- /mdloom:compiled -->
