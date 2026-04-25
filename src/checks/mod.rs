pub mod ascii_box;
pub mod ascii_char;
pub mod ascii_flow;
pub mod markdown;

use crate::diagnostic::Diagnostic;
use std::path::Path;

pub trait Check: Send + Sync {
    fn name(&self) -> &'static str;
    fn check(&self, path: &Path, content: &str) -> Vec<Diagnostic>;
}
