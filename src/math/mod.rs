/// proof math — re-exports proof_math.
///
/// The LaTeX math rendering implementation lives in the standalone
/// `proof-math` crate. This module re-exports everything for use
/// throughout proof.

pub use proof_math::{
    expand_inline_math,
    render_display_math,
    MathAlign,
    MathDiag,
    DiagSeverity,
};

// Re-export sub-modules for tests that import crate::math::symbols etc.
pub use proof_math::symbols;
pub use proof_math::tokenizer;
pub use proof_math::superscript;
pub use proof_math::tier2;
pub use proof_math::fraction;
pub use proof_math::integral;
pub use proof_math::matrix;
pub use proof_math::render;
