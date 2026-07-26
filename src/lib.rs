#![allow(
    clippy::approx_constant,
    clippy::doc_lazy_continuation,
    clippy::empty_line_after_doc_comments,
    clippy::explicit_counter_loop,
    clippy::manual_strip,
    clippy::needless_range_loop,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::redundant_field_names,
    clippy::single_match,
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::unnecessary_map_or,
    clippy::unnecessary_sort_by,
    clippy::unnecessary_to_owned
)]

pub mod ai;
pub mod artifact;
pub mod backfill;
pub mod cache;
pub mod chart;
pub mod checks;
pub mod compile;
pub(crate) mod compile_cache;
#[allow(dead_code)]
pub mod compile_chart;
pub(crate) mod compile_crop;
pub(crate) mod compile_dashboard;
#[allow(dead_code)]
pub mod compile_directive;
pub(crate) mod compile_element;
pub(crate) mod compile_figure;
pub(crate) mod compile_format;
pub(crate) mod compile_math;
pub(crate) mod compile_output;
#[allow(dead_code)]
pub mod compile_prose;
pub(crate) mod compile_region;
pub(crate) mod compile_slides;
#[allow(dead_code)]
pub mod compile_source;
pub(crate) mod compile_symbol;
#[allow(dead_code)]
pub mod compile_toc;
#[allow(dead_code)]
pub mod compile_tree;
pub mod compile_types;
pub(crate) mod compile_validation;
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
mod mdport_output;
pub mod publication;
pub mod publish;
pub mod runner;
pub mod slide;
pub mod spec_gen;
pub mod symbol;
pub mod tree;
pub mod unused;

pub use config::MdloomConfig;
pub use diagnostic::{Diagnostic, RichContext, Severity};
pub use diagnostic_registry::{lookup as lookup_diagnostic_code, DiagnosticCode, DIAGNOSTIC_CODES};
pub use fix::{Confidence, Edit, Fix, FixOptions, FixPlan, FixResult};
pub use lint::lint_paths;
pub use runner::{RunSummary, Runner};
