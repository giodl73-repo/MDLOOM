use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,   // 1-based
    pub col: usize,    // 1-based, byte offset
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub file: PathBuf,
    pub span: Span,
    pub end_span: Option<Span>,
    pub severity: Severity,
    pub code: &'static str,
    pub message: String,
    pub note: Option<String>,
}

impl Diagnostic {
    pub fn error(file: PathBuf, line: usize, col: usize, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            file,
            span: Span { line, col },
            end_span: None,
            severity: Severity::Error,
            code,
            message: message.into(),
            note: None,
        }
    }

    pub fn warning(file: PathBuf, line: usize, col: usize, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            file,
            span: Span { line, col },
            end_span: None,
            severity: Severity::Warning,
            code,
            message: message.into(),
            note: None,
        }
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn with_end(mut self, end_line: usize, end_col: usize) -> Self {
        self.end_span = Some(Span { line: end_line, col: end_col });
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
