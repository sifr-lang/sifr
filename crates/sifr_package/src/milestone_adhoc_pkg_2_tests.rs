use crate::{
    check_projection, init_package, repair_projection, InitPackageKind, InitPackageOptions,
};
use sifr_diagnostics::DiagnosticCode;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn init_lib_creates_canonical_src_layout_and_cargo_projection() {
    let temp = TestWorkspace::new("init_lib_projection");
    let package = temp.root.join("demo_json");
    let written = init_package(&InitPackageOptions {
        target_dir: package.clone(),
        sifr_name: "demo_json".to_string(),
        kind: InitPackageKind::Lib,
        force: false,
    })
    .expect("init succeeds");

    assert!(written.contains(&package.join("sifr.toml")));
    assert!(package.join("src/__init__.sifr").is_file());
    assert!(package.join("src/lib.rs").is_file());
    let cargo = fs::read_to_string(package.join("Cargo.toml")).expect("Cargo.toml exists");
    assert!(cargo.contains("name = \"sifr-demo-json\""));
    assert!(cargo.contains("[package.metadata.sifr]"));
    assert!(cargo.contains("manifest = \"sifr.toml\""));
    assert!(cargo.contains("src/**/*.sifr"));
    assert!(check_projection(&package).diagnostics.is_empty());
}

#[test]
fn cargo_projection_init_bin_creates_main_target_without_manifest_bin_table() {
    let temp = TestWorkspace::new("init_bin_projection");
    let package = temp.root.join("demo_app");
    init_package(&InitPackageOptions {
        target_dir: package.clone(),
        sifr_name: "demo_app".to_string(),
        kind: InitPackageKind::Bin,
        force: false,
    })
    .expect("init succeeds");

    assert!(package.join("src/main.sifr").is_file());
    assert!(!fs::read_to_string(package.join("sifr.toml"))
        .expect("manifest exists")
        .contains("[[bin]]"));
}

#[test]
fn cargo_projection_repair_check_reports_missing_manifest_pointer_0703() {
    let temp = TestWorkspace::new("repair_missing_pointer");
    let package = temp.root.join("demo_json");
    init_package(&InitPackageOptions {
        target_dir: package.clone(),
        sifr_name: "demo_json".to_string(),
        kind: InitPackageKind::Lib,
        force: false,
    })
    .expect("init succeeds");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"sifr-demo-json\"\n",
    )
    .expect("break projection");

    let check = check_projection(&package);

    assert!(check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT
    }));
}

#[test]
fn cargo_projection_repair_check_reports_missing_required_include_0704() {
    let temp = TestWorkspace::new("repair_missing_include");
    let package = temp.root.join("demo_json");
    init_package(&InitPackageOptions {
        target_dir: package.clone(),
        sifr_name: "demo_json".to_string(),
        kind: InitPackageKind::Lib,
        force: false,
    })
    .expect("init succeeds");
    let cargo = fs::read_to_string(package.join("Cargo.toml"))
        .expect("Cargo.toml exists")
        .replace("\"src/**/*.sifr\", ", "");
    fs::write(package.join("Cargo.toml"), cargo).expect("break include");

    let check = check_projection(&package);

    assert!(check
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_INCLUDE_DRIFT));
}

#[test]
fn cargo_projection_repair_regenerates_missing_pure_marker() {
    let temp = TestWorkspace::new("repair_marker");
    let package = temp.root.join("demo_json");
    init_package(&InitPackageOptions {
        target_dir: package.clone(),
        sifr_name: "demo_json".to_string(),
        kind: InitPackageKind::Lib,
        force: false,
    })
    .expect("init succeeds");
    fs::remove_file(package.join("src/lib.rs")).expect("remove marker");

    let check = check_projection(&package);
    assert!(check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_PURE_MARKER_MISSING
    }));

    let repair = repair_projection(&package, false);
    assert!(repair.diagnostics.is_empty());
    assert!(package.join("src/lib.rs").is_file());
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
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
