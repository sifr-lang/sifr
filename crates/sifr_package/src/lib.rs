pub mod cargo;
mod compiler_components;
pub mod diag;
mod digest;
pub mod graph;
pub mod imports;
pub mod manifest;
pub mod ops;
pub mod projection;
mod projection_bridge;
mod projection_rust_keywords;
pub mod python;
pub mod source;
mod sql_capabilities;
mod sql_profiles;

pub use crate::cargo::commands::{
    CargoCommandPlan, CargoFeatureSelection, CargoPackageArchiveOptions, CargoPackageMutation,
    CargoPackageSelection, CargoPublishOptions, CargoVendorOptions,
};
pub use crate::cargo::errors::{CargoAction, map_cargo_failure};
#[doc(hidden)]
pub use crate::cargo::invocation_trace::{
    CargoInvocation, capture_cargo_invocations, record_cargo_invocation,
};
pub use crate::cargo::load::{
    PackageGraphLoadFailure, PackageGraphLoadFailureKind, PackageGraphSnapshot,
    load_package_graph_snapshot,
};
pub use crate::cargo::lock_modes::{CargoLockMode, validate_offline_source_availability};
pub use crate::cargo::metadata::{
    CargoDependency, CargoMetadata, CargoPackage, CargoPackageId, CargoResolveEdge, CargoTarget,
    NormalizedCargoMetadata, parse_metadata_json,
};
pub use crate::cargo::package::{
    CargoPackageRole, PackageArchiveEntry, PackageArchiveValidation, PackageDryRunPlan,
    package_dry_run_plan, required_archive_entries, validate_package_archive,
};
pub use crate::cargo::package_lock_drift_reason;
pub use crate::cargo::trust::{BackendTrustSummary, validate_backend_trust};
pub use crate::compiler_components::{
    PackageCompilerComponent, compiler_component_registrations, resolve_package_component,
};
pub use crate::diag::{PackageDiagnostic, PackageDiagnosticOrigin};
pub use crate::graph::changed::{
    ChangedPackageSelection, ChangedPathSelection, select_changed_packages,
};
pub use crate::graph::derive::{
    BackendCrateMetadata, PackageClassification, SifrPackageGraph, SifrPackageId,
    SifrPackageMetadata, derive_package_graph,
};
pub use crate::graph::digest::{
    GraphDigest, PackageBuildCacheInputs, digest_graph_inputs, digest_package_build_cache_inputs,
    digest_package_graph, digest_package_source_map, digest_package_source_snapshot,
    digest_python_authoring_environment_probe, digest_python_environment_probe,
};
pub use crate::graph::filters::{
    PackageFilter, PackageFilterTerm, apply_package_filters, parse_package_filter,
};
pub use crate::graph::scopes::{DirectDependencyScope, ScopedImport, ScopedImportSource};
pub use crate::graph::type_identity::{PackageTypeIdentity, TypeIdentityMismatch};
pub use crate::graph::workspace::{
    WorkspacePackageSelection, explicit_package_selection, select_sifr_workspace_members,
    selected_workspace_members,
};
pub use crate::imports::source_map::{
    DottedModulePath, PackageImportAmbiguity, PackageImportOrigin, PackageImportResolution,
    PackageImportResolutionResult, PackageModuleKey, PackageModuleSource, PackageSourceMap,
};
pub use crate::manifest::CompilerComponentConfig;
pub use crate::manifest::metadata::{CargoSifrAliasMetadata, CargoSifrMetadata};
pub use crate::manifest::package_sections::{SifrDependency, SifrScript};
pub use crate::manifest::sifr::{
    CompilerRequirement, ImportRoot, PackageSourceRoot, PythonConfig, RustInteropConfig,
    SifrEdition, SifrManifest, SifrPackageName, TrustPolicy,
};
pub use crate::manifest::{SchemaSourceKind, SqlConfig, SqlProfileConfig};
pub use crate::ops::publish::{
    PublishPlan, VendorPlan, package_plan, publish_plan, publish_plan_with_options, vendor_plan,
    vendor_plan_with_options,
};
pub use crate::ops::read::{OutdatedPackageReport, OutdatedPackageSource, outdated_query_report};
pub use crate::ops::session::{
    PackageCommandPlan, PackageRunRequest, PackageSession, PackageSessionOptions,
    ResolvedRunTarget, ScriptOrigin,
};
pub use crate::projection::{
    InitPackageKind, InitPackageOptions, ProjectionCheck, ProjectionRepair, check_projection,
    init_package, repair_projection,
};
pub use crate::python::{
    ARROW_CERTIFICATION_SCHEMA_VERSION, ArrowCertification, ArrowCertifiedDistribution,
    ArrowCertifiedIdentityMethod, ArrowCertifiedKind, ArrowCertifiedSchemaMode,
    DeferredPythonEnvironment, DlpackCertification, DlpackCertifiedDevice,
    DlpackCertifiedStreamPolicy, PYTHON_BINDING_SCHEMA_VERSION, PYTHON_BINDINGS_FILE,
    PYTHON_BRIDGE_INVENTORY, PYTHON_BRIDGE_ROOT, PYTHON_BRIDGE_RUNTIME_ROOT,
    PYTHON_CERTIFICATION_SCHEMA_VERSION, PYTHON_CERTIFICATIONS_FILE, PythonBinding,
    PythonBindingArtifact, PythonBindingDistribution, PythonBindingSource, PythonBindingSourceKind,
    PythonBridgeImport, PythonBridgeInventory, PythonBridgeModule, PythonCertificationArtifact,
    PythonDistributionProbe, PythonEnvironmentProbe, PythonEnvironmentProbeRequest,
    PythonEnvironmentResolution, PythonEnvironmentSelection, PythonRequirementContribution,
    PythonRequirementKind, ResolvedPythonBridgeGraph, ResolvedPythonBridgeImport,
    ResolvedPythonBridgeModule, ResolvedPythonBridgePackage, ResolvedPythonEnvironment,
    arrow_fixture_digest, arrow_fixture_path, discover_python_bridge_inventory,
    load_python_bindings, load_python_bindings_for_update, load_python_certifications,
    load_python_certifications_for_dlpack_update, load_python_certifications_for_update,
    probe_python_environment, python_binding_generated_digest, python_binding_source_fingerprint,
    required_python_binding_archive_entries, required_python_certification_archive_entries,
    resolve_python_bridge_graph, resolve_python_environment, resolve_python_environment_for_check,
    resolve_python_environment_with_requirements, resolved_python_bridge_package_key,
    resolved_python_bridge_runtime_package, safe_python_binding_output,
    select_root_python_environment, validate_python_bindings,
    validate_python_bindings_with_generated_source, validate_python_bridge_inventory_manifest,
    validate_python_environment_probe, write_python_bindings, write_python_bridge_inventory,
    write_python_certifications,
};
pub use crate::source::layout::{MarkerValidation, validate_pure_marker_source};
pub use crate::sql_capabilities::{PackageCapabilityResolutionError, ResolvedPackageCapabilities};
pub use crate::sql_profiles::{ResolvedSqlProfile, resolve_sql_profiles};

