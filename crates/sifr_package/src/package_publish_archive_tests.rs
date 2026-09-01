use crate::cargo::lock_modes::CargoLockMode;
use crate::cargo::package::{
    PackageArchiveEntry, package_dry_run_plan, required_archive_entries, validate_package_archive,
};
use crate::cargo::trust::validate_backend_trust;
use crate::diag::PackageDiagnostic;
use crate::graph::derive::{
    BackendCrateMetadata, PackageClassification, SifrPackageGraph, SifrPackageId,
    SifrPackageMetadata,
};
use crate::imports::source_map::{
    DottedModulePath, PackageModuleKey, PackageModuleSource, PackageSourceMap,
};
use crate::manifest::sifr::{
    CompilerRequirement, PackageSourceRoot, PythonConfig, RustInteropConfig, SifrEdition,
    SifrManifest, SifrPackageName, TrustPolicy,
};
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeMap;
use std::path::PathBuf;

mod command_plan_tests;
#[test]
fn archive_missing_sifr_source_reports_0401() {
    let package = package(TrustPolicy::default());
    let source_map = PackageSourceMap::default();
    let diagnostics = validate_package_archive(&package, &source_map, &[entry("sifr.toml")])
        .expect_err("archive with no .sifr entries should fail");

    assert!(
        diagnostics.iter().any(
            |diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_ARCHIVE_MISSING_SIFR_SOURCE
        )
    );
}

#[test]
fn archive_missing_required_entry_reports_0403() {
    let package = package(TrustPolicy::default());
    let source_map = source_map(&package);
    let diagnostics = validate_package_archive(&package, &source_map, &[entry("sifr.toml")])
        .expect_err("missing source file should fail");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_INCLUDE_EXCLUDE_OMITS_SOURCE
            && diagnostic.message.contains("src/__init__.sifr")
    }));
}

#[test]
fn archive_requires_checked_in_schema_profile_sources() {
    let mut package = package(TrustPolicy::default());
    package.manifest.sql = SifrManifest::parse(
        &package.cargo_package_id,
        &package.sifr_manifest,
        r#"[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[sql.profiles.app]
family = "postgresql"
provider = "postgres"
source = ["db/schema.sql", "db/types.sql"]
server-version = "18"
pooling = "session"
schema-evidence = "migration-head"
schema-strictness = "exact"
"#,
    )
    .expect("profile manifest")
    .sql;
    let required = required_archive_entries(&package, &source_map(&package));
    assert!(required.contains(&PathBuf::from("db/schema.sql")));
    assert!(required.contains(&PathBuf::from("db/types.sql")));
}

#[test]
fn archive_traversal_reports_0404() {
    let package = package(TrustPolicy::default());
    let source_map = source_map(&package);
    let diagnostics = validate_package_archive(
        &package,
        &source_map,
        &[
            entry("sifr.toml"),
            entry("src/__init__.sifr"),
            entry("src/main.sifr"),
            entry("../escape.sifr"),
        ],
    )
    .expect_err("traversal path should fail");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_ARCHIVE_TRAVERSAL)
    );
}

#[test]
fn publish_validation_failed_reports_0402() {
    let diagnostic = PackageDiagnostic::publish_validation_failed(
        &cargo_id("sifr-app"),
        "registry rejected package metadata",
    );

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_PUBLISH_VALIDATION_FAILED
    );
    assert!(diagnostic.message.contains("registry rejected"));
}

#[test]
fn package_dry_run_includes_cargo_package_and_publish_dry_run_commands() {
    let package = package(TrustPolicy {
        native: vec!["reqwest".to_string()],
        ..TrustPolicy::default()
    });
    let graph = graph(package.clone(), vec![backend("reqwest")]);
    validate_backend_trust(&graph).expect("trust should pass");
    let source_map = source_map(&package);

    let plan = package_dry_run_plan(
        &graph,
        &source_map,
        &package.package_id,
        &[
            entry("Cargo.toml"),
            entry("sifr.toml"),
            entry("src/__init__.sifr"),
            entry("src/main.sifr"),
        ],
        CargoLockMode::Locked,
    )
    .expect("dry-run plan should validate");

    assert_eq!(plan.cargo_package.args, ["package", "--locked"]);
    assert_eq!(
        plan.cargo_publish_dry_run.args,
        ["publish", "--locked", "--dry-run"]
    );
}

