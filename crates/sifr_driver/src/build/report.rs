use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::diagnostics::RenderedDiagnostic;
use sifr_stdlib_manifest::SysrootDependencyPlan;

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
pub struct BuildSysrootReport {
    root: PathBuf,
    toolchain_id: String,
    content_sha256: String,
    dependency_inputs: String,
    dependency_fingerprint: String,
}

impl BuildSysrootReport {
    pub fn from_dependency_plan(dependency_plan: &SysrootDependencyPlan) -> Self {
        Self {
            root: dependency_plan.sysroot_root.clone(),
            toolchain_id: dependency_plan.toolchain_id.clone(),
            content_sha256: dependency_plan.sysroot_content_sha256.clone(),
            dependency_inputs: dependency_plan.dependency_input_fingerprint(),
            dependency_fingerprint: dependency_plan.cache_fingerprint.clone(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn toolchain_id(&self) -> &str {
        &self.toolchain_id
    }

    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    pub fn dependency_inputs(&self) -> &str {
        &self.dependency_inputs
    }

    pub fn dependency_fingerprint(&self) -> &str {
        &self.dependency_fingerprint
    }
}

#[derive(Clone, Debug)]
pub struct BuildReport {
    entrypoint_path: PathBuf,
    mode: BuildCompilationMode,
    target: &'static str,
    sysroot: BuildSysrootReport,
    binary_path: PathBuf,
    binary_size_bytes: Option<u64>,
    total_elapsed: Duration,
    stages: Vec<BuildStageReport>,
    frontend_diagnostics: Vec<RenderedDiagnostic>,
    cache_hit: bool,
    query_signature_artifact_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct BuildReportInput {
    pub entrypoint_path: PathBuf,
    pub mode: BuildCompilationMode,
    pub sysroot: BuildSysrootReport,
    pub binary_path: PathBuf,
    pub total_elapsed: Duration,
    pub stages: Vec<BuildStageReport>,
    pub frontend_diagnostics: Vec<RenderedDiagnostic>,
    pub cache_hit: bool,
    pub query_signature_artifact_path: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PythonInteropCheckReport {
    pub declarations: Vec<PythonDeclarationCheck>,
    pub required_import_roots: Vec<String>,
    pub target_probes: Vec<PythonTargetCheck>,
    pub bridge_package_count: usize,
    pub requires_async_loop: bool,
    pub environment: PythonEnvironmentCheck,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PythonDeclarationCheck {
    pub module_name: Option<String>,
    pub function_name: String,
    pub target: Option<String>,
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PythonTargetCheck {
    pub target: String,
    pub status: PythonTargetCheckStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PythonTargetCheckStatus {
    Deferred,
    Verified,
    RuntimeChecked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PythonEnvironmentCheck {
    NotRequired,
    Deferred,
    Resolved {
        interpreter: PathBuf,
        digest: String,
    },
}

impl BuildReport {
    pub fn new(input: BuildReportInput) -> Self {
        let BuildReportInput {
            entrypoint_path,
            mode,
            sysroot,
            binary_path,
            total_elapsed,
            stages,
            frontend_diagnostics,
            cache_hit,
            query_signature_artifact_path,
        } = input;
        Self {
            entrypoint_path,
            mode,
            target: "release native",
            sysroot,
            binary_size_bytes: binary_size(&binary_path),
            binary_path,
            total_elapsed,
            stages,
            frontend_diagnostics,
            cache_hit,
            query_signature_artifact_path,
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

    pub const fn sysroot(&self) -> &BuildSysrootReport {
        &self.sysroot
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

    pub fn query_signature_artifact_path(&self) -> Option<&Path> {
        self.query_signature_artifact_path.as_deref()
    }
}

fn binary_size(binary_path: &Path) -> Option<u64> {
    std::fs::metadata(binary_path)
        .ok()
        .filter(std::fs::Metadata::is_file)
        .map(|metadata| metadata.len())
}