#[cfg(test)]
mod cargo_backend_integration_tests;
#[cfg(test)]
mod package_dependency_scope_tests;
#[cfg(test)]
mod package_import_resolution_tests;
#[cfg(test)]
mod package_projection_tests;
#[cfg(test)]
mod package_public_api_tests;
#[cfg(test)]
mod package_publish_archive_tests;
#[cfg(test)]
mod package_rust_bridge_archive_tests;
#[cfg(test)]
mod package_session_tests;
#[cfg(test)]
mod package_source_map_tests;
#[cfg(test)]
mod package_verification_matrix_tests;
#[cfg(test)]
mod package_workspace_query_tests;
#[cfg(test)]
mod sql_capabilities_tests;
#[cfg(test)]
mod sql_profile_tests;

#[cfg(test)]
mod tests {
    use crate::cargo::metadata::parse_metadata_json;
    use crate::graph::derive::{PackageClassification, derive_package_graph};
    use crate::graph::digest::digest_graph_inputs;
    use sifr_diagnostics::DiagnosticCode;
    use sifr_frontend::DiskSourceProvider;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn pure_sifr_package_graph_derives_from_cargo_metadata() {
        let temp = TestWorkspace::new("pure_graph");
        let package_root = temp.package("sifr-demo-json");
        write_pure_package(&package_root, "sifr-demo-json", "demo_json");

        let metadata = parse_metadata_json(&metadata_json(&temp.root, &[&package_root]))
            .expect("metadata should parse");
        let graph = derive_package_graph(metadata, &mut DiskSourceProvider::new())
            .expect("graph should derive");

        assert_eq!(graph.packages.len(), 1);
        let package = graph.packages.values().next().expect("package exists");
        assert_eq!(package.sifr_name.0, "demo_json");
        assert_eq!(
            graph.classifications.get(&package.cargo_package_id),
            Some(&PackageClassification::SifrSource(
                package.package_id.clone()
            ))
        );
    }

