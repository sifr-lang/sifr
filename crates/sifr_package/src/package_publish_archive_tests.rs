use crate::cargo::commands::{
    CargoCommandPlan, CargoFeatureSelection, CargoPackageArchiveOptions, CargoPackageSelection,
    CargoPublishOptions, CargoVendorOptions,
};
use crate::cargo::lock_modes::CargoLockMode;
use crate::cargo::package::{package_dry_run_plan, validate_package_archive, PackageArchiveEntry};
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
    CompilerRequirement, ImportRoot, PackageSourceRoot, PythonConfig, SifrEdition, SifrManifest,
    SifrPackageName, TrustPolicy,
};
use crate::ops::plan::PackageOperation;
use crate::ops::publish::{publish_plan, vendor_plan};
use crate::ops::session::PackageSession;
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
            && diagnostic.message.contains("src/__init__.sifr")
    }));
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

    assert!(diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_ARCHIVE_TRAVERSAL));
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
        build_scripts: Vec::new(),
        proc_macros: Vec::new(),
        python: Vec::new(),
        python_native: Vec::new(),
    });
    let graph = graph(package.clone(), vec![backend("reqwest")]);
    validate_backend_trust(&graph).expect("trust should pass");
    let source_map = source_map(&package);

    let plan = package_dry_run_plan(
        &graph,
        &source_map,
        &package.package_id,
        &[
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

#[test]
fn package_publish_vendor_command_plans_cover_release_flags() {
    let features = CargoFeatureSelection {
        features: vec!["serde".to_string(), "json".to_string()],
        all_features: false,
        no_default_features: true,
    };
    let selection = CargoPackageSelection {
        workspace: true,
        packages: vec!["sifr-app".to_string()],
        excludes: vec!["sifr-tools".to_string()],
    };
    let package = CargoCommandPlan::package_with_options(
        PathBuf::from("/ws"),
        CargoLockMode::Frozen,
        &features,
        &selection,
        &CargoPackageArchiveOptions {
            list: true,
            no_verify: true,
            no_metadata: true,
            allow_dirty: true,
            exclude_lockfile: true,
        },
    );
    assert_eq!(
        package.args,
        [
            "package",
            "--frozen",
            "--workspace",
            "-p",
            "sifr-app",
            "--exclude",
            "sifr-tools",
            "--no-default-features",
            "--features",
            "json,serde",
            "--list",
            "--no-verify",
            "--no-metadata",
            "--allow-dirty",
            "--exclude-lockfile"
        ]
    );

    let publish = CargoCommandPlan::publish_with_options(
        PathBuf::from("/ws"),
        CargoLockMode::Locked,
        &features,
        &selection,
        &CargoPublishOptions {
            dry_run: true,
            no_verify: true,
            allow_dirty: true,
        },
    );
    assert_eq!(
        publish.args,
        [
            "publish",
            "--locked",
            "--workspace",
            "-p",
            "sifr-app",
            "--exclude",
            "sifr-tools",
            "--no-default-features",
            "--features",
            "json,serde",
            "--dry-run",
            "--no-verify",
            "--allow-dirty"
        ]
    );

    let vendor = CargoCommandPlan::vendor_with_options(
        PathBuf::from("/ws"),
        CargoLockMode::Offline,
        &PathBuf::from("vendor"),
        &CargoVendorOptions {
            sync: vec![PathBuf::from("member/Cargo.toml")],
            no_delete: true,
            respect_source_config: true,
            versioned_dirs: true,
        },
    );
    assert_eq!(
        vendor.args,
        [
            "vendor",
            "--offline",
            "--sync",
            "member/Cargo.toml",
            "--no-delete",
            "--respect-source-config",
            "--versioned-dirs",
            "vendor"
        ]
    );
}

#[test]
fn package_publish_vendor_session_plans_route_through_package_session() {
    let session = PackageSession {
        workspace_root: PathBuf::from("/ws"),
        manifest_path: Some(PathBuf::from("/ws/sifr.toml")),
        source_root: Some(PathBuf::from("/ws/src")),
        source_roots: vec![PathBuf::from("/ws/src")],
        manifest_less_mode: false,
        lock_mode: CargoLockMode::Locked,
        manifest: None,
    };
    let selection = CargoPackageSelection {
        workspace: false,
        packages: vec!["sifr-app".to_string()],
        excludes: Vec::new(),
    };
    let package = session.plan_package(
        &CargoFeatureSelection::default(),
        &selection,
        &CargoPackageArchiveOptions::default(),
    );
    assert_eq!(package.operation.operation, PackageOperation::Package);
    assert_eq!(
        package.cargo.expect("package cargo plan").args,
        ["package", "--locked", "-p", "sifr-app"]
    );

    let publish = session.plan_publish(
        &CargoFeatureSelection::default(),
        &selection,
        &CargoPublishOptions {
            dry_run: true,
            ..CargoPublishOptions::default()
        },
    );
    assert_eq!(publish.operation.operation, PackageOperation::Publish);
    assert_eq!(
        publish.cargo.expect("publish cargo plan").args,
        ["publish", "--locked", "-p", "sifr-app", "--dry-run"]
    );

    let vendor = session.plan_vendor(
        &PathBuf::from("vendor"),
        &CargoVendorOptions {
            versioned_dirs: true,
            ..CargoVendorOptions::default()
        },
    );
    assert_eq!(vendor.operation.operation, PackageOperation::Vendor);
    assert_eq!(
        vendor.cargo.expect("vendor cargo plan").args,
        ["vendor", "--locked", "--versioned-dirs", "vendor"]
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
            source_roots: vec![PackageSourceRoot(PathBuf::from("src"))],
            exports: vec![ImportRoot("app".to_string())],
            source_features: BTreeMap::new(),
            scripts: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            trust,
            python: PythonConfig::default(),
            production_schema: false,
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
