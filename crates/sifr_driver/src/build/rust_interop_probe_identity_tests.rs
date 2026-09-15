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

fn copy_vendor_tree(source: &std::path::Path, destination: &std::path::Path) {
    fs::create_dir_all(destination).expect("create test vendor directory");
    for entry in fs::read_dir(source).expect("read test vendor source") {
        let entry = entry.expect("read vendor entry");
        let target = destination.join(entry.file_name());
        if entry.file_type().expect("vendor entry type").is_dir() {
            copy_vendor_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy vendor file");
        }
    }
}

#[test]
fn vendored_probe_preserves_compiler_dependency_freshness() {
    use super::super::rust_interop_probe_paths::probe_cargo_target_dir_with_env;
    use super::probe_cargo_vendor_args;
    use std::path::Path;

    let projects = ProbeProjects(std::env::temp_dir().join(format!(
        "sifr_probe_storage_{}_{}",
        std::process::id(),
        super::super::rust_interop_probe_nonce::unique_probe_nonce()
    )));
    let original_vendor = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor")
        .canonicalize()
        .expect("workspace vendor");
    let probe_vendor = projects.0.join("probe-vendor");
    copy_vendor_tree(
        &original_vendor.join("unicode-ident"),
        &probe_vendor.join("unicode-ident"),
    );
    let compiler = projects.0.join("compiler");
    let probe = projects.0.join("probe");
    for (root, name, dependency_kind) in [
        (&compiler, "compiler-storage-fixture", "dependencies"),
        (&probe, "probe-storage-fixture", "build-dependencies"),
    ] {
        fs::create_dir_all(root.join("src")).expect("create source");
        fs::write(root.join("Cargo.toml"), format!(
            "[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[{dependency_kind}]\nunicode-ident = \"=1.0.24\"\n"
        )).expect("write storage fixture manifest");
        fs::write(root.join("src/lib.rs"), "").expect("write storage fixture");
    }
    fs::write(
        compiler.join("src/lib.rs"),
        "pub fn is_identifier(c: char) -> bool { unicode_ident::is_xid_start(c) }",
    )
    .expect("write compiler dependency use");
    fs::write(
        probe.join("build.rs"),
        "fn main() { assert!(unicode_ident::is_xid_start('a')); }",
    )
    .expect("write host dependency probe");
    let target = projects.0.join("target");
    let probe_target =
        probe_cargo_target_dir_with_env(Some(target.clone().into_os_string()), &projects.0);
    for (root, vendor, target, command, must_be_fresh) in [
        (&compiler, &original_vendor, &target, "build", false),
        (&probe, &probe_vendor, &probe_target, "check", false),
        (&compiler, &original_vendor, &target, "build", true),
    ] {
        let output = Command::new("cargo")
            .args(probe_cargo_vendor_args(Some(vendor)))
            .args([command, "--offline", "--message-format=json"])
            .current_dir(root)
            .env("CARGO_TARGET_DIR", target)
            .output()
            .expect("run storage regression Cargo");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if must_be_fresh {
            let artifacts: Vec<serde_json::Value> = String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
                .filter(|value| value["reason"] == "compiler-artifact")
                .collect();
            assert!(
                artifacts.len() >= 2,
                "compiler and dependency artifacts missing"
            );
            assert!(
                artifacts.iter().all(|value| value["fresh"] == true),
                "probe invalidated compiler artifacts: {artifacts:?}"
            );
        }
    }
}
