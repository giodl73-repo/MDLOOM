# proof Diagnostic Codes

| code | severity | module | meaning |
|------|----------|--------|---------|
| ascii_box_width | error | ascii_box | Box top/bottom width does not match declared width |
| ascii_box_col | error | ascii_box | Column separator misaligned |
| ascii_box_open | error | ascii_box | Unclosed box — missing bottom border |
| ascii_flow_node | warning | ascii_flow | Node label missing or malformed |
| ascii_flow_edge | warning | ascii_flow | Edge connector does not connect two nodes |
| ascii_tree_indent | error | ascii_tree | Inconsistent indentation level |
| ascii_tree_root | error | ascii_tree | Multiple or missing root nodes |
| ascii_barchart_scale | warning | ascii_barchart | Bar scale inconsistent with declared max |
| markdown_h1 | warning | markdown | File missing H1 heading |
| markdown_h2 | warning | markdown | Required H2 section missing |
| markdown_link | warning | markdown | Broken or missing internal link |
| markdown_table | warning | markdown_table | Table column count inconsistent |
| MATH-001 | warning | math | Unknown LaTeX command — passed through |
| MATH-002 | warning | math | Inline math spans multiple lines |
| MATH-003 | error | math | Unmatched \begin{env} / \end{env} |
| MATH-004 | warning | math | Display math exceeds declared width — clipped |
| MATH-005 | warning | math | Tier 3 construct in inline context — simplified |
| MATH-006 | warning | math | Unmatched braces or malformed operator |
| SYMBOL-001 | warning | symbol | Symbol name not found in library |
| COMPILE-001 | error | compile | md:// URI could not be resolved |
| COMPILE-002 | error | compile | Directive invalid in current file type |
| COMPILE-003 | error | compile | Element data source missing required field |
| DASHBOARD-001 | error | dashboard | Region declared but not rendered |
| DASHBOARD-002 | error | dashboard | Region content exceeds bounding box |
| DASHBOARD-003 | error | dashboard | Two regions overlap |
