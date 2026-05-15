//! proof:chart — multi-kind ASCII charts (bar, line, area, stacked-bar,
//! waterfall, scatter, heatmap, candlestick, gantt, timeline).
//!
//! Distinct from `proof:element kind=sparkline`: sparkline is a one-line glyph
//! sequence intended for inline use; this module produces multi-line ASCII
//! charts with axes, labels, and titles for use in dashboards and prose docs.

mod area;
mod bar;
mod candlestick;
mod gantt;
mod heatmap;
mod line;
pub mod render;
mod scatter;
mod stacked_bar;
mod timeline;
mod waterfall;

pub use render::{render_chart, ChartAttrs, ChartData, ChartError, ChartKind, ChartPoint};
