use super::bind_probe_package_identity;
use std::fs::{self, File, FileTimes};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

const MANIFEST: &str =
    "[package]\nname = \"sifr-rust-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n";

struct ProbeProjects(PathBuf);

impl Drop for ProbeProjects {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn probe_package_identity_binds_manifest_and_source() {
    let source = "pub fn probe() {}";
    let identity = bind_probe_package_identity(MANIFEST, source);
    assert_eq!(identity, bind_probe_package_identity(MANIFEST, source));
    assert_ne!(
        identity,
        bind_probe_package_identity(MANIFEST, "pub fn other() {}")
    );
    assert_ne!(
        identity,
        bind_probe_package_identity(&format!("{MANIFEST}\n[features]\nextra=[]\n"), source)
    );
}

#[test]
fn shared_cargo_target_rejects_a_different_probe_contract() {
    let projects = ProbeProjects(std::env::temp_dir().join(format!(
        "sifr_probe_identity_{}_{}",
        std::process::id(),
        super::super::rust_interop_probe_nonce::unique_probe_nonce()
    )));
    let old = SystemTime::now() - Duration::from_secs(60);
    // Both roots exist before the first check. This models concurrent probes
    // waiting on Cargo's shared target lock, without timing-dependent sleeps.
    for (name, source) in [
        ("valid", "pub fn probe() {}"),
        ("invalid", "pub fn probe() { let _: bool = String::new(); }"),
    ] {
        let root = projects.0.join(name);
        fs::create_dir_all(root.join("src")).expect("create probe root");
        fs::write(
            root.join("Cargo.toml"),
            bind_probe_package_identity(MANIFEST, source),
        )
        .expect("write probe manifest");
        let path = root.join("src/lib.rs");
        fs::write(&path, source).expect("write probe source");
        File::options()
            .write(true)
            .open(path)
            .expect("open probe source")
            .set_times(FileTimes::new().set_modified(old))
            .expect("set pre-check source timestamp");
    }
    for (name, expected_success) in [("valid", true), ("invalid", false)] {
        let output = Command::new("cargo")
            .args(["check", "--offline", "--quiet"])
            .current_dir(projects.0.join(name))
            .env("CARGO_TARGET_DIR", projects.0.join("target"))
            .output()
            .expect("run real Cargo probe");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.success(),
            expected_success,
            "{name}: {stderr}"
        );
        if !expected_success {
            assert!(stderr.contains("E0308"), "{stderr}");
        }
    }
}
