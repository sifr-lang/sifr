use super::implementation::{AnalysisHost, unknown_file};
use crate::snapshot::AnalysisError;
use sifr_frontend::FileId;
use std::path::Path;

impl AnalysisHost {
    pub fn path_for_file(&self, file: FileId) -> Result<&Path, AnalysisError> {
        self.session
            .context()
            .and_then(|context| context.path_for_file(file))
            .ok_or_else(|| unknown_file(file))
    }

    pub fn source_text_for_file(&self, file: FileId) -> Result<&str, AnalysisError> {
        self.session
            .context()
            .and_then(|context| context.source_text_for_file(file))
            .ok_or_else(|| unknown_file(file))
    }
}
