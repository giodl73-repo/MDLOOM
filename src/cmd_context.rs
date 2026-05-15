use std::path::PathBuf;

pub(crate) struct GlobalOptions {
    config: Option<PathBuf>,
    format: String,
    errors_only: bool,
    no_fail: bool,
    output: Option<PathBuf>,
}

impl GlobalOptions {
    pub(crate) fn new(
        config: Option<PathBuf>,
        format: String,
        errors_only: bool,
        no_fail: bool,
        output: Option<PathBuf>,
    ) -> Self {
        Self {
            config,
            format,
            errors_only,
            no_fail,
            output,
        }
    }

    pub(crate) fn config(&self) -> &Option<PathBuf> {
        &self.config
    }

    pub(crate) fn format(&self) -> &str {
        &self.format
    }

    pub(crate) fn errors_only(&self) -> bool {
        self.errors_only
    }

    pub(crate) fn no_fail(&self) -> bool {
        self.no_fail
    }

    pub(crate) fn output(&self) -> &Option<PathBuf> {
        &self.output
    }
}
