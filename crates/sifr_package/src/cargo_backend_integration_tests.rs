use crate::cargo::commands::{CargoCommandPlan, CargoFeatureSelection, CargoPackageMutation};
use crate::cargo::errors::{map_cargo_failure, CargoAction};
use crate::cargo::lock_modes::{validate_offline_source_availability, CargoLockMode};
use crate::cargo::metadata::{CargoPackage, CargoPackageId, CargoTarget, NormalizedCargoMetadata};
use crate::cargo::trust::validate_backend_trust;
use crate::graph::derive::{
    BackendCrateMetadata, PackageClassification, SifrPackageGraph, SifrPackageId,
    SifrPackageMetadata,
};
use crate::graph::digest::{digest_package_build_cache_inputs, PackageBuildCacheInputs};
use crate::manifest::metadata::CargoSifrMetadata;
use crate::manifest::sifr::{
    CompilerRequirement, ImportRoot, PackageSourceRoot, PythonConfig, RustInteropConfig,
    SifrEdition, SifrManifest, SifrPackageName, TrustPolicy,
};
use sifr_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

#[test]
fn cargo_command_plans_preserve_lock_mode_and_feature_semantics() {
    let features = CargoFeatureSelection {
        features: vec!["json".to_string(), "tls".to_string()],
        all_features: false,
        no_default_features: true,
    };

    let metadata = CargoCommandPlan::metadata(PathBuf::from("/ws"), CargoLockMode::Frozen);
    assert_eq!(
        metadata.args,
        ["metadata", "--format-version", "1", "--frozen"]
    );

    let build = CargoCommandPlan::build(
        PathBuf::from("/ws"),
        CargoLockMode::Locked,
        &features,
        Some("aarch64-apple-darwin"),
    );
    assert_eq!(
        build.args,
        [
            "build",
            "--locked",
            "--no-default-features",
            "--features",
            "json,tls",
            "--target",
            "aarch64-apple-darwin"
        ]
    );

    let mutation = CargoPackageMutation {
        package_spec: "sifr-json@1".to_string(),
        rename: Some("json_v1".to_string()),
        features: vec!["serde".to_string()],
    };
    let add = CargoCommandPlan::add(PathBuf::from("/ws"), &mutation);
    assert_eq!(
        add.args,
        [
            "add",
            "sifr-json@1",
            "--rename",
            "json_v1",
            "--features",
            "serde"
        ]
    );
}

#[test]
fn offline_mode_reports_missing_sifr_source_package() {
    let metadata = NormalizedCargoMetadata {
        packages: BTreeMap::from([(
            CargoPackageId("registry+sifr-json@1.0.0".to_string()),
            cargo_package("/definitely/not/materialized/sifr-json/Cargo.toml"),
        )]),
        resolve_edges: Vec::new(),
        workspace_members: BTreeSet::new(),
        workspace_default_members: BTreeSet::new(),
        target_directory: PathBuf::from("/ws/target"),
        workspace_root: PathBuf::from("/ws"),
    };

    let diagnostics = validate_offline_source_availability(&metadata, CargoLockMode::Offline)
        .expect_err("offline unavailable source should fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_SOURCE_UNAVAILABLE_OFFLINE
    );
}

#[test]
fn cargo_failure_mapping_redacts_private_credentials() {
    let diagnostic = map_cargo_failure(
        CargoAction::Fetch,
        "403 forbidden for https://user:secret@example.invalid token=abcd",
    );

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED
    );
    assert!(diagnostic.message.contains("[redacted]"));
    assert!(!diagnostic.message.contains("secret"));
    assert!(!diagnostic.message.contains("abcd"));
    assert!(diagnostic.message.contains("403 forbidden"));

    let generic = map_cargo_failure(CargoAction::Metadata, "manifest parse failed");
    assert_eq!(generic.code, DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED);
}

