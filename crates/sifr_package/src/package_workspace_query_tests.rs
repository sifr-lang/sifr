use crate::cargo::metadata::{
    CargoPackage, CargoPackageId, CargoResolveEdge, CargoTarget, NormalizedCargoMetadata,
};
use crate::graph::changed::select_changed_packages;
use crate::graph::derive::{
    PackageClassification, SifrPackageGraph, SifrPackageId, SifrPackageMetadata,
};
use crate::graph::filters::{apply_package_filters, parse_package_filter};
use crate::graph::workspace::{explicit_package_selection, select_sifr_workspace_members};
use crate::manifest::sifr::{
    CompilerRequirement, ImportRoot, PackageSourceRoot, PythonConfig, RustInteropConfig,
    SifrEdition, SifrManifest, SifrPackageName, TrustPolicy,
};
use crate::ops::read::{outdated_query_report, OutdatedPackageSource};
use sifr_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[test]
fn filters_select_dependency_and_dependent_closures_with_negation() {
    let (graph, _) = graph_and_metadata();

    let selected = apply_package_filters(
        &graph,
        &[
            parse_package_filter("app...").expect("parse dependency closure"),
            parse_package_filter("!shared").expect("parse negation"),
        ],
    )
    .expect("filters apply");
    assert!(selected.contains(&package_id("app")));
    assert!(selected.contains(&package_id("lib")));
    assert!(!selected.contains(&package_id("shared")));

    let dependents = apply_package_filters(
        &graph,
        &[parse_package_filter("...^shared").expect("parse dependents")],
    )
    .expect("dependents apply");
    assert_eq!(
        dependents,
        BTreeSet::from([package_id("app"), package_id("lib")])
    );
}

#[test]
fn ambiguous_filter_reports_0601() {
    let (mut graph, _) = graph_and_metadata();
    let duplicate = package(
        "other-app",
        "sifr-other-app",
        "app",
        "/ws/other",
        "other_app",
        None,
    );
    graph
        .packages
        .insert(duplicate.package_id.clone(), duplicate);

    let diagnostics = apply_package_filters(
        &graph,
        &[parse_package_filter("app").expect("parse package filter")],
    )
    .expect_err("ambiguous Sifr name should fail");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_SELECTOR_AMBIGUOUS
    );
}

#[test]
fn changed_file_mapping_reports_0603() {
    let (graph, _) = graph_and_metadata();
    let diagnostics = select_changed_packages(&graph, &[PathBuf::from("/outside/file.sifr")])
        .expect_err("outside path should fail");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_CHANGED_FILE_MAPPING_FAILED
    );

    let changed = select_changed_packages(&graph, &[PathBuf::from("/ws/lib/sifr/lib/a.sifr")])
        .expect("package path should map");
    assert_eq!(changed.package_ids, BTreeSet::from([package_id("lib")]));
}

#[test]
fn explicit_rust_only_selection_reports_0102() {
    let (graph, metadata) = graph_and_metadata();
    let diagnostics = explicit_package_selection(&metadata, &graph, &["rust-helper".to_string()])
        .expect_err("Rust-only package selection should fail");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_SELECTED_RUST_ONLY
    );
}

#[test]
fn rust_only_member_depending_on_sifr_reports_0106() {
    let (graph, metadata) = graph_and_metadata();
    let diagnostics =
        select_sifr_workspace_members(&metadata, &graph).expect_err("Rust to Sifr edge fails");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_RUST_ONLY_DEPENDS_ON_SIFR
    );
}

#[test]
fn workspace_duplicate_import_roots_report_0602() {
    let (mut graph, mut metadata) = graph_and_metadata_without_rust_violation();
    let duplicate = package(
        "duplicate",
        "sifr-duplicate",
        "duplicate",
        "/ws/dup",
        "lib",
        None,
    );
    graph.classifications.insert(
        duplicate.cargo_package_id.clone(),
        PackageClassification::SifrSource(duplicate.package_id.clone()),
    );
    graph
        .packages
        .insert(duplicate.package_id.clone(), duplicate);
    metadata
        .workspace_members
        .insert(cargo_id("sifr-duplicate"));

    let diagnostics =
        select_sifr_workspace_members(&metadata, &graph).expect_err("duplicate exports fail");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_DUPLICATE_WORKSPACE_IMPORT_ROOT
    );
}

