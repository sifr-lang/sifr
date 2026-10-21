use super::report::BuildReport;
use std::path::{Path, PathBuf};

pub struct CachedBinaryArtifact {
    #[cfg(test)]
    pub(super) generated_project_root: Option<PathBuf>,
    pub(super) _cache_lease: super::workspace::CachedArtifactEntry,
    pub(super) input_leases: super::native_child_leases::NativeChildLeases,
    pub(super) binary_path: PathBuf,
    pub(super) build_report: BuildReport,
}

impl CachedBinaryArtifact {
    /// Create an executable command that retains this cached artifact in its
    /// intended child. A bare path used by Command::new carries no ownership.
    pub fn command(&self) -> std::io::Result<std::process::Command> {
        let mut command = std::process::Command::new(&self.binary_path);
        self._cache_lease.bind_command(&mut command)?;
        self.input_leases.bind(&mut command)?;
        Ok(command)
    }

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
