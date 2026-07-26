# Directive Parser Boundary Closeout

## Mission

Start reducing `compile.rs` into compiler phase modules without changing source
syntax or renderer behavior.

## Changes

- Added `mdloom_lib::compile_directive`.
- Moved mdloom directive kind classification out of `compile.rs`.
- Moved mdloom directive fence span scanning out of `compile.rs`.
- Moved mdloom directive header attribute slicing out of `compile.rs` and onto
  `DirectiveSpan`.
- Moved shared directive `key=value` attribute extraction out of `compile.rs`.
- Moved `mdloom:element` directive attribute/kind/source/field/inline-value
  parsing out of `compile.rs`.
- Moved `mdloom:row` foreach/separator/width/no-chrome and row-element body
  parsing out of `compile.rs`.
- Moved `mdloom:tree` directive attribute/kind/source/inline-body parsing out
  of `compile.rs`.
- Moved `mdloom:layout` directive attribute/body URI parsing and config
  conversion out of `compile.rs`.
- Moved `mdloom:chart` directive attribute/source/field/body parsing out of
  `compile.rs`.
- Moved `mdloom:math` directive attribute/expression-body parsing out of
  `compile.rs`.
- Moved `mdloom:toc` directive payload parsing out of `compile.rs`.
- Moved `mdloom:xref` directive payload parsing out of `compile.rs`.
- Moved `mdloom:blockquote` directive attribute/text-body parsing out of
  `compile.rs`.
- Moved `mdloom:symbol` directive payload parsing out of `compile.rs`.
- Moved `mdloom:shape` directive payload parsing behind the directive parser
  boundary.
- Moved `mdloom:region` directive name/body parsing out of `compile.rs`.
- Moved `mdloom:include` directive pin/body URI parsing out of `compile.rs`.
- Moved `mdloom:table` directive payload parsing out of `compile.rs`.
- Moved typed `Directive` ownership and directive collection into
  `compile_directive`; kept `compile.rs` as the compile/render facade.
- Moved the public quick-inspection `parse_directives` implementation behind
  `compile_directive`.
- Added `mdloom_lib::compile_prose` and moved prose-only `mdloom:xref` and
  `mdloom:blockquote` rendering helpers out of `compile.rs`.
- Added `mdloom_lib::compile_source` and moved shared compile-time source
  resolution out of `compile.rs`.
- Added `mdloom_lib::compile_chart` and moved `mdloom:chart` data resolution and
  markdown table extraction out of `compile.rs`.
- Added `mdloom_lib::compile_tree` and moved inline tree/outline rendering
  helpers out of `compile.rs`.
- Added `mdloom_lib::compile_toc` and moved `mdloom:toc` heading collection,
  section narrowing, and list/tree/numbered formatting out of `compile.rs`.
- Preserved existing directive aliases, including `mdloom:numbered-list` and
  `mdloom:ol` mapping to `ol`.
- Added focused unit coverage for known directive kinds, aliases, unknown
  directives, and non-mdloom fences.
- Re-ran existing compile parser coverage to prove behavior stayed stable.

## Validation

- `cargo test classifies_known_directive_kinds`
- `cargo test ignores_unknown_or_non_mdloom_fences`
- `cargo test scans_directive_spans_with_body_and_closing_line`
- `cargo test scans_multiple_directive_spans`
- `cargo test slices_directive_header_attrs`
- `cargo test extracts_quoted_and_unquoted_attr_values`
- `cargo test extract_attr_value_respects_word_boundaries`
- `cargo test parses_element_attrs_with_defaults_and_flags`
- `cargo test parses_element_directive_kind_source_field_and_inline_value`
- `cargo test parses_tree_attrs_with_defaults_and_lists`
- `cargo test parses_tree_directive_kind_source_and_inline_body`
- `cargo test parses_layout_attrs_with_defaults_and_flags`
- `cargo test parses_layout_directive_attrs_and_body_uris`
- `cargo test parses_chart_attrs_with_defaults_and_aliases`
- `cargo test parses_chart_directive_source_fields_and_inline_body`
- `cargo test parses_math_attrs_with_defaults`
- `cargo test parses_math_directive_attrs_and_expression_body`
- `cargo test parses_toc_attrs_with_body_source_and_aliases`
- `cargo test parses_toc_directive_payload`
- `cargo test parses_xref_attrs_with_uri_source_and_body_fallback`
- `cargo test parses_xref_directive_payload`
- `cargo test parses_blockquote_attrs_with_defaults_and_aliases`
- `cargo test parses_blockquote_directive_attrs_and_text_body`
- `cargo test parses_symbol_attrs_with_defaults`
- `cargo test parses_symbol_directive_payload`
- `cargo test parses_shape_attrs_with_symbol_defaults`
- `cargo test parses_shape_directive_payload`
- `cargo test parses_region_directive_name_and_body`
- `cargo test parses_include_directive_uri_and_pin`
- `cargo test parses_table_uri_from_body`
- `cargo test parses_table_directive_payload`
- `cargo test parses_foreach_positional_and_source_attr_forms`
- `cargo test parses_row_element_lines`
- `cargo test parses_row_directive_attrs_and_elements`
- `cargo test test_parse_include_directive`
- `cargo test test_parse_layout_directive`
- `cargo test test_parse_table_directive`
- `cargo test test_parse_no_directives`
- `cargo test test_parse_multiple_directives`
- `cargo test test_collect_directives_include`
- `cargo test test_collect_directives_row_explicit_separator`
- `cargo test test_collect_directives_element_kind_value`
- `cargo test test_parse_foreach_extracts_var_and_uri`
- `cargo test test_parse_row_element_line_label`
- `cargo test test_attrs_parse_gap`
- `cargo test test_attrs_parse_labels_quoted`
- `cargo test test_attrs_parse_border_flag`
- `cargo test test_attrs_parse_combined`
- `cargo test toc_directive_parses_section_attr`
- `cargo test xref_parses_label_override`
- `cargo test xref_parses_uri_and_format`
- `cargo test xref`
- `cargo test blockquote_collected_from_directive_block`
- `cargo test blockquote`
- `cargo test source`
- `cargo test chart`
- `cargo test tree`
- `cargo test outline`
- `cargo test toc`
- `cargo test test_collect_directives_region`
- `cargo test include_pin_attribute_parsed`
- `cargo test compile_directive`
- `cargo fmt && cargo test && cargo build && git --no-pager diff --check`

The sibling `mdpath` crate still emits its known warning set during workspace
test/build; this wave did not change sibling repository code.

## Carry-forward

Next directive split work should continue extracting artifact-family renderers
behind the compile facade.
