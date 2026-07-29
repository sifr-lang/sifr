use super::report::BuildReport;
use std::path::{Path, PathBuf};

pub struct CachedBinaryArtifact {
    pub(super) binary_path: PathBuf,
    pub(super) build_report: BuildReport,
}

impl CachedBinaryArtifact {
    #[must_use]
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    #[must_use]
    pub fn build_report(&self) -> &BuildReport {
        &self.build_report
    }
}
