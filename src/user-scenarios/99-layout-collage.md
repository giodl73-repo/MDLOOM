# US-99 — Side-by-side layout of three figures

`mdloom:layout` composes multiple figures into a single ASCII collage with
gaps and labels. Useful in slides and dashboards where you want a row of
visualizations.

<!-- mdloom:compiled from="mdloom:layout"
     uris="md://src/data/features.md#:table:0,md://src/data/features.md#:table:0,md://src/data/features.md#:table:0" -->
```
                                      One Two Three                                          name | category | status | directive | output                                                name | category | status | directive | output
name | category | status | directive | output                                                ------ | ---------- | -------- | ----------- | --------                                      ------ | ---------- | -------- | ----------- | --------
------ | ---------- | -------- | ----------- | --------                                      LaTeX math inline | math | stable | $...$ | inline Unicode                                   LaTeX math inline | math | stable | $...$ | inline Unicode
LaTeX math inline | math | stable | $...$ | inline Unicode                                   LaTeX math display | math | stable | mdloom:math | multi-line ASCII art                       LaTeX math display | math | stable | mdloom:math | multi-line ASCII art
LaTeX math display | math | stable | mdloom:math | multi-line ASCII art                       Symbol expansion | symbols | stable | [sym:name] | Unicode glyph                             Symbol expansion | symbols | stable | [sym:name] | Unicode glyph
Symbol expansion | symbols | stable | [sym:name] | Unicode glyph                             Symbol block | symbols | stable | mdloom:symbol | ASCII art block                             Symbol block | symbols | stable | mdloom:symbol | ASCII art block
Symbol block | symbols | stable | mdloom:symbol | ASCII art block                             Shape renderer | symbols | stable | mdloom:shape | ASCII art shape                            Shape renderer | symbols | stable | mdloom:shape | ASCII art shape
Shape renderer | symbols | stable | mdloom:shape | ASCII art shape                            Element value | elements | stable | mdloom:element kind=value | numeric cell                  Element value | elements | stable | mdloom:element kind=value | numeric cell
Element value | elements | stable | mdloom:element kind=value | numeric cell                  Element delta | elements | stable | mdloom:element kind=delta | delta with arrow              Element delta | elements | stable | mdloom:element kind=delta | delta with arrow
Element delta | elements | stable | mdloom:element kind=delta | delta with arrow              Element sparkline | elements | stable | mdloom:element kind=sparkline | ASCII sparkline       Element sparkline | elements | stable | mdloom:element kind=sparkline | ASCII sparkline
Element sparkline | elements | stable | mdloom:element kind=sparkline | ASCII sparkline       Element mini-bar | elements | stable | mdloom:element kind=mini-bar | ASCII bar chart         Element mini-bar | elements | stable | mdloom:element kind=mini-bar | ASCII bar chart
Element mini-bar | elements | stable | mdloom:element kind=mini-bar | ASCII bar chart         Element label | elements | stable | mdloom:element kind=label | text label                    Element label | elements | stable | mdloom:element kind=label | text label
Element label | elements | stable | mdloom:element kind=label | text label                    Element badge | elements | stable | mdloom:element kind=badge | bracketed badge               Element badge | elements | stable | mdloom:element kind=badge | bracketed badge
Element badge | elements | stable | mdloom:element kind=badge | bracketed badge               Row compositor | elements | stable | mdloom:row | column-pinned row                           Row compositor | elements | stable | mdloom:row | column-pinned row
Row compositor | elements | stable | mdloom:row | column-pinned row                           Slide title | slides | stable | mdloom:slide layout=title | title card                        Slide title | slides | stable | mdloom:slide layout=title | title card
Slide title | slides | stable | mdloom:slide layout=title | title card                        Slide title-content | slides | stable | mdloom:slide layout=title-content | two-zone slide    Slide title-content | slides | stable | mdloom:slide layout=title-content | two-zone slide
Slide title-content | slides | stable | mdloom:slide layout=title-content | two-zone slide    Slide two-column | slides | stable | mdloom:slide layout=two-column | split layout            Slide two-column | slides | stable | mdloom:slide layout=two-column | split layout
Slide two-column | slides | stable | mdloom:slide layout=two-column | split layout            Slide section | slides | stable | mdloom:slide layout=section | section divider               Slide section | slides | stable | mdloom:slide layout=section | section divider
Slide section | slides | stable | mdloom:slide layout=section | section divider               Slide stats | slides | stable | mdloom:slide layout=stats | stat row                          Slide stats | slides | stable | mdloom:slide layout=stats | stat row
Slide stats | slides | stable | mdloom:slide layout=stats | stat row                          Slide blank | slides | stable | mdloom:slide layout=blank | empty canvas                      Slide blank | slides | stable | mdloom:slide layout=blank | empty canvas
Slide blank | slides | stable | mdloom:slide layout=blank | empty canvas                      Slide bullets | slides | stable | mdloom:bullets | bullet list                                Slide bullets | slides | stable | mdloom:bullets | bullet list
Slide bullets | slides | stable | mdloom:bullets | bullet list                                Slide callout | slides | stable | mdloom:callout | callout box                                Slide callout | slides | stable | mdloom:callout | callout box
Slide callout | slides | stable | mdloom:callout | callout box                                Slide divider | slides | stable | mdloom:divider | horizontal rule                            Slide divider | slides | stable | mdloom:divider | horizontal rule
Slide divider | slides | stable | mdloom:divider | horizontal rule                            Slide quote | slides | stable | mdloom:quote | attributed quote                               Slide quote | slides | stable | mdloom:quote | attributed quote
Slide quote | slides | stable | mdloom:quote | attributed quote                               Slide centered | slides | stable | mdloom:centered | centered text                            Slide centered | slides | stable | mdloom:centered | centered text
Slide centered | slides | stable | mdloom:centered | centered text                            Dashboard canvas | dashboard | stable | mdloom:region | canvas grid                           Dashboard canvas | dashboard | stable | mdloom:region | canvas grid
Dashboard canvas | dashboard | stable | mdloom:region | canvas grid                           Tree dirtree | trees | stable | mdloom:tree kind=dirtree | filesystem tree                    Tree dirtree | trees | stable | mdloom:tree kind=dirtree | filesystem tree
Tree dirtree | trees | stable | mdloom:tree kind=dirtree | filesystem tree                    Tree org | trees | stable | mdloom:tree kind=org | org chart                                  Tree org | trees | stable | mdloom:tree kind=org | org chart
Tree org | trees | stable | mdloom:tree kind=org | org chart                                  Tree taxonomy | trees | stable | mdloom:tree kind=taxonomy | taxonomy tree                    Tree taxonomy | trees | stable | mdloom:tree kind=taxonomy | taxonomy tree
Tree taxonomy | trees | stable | mdloom:tree kind=taxonomy | taxonomy tree                    Tree dependency | trees | stable | mdloom:tree kind=dependency | dependency graph             Tree dependency | trees | stable | mdloom:tree kind=dependency | dependency graph
Tree dependency | trees | stable | mdloom:tree kind=dependency | dependency graph             Tree outline | trees | stable | mdloom:tree kind=outline | numbered outline                   Tree outline | trees | stable | mdloom:tree kind=outline | numbered outline
Tree outline | trees | stable | mdloom:tree kind=outline | numbered outline                   Figure import | figures | beta | mdloom:include kind=figure | ASCII image                     Figure import | figures | beta | mdloom:include kind=figure | ASCII image
Figure import | figures | beta | mdloom:include kind=figure | ASCII image                     DaVinci pin | figures | beta | mdloom pin | invariant storage                                 DaVinci pin | figures | beta | mdloom pin | invariant storage
DaVinci pin | figures | beta | mdloom pin | invariant storage                                 Lint check | linting | stable | mdloom check | diagnostic report                              Lint check | linting | stable | mdloom check | diagnostic report
Lint check | linting | stable | mdloom check | diagnostic report                              Auto-fix | linting | stable | mdloom fix | patched files                                      Auto-fix | linting | stable | mdloom fix | patched files
Auto-fix | linting | stable | mdloom fix | patched files                                      Compile pipeline | compile | stable | mdloom compile | resolved output                        Compile pipeline | compile | stable | mdloom compile | resolved output
Compile pipeline | compile | stable | mdloom compile | resolved output
```
<!-- /mdloom:compiled -->

The same source repeated three times — purely to demonstrate the layout
geometry. Real usage would pull three different figures.
