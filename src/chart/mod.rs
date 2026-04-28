//! proof:chart — full bar and line charts.
//!
//! Distinct from `proof:element kind=sparkline`: sparkline is a one-line glyph
//! sequence intended for inline use; this module produces multi-line ASCII
//! charts with axes, labels, and titles for use in dashboards and prose docs.

mod bar;
mod line;
pub mod render;

pub use render::{render_chart, ChartAttrs, ChartKind, ChartData, ChartError, ChartPoint};
