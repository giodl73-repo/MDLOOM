pub mod checks;
pub mod config;
pub mod diagnostic;
pub mod runner;

pub use config::GlintConfig;
pub use diagnostic::{Diagnostic, Severity};
pub use runner::Runner;
