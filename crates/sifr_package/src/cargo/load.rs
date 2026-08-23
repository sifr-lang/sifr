use crate::{
    CargoCommandPlan, CargoLockMode, NormalizedCargoMetadata, PackageDiagnostic, PackageSourceMap,
    SifrPackageGraph, derive_package_graph, map_cargo_failure, parse_metadata_json,
    record_cargo_invocation,
};
use sifr_frontend::SourceProvider;
use std::path::Path;
use std::process::Command;

/// Read-only Cargo-backed package graph and Sifr source namespace snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageGraphSnapshot {
    pub metadata: NormalizedCargoMetadata,
    pub graph: SifrPackageGraph,
    pub source_map: PackageSourceMap,
}

/// Structured failure from the shared Cargo-backed graph loader.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageGraphLoadFailure {
    pub plan: CargoCommandPlan,
    pub kind: PackageGraphLoadFailureKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageGraphLoadFailureKind {
    Spawn {
        message: String,
    },
    Command {
        exit_status: Option<i32>,
        output: String,
    },
    Package {
        diagnostics: Vec<PackageDiagnostic>,
        usage_error: bool,
    },
}

impl PackageGraphLoadFailure {
    #[must_use]
    pub fn into_diagnostics(self) -> Vec<PackageDiagnostic> {
        match self.kind {
            PackageGraphLoadFailureKind::Spawn { message } => {
                vec![map_cargo_failure(self.plan.action, &message)]
            }
            PackageGraphLoadFailureKind::Command { output, .. } => {
                vec![map_cargo_failure(self.plan.action, &output)]
            }
            PackageGraphLoadFailureKind::Package { diagnostics, .. } => diagnostics,
        }
    }
}

/// Load the canonical package graph without mutating the lockfile or fetching.
pub fn load_package_graph_snapshot(
    workspace_root: &Path,
    lock_mode: CargoLockMode,
    provider: &mut impl SourceProvider,
) -> Result<PackageGraphSnapshot, PackageGraphLoadFailure> {
    let plan = CargoCommandPlan::metadata(workspace_root.to_path_buf(), lock_mode);
    let mut command = Command::new(&plan.program);
    command.args(&plan.args).current_dir(&plan.current_dir);
    record_cargo_invocation("package-metadata", lock_mode, &command);
    let output = command.output().map_err(|error| PackageGraphLoadFailure {
        plan: plan.clone(),
        kind: PackageGraphLoadFailureKind::Spawn {
            message: error.to_string(),
        },
    })?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.as_ref()
        } else {
            stderr.as_ref()
        };
        return Err(PackageGraphLoadFailure {
            plan,
            kind: PackageGraphLoadFailureKind::Command {
                exit_status: output.status.code(),
                output: detail.to_string(),
            },
        });
    }
    let metadata =
        parse_metadata_json(&String::from_utf8_lossy(&output.stdout)).map_err(|error| {
            PackageGraphLoadFailure {
                plan: plan.clone(),
                kind: PackageGraphLoadFailureKind::Package {
                    diagnostics: vec![error],
                    usage_error: true,
                },
            }
        })?;
    let normalized = metadata.clone().normalize();
    let graph = derive_package_graph(metadata, provider).map_err(|diagnostics| {
        PackageGraphLoadFailure {
            plan: plan.clone(),
            kind: PackageGraphLoadFailureKind::Package {
                diagnostics,
                usage_error: false,
            },
        }
    })?;
    let source_map = PackageSourceMap::build(&graph, provider).map_err(|diagnostics| {
        PackageGraphLoadFailure {
            plan,
            kind: PackageGraphLoadFailureKind::Package {
                diagnostics,
                usage_error: false,
            },
        }
    })?;
    Ok(PackageGraphSnapshot {
        metadata: normalized,
        graph,
        source_map,
    })
}
