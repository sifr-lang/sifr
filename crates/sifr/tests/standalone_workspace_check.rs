mod test_support;

use std::path::Path;
use std::process::{Command, Output};
use test_support::TestUnwrap as _;

fn check(root: &Path, flags: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sifr"))
        .arg("check")
        .arg("main.sifr")
        .args(flags)
        .current_dir(root)
        .output()
        .test_unwrap("run isolated standalone workspace check")
}

#[test]
fn standalone_file_does_not_resolve_unrelated_members_but_package_and_flags_do() {
    let workspace = tempfile::tempdir().test_unwrap("workspace");
    let root = workspace.path();
    std::fs::write(
        root.join("sifr.toml"),
        "[package]\nname = \"standalone\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n[source]\nroot = \".\"\n",
    ).test_unwrap("source workspace manifest");
    std::fs::write(root.join("main.sifr"), "def main():\n    pass\n")
        .test_unwrap("standalone source");
    let broken_workspace = "[workspace]\nmembers = [\"broken-unrelated-member\"]\n";
    std::fs::write(root.join("Cargo.toml"), broken_workspace).test_unwrap("virtual workspace");
    let normal = check(root, &[]);
    assert!(
        normal.status.success(),
        "{}",
        String::from_utf8_lossy(&normal.stderr)
    );
    #[cfg(unix)]
    assert_frontend_does_not_probe_native_tools(root);
    for flag in ["--locked", "--offline", "--frozen"] {
        let output = check(root, &[flag]);
        assert!(
            !output.status.success(),
            "{flag} must retain Cargo validation"
        );
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("broken-unrelated-member"),
            "{flag}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::create_dir(root.join("src")).test_unwrap("Rust source directory");
    std::fs::write(root.join("src/lib.rs"), "").test_unwrap("Rust package target");
    std::fs::write(root.join("Cargo.toml"), format!(
        "[package]\nname = \"actual-root-package\"\nversion = \"0.0.0\"\nedition = \"2024\"\n{broken_workspace}"
    )).test_unwrap("actual Cargo root package");
    let package = check(root, &[]);
    assert!(
        !package.status.success(),
        "actual packages retain Cargo validation"
    );
    assert!(
        String::from_utf8_lossy(&package.stderr).contains("broken-unrelated-member"),
        "{}",
        String::from_utf8_lossy(&package.stderr)
    );
}

#[cfg(unix)]
fn assert_frontend_does_not_probe_native_tools(root: &Path) {
    use std::os::unix::fs::PermissionsExt as _;
    let tools = root.join("recording-tools");
    let log = root.join("native-probes.log");
    std::fs::create_dir(&tools).test_unwrap("recording tool directory");
    for name in ["rustc", "cargo", "rustup"] {
        let path = tools.join(name);
        std::fs::write(
            &path,
            r#"#!/bin/sh
printf '%s\n' "$0 $*" >> "$SIFR_DX15_PROBE_LOG"
exit 77
"#,
        )
        .test_unwrap("recording native tool");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .test_unwrap("executable recording tool");
    }
    let invoke = |arguments: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_sifr"))
            .args(arguments)
            .current_dir(root)
            .env("PATH", &tools)
            .env("RUSTC", tools.join("rustc"))
            .env("CARGO", tools.join("cargo"))
            .env("SIFR_DX15_PROBE_LOG", &log)
            .output()
            .test_unwrap("invoke with recording native tools")
    };
    let checked = invoke(&["check", "--no-incremental", "main.sifr"]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert!(
        !log.exists(),
        "frontend-only checks must not invoke unused native probes"
    );
    let built = invoke(&[
        "build",
        "--materialize-only",
        "main.sifr",
        "-o",
        "native-output",
    ]);
    assert!(
        !built.status.success(),
        "native work must retain authoritative tool validation"
    );
    assert!(
        log.exists(),
        "native work still resolves its required tools"
    );
}
