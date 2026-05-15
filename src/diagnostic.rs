use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Span {
    pub line: usize, // 1-based
    pub col: usize,  // 1-based, visual column
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// Rich context block — populated by checks, serialized only in --format rich.
/// Gives an AI reviewer everything needed to decide the fix direction without
/// reading the whole file.
#[derive(Debug, Clone, Serialize, Default)]
pub struct RichContext {
    /// For ascii_box errors: the line where this box's top border was found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub box_opens_at: Option<usize>,

    /// The top border line of the box (defines expected column positions).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_line: Option<String>,

    /// Column positions (1-based visual) that column separators must occupy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_cols: Option<Vec<usize>>,

    /// Actual column positions found on the failing line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_cols: Option<Vec<usize>>,

    /// Surrounding file lines: line_number → line_content.
    /// Populated by the runner for all diagnostics (typically ±3 lines).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub lines: BTreeMap<usize, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub span: Span,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_span: Option<Span>,
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Rich context — always computed by checks, only emitted in --format rich.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rich: Option<RichContext>,

    /// Groups related diagnostics from the same source object (same box, table, chart).
    /// Used by `proof draft` to cluster errors for AI review.
    /// Format: "<type>-l<line>" e.g. "box-l38", "table-l12", "chart-l20"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
}

impl Diagnostic {
    pub fn error(
        file: PathBuf,
        line: usize,
        col: usize,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file,
            span: Span { line, col },
            end_span: None,
            severity: Severity::Error,
            code,
            message: message.into(),
            note: None,
            rich: None,
            group_id: None,
        }
    }

    pub fn warning(
        file: PathBuf,
        line: usize,
        col: usize,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file,
            span: Span { line, col },
            end_span: None,
            severity: Severity::Warning,
            code,
            message: message.into(),
            note: None,
            rich: None,
            group_id: None,
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_rich(mut self, ctx: RichContext) -> Self {
        self.rich = Some(ctx);
        self
    }

    pub fn with_group(mut self, group_id: impl Into<String>) -> Self {
        self.group_id = Some(group_id.into());
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}[{}]: {}",
            self.file.display(),
            self.span,
            self.severity,
            self.code,
            self.message
        )?;
        if let Some(note) = &self.note {
            write!(f, "\n  note: {}", note)?;
        }
        Ok(())
    }
}
