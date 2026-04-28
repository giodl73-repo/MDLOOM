# proof Feature Registry

| name | category | status | directive | output |
|------|----------|--------|-----------|--------|
| LaTeX math inline | math | stable | $...$ | inline Unicode |
| LaTeX math display | math | stable | proof:math | multi-line ASCII art |
| Symbol expansion | symbols | stable | [sym:name] | Unicode glyph |
| Symbol block | symbols | stable | proof:symbol | ASCII art block |
| Shape renderer | symbols | stable | proof:shape | ASCII art shape |
| Element value | elements | stable | proof:element kind=value | numeric cell |
| Element delta | elements | stable | proof:element kind=delta | delta with arrow |
| Element sparkline | elements | stable | proof:element kind=sparkline | ASCII sparkline |
| Element mini-bar | elements | stable | proof:element kind=mini-bar | ASCII bar chart |
| Element label | elements | stable | proof:element kind=label | text label |
| Element badge | elements | stable | proof:element kind=badge | bracketed badge |
| Row compositor | elements | stable | proof:row | column-pinned row |
| Slide title | slides | stable | proof:slide layout=title | title card |
| Slide title-content | slides | stable | proof:slide layout=title-content | two-zone slide |
| Slide two-column | slides | stable | proof:slide layout=two-column | split layout |
| Slide section | slides | stable | proof:slide layout=section | section divider |
| Slide stats | slides | stable | proof:slide layout=stats | stat row |
| Slide blank | slides | stable | proof:slide layout=blank | empty canvas |
| Slide bullets | slides | stable | proof:bullets | bullet list |
| Slide callout | slides | stable | proof:callout | callout box |
| Slide divider | slides | stable | proof:divider | horizontal rule |
| Slide quote | slides | stable | proof:quote | attributed quote |
| Slide centered | slides | stable | proof:centered | centered text |
| Dashboard canvas | dashboard | stable | proof:region | canvas grid |
| Tree dirtree | trees | stable | proof:tree kind=dirtree | filesystem tree |
| Tree org | trees | stable | proof:tree kind=org | org chart |
| Tree taxonomy | trees | stable | proof:tree kind=taxonomy | taxonomy tree |
| Tree dependency | trees | stable | proof:tree kind=dependency | dependency graph |
| Tree outline | trees | stable | proof:tree kind=outline | numbered outline |
| Figure import | figures | beta | proof:include kind=figure | ASCII image |
| DaVinci pin | figures | beta | proof pin | invariant storage |
| Lint check | linting | stable | proof check | diagnostic report |
| Auto-fix | linting | stable | proof fix | patched files |
| Compile pipeline | compile | stable | proof compile | resolved output |
