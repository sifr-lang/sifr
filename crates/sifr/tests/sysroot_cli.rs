#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TestSysroot {
    root: PathBuf,
}

impl TestSysroot {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should move forward")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "sifr_cli_sysroot_{label}_{}_{}",
            std::process::id(),
            unique
        ));
        write_skeleton(&root);
        Self { root }
    }
}

impl Drop for TestSysroot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn print_sysroot_uses_explicit_installed_layout_outside_repo() {
    let sysroot = TestSysroot::new("plain");
    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args([
            "--sysroot",
            &sysroot.root.display().to_string(),
            "--print",
            "sysroot",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .expect("sifr should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    assert_eq!(stdout(&output).trim(), sysroot.root.display().to_string());
}

#[test]
fn print_sysroot_json_reports_identity_and_layout_paths() {
    let sysroot = TestSysroot::new("json");
    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args([
            "--sysroot",
            &sysroot.root.display().to_string(),
            "--print",
            "sysroot",
            "--json",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .expect("sifr should run");

    assert!(output.status.success(), "stderr: {}", stderr(&output));
    let value: serde_json::Value =
        serde_json::from_str(&stdout(&output)).expect("JSON output should parse");
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["root"], sysroot.root.display().to_string());
    assert_eq!(
        value["toolchain_id"],
        format!("{}-test-target", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        value["paths"]["runtime_crate_manifest"],
        sysroot
            .root
            .join("crates/sifr_runtime/Cargo.toml")
            .display()
            .to_string()
    );
    assert_eq!(
        value["paths"]["stdlib_crate_manifest"],
        sysroot
            .root
            .join("crates/sifr_stdlib/Cargo.toml")
            .display()
            .to_string()
    );
    assert_eq!(
        value["paths"]["stdlib_public_sources"],
        sysroot
            .root
            .join("lib/sifr/stdlib/sifr")
            .display()
            .to_string()
    );
}

#[test]
fn explicit_sysroot_missing_vendor_reports_boundary_error() {
    let sysroot = TestSysroot::new("missing_vendor");
    std::fs::remove_dir_all(sysroot.root.join("vendor")).expect("remove vendor dir");

    let output = Command::new(env!("CARGO_BIN_EXE_sifr"))
        .args([
            "--sysroot",
            &sysroot.root.display().to_string(),
            "--print",
            "sysroot",
        ])
        .current_dir(std::env::temp_dir())
        .output()
        .expect("sifr should run");

    assert!(!output.status.success(), "stdout: {}", stdout(&output));
    let stderr = stderr(&output);
    assert!(stderr.contains("Sifr sysroot is missing vendor"));
    assert!(stderr.contains("binary path:"));
    assert!(stderr.contains("attempted sysroot:"));
    assert!(stderr.contains(&sysroot.root.join("vendor").display().to_string()));
}

fn write_skeleton(root: &Path) {
    std::fs::create_dir_all(root.join(".cargo")).expect("cargo dir");
    std::fs::create_dir_all(root.join("vendor")).expect("vendor dir");
    std::fs::create_dir_all(root.join("lib/sifr/stdlib/sifr")).expect("public stdlib dir");
    std::fs::create_dir_all(root.join("lib/sifr/stdlib/_sifr")).expect("private stdlib dir");
    std::fs::write(
        root.join("sysroot.toml"),
        format!(
            r#""schema-version" = 1
"sifr-version" = "{}"
"target-triple" = "test-target"
"built-by-compiler-commit" = "abc123"
"sysroot-content-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
"cargo-lock-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
"#,
            env!("CARGO_PKG_VERSION")
        ),
    )
    .expect("manifest");
    std::fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/sifr_runtime", "crates/sifr_structural_identity", "crates/sifr_stdlib"]
resolver = "3"
"#,
    )
    .expect("workspace manifest");
    std::fs::write(root.join("Cargo.lock"), "").expect("lockfile");
    std::fs::write(root.join(".cargo/config.toml"), "").expect("cargo config");
    write_minimal_crate(root, "sifr_runtime");
    write_minimal_crate(root, "sifr_structural_identity");
    write_minimal_crate(root, "sifr_stdlib");
}

fn write_minimal_crate(root: &Path, name: &str) {
    let crate_dir = root.join("crates").join(name);
    std::fs::create_dir_all(crate_dir.join("src")).expect("crate src dir");
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n"),
    )
    .expect("crate manifest");
    std::fs::write(crate_dir.join("src/lib.rs"), "").expect("crate lib");
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
