use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::diagnostics::RenderedDiagnostic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildCompilationMode {
    SingleFile,
    Project,
    PackageProject,
}

impl BuildCompilationMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SingleFile => "single-file",
            Self::Project => "project",
            Self::PackageProject => "package-project",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildStageReport {
    label: String,
    elapsed: Duration,
}

impl BuildStageReport {
    pub fn new(label: impl Into<String>, elapsed: Duration) -> Self {
        Self {
            label: label.into(),
            elapsed,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }
}

#[derive(Clone, Debug)]
pub struct BuildReport {
    entrypoint_path: PathBuf,
    mode: BuildCompilationMode,
    target: &'static str,
    binary_path: PathBuf,
    binary_size_bytes: Option<u64>,
    total_elapsed: Duration,
    stages: Vec<BuildStageReport>,
    frontend_diagnostics: Vec<RenderedDiagnostic>,
    cache_hit: bool,
}

impl BuildReport {
    pub fn new(
        entrypoint_path: PathBuf,
        mode: BuildCompilationMode,
        binary_path: PathBuf,
        total_elapsed: Duration,
        stages: Vec<BuildStageReport>,
        frontend_diagnostics: Vec<RenderedDiagnostic>,
        cache_hit: bool,
    ) -> Self {
        Self {
            entrypoint_path,
            mode,
            target: "release native",
            binary_size_bytes: binary_size(&binary_path),
            binary_path,
            total_elapsed,
            stages,
            frontend_diagnostics,
            cache_hit,
        }
    }

    pub fn entrypoint_path(&self) -> &Path {
        &self.entrypoint_path
    }

    pub const fn mode(&self) -> BuildCompilationMode {
        self.mode
    }

    pub const fn target(&self) -> &'static str {
        self.target
    }

    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    pub const fn binary_size_bytes(&self) -> Option<u64> {
        self.binary_size_bytes
    }

    pub const fn total_elapsed(&self) -> Duration {
        self.total_elapsed
    }

    pub fn stages(&self) -> &[BuildStageReport] {
        &self.stages
    }

    pub fn frontend_diagnostics(&self) -> &[RenderedDiagnostic] {
        &self.frontend_diagnostics
    }

    pub const fn cache_hit(&self) -> bool {
        self.cache_hit
    }
}

fn binary_size(binary_path: &Path) -> Option<u64> {
    std::fs::metadata(binary_path)
        .ok()
        .filter(std::fs::Metadata::is_file)
        .map(|metadata| metadata.len())
}