#[test]
fn backend_trust_reports_untrusted_direct_backend_crate() {
    let graph = package_graph(TrustPolicy::default(), vec![backend("reqwest")]);

    let diagnostics = validate_backend_trust(&graph).expect_err("untrusted backend should fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_BACKEND_TRUST_VIOLATION
    );
    assert!(diagnostics[0].message.contains("reqwest"));
}

#[test]
fn backend_trust_rejects_stale_non_direct_trust_entry() {
    let graph = package_graph(
        TrustPolicy {
            native: vec!["unused-native".to_string()],
            build_scripts: Vec::new(),
            proc_macros: Vec::new(),
            python: Vec::new(),
            python_native: Vec::new(),
            rust_build_scripts: Vec::new(),
            rust_proc_macros: Vec::new(),
            native_links: Vec::new(),
            unsafe_rust_bridges: Vec::new(),
            build_env: Vec::new(),
            rust_no_panic: Vec::new(),
            rust_panic_abort: Vec::new(),
        },
        Vec::new(),
    );

    let diagnostics = validate_backend_trust(&graph).expect_err("stale trust should fail");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].code,
        DiagnosticCode::PACKAGE_TRUST_NON_DIRECT_DEPENDENCY
    );
}

#[test]
fn rust_interop_manifest_parses_config_and_trust_policy() {
    let cargo_package_id = CargoPackageId("path+file:///ws/app#sifr-app@0.1.0".to_string());
    let manifest = SifrManifest::parse(
        &cargo_package_id,
        &PathBuf::from("/ws/app/sifr.toml"),
        r#"
[package]
name = "app"
edition = "2026"
sifr-version = ">=0.3,<0.4"

[source]
roots = ["sifr"]

[rust]
bridges = ["src/bridges", "rust/interop"]
direct-crate-bindings = true

[trust]
rust-build-scripts = ["openssl_sys"]
rust-proc-macros = ["serde_derive"]
native-links = ["ssl"]
unsafe-rust-bridges = ["app.hash"]
build-env = ["OPENSSL_DIR"]
rust-no-panic = ["app.hash.digest"]
rust-panic-abort = ["app.exit"]
"#,
    )
    .expect("manifest should parse");

    assert_eq!(manifest.rust.bridges.len(), 2);
    assert!(manifest.rust.direct_crate_bindings);
    assert_eq!(manifest.trust.rust_build_scripts, ["openssl_sys"]);
    assert_eq!(manifest.trust.rust_proc_macros, ["serde_derive"]);
    assert_eq!(manifest.trust.native_links, ["ssl"]);
    assert_eq!(manifest.trust.unsafe_rust_bridges, ["app.hash"]);
    assert_eq!(manifest.trust.build_env, ["OPENSSL_DIR"]);
    assert_eq!(manifest.trust.rust_no_panic, ["app.hash.digest"]);
    assert_eq!(manifest.trust.rust_panic_abort, ["app.exit"]);
    assert!(manifest.declares_rust_backend());
}

#[test]
fn cargo_metadata_parses_native_links_evidence() {
    let metadata = crate::cargo::metadata::parse_metadata_json(
        r#"
{
  "packages": [{
    "id": "path+file:///ws/native#native@0.1.0",
    "name": "native",
    "version": "0.1.0",
    "source": null,
    "links": "native",
    "manifest_path": "/ws/native/Cargo.toml",
    "dependencies": [],
    "targets": [{
      "name": "build-script-build",
      "kind": ["custom-build"],
      "crate_types": ["bin"],
      "src_path": "/ws/native/build.rs"
    }],
    "features": {},
    "metadata": {}
  }],
  "resolve": null,
  "workspace_members": [],
  "workspace_default_members": [],
  "target_directory": "/ws/target",
  "workspace_root": "/ws"
}
"#,
    )
    .expect("metadata should parse");

    let package = metadata.packages.first().expect("package exists");
    assert_eq!(package.links.as_deref(), Some("native"));
    assert!(package
        .targets
        .iter()
        .any(|target| target.kind.contains("custom-build")));
}

