use crate::cargo::metadata::CargoPackageId;
use crate::cargo::package::{
    PackageArchiveEntry, required_archive_entries, validate_package_archive,
};
use crate::graph::derive::{SifrPackageId, SifrPackageMetadata};
use crate::imports::source_map::{
    DottedModulePath, PackageModuleKey, PackageModuleSource, PackageSourceMap,
};
use crate::manifest::sifr::{
    CompilerRequirement, PackageSourceRoot, RustInteropConfig, SifrEdition, SifrManifest,
    SifrPackageName, TrustPolicy,
};
use sifr_diagnostics::DiagnosticCode;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn rust_bridge_archive_requires_managed_projection_and_user_bridge_files() {
    let fixture = RustBridgeFixture::new("archive_required_bridge_entries");
    let package = fixture.package();
    let source_map = fixture.source_map(&package);

    let required = required_archive_entries(&package, &source_map);

    assert!(required.contains(&PathBuf::from("Cargo.toml")));
    assert!(required.contains(&PathBuf::from("sifr.toml")));
    assert!(required.contains(&PathBuf::from("src/__init__.sifr")));
    assert!(required.contains(&PathBuf::from("src/lib.rs")));
    assert!(required.contains(&PathBuf::from("src/bridges/mod.rs")));
    assert!(required.contains(&PathBuf::from("src/bridges/tokenizer.rs")));
    assert!(required.contains(&PathBuf::from("src/__sifr_bridge/mod.rs")));
}

#[test]
fn rust_bridge_archive_validation_reports_missing_bridge_projection_entry() {
    let fixture = RustBridgeFixture::new("archive_missing_bridge_entries");
    let package = fixture.package();
    let source_map = fixture.source_map(&package);

    let diagnostics = validate_package_archive(
        &package,
        &source_map,
        &[
            entry("Cargo.toml"),
            entry("sifr.toml"),
            entry("src/__init__.sifr"),
            entry("src/lib.rs"),
            entry("src/bridges/tokenizer.rs"),
        ],
    )
    .expect_err("missing managed bridge projection should fail");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_INCLUDE_EXCLUDE_OMITS_SOURCE
            && diagnostic.message.contains("src/bridges/mod.rs")
    }));
}

struct RustBridgeFixture {
    root: PathBuf,
}

impl RustBridgeFixture {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("sifr_package_{name}_{nonce}"));
        fs::create_dir_all(root.join("src/bridges")).expect("create bridges");
        fs::create_dir_all(root.join("src/__sifr_bridge")).expect("create generated bridge dir");
        fs::write(root.join("src/__init__.sifr"), "").expect("write source");
        fs::write(
            root.join("src/lib.rs"),
            "// sifr-managed: rust-interop bridge projection v1\n",
        )
        .expect("write lib projection");
        fs::write(
            root.join("src/bridges/mod.rs"),
            "// sifr-managed: rust-interop bridge projection v1\npub mod tokenizer;\n",
        )
        .expect("write bridge projection");
        fs::write(
            root.join("src/bridges/tokenizer.rs"),
            "pub fn tokenize() {}\n",
        )
        .expect("write user bridge");
        fs::write(
            root.join("src/__sifr_bridge/mod.rs"),
            "// sifr-managed: rust-interop bridge projection v1\n",
        )
        .expect("write generated bridge projection");
        Self { root }
    }

    fn package(&self) -> SifrPackageMetadata {
        SifrPackageMetadata {
            package_id: package_id(),
            cargo_package_id: cargo_id(),
            cargo_package_name: "sifr-demo-json".to_string(),
            cargo_version: "0.1.0".to_string(),
            cargo_source: None,
            package_root: self.root.clone(),
            sifr_manifest: self.root.join("sifr.toml"),
            sifr_name: SifrPackageName("demo_json".to_string()),
            manifest: manifest(),
            aliases: BTreeMap::new(),
        }
    }

    fn source_map(&self, package: &SifrPackageMetadata) -> PackageSourceMap {
        let mut source_map = PackageSourceMap::default();
        source_map.modules.insert(
            PackageModuleKey {
                package_id: package.package_id.clone(),
                module_path: DottedModulePath("demo_json".to_string()),
            },
            PackageModuleSource {
                package_id: package.package_id.clone(),
                cargo_package_id: package.cargo_package_id.clone(),
                module_path: DottedModulePath("demo_json".to_string()),
                file_path: self.root.join("src/__init__.sifr"),
                source_root: self.root.join("src"),
            },
        );
        source_map
    }
}

impl Drop for RustBridgeFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn manifest() -> SifrManifest {
    SifrManifest {
        package_name: SifrPackageName("demo_json".to_string()),
        edition: SifrEdition("2026".to_string()),
        compiler_requirement: CompilerRequirement(">=0.3,<0.4".to_string()),
        default_run: None,
        source_root: PackageSourceRoot(PathBuf::from("src")),
        source_features: BTreeMap::new(),
        scripts: BTreeMap::new(),
        dependencies: BTreeMap::new(),
        dev_dependencies: BTreeMap::new(),
        compiler_components: BTreeMap::new(),
        trust: TrustPolicy::default(),
        python: crate::manifest::sifr::PythonConfig::default(),
        rust: RustInteropConfig {
            bridges: vec![PathBuf::from("src/bridges")],
            direct_crate_bindings: false,
        },
    }
}

fn package_id() -> SifrPackageId {
    SifrPackageId("sifr-demo-json@0.1.0#path".to_string())
}

fn cargo_id() -> CargoPackageId {
    CargoPackageId("path+file:///tmp/sifr-demo-json#0.1.0".to_string())
}

fn entry(path: &str) -> PackageArchiveEntry {
    PackageArchiveEntry {
        relative_path: PathBuf::from(path),
    }
}
