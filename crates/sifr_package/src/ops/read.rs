use crate::cargo::lock_modes::CargoLockMode;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::{SifrPackageGraph, SifrPackageId};
use crate::ops::plan::{OperationPlan, PackageOperation};

#[must_use]
pub const fn read_graph_operation(lock_mode: CargoLockMode) -> OperationPlan {
    OperationPlan {
        operation: PackageOperation::ReadGraph,
        lock_mode,
        mutates_manifests: false,
        mutates_lockfile: false,
        requires_network: false,
        writes_projection: false,
        manifest_less_mode: false,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutdatedPackageReport {
    pub package_id: SifrPackageId,
    pub current_version: String,
    pub newest_compatible_version: Option<String>,
    pub source: OutdatedPackageSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutdatedPackageSource {
    Registry {
        source: String,
    },
    Git {
        source: String,
        remote_check_allowed: bool,
    },
    PathPinned,
    Unknown,
}

pub fn outdated_query_report(
    graph: &SifrPackageGraph,
    allow_network: bool,
) -> Result<Vec<OutdatedPackageReport>, Vec<PackageDiagnostic>> {
    let mut diagnostics = Vec::new();
    let reports = graph
        .packages
        .values()
        .map(|package| {
            let source = match package.cargo_source.as_deref() {
                None => OutdatedPackageSource::PathPinned,
                Some(source) if source.starts_with("registry+") => {
                    OutdatedPackageSource::Registry {
                        source: source.to_string(),
                    }
                }
                Some(source) if source.starts_with("git+") => OutdatedPackageSource::Git {
                    source: source.to_string(),
                    remote_check_allowed: allow_network,
                },
                Some(source) => {
                    diagnostics.push(PackageDiagnostic::outdated_query_unsupported(
                        &package.cargo_package_id,
                        source,
                    ));
                    OutdatedPackageSource::Unknown
                }
            };
            OutdatedPackageReport {
                package_id: package.package_id.clone(),
                current_version: package.cargo_version.clone(),
                newest_compatible_version: None,
                source,
            }
        })
        .collect::<Vec<_>>();

    if diagnostics.is_empty() {
        Ok(reports)
    } else {
        Err(diagnostics)
    }
}
