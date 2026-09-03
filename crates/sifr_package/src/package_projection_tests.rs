use crate::test_support::TestUnwrap as _;

use crate::{
    InitPackageKind, InitPackageOptions, check_projection, init_package, repair_projection,
};
use sifr_diagnostics::DiagnosticCode;
use sifr_frontend::DiskSourceProvider;
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
    .test_unwrap("init succeeds");

    assert!(written.contains(&package.join("sifr.toml")));
    assert!(package.join("src/__init__.sifr").is_file());
    assert!(package.join("src/lib.rs").is_file());
    let cargo = fs::read_to_string(package.join("Cargo.toml")).test_unwrap("Cargo.toml exists");
    assert!(cargo.contains("name = \"sifr-demo-json\""));
    assert!(cargo.contains("[package.metadata.sifr]"));
    assert!(cargo.contains("manifest = \"sifr.toml\""));
    assert!(cargo.contains("src/**/*.sifr"));
    assert!(
        check_projection(&package, &mut DiskSourceProvider::new())
            .diagnostics
            .is_empty()
    );
}

#[test]
fn projection_repair_generates_canonical_python_bridge_inventory() {
    let temp = TestWorkspace::new("python_bridge_inventory_projection");
    let package = temp.root.join("demo_json");
    init_package(&InitPackageOptions {
        target_dir: package.clone(),
        sifr_name: "demo_json".to_string(),
        kind: InitPackageKind::Lib,
        force: false,
    })
    .test_unwrap("init succeeds");
    fs::create_dir_all(package.join("src/python_bridges")).test_unwrap("create bridge root");
    fs::write(
        package.join("src/python_bridges/adapter.py"),
        "import json\nVALUE = 1\n",
    )
    .test_unwrap("write bridge");

    let check = check_projection(&package, &mut DiskSourceProvider::new());
    assert!(check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PYIMP_INVALID_BRIDGE_SOURCE
            && diagnostic.message.contains("missing or unreadable")
    }));

    let repair = repair_projection(&package, false, &mut DiskSourceProvider::new());
    let inventory = package.join("src/python_bridges/__sifr_inventory__.json");
    assert!(repair.wrote_files.contains(&inventory));
    assert!(inventory.is_file());
    assert!(repair.diagnostics.is_empty());
    let cargo = fs::read_to_string(package.join("Cargo.toml")).test_unwrap("Cargo.toml exists");
    assert!(cargo.contains("src/**/*.py"));
    assert!(cargo.contains("src/python_bridges/__sifr_inventory__.json"));
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
    .test_unwrap("init succeeds");

    assert!(package.join("src/main.sifr").is_file());
    assert!(
        !fs::read_to_string(package.join("sifr.toml"))
            .test_unwrap("manifest exists")
            .contains("[[bin]]")
    );
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
    .test_unwrap("init succeeds");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"sifr-demo-json\"\n",
    )
    .test_unwrap("break projection");

    let check = check_projection(&package, &mut DiskSourceProvider::new());

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
    .test_unwrap("init succeeds");
    let cargo = fs::read_to_string(package.join("Cargo.toml"))
        .test_unwrap("Cargo.toml exists")
        .replace("\"src/**/*.sifr\", ", "");
    fs::write(package.join("Cargo.toml"), cargo).test_unwrap("break include");

    let check = check_projection(&package, &mut DiskSourceProvider::new());

    assert!(
        check
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_INCLUDE_DRIFT)
    );
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
    .test_unwrap("init succeeds");
    fs::remove_file(package.join("src/lib.rs")).test_unwrap("remove marker");

    let check = check_projection(&package, &mut DiskSourceProvider::new());
    assert!(check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_PURE_MARKER_MISSING
    }));

    let repair = repair_projection(&package, false, &mut DiskSourceProvider::new());
    assert!(repair.diagnostics.is_empty());
    assert!(package.join("src/lib.rs").is_file());
}

