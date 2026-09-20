use super::report::BuildReport;
use std::path::{Path, PathBuf};

pub struct CachedBinaryArtifact {
    #[cfg(test)]
    pub(super) generated_project_root: Option<PathBuf>,
    pub(super) _cache_lease: super::workspace::CachedArtifactEntry,
    pub(super) binary_path: PathBuf,
    pub(super) build_report: BuildReport,
}

impl CachedBinaryArtifact {
    /// Test observations use the actual materialized Cargo source, independently
    /// of the immutable captured executable bundle.
    #[cfg(test)]
    pub(crate) fn generated_project_root(&self) -> &Path {
        self.generated_project_root
            .as_deref()
            .expect("cached build must retain its actual materialization report")
    }

    #[must_use]
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    #[must_use]
    pub fn build_report(&self) -> &BuildReport {
        &self.build_report
    }
}
