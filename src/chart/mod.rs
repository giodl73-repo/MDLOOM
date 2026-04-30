//! proof:chart — multi-kind ASCII charts (bar, line, area, stacked-bar,
//! waterfall, scatter, heatmap, candlestick, gantt, timeline).
//!
//! Distinct from `proof:element kind=sparkline`: sparkline is a one-line glyph
//! sequence intended for inline use; this module produces multi-line ASCII
//! charts with axes, labels, and titles for use in dashboards and prose docs.

mod bar;
mod line;
mod area;
mod stacked_bar;
mod waterfall;
mod scatter;
mod heatmap;
mod candlestick;
mod gantt;
mod timeline;
pub mod render;

pub use render::{render_chart, ChartAttrs, ChartKind, ChartData, ChartError, ChartPoint};
