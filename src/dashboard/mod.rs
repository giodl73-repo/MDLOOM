pub mod canvas;
pub mod region;

pub use canvas::Canvas;
pub use region::{DashboardMeta, RegionGeometry, DashboardError,
                 parse_dashboard_frontmatter, validate_regions, compile_dashboard};
