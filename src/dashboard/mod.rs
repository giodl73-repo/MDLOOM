pub mod canvas;
pub mod region;

pub use canvas::Canvas;
pub use region::{
    compile_dashboard, parse_dashboard_frontmatter, validate_regions, DashboardError,
    DashboardMeta, RegionGeometry,
};
