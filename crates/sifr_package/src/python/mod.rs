mod bridge_inventory;
mod bridge_resolution;
mod environment;
mod probe_validation;
mod requirements;
mod selection;
mod trust_policy;

pub use bridge_inventory::{
    discover_python_bridge_inventory, required_python_bridge_archive_entries,
    validate_python_bridge_inventory_manifest, write_python_bridge_inventory, PythonBridgeImport,
    PythonBridgeInventory, PythonBridgeModule, PYTHON_BRIDGE_INVENTORY, PYTHON_BRIDGE_ROOT,
};
pub(crate) use bridge_inventory::{
    python_bridge_projection_diagnostics, repair_python_bridge_inventory,
};
pub use bridge_resolution::{
    resolve_python_bridge_graph, resolved_python_bridge_package_key,
    resolved_python_bridge_runtime_package, ResolvedPythonBridgeGraph, ResolvedPythonBridgeImport,
    ResolvedPythonBridgeModule, ResolvedPythonBridgePackage, PYTHON_BRIDGE_RUNTIME_ROOT,
};
pub use environment::{
    probe_python_environment, resolve_python_environment,
    resolve_python_environment_with_requirements, PythonDistributionProbe, PythonEnvironmentProbe,
    PythonEnvironmentProbeRequest, PythonImportProbe, ResolvedPythonEnvironment,
};
pub use probe_validation::validate_python_environment_probe;
pub use requirements::{
    canonical_python_requirements, CanonicalPythonRequirement, CanonicalPythonRequirements,
    PythonRequirementContribution, PythonRequirementKind,
};

#[cfg(all(test, unix))]
mod bridge_inventory_symlink_tests;
#[cfg(test)]
mod bridge_inventory_tests;
#[cfg(test)]
mod bridge_resolution_tests;
#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod trust_policy_tests;
