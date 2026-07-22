use crate::{
    derive_package_graph, map_cargo_failure, parse_metadata_json, CargoCommandPlan, CargoLockMode,
    NormalizedCargoMetadata, PackageDiagnostic, PackageSourceMap, SifrPackageGraph,
};
use std::path::Path;
use std::process::Command;

/// Read-only Cargo-backed package graph and Sifr source namespace snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageGraphSnapshot {
    pub metadata: NormalizedCargoMetadata,
    pub graph: SifrPackageGraph,
    pub source_map: PackageSourceMap,
}

/// Load the canonical package graph without mutating the lockfile or fetching.
pub fn load_package_graph_snapshot(
    workspace_root: &Path,
    lock_mode: CargoLockMode,
) -> Result<PackageGraphSnapshot, Vec<PackageDiagnostic>> {
    let plan = CargoCommandPlan::metadata(workspace_root.to_path_buf(), lock_mode);
    let output = Command::new(&plan.program)
        .args(&plan.args)
        .current_dir(&plan.current_dir)
        .output()
        .map_err(|error| vec![map_cargo_failure(plan.action, &error.to_string())])?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.as_ref()
        } else {
            stderr.as_ref()
        };
        return Err(vec![map_cargo_failure(plan.action, detail)]);
    }
    let metadata = parse_metadata_json(&String::from_utf8_lossy(&output.stdout))
        .map_err(|error| vec![error])?;
    let normalized = metadata.clone().normalize();
    let graph = derive_package_graph(metadata)?;
    let source_map = PackageSourceMap::build(&graph)?;
    Ok(PackageGraphSnapshot {
        metadata: normalized,
        graph,
        source_map,
    })
}
