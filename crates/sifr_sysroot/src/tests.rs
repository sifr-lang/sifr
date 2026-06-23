use super::{
    canonical_sysroot_tree_digest, parse_sysroot_manifest, resolve_sysroot_with,
    CanonicalDigestPolicy, SysrootErrorKind, SysrootResolutionInput, COMPILER_SIFR_VERSION,
    SYSROOT_MANIFEST_FIELDS,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempRoot {
    path: PathBuf,
}

impl TempRoot {
    fn new(label: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be monotonic enough for test paths")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "sifr_sysroot_{label}_{}_{}",
            std::process::id(),
            unique
        ));
        fs::create_dir_all(&path).expect("temp root should be created");
        Self { path }
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn documented_manifest_fields_match_parser_schema() {
    assert_eq!(
        SYSROOT_MANIFEST_FIELDS,
        &[
            "schema-version",
            "sifr-version",
            "target-triple",
            "built-by-compiler-commit",
            "sysroot-content-sha256",
            "cargo-lock-sha256",
        ]
    );
    let manifest = parse_sysroot_manifest(&valid_manifest(
        COMPILER_SIFR_VERSION,
        "x86_64-unknown-linux-gnu",
    ))
    .expect("valid manifest should parse");
    assert_eq!(
        manifest.toolchain_id(),
        format!("{COMPILER_SIFR_VERSION}-x86_64-unknown-linux-gnu")
    );
}

#[test]
fn manifest_rejects_malformed_and_unsupported_schema_versions() {
    let malformed = parse_sysroot_manifest("schema-version = \"one\"")
        .expect_err("malformed manifest should fail");
    assert_eq!(malformed.kind, SysrootErrorKind::MalformedManifest);

    let unsupported = parse_sysroot_manifest(
        r#"
"schema-version" = 2
"sifr-version" = "0.0.0"
"target-triple" = "x"
"built-by-compiler-commit" = "abc"
"sysroot-content-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
"cargo-lock-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
"#,
    )
    .expect_err("unsupported schema should fail");
    assert_eq!(unsupported.kind, SysrootErrorKind::UnsupportedSchemaVersion);
}

#[test]
fn manifest_rejects_version_mismatch() {
    let mismatch = parse_sysroot_manifest(&valid_manifest("999.999.999", "target"))
        .expect_err("version mismatch should fail");
    assert_eq!(mismatch.kind, SysrootErrorKind::VersionMismatch);
    assert!(mismatch.message.contains(COMPILER_SIFR_VERSION));
}

#[test]
fn manifest_unknown_required_fields_fail_but_optional_fields_are_ignored() {
    let unknown = format!(
        "{}\n\"future-required\" = true\n",
        valid_manifest(COMPILER_SIFR_VERSION, "target")
    );
    let error = parse_sysroot_manifest(&unknown).expect_err("unknown required field should fail");
    assert_eq!(error.kind, SysrootErrorKind::UnknownManifestField);

    let optional = format!(
        "{}\n\"optional-note\" = \"ignored\"\n",
        valid_manifest(COMPILER_SIFR_VERSION, "target")
    );
    assert!(parse_sysroot_manifest(&optional).is_ok());
}

#[test]
fn resolver_prefers_explicit_then_environment_then_installed_then_development() {
    let explicit = complete_sysroot("explicit", COMPILER_SIFR_VERSION);
    let env = complete_sysroot("env", COMPILER_SIFR_VERSION);
    let installed = complete_installed_layout("installed", COMPILER_SIFR_VERSION);
    let development = complete_source_tree_sysroot("development", COMPILER_SIFR_VERSION);
    let input = SysrootResolutionInput {
        explicit_sysroot: Some(explicit.path.clone()),
        env_sysroot: Some(env.path.clone()),
        current_exe: installed.path.join("bin").join("sifr"),
        current_dir: development.path.join("nested"),
        allow_source_tree_development: true,
    };
    fs::create_dir_all(&input.current_dir).expect("nested current dir");
    let resolved = resolve_sysroot_with(&input).expect("explicit sysroot should resolve");
    assert_eq!(resolved.root, explicit.path);

    let input_without_explicit = SysrootResolutionInput {
        explicit_sysroot: None,
        ..input.clone()
    };
    let resolved =
        resolve_sysroot_with(&input_without_explicit).expect("env sysroot should resolve");
    assert_eq!(resolved.root, env.path);

    let input_without_env = SysrootResolutionInput {
        env_sysroot: None,
        ..input_without_explicit
    };
    let resolved =
        resolve_sysroot_with(&input_without_env).expect("installed sysroot should resolve");
    assert_eq!(resolved.root, installed.path);
}

#[test]
fn source_tree_development_requires_explicit_gate() {
    let development = complete_source_tree_sysroot("development_only", COMPILER_SIFR_VERSION);
    let current_dir = development.path.join("nested");
    fs::create_dir_all(&current_dir).expect("nested current dir");
    let denied = SysrootResolutionInput {
        explicit_sysroot: None,
        env_sysroot: None,
        current_exe: PathBuf::from("/tmp/sifr"),
        current_dir: current_dir.clone(),
        allow_source_tree_development: false,
    };
    let error = resolve_sysroot_with(&denied).expect_err("development sysroot should be gated");
    assert_eq!(error.kind, SysrootErrorKind::NoCandidate);

    let allowed = SysrootResolutionInput {
        allow_source_tree_development: true,
        ..denied
    };
    let resolved = resolve_sysroot_with(&allowed).expect("development sysroot should resolve");
    assert_eq!(resolved.root, development.path);
}

#[test]
fn missing_runtime_crate_reports_sysroot_boundary() {
    let root = complete_sysroot("missing_runtime", COMPILER_SIFR_VERSION);
    fs::remove_file(root.path.join("crates/sifr_runtime/Cargo.toml")).expect("remove runtime");
    let input = SysrootResolutionInput {
        explicit_sysroot: Some(root.path.clone()),
        env_sysroot: None,
        current_exe: PathBuf::from("/tool/bin/sifr"),
        current_dir: root.path.clone(),
        allow_source_tree_development: false,
    };
    let error = resolve_sysroot_with(&input).expect_err("missing runtime should fail");
    assert_eq!(error.kind, SysrootErrorKind::MissingAsset);
    let message = error.boundary_message();
    assert!(message.contains("binary path: /tool/bin/sifr"));
    assert!(message.contains("attempted sysroot:"));
    assert!(message.contains("crates/sifr_runtime/Cargo.toml"));
}

#[test]
fn layout_validation_checks_all_skeleton_assets() {
    for (label, path) in [
        ("cargo_manifest", "Cargo.toml"),
        ("cargo_lock", "Cargo.lock"),
        ("cargo_config", ".cargo/config.toml"),
        ("stdlib_root", "lib/sifr/stdlib"),
        ("stdlib_public_sources", "lib/sifr/stdlib/sifr"),
        ("stdlib_private_sources", "lib/sifr/stdlib/_sifr"),
        ("vendor", "vendor"),
        ("runtime_manifest", "crates/sifr_runtime/Cargo.toml"),
        ("stdlib_manifest", "crates/sifr_stdlib/Cargo.toml"),
    ] {
        let root = complete_sysroot(label, COMPILER_SIFR_VERSION);
        remove_asset(&root.path.join(path));
        let input = SysrootResolutionInput {
            explicit_sysroot: Some(root.path.clone()),
            env_sysroot: None,
            current_exe: PathBuf::from("/tool/bin/sifr"),
            current_dir: root.path.clone(),
            allow_source_tree_development: false,
        };
        let error = resolve_sysroot_with(&input).expect_err("missing asset should fail");
        assert_eq!(error.kind, SysrootErrorKind::MissingAsset);
        assert_eq!(error.asset_path, Some(root.path.join(path)));
    }
}

#[test]
fn workspace_validation_requires_generated_stdlib_member() {
    let root = complete_sysroot("missing_stdlib_member", COMPILER_SIFR_VERSION);
    fs::write(
        root.path.join("Cargo.toml"),
        r#"
[workspace]
members = ["crates/sifr_runtime"]
resolver = "2"
"#,
    )
    .expect("workspace manifest");
    let input = SysrootResolutionInput {
        explicit_sysroot: Some(root.path.clone()),
        env_sysroot: None,
        current_exe: PathBuf::from("/tool/bin/sifr"),
        current_dir: root.path.clone(),
        allow_source_tree_development: false,
    };

    let error = resolve_sysroot_with(&input).expect_err("missing stdlib member should fail");

    assert_eq!(error.kind, SysrootErrorKind::InvalidWorkspace);
    assert!(error
        .message
        .contains("workspace member crates/sifr_stdlib"));
}

#[test]
fn installed_layout_workspace_supports_offline_cargo_metadata() {
    let root = complete_sysroot("offline_metadata", COMPILER_SIFR_VERSION);

    let output = Command::new("cargo")
        .args([
            "metadata",
            "--offline",
            "--no-deps",
            "--format-version",
            "1",
        ])
        .current_dir(&root.path)
        .output()
        .expect("cargo metadata should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn canonical_digest_sorts_paths_and_normalizes_line_endings() {
    let root = TempRoot::new("digest");
    fs::create_dir_all(root.path.join("b")).expect("dir");
    fs::write(root.path.join("b/two.sifr"), "two\r\n").expect("write two");
    fs::write(root.path.join("a.sifr"), "one\r").expect("write one");
    fs::write(root.path.join("ignore.bin"), [1, 2, 3]).expect("write ignored");

    let digest = canonical_sysroot_tree_digest(&root.path, &CanonicalDigestPolicy::default())
        .expect("digest should compute");
    assert_eq!(digest.algorithm, "sha256");
    assert_eq!(
        digest
            .entries
            .iter()
            .map(|entry| entry.relative_path.as_str())
            .collect::<Vec<_>>(),
        ["a.sifr", "b/two.sifr"]
    );
    assert_eq!(digest.entries[0].bytes, b"one\n");
    assert_eq!(digest.entries[1].bytes, b"two\n");
}

fn complete_installed_layout(label: &str, version: &str) -> TempRoot {
    let root = complete_sysroot(label, version);
    fs::create_dir_all(root.path.join("bin")).expect("bin dir");
    root
}

fn complete_sysroot(label: &str, version: &str) -> TempRoot {
    let root = TempRoot::new(label);
    write_complete_sysroot(&root.path, version);
    root
}

fn complete_source_tree_sysroot(label: &str, version: &str) -> TempRoot {
    let root = complete_sysroot(label, version);
    fs::create_dir_all(root.path.join("stdlib/sifr")).expect("source-tree public stdlib root");
    fs::create_dir_all(root.path.join("stdlib/_sifr")).expect("source-tree private stdlib root");
    root
}

fn write_complete_sysroot(root: &Path, version: &str) {
    fs::write(
        root.join("sysroot.toml"),
        valid_manifest(version, "test-target"),
    )
    .expect("manifest");
    fs::write(
        root.join("Cargo.toml"),
        r#"
[workspace]
members = ["crates/sifr_runtime", "crates/sifr_stdlib"]
resolver = "2"
"#,
    )
    .expect("workspace manifest");
    fs::write(root.join("Cargo.lock"), "").expect("lock");
    fs::create_dir_all(root.join(".cargo")).expect("cargo config dir");
    fs::write(root.join(".cargo/config.toml"), "").expect("cargo config");
    fs::create_dir_all(root.join("vendor")).expect("vendor");
    fs::create_dir_all(root.join("lib/sifr/stdlib/sifr")).expect("public stdlib root");
    fs::create_dir_all(root.join("lib/sifr/stdlib/_sifr")).expect("private stdlib root");
    write_minimal_crate(root, "sifr_runtime");
    write_minimal_crate(root, "sifr_stdlib");
}

fn write_minimal_crate(root: &Path, name: &str) {
    let crate_dir = root.join("crates").join(name);
    fs::create_dir_all(crate_dir.join("src")).expect("crate src dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n"),
    )
    .expect("crate manifest");
    fs::write(crate_dir.join("src/lib.rs"), "").expect("crate lib");
}

fn valid_manifest(version: &str, target: &str) -> String {
    format!(
        r#""schema-version" = 1
"sifr-version" = "{version}"
"target-triple" = "{target}"
"built-by-compiler-commit" = "abc123"
"sysroot-content-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
"cargo-lock-sha256" = "0000000000000000000000000000000000000000000000000000000000000000"
"#
    )
}

fn remove_asset(path: &Path) {
    if path.is_dir() {
        fs::remove_dir_all(path).expect("remove dir");
    } else {
        fs::remove_file(path).expect("remove file");
    }
}
