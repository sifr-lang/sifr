mod arrow_certification;
mod binding_authoring;
mod binding_validation;
mod bridge_inventory;
mod bridge_resolution;
mod dlpack_certification;
mod environment;
mod probe_validation;
mod requirements;
mod selection;
mod trust_policy;

pub use arrow_certification::{
    ARROW_CERTIFICATION_SCHEMA_VERSION, ArrowCertification, ArrowCertifiedDistribution,
    ArrowCertifiedIdentityMethod, ArrowCertifiedKind, ArrowCertifiedSchemaMode,
    DlpackCertification, DlpackCertifiedDevice, DlpackCertifiedStreamPolicy,
    PYTHON_CERTIFICATION_SCHEMA_VERSION, PYTHON_CERTIFICATIONS_FILE, PythonCertificationArtifact,
    fixture_digest as arrow_fixture_digest, load_python_certifications,
    load_python_certifications_for_dlpack_update, load_python_certifications_for_update,
    required_python_certification_archive_entries, safe_fixture_path as arrow_fixture_path,
    validate_python_certifications, write_python_certifications,
};
pub use binding_authoring::{
    PYTHON_BINDING_SCHEMA_VERSION, PYTHON_BINDINGS_FILE, PythonBinding, PythonBindingArtifact,
    PythonBindingDistribution, PythonBindingSource, PythonBindingSourceKind, load_python_bindings,
    load_python_bindings_for_update, python_binding_generated_digest,
    python_binding_source_fingerprint, required_python_binding_archive_entries,
    safe_python_binding_output, write_python_bindings,
};
pub use binding_validation::{
    validate_python_bindings, validate_python_bindings_with_generated_source,
};
pub use bridge_inventory::{
    PYTHON_BRIDGE_INVENTORY, PYTHON_BRIDGE_ROOT, PythonBridgeImport, PythonBridgeInventory,
    PythonBridgeModule, discover_python_bridge_inventory, required_python_bridge_archive_entries,
    validate_python_bridge_inventory_manifest, write_python_bridge_inventory,
};
pub(crate) use bridge_inventory::{
    python_bridge_projection_diagnostics, repair_python_bridge_inventory,
};
pub use bridge_resolution::{
    PYTHON_BRIDGE_RUNTIME_ROOT, ResolvedPythonBridgeGraph, ResolvedPythonBridgeImport,
    ResolvedPythonBridgeModule, ResolvedPythonBridgePackage, resolve_python_bridge_graph,
    resolved_python_bridge_package_key, resolved_python_bridge_runtime_package,
};
pub use environment::{
    DeferredPythonEnvironment, PythonDistributionProbe, PythonEnvironmentProbe,
    PythonEnvironmentProbeRequest, PythonEnvironmentResolution, PythonImportProbe,
    ResolvedPythonEnvironment, probe_python_environment, resolve_python_environment,
    resolve_python_environment_for_check, resolve_python_environment_with_requirements,
};
pub use probe_validation::validate_python_environment_probe;
pub use requirements::{
    CanonicalPythonRequirement, CanonicalPythonRequirements, PythonRequirementContribution,
    PythonRequirementKind, canonical_python_requirements,
};
pub use selection::{PythonEnvironmentSelection, select_root_python_environment};

#[cfg(test)]
mod arrow_certification_tests;
#[cfg(test)]
mod binding_authoring_tests;
#[cfg(all(test, unix))]
mod bridge_inventory_symlink_tests;
#[cfg(test)]
mod bridge_inventory_tests;
#[cfg(test)]
mod bridge_resolution_tests;
#[cfg(test)]
mod cache_tests;
#[cfg(test)]
mod probe_validation_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod trust_policy_tests;
