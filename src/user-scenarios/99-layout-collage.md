# US-99 — Side-by-side layout of three figures

`proof:layout` composes multiple figures into a single ASCII collage with
gaps and labels. Useful in slides and dashboards where you want a row of
visualizations.

<!-- proof:compiled from="proof:layout"
     uris="md://src/data/features.md#:table:0,md://src/data/features.md#:table:0,md://src/data/features.md#:table:0" -->
```
                                      One Two Three                                          name | category | status | directive | output                                                name | category | status | directive | output
name | category | status | directive | output                                                ------ | ---------- | -------- | ----------- | --------                                      ------ | ---------- | -------- | ----------- | --------
------ | ---------- | -------- | ----------- | --------                                      LaTeX math inline | math | stable | $...$ | inline Unicode                                   LaTeX math inline | math | stable | $...$ | inline Unicode
LaTeX math inline | math | stable | $...$ | inline Unicode                                   LaTeX math display | math | stable | proof:math | multi-line ASCII art                       LaTeX math display | math | stable | proof:math | multi-line ASCII art
LaTeX math display | math | stable | proof:math | multi-line ASCII art                       Symbol expansion | symbols | stable | [sym:name] | Unicode glyph                             Symbol expansion | symbols | stable | [sym:name] | Unicode glyph
Symbol expansion | symbols | stable | [sym:name] | Unicode glyph                             Symbol block | symbols | stable | proof:symbol | ASCII art block                             Symbol block | symbols | stable | proof:symbol | ASCII art block
Symbol block | symbols | stable | proof:symbol | ASCII art block                             Shape renderer | symbols | stable | proof:shape | ASCII art shape                            Shape renderer | symbols | stable | proof:shape | ASCII art shape
Shape renderer | symbols | stable | proof:shape | ASCII art shape                            Element value | elements | stable | proof:element kind=value | numeric cell                  Element value | elements | stable | proof:element kind=value | numeric cell
Element value | elements | stable | proof:element kind=value | numeric cell                  Element delta | elements | stable | proof:element kind=delta | delta with arrow              Element delta | elements | stable | proof:element kind=delta | delta with arrow
Element delta | elements | stable | proof:element kind=delta | delta with arrow              Element sparkline | elements | stable | proof:element kind=sparkline | ASCII sparkline       Element sparkline | elements | stable | proof:element kind=sparkline | ASCII sparkline
Element sparkline | elements | stable | proof:element kind=sparkline | ASCII sparkline       Element mini-bar | elements | stable | proof:element kind=mini-bar | ASCII bar chart         Element mini-bar | elements | stable | proof:element kind=mini-bar | ASCII bar chart
Element mini-bar | elements | stable | proof:element kind=mini-bar | ASCII bar chart         Element label | elements | stable | proof:element kind=label | text label                    Element label | elements | stable | proof:element kind=label | text label
Element label | elements | stable | proof:element kind=label | text label                    Element badge | elements | stable | proof:element kind=badge | bracketed badge               Element badge | elements | stable | proof:element kind=badge | bracketed badge
Element badge | elements | stable | proof:element kind=badge | bracketed badge               Row compositor | elements | stable | proof:row | column-pinned row                           Row compositor | elements | stable | proof:row | column-pinned row
Row compositor | elements | stable | proof:row | column-pinned row                           Slide title | slides | stable | proof:slide layout=title | title card                        Slide title | slides | stable | proof:slide layout=title | title card
Slide title | slides | stable | proof:slide layout=title | title card                        Slide title-content | slides | stable | proof:slide layout=title-content | two-zone slide    Slide title-content | slides | stable | proof:slide layout=title-content | two-zone slide
Slide title-content | slides | stable | proof:slide layout=title-content | two-zone slide    Slide two-column | slides | stable | proof:slide layout=two-column | split layout            Slide two-column | slides | stable | proof:slide layout=two-column | split layout
Slide two-column | slides | stable | proof:slide layout=two-column | split layout            Slide section | slides | stable | proof:slide layout=section | section divider               Slide section | slides | stable | proof:slide layout=section | section divider
Slide section | slides | stable | proof:slide layout=section | section divider               Slide stats | slides | stable | proof:slide layout=stats | stat row                          Slide stats | slides | stable | proof:slide layout=stats | stat row
Slide stats | slides | stable | proof:slide layout=stats | stat row                          Slide blank | slides | stable | proof:slide layout=blank | empty canvas                      Slide blank | slides | stable | proof:slide layout=blank | empty canvas
Slide blank | slides | stable | proof:slide layout=blank | empty canvas                      Slide bullets | slides | stable | proof:bullets | bullet list                                Slide bullets | slides | stable | proof:bullets | bullet list
Slide bullets | slides | stable | proof:bullets | bullet list                                Slide callout | slides | stable | proof:callout | callout box                                Slide callout | slides | stable | proof:callout | callout box
Slide callout | slides | stable | proof:callout | callout box                                Slide divider | slides | stable | proof:divider | horizontal rule                            Slide divider | slides | stable | proof:divider | horizontal rule
Slide divider | slides | stable | proof:divider | horizontal rule                            Slide quote | slides | stable | proof:quote | attributed quote                               Slide quote | slides | stable | proof:quote | attributed quote
Slide quote | slides | stable | proof:quote | attributed quote                               Slide centered | slides | stable | proof:centered | centered text                            Slide centered | slides | stable | proof:centered | centered text
Slide centered | slides | stable | proof:centered | centered text                            Dashboard canvas | dashboard | stable | proof:region | canvas grid                           Dashboard canvas | dashboard | stable | proof:region | canvas grid
Dashboard canvas | dashboard | stable | proof:region | canvas grid                           Tree dirtree | trees | stable | proof:tree kind=dirtree | filesystem tree                    Tree dirtree | trees | stable | proof:tree kind=dirtree | filesystem tree
Tree dirtree | trees | stable | proof:tree kind=dirtree | filesystem tree                    Tree org | trees | stable | proof:tree kind=org | org chart                                  Tree org | trees | stable | proof:tree kind=org | org chart
Tree org | trees | stable | proof:tree kind=org | org chart                                  Tree taxonomy | trees | stable | proof:tree kind=taxonomy | taxonomy tree                    Tree taxonomy | trees | stable | proof:tree kind=taxonomy | taxonomy tree
Tree taxonomy | trees | stable | proof:tree kind=taxonomy | taxonomy tree                    Tree dependency | trees | stable | proof:tree kind=dependency | dependency graph             Tree dependency | trees | stable | proof:tree kind=dependency | dependency graph
Tree dependency | trees | stable | proof:tree kind=dependency | dependency graph             Tree outline | trees | stable | proof:tree kind=outline | numbered outline                   Tree outline | trees | stable | proof:tree kind=outline | numbered outline
Tree outline | trees | stable | proof:tree kind=outline | numbered outline                   Figure import | figures | beta | proof:include kind=figure | ASCII image                     Figure import | figures | beta | proof:include kind=figure | ASCII image
Figure import | figures | beta | proof:include kind=figure | ASCII image                     DaVinci pin | figures | beta | proof pin | invariant storage                                 DaVinci pin | figures | beta | proof pin | invariant storage
DaVinci pin | figures | beta | proof pin | invariant storage                                 Lint check | linting | stable | proof check | diagnostic report                              Lint check | linting | stable | proof check | diagnostic report
Lint check | linting | stable | proof check | diagnostic report                              Auto-fix | linting | stable | proof fix | patched files                                      Auto-fix | linting | stable | proof fix | patched files
Auto-fix | linting | stable | proof fix | patched files                                      Compile pipeline | compile | stable | proof compile | resolved output                        Compile pipeline | compile | stable | proof compile | resolved output
Compile pipeline | compile | stable | proof compile | resolved output
```
<!-- /proof:compiled -->

The same source repeated three times — purely to demonstrate the layout
geometry. Real usage would pull three different figures.