#[test]
fn workspace_duplicate_sifr_names_report_0607() {
    let (mut graph, mut metadata) = graph_and_metadata_without_rust_violation();
    let duplicate = package(
        "duplicate-name",
        "sifr-duplicate-name",
        "lib",
        "/ws/duplicate-name",
        "duplicate_lib",
        None,
    );
    graph.classifications.insert(
        duplicate.cargo_package_id.clone(),
        PackageClassification::SifrSource(duplicate.package_id.clone()),
    );
    graph
        .packages
        .insert(duplicate.package_id.clone(), duplicate.clone());
    metadata
        .workspace_members
        .insert(duplicate.cargo_package_id.clone());
    metadata.packages.insert(
        duplicate.cargo_package_id.clone(),
        cargo_package(&duplicate),
    );

    let diagnostics =
        select_sifr_workspace_members(&metadata, &graph).expect_err("duplicate names fail");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_DUPLICATE_WORKSPACE_SIFR_NAME
    );
}

#[test]
fn outdated_unknown_source_reports_0604() {
    let mut graph = graph_and_metadata().0;
    let package = graph
        .packages
        .get_mut(&package_id("app"))
        .expect("app package exists");
    package.cargo_source = Some("sparse+custom".to_string());

    let diagnostics = outdated_query_report(&graph, false).expect_err("unknown source should fail");

    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_OUTDATED_QUERY_UNSUPPORTED
    );
}

#[test]
fn outdated_query_classifies_path_registry_and_git_sources_read_only() {
    let graph = graph_and_metadata().0;
    let reports = outdated_query_report(&graph, false).expect("known sources classify");

    assert!(reports
        .iter()
        .any(|report| matches!(report.source, OutdatedPackageSource::PathPinned)));
    assert!(reports
        .iter()
        .any(|report| matches!(report.source, OutdatedPackageSource::Registry { .. })));
    assert!(reports.iter().any(|report| {
        matches!(
            report.source,
            OutdatedPackageSource::Git {
                remote_check_allowed: false,
                ..
            }
        )
    }));
}

fn graph_and_metadata() -> (SifrPackageGraph, NormalizedCargoMetadata) {
    let (graph, mut metadata) = graph_and_metadata_without_rust_violation();
    metadata.resolve_edges.push(CargoResolveEdge {
        from: cargo_id("rust-helper"),
        dependency_name: "sifr-lib".to_string(),
        to: cargo_id("sifr-lib"),
    });
    (graph, metadata)
}

