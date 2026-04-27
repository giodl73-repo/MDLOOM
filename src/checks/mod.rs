pub mod ascii_barchart;
pub mod ascii_box;
pub mod ascii_char;
pub mod ascii_flow;
pub mod ascii_tree;
pub mod markdown;
pub mod markdown_table;

use crate::diagnostic::Diagnostic;
use std::path::Path;

pub trait Check: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic>;
}