#[test]
fn package_build_cache_digest_changes_with_lock_source_and_target_inputs() {
    let mut first = PackageBuildCacheInputs {
        cargo_lock_digest: Some("lock-a".to_string()),
        cargo_metadata_digest: Some("metadata-a".to_string()),
        package_graph_digest: Some("graph-a".to_string()),
        package_source_map_digest: Some("source-map-a".to_string()),
        compiler_version: "0.3.0".to_string(),
        target: Some("aarch64-apple-darwin".to_string()),
        profile: "release".to_string(),
        features: vec!["tls".to_string(), "json".to_string()],
        selectors: vec!["package:app".to_string()],
        ..PackageBuildCacheInputs::default()
    };
    first
        .sifr_source_digests
        .insert("sifr/app/__init__.sifr".to_string(), "src-a".to_string());

    let mut reordered = first.clone();
    reordered.features = vec!["json".to_string(), "tls".to_string()];
    assert_eq!(
        digest_package_build_cache_inputs(&first),
        digest_package_build_cache_inputs(&reordered)
    );

    let mut changed = first.clone();
    changed.cargo_lock_digest = Some("lock-b".to_string());
    assert_ne!(
        digest_package_build_cache_inputs(&first),
        digest_package_build_cache_inputs(&changed)
    );

    let mut python_changed = first.clone();
    python_changed.python_probe_digest = Some("python-probe-b".to_string());
    assert_ne!(
        digest_package_build_cache_inputs(&first),
        digest_package_build_cache_inputs(&python_changed)
    );
}

fn cargo_package(manifest_path: &str) -> CargoPackage {
    CargoPackage {
        id: CargoPackageId("registry+sifr-json@1.0.0".to_string()),
        name: "sifr-json".to_string(),
        version: "1.0.0".to_string(),
        source: Some("registry+https://example.invalid".to_string()),
        links: None,
        manifest_path: PathBuf::from(manifest_path),
        dependencies: Vec::new(),
        targets: Vec::<CargoTarget>::new(),
        features: BTreeMap::new(),
        sifr_metadata: Some(CargoSifrMetadata {
            manifest: PathBuf::from("sifr.toml"),
            aliases: BTreeMap::new(),
        }),
    }
}

fn package_graph(
    trust: TrustPolicy,
    backend_crates: Vec<BackendCrateMetadata>,
) -> SifrPackageGraph {
    let package_id = SifrPackageId("sifr-app@0.1.0#path".to_string());
    let cargo_package_id = CargoPackageId("path+file:///ws/app#sifr-app@0.1.0".to_string());
    let metadata = SifrPackageMetadata {
        package_id: package_id.clone(),
        cargo_package_id: cargo_package_id.clone(),
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
            source_roots: vec![PackageSourceRoot(PathBuf::from("sifr"))],
            exports: vec![ImportRoot("app".to_string())],
            source_features: BTreeMap::new(),
            scripts: BTreeMap::new(),
            dependencies: BTreeMap::new(),
            dev_dependencies: BTreeMap::new(),
            trust,
            python: PythonConfig::default(),
            rust: RustInteropConfig::default(),
            production_schema: false,
        },
        aliases: BTreeMap::new(),
    };

    SifrPackageGraph {
        packages: BTreeMap::from([(package_id.clone(), metadata)]),
        cargo_edges: BTreeMap::new(),
        direct_dependency_scopes: BTreeMap::new(),
        backend_crates: BTreeMap::from([(package_id, backend_crates)]),
        classifications: BTreeMap::from([(
            cargo_package_id,
            PackageClassification::SifrSource(SifrPackageId("sifr-app@0.1.0#path".to_string())),
        )]),
    }
}

fn backend(name: &str) -> BackendCrateMetadata {
    BackendCrateMetadata {
        cargo_package_id: CargoPackageId(format!("registry+{name}@1.0.0")),
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
