/// mdloom math — re-exports mdloom_math.
///
/// The LaTeX math rendering implementation lives in the standalone
/// `mdloom-math` crate. This module re-exports everything for use
/// throughout mdloom.
pub use mdloom_math::{expand_inline_math, render_display_math, DiagSeverity, MathAlign, MathDiag};

// Re-export sub-modules for tests that import crate::math::symbols etc.
pub use mdloom_math::fraction;
pub use mdloom_math::integral;
pub use mdloom_math::matrix;
pub use mdloom_math::render;
pub use mdloom_math::superscript;
pub use mdloom_math::symbols;
pub use mdloom_math::tier2;
pub use mdloom_math::tokenizer;