#[test]
fn rust_bridge_projection_repair_writes_managed_projection_without_touching_user_bridge() {
    let temp = TestWorkspace::new("repair_rust_bridge_projection");
    let package = temp.root.join("demo_json");
    write_rust_bridge_package(&package);

    let repair = repair_projection(&package, false, &mut DiskSourceProvider::new());

    assert!(repair.diagnostics.is_empty());
    assert!(package.join("src/lib.rs").is_file());
    assert!(package.join("src/bridges/mod.rs").is_file());
    assert!(package.join("src/__sifr_bridge/mod.rs").is_file());
    assert_eq!(
        fs::read_to_string(package.join("src/bridges/tokenizer.rs"))
            .test_unwrap("bridge remains readable"),
        "pub fn tokenize() {}\n"
    );
    assert!(
        fs::read_to_string(package.join("src/lib.rs"))
            .test_unwrap("lib projection exists")
            .contains("pub mod __sifr_bridge;")
    );
    assert!(
        fs::read_to_string(package.join("src/bridges/mod.rs"))
            .test_unwrap("bridge projection exists")
            .contains("pub mod tokenizer;")
    );
    assert!(
        fs::read_to_string(package.join("Cargo.toml"))
            .test_unwrap("cargo projection exists")
            .contains("src/**/*.rs")
    );
    assert!(
        check_projection(&package, &mut DiskSourceProvider::new())
            .diagnostics
            .is_empty()
    );
}

#[test]
fn rust_bridge_projection_conflict_does_not_overwrite_user_authored_mod_rs() {
    let temp = TestWorkspace::new("repair_rust_bridge_conflict");
    let package = temp.root.join("demo_json");
    write_rust_bridge_package(&package);
    fs::write(
        package.join("src/bridges/mod.rs"),
        "pub fn user_owned() {}\n",
    )
    .test_unwrap("write conflict");

    let repair = repair_projection(&package, false, &mut DiskSourceProvider::new());

    assert!(repair.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT
            && diagnostic.message.contains("user-authored")
    }));
    assert_eq!(
        fs::read_to_string(package.join("src/bridges/mod.rs")).test_unwrap("read conflict"),
        "pub fn user_owned() {}\n"
    );
}

#[test]
fn rust_bridge_projection_rejects_reserved_generated_bridge_file() {
    let temp = TestWorkspace::new("repair_rust_bridge_reserved_namespace");
    let package = temp.root.join("demo_json");
    write_rust_bridge_package(&package);
    fs::write(package.join("src/__sifr_bridge.rs"), "pub mod user {}\n")
        .test_unwrap("write reserved conflict");

    let check = check_projection(&package, &mut DiskSourceProvider::new());

    assert!(check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT
            && diagnostic.message.contains("reserved")
    }));
}

#[test]
fn rust_bridge_projection_rejects_keyword_bridge_module_filename() {
    let temp = TestWorkspace::new("repair_rust_bridge_keyword_filename");
    let package = temp.root.join("demo_json");
    write_rust_bridge_package(&package);
    fs::write(
        package.join("src/bridges/match.rs"),
        "pub fn keyword() {}\n",
    )
    .test_unwrap("write keyword bridge file");

    let check = check_projection(&package, &mut DiskSourceProvider::new());

    assert!(check.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == DiagnosticCode::PACKAGE_PROJECTION_MANIFEST_POINTER_DRIFT
            && diagnostic.message.contains("valid Rust identifiers")
    }));
}

fn write_rust_bridge_package(package: &std::path::Path) {
    fs::create_dir_all(package.join("src/bridges")).test_unwrap("create bridges");
    fs::write(
        package.join("Cargo.toml"),
        "[package]\nname = \"sifr-demo-json\"\nversion = \"0.1.0\"\nedition = \"2024\"\ninclude = [\"Cargo.toml\", \"Cargo.lock\", \"sifr.toml\", \"src/**/*.sifr\", \"src/lib.rs\"]\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n",
    )
    .test_unwrap("write Cargo.toml");
    fs::write(
        package.join("sifr.toml"),
        "[package]\nname = \"demo_json\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[rust]\nbridges = [\"src/bridges\"]\n",
    )
    .test_unwrap("write sifr.toml");
    fs::write(package.join("src/__init__.sifr"), "").test_unwrap("write Sifr source");
    fs::write(
        package.join("src/bridges/tokenizer.rs"),
        "pub fn tokenize() {}\n",
    )
    .test_unwrap("write user bridge");
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .test_unwrap("clock should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("sifr_package_{name}_{nonce}"));
        fs::create_dir_all(&root).test_unwrap("create temp workspace");
        Self { root }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
