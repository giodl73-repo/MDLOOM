pub mod ai;
pub mod artifact;
pub mod backfill;
pub mod cache;
pub mod chart;
pub mod checks;
pub mod compile;
#[allow(dead_code)]
pub mod compile_chart;
pub(crate) mod compile_crop;
#[allow(dead_code)]
pub mod compile_directive;
pub(crate) mod compile_math;
#[allow(dead_code)]
pub mod compile_prose;
#[allow(dead_code)]
pub mod compile_source;
pub(crate) mod compile_symbol;
#[allow(dead_code)]
pub mod compile_toc;
#[allow(dead_code)]
pub mod compile_tree;
pub mod config;
pub mod crop_side_info;
pub mod dashboard;
pub mod davinci;
pub mod depends;
pub mod diagnostic;
pub mod diagnostic_registry;
pub mod draft;
pub mod element;
pub mod figure;
pub mod fix;
pub mod frontmatter;
pub mod layout;
pub mod lint;
pub mod math;
pub mod publish;
pub mod runner;
pub mod slide;
pub mod spec_gen;
pub mod symbol;
pub mod tree;
pub mod unused;

pub use config::GlintConfig;
pub use diagnostic::{Diagnostic, RichContext, Severity};
pub use diagnostic_registry::{lookup as lookup_diagnostic_code, DiagnosticCode, DIAGNOSTIC_CODES};
pub use fix::{Confidence, Edit, Fix, FixOptions, FixPlan, FixResult};
pub use lint::lint_paths;
pub use runner::{RunSummary, Runner};
