pub mod cargo;
pub mod diag;
pub mod graph;
pub mod imports;
pub mod manifest;
pub mod ops;
pub mod projection;
pub mod python;
pub mod source;

pub use crate::cargo::commands::{
    CargoCommandPlan, CargoFeatureSelection, CargoPackageArchiveOptions, CargoPackageMutation,
    CargoPackageSelection, CargoPublishOptions, CargoVendorOptions,
};
pub use crate::cargo::errors::{map_cargo_failure, CargoAction};
pub use crate::cargo::lock_modes::{validate_offline_source_availability, CargoLockMode};
pub use crate::cargo::metadata::{
    parse_metadata_json, CargoDependency, CargoMetadata, CargoPackage, CargoPackageId,
    CargoResolveEdge, CargoTarget, NormalizedCargoMetadata,
};
pub use crate::cargo::package::{
    package_dry_run_plan, required_archive_entries, validate_package_archive, CargoPackageRole,
    PackageArchiveEntry, PackageArchiveValidation, PackageDryRunPlan,
};
pub use crate::cargo::trust::{validate_backend_trust, BackendTrustSummary};
pub use crate::diag::{PackageDiagnostic, PackageDiagnosticOrigin};
pub use crate::graph::changed::{
    select_changed_packages, ChangedPackageSelection, ChangedPathSelection,
};
pub use crate::graph::derive::{
    derive_package_graph, BackendCrateMetadata, PackageClassification, SifrPackageGraph,
    SifrPackageId, SifrPackageMetadata,
};
pub use crate::graph::digest::{
    digest_graph_inputs, digest_package_build_cache_inputs, digest_package_graph,
    digest_package_source_map, digest_python_environment_probe, GraphDigest,
    PackageBuildCacheInputs,
};
pub use crate::graph::filters::{
    apply_package_filters, parse_package_filter, PackageFilter, PackageFilterTerm,
};
pub use crate::graph::scopes::{DirectDependencyScope, ScopedImport, ScopedImportSource};
pub use crate::graph::type_identity::{PackageTypeIdentity, TypeIdentityMismatch};
pub use crate::graph::workspace::{
    explicit_package_selection, select_sifr_workspace_members, selected_workspace_members,
    WorkspacePackageSelection,
};
pub use crate::imports::source_map::{
    DottedModulePath, PackageImportAmbiguity, PackageImportOrigin, PackageImportResolution,
    PackageImportResolutionResult, PackageModuleKey, PackageModuleSource, PackageSourceMap,
};
pub use crate::manifest::metadata::{CargoSifrAliasMetadata, CargoSifrMetadata};
pub use crate::manifest::package_sections::{SifrDependency, SifrScript};
pub use crate::manifest::sifr::{
    CompilerRequirement, ImportRoot, PackageSourceRoot, PythonConfig, RustInteropConfig,
    SifrEdition, SifrManifest, SifrPackageName, TrustPolicy,
};
pub use crate::ops::publish::{
    package_plan, publish_plan, publish_plan_with_options, vendor_plan, vendor_plan_with_options,
    PublishPlan, VendorPlan,
};
pub use crate::ops::read::{outdated_query_report, OutdatedPackageReport, OutdatedPackageSource};
pub use crate::ops::session::{
    PackageCommandPlan, PackageRunRequest, PackageSession, PackageSessionOptions,
    ResolvedRunTarget, ScriptOrigin,
};
pub use crate::projection::{
    check_projection, init_package, repair_projection, InitPackageKind, InitPackageOptions,
    ProjectionCheck, ProjectionRepair,
};
pub use crate::python::{
    probe_python_environment, resolve_python_environment, validate_python_environment_probe,
    PythonEnvironmentProbe, PythonEnvironmentProbeRequest, ResolvedPythonEnvironment,
};
pub use crate::source::layout::{validate_pure_marker_source, MarkerValidation};

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
mod package_session_tests;
#[cfg(test)]
mod package_source_map_tests;
#[cfg(test)]
mod package_verification_matrix_tests;
#[cfg(test)]
mod package_workspace_query_tests;

#[cfg(test)]
mod tests {
    use crate::cargo::metadata::parse_metadata_json;
    use crate::graph::derive::{derive_package_graph, PackageClassification};
    use crate::graph::digest::digest_graph_inputs;
    use sifr_diagnostics::DiagnosticCode;
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
        let graph = derive_package_graph(metadata).expect("graph should derive");

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
        let diagnostics = derive_package_graph(metadata).expect_err("marker must fail");

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
        let diagnostics = derive_package_graph(metadata).expect_err("manifest must fail");

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
        fs::create_dir_all(package_root.join(format!("sifr/{sifr_name}"))).expect("create sifr");
        fs::write(
            package_root.join("src/lib.rs"),
            "// Pure Sifr package marker. Sifr source lives in sifr.toml source roots.\n",
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
                "[package]\nname = \"{sifr_name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroots = [\"sifr\"]\n\n[exports]\nmodules = [\"{sifr_name}\"]\n"
            ),
        )
        .expect("write sifr.toml");
        fs::write(
            package_root.join(format!("sifr/{sifr_name}/__init__.sifr")),
            "",
        )
        .expect("write init");
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