#[test]
fn package_dry_run_reports_backend_trust_failures_before_publish() {
    let package = package(TrustPolicy::default());
    let graph = graph(package.clone(), vec![backend("reqwest")]);
    let source_map = source_map(&package);

    let diagnostics = package_dry_run_plan(
        &graph,
        &source_map,
        &package.package_id,
        &[
            entry("sifr.toml"),
            entry("src/__init__.sifr"),
            entry("src/main.sifr"),
        ],
        CargoLockMode::Normal,
    )
    .expect_err("untrusted backend should fail dry-run");

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_BACKEND_TRUST_VIOLATION)
    );
}

fn package(trust: TrustPolicy) -> SifrPackageMetadata {
    SifrPackageMetadata {
        package_id: SifrPackageId("sifr-app@0.1.0#path".to_string()),
        cargo_package_id: cargo_id("sifr-app"),
        cargo_package_name: "sifr-app".to_string(),
        cargo_version: "0.1.0".to_string(),
        cargo_source: None,
        package_root: PathBuf::from("/ws/app"),
        sifr_manifest: PathBuf::from("/ws/app/sifr.toml"),
        sifr_name: SifrPackageName("app".to_string()),
        manifest: SifrManifest {
            package_name: SifrPackageName("app".to_string()),
            edition: SifrEdition("2026".to_string()),
            compiler_requirement: CompilerRequirement(">=0.3,<0.4".to_string()),
            default_run: None,
            source_root: PackageSourceRoot(PathBuf::from("src")),
            source_features: BTreeMap::new(),
            scripts: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            compiler_components: BTreeMap::new(),
            sql: crate::SqlConfig::default(),
            trust,
            python: PythonConfig::default(),
            rust: RustInteropConfig::default(),
        },
        aliases: BTreeMap::new(),
    }
}
fn source_map(package: &SifrPackageMetadata) -> PackageSourceMap {
    let init = PackageModuleSource {
        package_id: package.package_id.clone(),
        cargo_package_id: package.cargo_package_id.clone(),
        module_path: DottedModulePath("app".to_string()),
        file_path: PathBuf::from("/ws/app/src/__init__.sifr"),
        source_root: PathBuf::from("/ws/app/src"),
    };
    let main = PackageModuleSource {
        package_id: package.package_id.clone(),
        cargo_package_id: package.cargo_package_id.clone(),
        module_path: DottedModulePath("app.main".to_string()),
        file_path: PathBuf::from("/ws/app/src/main.sifr"),
        source_root: PathBuf::from("/ws/app/src"),
    };
    PackageSourceMap {
        roots: BTreeMap::new(),
        modules: BTreeMap::from([
            (
                PackageModuleKey {
                    package_id: package.package_id.clone(),
                    module_path: init.module_path.clone(),
                },
                init,
            ),
            (
                PackageModuleKey {
                    package_id: package.package_id.clone(),
                    module_path: main.module_path.clone(),
                },
                main,
            ),
        ]),
        ambiguous_modules: BTreeMap::new(),
        public_apis: BTreeMap::new(),
        fatal_diagnostics: Vec::new(),
    }
}
fn graph(
    package: SifrPackageMetadata,
    backend_crates: Vec<BackendCrateMetadata>,
) -> SifrPackageGraph {
    SifrPackageGraph {
        packages: BTreeMap::from([(package.package_id.clone(), package.clone())]),
        cargo_edges: BTreeMap::new(),
        direct_dependency_scopes: BTreeMap::new(),
        backend_crates: BTreeMap::from([(package.package_id.clone(), backend_crates)]),
        classifications: BTreeMap::from([(
            package.cargo_package_id.clone(),
            PackageClassification::SifrSource(package.package_id.clone()),
        )]),
    }
}

fn backend(name: &str) -> BackendCrateMetadata {
    BackendCrateMetadata {
        cargo_package_id: cargo_id(name),
        dependency_name: name.to_string(),
        dependency_kind: None,
        cargo_package_name: name.to_string(),
        cargo_version: "1.0.0".to_string(),
        cargo_source: Some("registry+https://example.invalid".to_string()),
        cargo_manifest_path: PathBuf::from(format!("/ws/{name}/Cargo.toml")),
        links: None,
        has_build_script: false,
        has_proc_macro: false,
    }
}

fn entry(path: &str) -> PackageArchiveEntry {
    PackageArchiveEntry {
        relative_path: PathBuf::from(path),
    }
}

fn cargo_id(name: &str) -> crate::CargoPackageId {
    crate::CargoPackageId(format!("path+file:///ws/{name}#{name}@0.1.0"))
}