    #[test]
    fn non_trivial_pure_marker_reports_package_0501() {
        let temp = TestWorkspace::new("bad_marker");
        let package_root = temp.package("sifr-demo-json");
        write_pure_package(&package_root, "sifr-demo-json", "demo_json");
        fs::write(package_root.join("src/lib.rs"), "pub fn hidden() {}\n").expect("write marker");

        let metadata = parse_metadata_json(&metadata_json(&temp.root, &[&package_root]))
            .expect("metadata should parse");
        let diagnostics = derive_package_graph(metadata, &mut DiskSourceProvider::new())
            .expect_err("marker must fail");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            DiagnosticCode::PACKAGE_NON_TRIVIAL_PURE_MARKER
        );
    }

    #[test]
    fn missing_manifest_reports_package_0002() {
        let temp = TestWorkspace::new("missing_manifest");
        let package_root = temp.package("sifr-demo-json");
        write_pure_package(&package_root, "sifr-demo-json", "demo_json");
        fs::remove_file(package_root.join("sifr.toml")).expect("remove manifest");

        let metadata = parse_metadata_json(&metadata_json(&temp.root, &[&package_root]))
            .expect("metadata should parse");
        let diagnostics = derive_package_graph(metadata, &mut DiskSourceProvider::new())
            .expect_err("manifest must fail");

        assert_eq!(
            diagnostics[0].code,
            DiagnosticCode::PACKAGE_MISSING_OR_INVALID_SIFR_MANIFEST
        );
    }

    #[test]
    fn misplaced_compiler_metadata_reports_package_0003() {
        let temp = TestWorkspace::new("misplaced_metadata");
        let package_root = temp.package("sifr-demo-json");
        write_pure_package(&package_root, "sifr-demo-json", "demo_json");

        let json = metadata_json_with_sifr_metadata(
            &temp.root,
            &package_root,
            r#"{"sifr":{"manifest":"sifr.toml","package":{"name":"demo_json"}}}"#,
        );
        let diagnostic = parse_metadata_json(&json).expect_err("misplaced metadata must fail");

        assert_eq!(
            diagnostic.code,
            DiagnosticCode::PACKAGE_UNSUPPORTED_CARGO_SIFR_METADATA
        );
    }

    #[test]
    fn shuffled_cargo_metadata_has_stable_digest() {
        let temp = TestWorkspace::new("stable_digest");
        let first = temp.package("sifr-demo-json");
        let second = temp.package("sifr-demo-http");
        write_pure_package(&first, "sifr-demo-json", "demo_json");
        write_pure_package(&second, "sifr-demo-http", "demo_http");

        let normal = parse_metadata_json(&metadata_json(&temp.root, &[&first, &second]))
            .expect("normal metadata parses")
            .normalize();
        let shuffled = parse_metadata_json(&metadata_json(&temp.root, &[&second, &first]))
            .expect("shuffled metadata parses")
            .normalize();

        assert_eq!(digest_graph_inputs(&normal), digest_graph_inputs(&shuffled));
    }

    fn write_pure_package(package_root: &Path, cargo_name: &str, sifr_name: &str) {
        fs::create_dir_all(package_root.join("src")).expect("create src");
        fs::write(
            package_root.join("src/lib.rs"),
            "// Pure Sifr package marker. Sifr source lives in the sifr.toml source root.\n",
        )
        .expect("write marker");
        fs::write(
            package_root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{cargo_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
            ),
        )
        .expect("write Cargo.toml");
        fs::write(
            package_root.join("sifr.toml"),
            format!(
                "[package]\nname = \"{sifr_name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
            ),
        )
        .expect("write sifr.toml");
        fs::write(package_root.join("src/__init__.sifr"), "").expect("write init");
    }

    fn metadata_json(workspace_root: &Path, package_roots: &[&PathBuf]) -> String {
        let package_json = package_roots
            .iter()
            .map(|package_root| {
                metadata_package_json(package_root, r#"{"sifr":{"manifest":"sifr.toml"}}"#, "[]")
            })
            .collect::<Vec<_>>()
            .join(",");
        let members = package_roots
            .iter()
            .map(|package_root| {
                let name = package_root
                    .file_name()
                    .expect("package name")
                    .to_string_lossy();
                format!(r#""path+file://{}#{}@0.1.0""#, package_root.display(), name)
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{
                "packages":[{package_json}],
                "workspace_members":[{members}],
                "target_directory":"{}/target",
                "workspace_root":"{}"
            }}"#,
            workspace_root.display(),
            workspace_root.display()
        )
    }

    fn metadata_json_with_sifr_metadata(
        workspace_root: &Path,
        package_root: &Path,
        metadata: &str,
    ) -> String {
        format!(
            r#"{{
                "packages":[{}],
                "workspace_members":["path+file://{}#sifr-demo-json@0.1.0"],
                "target_directory":"{}/target",
                "workspace_root":"{}"
            }}"#,
            metadata_package_json(package_root, metadata, "[]"),
            package_root.display(),
            workspace_root.display(),
            workspace_root.display()
        )
    }

    fn metadata_package_json(package_root: &Path, metadata: &str, dependencies: &str) -> String {
        let name = package_root
            .file_name()
            .expect("package name")
            .to_string_lossy();
        format!(
            r#"{{
                "id":"path+file://{}#{}@0.1.0",
                "name":"{}",
                "version":"0.1.0",
                "source":null,
                "manifest_path":"{}/Cargo.toml",
                "dependencies":{dependencies},
                "targets":[{{
                    "name":"{}",
                    "kind":["lib"],
                    "crate_types":["lib"],
                    "src_path":"{}/src/lib.rs"
                }}],
                "features":{{}},
                "metadata":{metadata}
            }}"#,
            package_root.display(),
            name,
            name,
            package_root.display(),
            name,
            package_root.display()
        )
    }

    struct TestWorkspace {
        root: PathBuf,
    }

    impl TestWorkspace {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!("sifr_package_{name}_{nonce}"));
            fs::create_dir_all(&root).expect("create temp workspace");
            Self { root }
        }

        fn package(&self, name: &str) -> PathBuf {
            let path = self.root.join(name);
            fs::create_dir_all(&path).expect("create package");
            path
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
