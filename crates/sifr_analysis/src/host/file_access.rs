use super::implementation::{AnalysisHost, unknown_file};
use crate::snapshot::AnalysisError;
use sifr_frontend::FileId;
use std::path::Path;

impl AnalysisHost {
    pub fn path_for_file(&self, file: FileId) -> Result<&Path, AnalysisError> {
        self.session
            .context()
            .and_then(|context| context.path_for_file(file))
            .or_else(|| self.stdlib_navigation.path(file.as_u32()))
            .ok_or_else(|| unknown_file(file))
    }

    pub fn stdlib_navigation(&self) -> std::sync::Arc<sifr_driver::StdlibNavigation> {
        self.stdlib_navigation.clone()
    }
    pub fn source_text_for_file(&self, file: FileId) -> Result<&str, AnalysisError> {
        if self.stdlib_navigation.path(file.as_u32()).is_some() {
            return self
                .stdlib_navigation
                .source(file.as_u32())
                .map_err(|message| {
                    AnalysisError::new(crate::snapshot::AnalysisErrorKind::UnknownFile, message)
                })?
                .ok_or_else(|| unknown_file(file));
        }

        self.session
            .context()
            .and_then(|context| context.source_text_for_file(file))
            .ok_or_else(|| unknown_file(file))
    }
}
