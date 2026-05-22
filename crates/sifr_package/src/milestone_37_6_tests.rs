use crate::cargo::lock_modes::CargoLockMode;
use crate::cargo::package::{package_dry_run_plan, validate_package_archive, PackageArchiveEntry};
use crate::cargo::trust::validate_backend_trust;
use crate::graph::derive::{
    BackendCrateMetadata, PackageClassification, SifrPackageGraph, SifrPackageId,
    SifrPackageMetadata,
};
use crate::imports::source_map::{
    DottedModulePath, PackageModuleKey, PackageModuleSource, PackageSourceMap,
};
use crate::manifest::sifr::{
    CompilerRequirement, ImportRoot, PackageSourceRoot, SifrEdition, SifrManifest, SifrPackageName,
    TrustPolicy,
};
use crate::ops::publish::{publish_plan, vendor_plan};
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn archive_missing_sifr_source_reports_0401() {
    let package = package(TrustPolicy::default());
    let source_map = PackageSourceMap::default();
    let diagnostics = validate_package_archive(&package, &source_map, &[entry("sifr.toml")])
        .expect_err("archive with no .sifr entries should fail");

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_ARCHIVE_MISSING_SIFR_SOURCE));
}

#[test]
fn archive_missing_required_entry_reports_0403() {
    let package = package(TrustPolicy::default());
    let source_map = source_map(&package);
    let diagnostics = validate_package_archive(&package, &source_map, &[entry("sifr.toml")])
        .expect_err("missing source file should fail");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_INCLUDE_EXCLUDE_OMITS_SOURCE
            && diagnostic.message.contains("sifr/app/__init__.sifr")
    }));
}

#[test]
fn archive_traversal_reports_0402() {
    let package = package(TrustPolicy::default());
    let source_map = source_map(&package);
    let diagnostics = validate_package_archive(
        &package,
        &source_map,
        &[
            entry("sifr.toml"),
            entry("sifr/app/__init__.sifr"),
            entry("../escape.sifr"),
        ],
    )
    .expect_err("traversal path should fail");

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_PUBLISH_VALIDATION_FAILED));
}

#[test]
fn package_dry_run_includes_cargo_package_and_publish_dry_run_commands() {
    let package = package(TrustPolicy {
        native: vec!["reqwest".to_string()],
        build_scripts: Vec::new(),
        proc_macros: Vec::new(),
    });
    let graph = graph(package.clone(), vec![backend("reqwest")]);
    validate_backend_trust(&graph).expect("trust should pass");
    let source_map = source_map(&package);

    let plan = package_dry_run_plan(
        &graph,
        &source_map,
        &package.package_id,
        &[entry("sifr.toml"), entry("sifr/app/__init__.sifr")],
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
        &[entry("sifr.toml"), entry("sifr/app/__init__.sifr")],
        CargoLockMode::Normal,
    )
    .expect_err("untrusted backend should fail dry-run");

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_BACKEND_TRUST_VIOLATION));
}

#[test]
fn publish_and_vendor_plans_delegate_to_cargo_with_redaction_ready_commands() {
    let publish = publish_plan(PathBuf::from("/ws/app"), CargoLockMode::Frozen, true);
    assert_eq!(
        publish.cargo_command.args,
        ["publish", "--frozen", "--dry-run"]
    );

    let vendor = vendor_plan(
        PathBuf::from("/ws"),
        CargoLockMode::Locked,
        PathBuf::from("vendor"),
    );
    assert_eq!(vendor.cargo_command.args, ["vendor", "--locked", "vendor"]);
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
            source_roots: vec![PackageSourceRoot(PathBuf::from("sifr"))],
            exports: vec![ImportRoot("app".to_string())],
            source_features: BTreeMap::new(),
            trust,
            production_schema: false,
        },
        aliases: BTreeMap::new(),
    }
}

fn source_map(package: &SifrPackageMetadata) -> PackageSourceMap {
    let module = PackageModuleSource {
        package_id: package.package_id.clone(),
        cargo_package_id: package.cargo_package_id.clone(),
        module_path: DottedModulePath("app".to_string()),
        file_path: PathBuf::from("/ws/app/sifr/app/__init__.sifr"),
        source_root: PathBuf::from("/ws/app/sifr"),
    };
    PackageSourceMap {
        roots: BTreeMap::new(),
        modules: BTreeMap::from([(
            PackageModuleKey {
                package_id: package.package_id.clone(),
                module_path: module.module_path.clone(),
            },
            module,
        )]),
        public_apis: BTreeMap::new(),
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
        cargo_package_name: name.to_string(),
        cargo_version: "1.0.0".to_string(),
        cargo_source: Some("registry+https://example.invalid".to_string()),
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
