use super::*;
use sifr_package::CargoLockMode;
use std::fs;
use std::path::PathBuf;

fn fixture(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "sifr-cargo-family-{label}-{}-{}",
        std::process::id(),
        super::super::rust_interop_probe_nonce::unique_probe_nonce()
    ));
    fs::create_dir_all(&path).expect("private fixture");
    path
}

#[test]
fn family_identity_preserves_source_resolution_and_profile_boundaries() {
    let root = fixture("identity");
    let runtime = root.join("runtime");
    let authority = root.join("Cargo.lock");
    fs::write(&authority, "version = 4\n").expect("authority");
    let mut policy = CargoResolutionPolicy::normal();
    policy.authoritative_locks.push(authority.clone());
    let identity = |policy: &CargoResolutionPolicy| {
        policy.native_family_sources(&runtime, CargoVendorMode::PackageOwned, &[])
    };
    let original = identity(&policy);
    assert_eq!(original, identity(&policy));
    let mut changed = policy.clone();
    changed.lock_mode = CargoLockMode::Locked;
    assert_ne!(original, identity(&changed));
    changed = policy.clone();
    changed.application_profile = crate::ApplicationProfile::Development;
    assert_ne!(original, identity(&changed));
    changed = policy.clone();
    changed.trusted_vendor_dirs.push(root.join("other-vendor"));
    assert_ne!(original, identity(&changed));
    changed = policy.clone();
    changed.authoritative_locks.push(root.join("other.lock"));
    assert_ne!(original, identity(&changed));
    assert_ne!(
        original,
        policy.native_family_sources(
            &root.join("other-runtime"),
            CargoVendorMode::PackageOwned,
            &[]
        )
    );
    assert_ne!(
        original,
        policy.native_family_sources(&runtime, CargoVendorMode::SysrootOnly, &[])
    );
    assert_ne!(
        original,
        policy.native_family_sources(
            &runtime,
            CargoVendorMode::PackageOwned,
            &[
                "--config".into(),
                "source.crates-io.replace-with=\"other\"".into()
            ]
        )
    );
    fs::write(&authority, "version = 4\n# changed authority\n").expect("new authority");
    assert_ne!(original, identity(&policy));
    fs::remove_dir_all(root).expect("cleanup");
}

#[test]
fn probe_and_build_reuse_native_preparation_but_recheck_changed_inputs() {
    let root = fixture("native");
    let dependency = root.join("dependency");
    fs::create_dir_all(dependency.join("src")).expect("dependency");
    fs::write(
        dependency.join("Cargo.toml"),
        "[package]\nname = \"family_dependency\"\nversion = \"0.0.0\"\nedition = \"2024\"\n",
    )
    .expect("manifest");
    fs::write(dependency.join("value.txt"), "first").expect("build input");
    fs::write(dependency.join("build.rs"), r#"use std::io::Write;
fn main() {
    std::fs::OpenOptions::new().create(true).append(true).open("runs.log").unwrap().write_all(b"run\n").unwrap();
    println!("cargo:rerun-if-changed=value.txt");
    println!("cargo:rustc-env=VALUE={}", std::fs::read_to_string("value.txt").unwrap());
}"#).expect("build script");
    fs::write(
        dependency.join("src/lib.rs"),
        "pub fn value() -> &'static str { env!(\"VALUE\") }",
    )
    .expect("dependency source");
    for (name, file, source) in [
        (
            "probe",
            "lib.rs",
            "pub fn probe() -> &'static str { family_dependency::value() }",
        ),
        (
            "application",
            "main.rs",
            "fn main() { println!(\"{}\", family_dependency::value()); }",
        ),
    ] {
        let project = root.join(name);
        fs::create_dir_all(project.join("src")).expect("project");
        fs::write(project.join("Cargo.toml"), format!("[package]\nname = \"family_{name}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[dependencies]\nfamily_dependency = {{ path = {dependency:?} }}\n{}", crate::application_profile::MANIFEST)).expect("project manifest");
        fs::write(project.join("src").join(file), source).expect("project source");
    }
    let policy = CargoResolutionPolicy::normal();
    let run = |name: &str, action: &str, success: bool| {
        let family = policy
            .native_family(&root, CargoVendorMode::PackageOwned, &[], "")
            .expect("family");
        let owned = policy.with_lease(family.lease()).expect("child lease");
        let mut command = owned.cargo_command().expect("frozen native toolchain");
        command
            .args([action, "--offline", "--release", "--message-format=json"])
            .arg("--manifest-path")
            .arg(root.join(name).join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", family.target());
        let output = command.output().expect("Cargo");
        assert_eq!(
            output.status.success(),
            success,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        (family.root.clone(), family.target(), output.stdout)
    };
    let (family_root, target, _) = run("probe", "check", true);
    let (_, application_target, artifacts) = run("application", "build", true);
    assert_eq!(target, application_target);
    assert_eq!(
        fs::read_to_string(dependency.join("runs.log")).expect("build-script count"),
        "run\n"
    );
    let artifacts: Vec<serde_json::Value> = String::from_utf8_lossy(&artifacts)
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    assert!(
        artifacts
            .iter()
            .any(|row| row["reason"] == "compiler-artifact"
                && row["target"]["name"] == "build-script-build"
                && row["fresh"] == true)
    );
    let binary = target.join("release").join(format!(
        "family_application{}",
        std::env::consts::EXE_SUFFIX
    ));
    let execute = || {
        let output = std::process::Command::new(&binary)
            .output()
            .expect("execute actual program");
        assert!(output.status.success());
        output.stdout
    };
    assert_eq!(execute(), b"first\n");
    fs::write(
        root.join("probe/src/lib.rs"),
        "pub fn probe() { let _: bool = family_dependency::value(); }",
    )
    .expect("invalid probe");
    run("probe", "check", false);
    fs::write(dependency.join("value.txt"), "second").expect("changed build input");
    run("application", "build", true);
    assert_eq!(execute(), b"second\n");
    assert_eq!(
        fs::read_to_string(dependency.join("runs.log")).expect("rebuild count"),
        "run\nrun\n"
    );
    fs::write(
        root.join("application/src/main.rs"),
        "fn main() { absent(); }",
    )
    .expect("invalid replacement");
    run("application", "build", false);
    fs::remove_dir_all(family_root).expect("owned family cleanup");
    fs::remove_dir_all(root).expect("fixture cleanup");
}
