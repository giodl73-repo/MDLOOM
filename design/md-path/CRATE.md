# md-path — Crate Design

## What it is

`md-path` is a standalone Rust library crate that implements the `md://` URI
scheme for addressing elements within markdown documents.

It is deliberately separated from `proof` so that:
- Editors, CI systems, and other tools can adopt `md://` without depending on proof
- The addressing scheme is an open standard; proof is the reference implementation
- The resolver can be tested independently of proof's validation logic

## Crate name

`md-path` — describes what it does: gives markdown content a stable path/address.

## Workspace structure

```
proof/                     ← Cargo workspace root
├── Cargo.toml             ← [workspace] members = ["proof", "md-path"]
├── proof/                 ← The proof CLI crate
│   ├── Cargo.toml         ← depends on md-path
│   └── src/
│       └── main.rs
└── md-path/               ← The URI resolver library crate
    ├── Cargo.toml
    └── src/
        ├── lib.rs         ← public API
        ├── uri.rs         ← FigUri struct + parser
        ├── resolver.rs    ← resolution algorithm
        ├── heading.rs     ← heading normalization + section parsing
        ├── label.rs       ← label detection (3 priority rules)
        ├── selector.rs    ← type/kind/selector/sub-selector parsing
        └── query.rs       ← OData-style query parameter parsing
```

## Public API (sketch)

```rust
// md_path::lib.rs

pub use uri::MdUri;
pub use resolver::{Resolver, ResolvedElement, ElementKind};
pub use error::MdPathError;

/// Parse an md:// URI from a string.
/// Returns Err if the URI is syntactically invalid.
pub fn parse(uri: &str) -> Result<MdUri, MdPathError>;

/// Resolve an md:// URI against a root directory.
/// Returns the resolved element with content and metadata.
pub fn resolve(uri: &MdUri, root: &Path) -> Result<ResolvedElement, MdPathError>;
```

```rust
pub struct MdUri {
    pub path: PathBuf,
    pub heading_path: Vec<String>,        // normalized segments
    pub element_type: Option<ElementType>, // figure, table, chart, text, heading
    pub kind: Option<String>,             // flowchart, key-value, etc.
    pub selector: Selector,               // Named(String) | Index(usize) | None
    pub sub_selectors: Vec<SubSelector>,  // [row=X], [col=Y], [box=Z]
    pub query: Option<QueryParams>,       // ?select, ?filter, ?count, ?top, ?skip
}

pub struct ResolvedElement {
    pub uri: MdUri,
    pub file: PathBuf,
    pub line_start: usize,    // 1-based
    pub line_end: usize,      // 1-based
    pub content: String,      // element content (without fences for figures)
    pub label: Option<String>, // detected label
    pub section_heading: Option<String>,
    pub element_type: ElementType,
    pub kind: Option<String>,
}

pub enum ElementType { Figure, Table, Chart, Text, Heading, Section }
pub enum Selector { Named(String), Index(usize), None }
```

## Design directory

All md-path design docs live in `proof/design/md-path/`:

- `FIG-SPEC.md` — full specification (in `proof/design/`, shared)
- `PITFALLS.md` — this directory: failure mode catalog
- `INVARIANTS.md` — this directory: properties that must always hold
- `CRATE.md` — this file: crate structure

## Dependencies (planned)

```toml
[dependencies]
# Markdown parsing
pulldown-cmark = "0.10"   # CommonMark parser for fence/heading detection

# URI encoding
percent-encoding = "2"    # for encoding/decoding special chars in URIs

# Error handling
thiserror = "2"
anyhow = "1"              # for higher-level error context

# Regex (for label matching, filter expressions)
regex = "1"
```

## Test strategy

Tests live in `md-path/tests/` as integration tests using fixture .md files:

```
md-path/tests/
├── fixtures/
│   ├── label-detection.md       # 12+ figures covering all label rules
│   ├── table-ambiguity.md       # rows/cols with overlapping names
│   ├── nested-headings.md       # 3+ levels of hierarchy
│   ├── figure-kinds.md          # one example of each kind
│   ├── chart-kinds.md           # horizontal/vertical/timeline
│   ├── odata-filters.md         # filter syntax examples
│   ├── davinci-invariants.md    # violation of each invariant rule
│   └── normalization.md         # heading edge cases
├── normalization.rs
├── label_detection.rs
├── selector_parsing.rs
├── resolution.rs
├── sub_selectors.rs
├── query_params.rs
└── error_cases.rs
```

## Status

Not yet implemented. Specification complete. See `proof/design/FIG-SPEC.md`.