fn graph_and_metadata_without_rust_violation() -> (SifrPackageGraph, NormalizedCargoMetadata) {
    let app = package("app", "sifr-app", "app", "/ws/app", "app", None);
    let lib = package(
        "lib",
        "sifr-lib",
        "lib",
        "/ws/lib",
        "lib",
        Some("registry+https://example.invalid".to_string()),
    );
    let shared = package(
        "shared",
        "sifr-shared",
        "shared",
        "/ws/shared",
        "shared",
        Some("git+https://example.invalid/shared".to_string()),
    );
    let rust_id = cargo_id("rust-helper");

    let graph = SifrPackageGraph {
        packages: BTreeMap::from([
            (app.package_id.clone(), app.clone()),
            (lib.package_id.clone(), lib.clone()),
            (shared.package_id.clone(), shared.clone()),
        ]),
        cargo_edges: BTreeMap::from([
            (
                app.package_id.clone(),
                BTreeSet::from([lib.package_id.clone(), shared.package_id.clone()]),
            ),
            (
                lib.package_id.clone(),
                BTreeSet::from([shared.package_id.clone()]),
            ),
        ]),
        direct_dependency_scopes: BTreeMap::new(),
        backend_crates: BTreeMap::new(),
        classifications: BTreeMap::from([
            (
                app.cargo_package_id.clone(),
                PackageClassification::SifrSource(app.package_id.clone()),
            ),
            (
                lib.cargo_package_id.clone(),
                PackageClassification::SifrSource(lib.package_id.clone()),
            ),
            (
                shared.cargo_package_id.clone(),
                PackageClassification::SifrSource(shared.package_id.clone()),
            ),
            (rust_id.clone(), PackageClassification::BackendRust),
        ]),
    };
    let metadata = NormalizedCargoMetadata {
        packages: BTreeMap::from([
            (app.cargo_package_id.clone(), cargo_package(&app)),
            (lib.cargo_package_id.clone(), cargo_package(&lib)),
            (shared.cargo_package_id.clone(), cargo_package(&shared)),
            (rust_id.clone(), rust_package("rust-helper")),
        ]),
        resolve_edges: Vec::new(),
        workspace_members: BTreeSet::from([
            app.cargo_package_id.clone(),
            lib.cargo_package_id.clone(),
            shared.cargo_package_id.clone(),
            rust_id,
        ]),
        workspace_default_members: BTreeSet::new(),
        target_directory: PathBuf::from("/ws/target"),
        workspace_root: PathBuf::from("/ws"),
    };
    (graph, metadata)
}

fn package(
    id: &str,
    cargo_name: &str,
    sifr_name: &str,
    root: &str,
    export: &str,
    source: Option<String>,
) -> SifrPackageMetadata {
    SifrPackageMetadata {
        package_id: package_id(id),
        cargo_package_id: cargo_id(cargo_name),
        cargo_package_name: cargo_name.to_string(),
        cargo_version: "0.1.0".to_string(),
        cargo_source: source,
        package_root: PathBuf::from(root),
        sifr_manifest: PathBuf::from(root).join("sifr.toml"),
        sifr_name: SifrPackageName(sifr_name.to_string()),
        manifest: SifrManifest {
            package_name: SifrPackageName(sifr_name.to_string()),
            edition: SifrEdition("2026".to_string()),
            compiler_requirement: CompilerRequirement(">=0.3,<0.4".to_string()),
            default_run: None,
            source_roots: vec![PackageSourceRoot(PathBuf::from("sifr"))],
            exports: vec![ImportRoot(export.to_string())],
            source_features: BTreeMap::new(),
            scripts: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            trust: TrustPolicy::default(),
            python: PythonConfig::default(),
            rust: RustInteropConfig::default(),
            production_schema: false,
        },
        aliases: BTreeMap::new(),
    }
}

fn cargo_package(package: &SifrPackageMetadata) -> CargoPackage {
    CargoPackage {
        id: package.cargo_package_id.clone(),
        name: package.cargo_package_name.clone(),
        version: package.cargo_version.clone(),
        source: package.cargo_source.clone(),
        links: None,
        manifest_path: package.package_root.join("Cargo.toml"),
        dependencies: Vec::new(),
        targets: Vec::<CargoTarget>::new(),
        features: BTreeMap::new(),
        sifr_metadata: None,
    }
}

fn rust_package(name: &str) -> CargoPackage {
    CargoPackage {
        id: cargo_id(name),
        name: name.to_string(),
        version: "0.1.0".to_string(),
        source: None,
        links: None,
        manifest_path: PathBuf::from(format!("/ws/{name}/Cargo.toml")),
        dependencies: Vec::new(),
        targets: Vec::<CargoTarget>::new(),
        features: BTreeMap::new(),
        sifr_metadata: None,
    }
}

fn package_id(name: &str) -> SifrPackageId {
    SifrPackageId(format!("sifr-{name}@0.1.0#path"))
}

fn cargo_id(name: &str) -> CargoPackageId {
    CargoPackageId(format!("path+file:///ws/{name}#{name}@0.1.0"))
}
