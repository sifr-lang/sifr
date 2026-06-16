use crate::cargo::commands::{CargoFeatureSelection, CargoPackageSelection};
use crate::cargo::errors::{map_cargo_failure, redact_cargo_stderr, CargoAction};
use crate::cargo::lock_modes::CargoLockMode;
use crate::ops::plan::PackageOperation;
use crate::ops::session::{PackageRunRequest, PackageSession, PackageSessionOptions};
use crate::SifrManifest;
use sifr_diagnostics::codes::{active_registry_entries, DiagnosticState};
use sifr_diagnostics::DiagnosticCode;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn package_session_plans_fetch_tree_and_package_check() {
    let temp = TestPackage::new("session_plan");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    );
    temp.write("src/main.sifr", "def main():\n    pass\n");
    let session = session(temp.path(), CargoLockMode::Locked);

    let fetch = session.plan_fetch();
    assert_eq!(fetch.operation.operation, PackageOperation::Fetch);
    assert_eq!(fetch.cargo.expect("fetch plan").args, ["fetch", "--locked"]);

    let tree = session.plan_tree(&["--depth".to_string(), "1".to_string()]);
    assert_eq!(
        tree.cargo.expect("tree plan").args,
        ["tree", "--locked", "--depth", "1"]
    );

    let check = session
        .plan_check(
            None,
            &CargoFeatureSelection::default(),
            &CargoPackageSelection::default(),
        )
        .expect("check plan");
    assert_eq!(check.operation.operation, PackageOperation::Check);
    assert_eq!(check.cargo.expect("check plan").args, ["check", "--locked"]);
}

#[test]
fn package_session_plans_check_workspace_package_selection() {
    let temp = TestPackage::new("session_check_selection");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    );
    let session = session(temp.path(), CargoLockMode::Locked);
    let selection = CargoPackageSelection {
        workspace: true,
        packages: vec!["demo-app".to_string()],
        excludes: vec!["demo-tools".to_string()],
    };

    let check = session
        .plan_check(None, &CargoFeatureSelection::default(), &selection)
        .expect("check plan");

    assert_eq!(
        check.cargo.expect("check plan").args,
        [
            "check",
            "--locked",
            "--workspace",
            "-p",
            "demo-app",
            "--exclude",
            "demo-tools"
        ]
    );
}

#[test]
fn package_session_stops_at_nested_cargo_workspace_without_root_sifr_manifest() {
    let temp = TestPackage::new("session_nested_cargo_workspace");
    temp.write_package_manifest(
        "[package]\nname = \"outer_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n",
    );
    temp.write(
        "nested/Cargo.toml",
        "[workspace]\nmembers = [\"packages/*\"]\nresolver = \"3\"\n",
    );
    temp.write(
        "nested/packages/app/Cargo.toml",
        "[package]\nname = \"sifr-demo-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    );
    temp.write(
        "nested/packages/app/sifr.toml",
        "[package]\nname = \"workspace_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n",
    );
    let workspace_root = temp.path().join("nested");

    let session = session(&workspace_root, CargoLockMode::Locked);

    assert!(session.manifest_less_mode);
    assert_eq!(session.workspace_root, workspace_root);
    let selection = CargoPackageSelection {
        workspace: true,
        excludes: vec!["sifr-demo-app".to_string()],
        ..CargoPackageSelection::default()
    };
    let check = session
        .plan_check(None, &CargoFeatureSelection::default(), &selection)
        .expect("workspace check plan");
    let cargo = check.cargo.expect("cargo plan");
    assert_eq!(cargo.current_dir, temp.path().join("nested"));
    assert_eq!(
        cargo.args,
        [
            "check",
            "--locked",
            "--workspace",
            "--exclude",
            "sifr-demo-app"
        ]
    );
}

#[test]
fn package_session_resolves_default_script_and_explicit_bin() {
    let temp = TestPackage::new("session_run");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\ndefault-run = \"admin\"\n\n[source]\nroot = \"src\"\n\n[scripts]\ndev = { command = \"run\", args = [] }\n",
    );
    temp.write("src/main.sifr", "def main():\n    pass\n");
    temp.write("src/bin/admin.sifr", "def main():\n    pass\n");
    let session = session(temp.path(), CargoLockMode::Normal);

    let default_run = session
        .plan_run(&PackageRunRequest::default())
        .expect("default run target");
    assert_eq!(
        default_run.cargo.expect("run plan").args,
        ["run", "--bin", "admin"]
    );

    let script = session
        .plan_run(&PackageRunRequest {
            target_or_path: Some("dev".to_string()),
            ..PackageRunRequest::default()
        })
        .expect("script plan");
    let origin = script.script_origin.expect("script origin");
    assert_eq!(origin.name, "dev");
    assert_eq!(origin.command, "run");
}

#[test]
fn package_session_reports_script_target_ambiguity() {
    let temp = TestPackage::new("session_ambiguity");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[scripts]\nadmin = { command = \"check\", args = [] }\n",
    );
    temp.write("src/bin/admin.sifr", "def main():\n    pass\n");
    let session = session(temp.path(), CargoLockMode::Normal);

    let diagnostic = session
        .plan_run(&PackageRunRequest {
            target_or_path: Some("admin".to_string()),
            ..PackageRunRequest::default()
        })
        .expect_err("ambiguous target should fail");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_RUN_TARGET_AMBIGUOUS
    );
}

#[test]
fn package_session_rejects_invalid_nested_target_name() {
    let temp = TestPackage::new("session_bad_target");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    );
    temp.write("src/bin/bad!name.sifr", "def main():\n    pass\n");
    let session = session(temp.path(), CargoLockMode::Normal);

    let diagnostic = session
        .plan_run(&PackageRunRequest {
            bin: Some("bad!name".to_string()),
            ..PackageRunRequest::default()
        })
        .expect_err("invalid target should fail");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_INVALID_APP_TARGET_NAME
    );
}

#[test]
fn package_session_rejects_explicit_file_outside_source_root() {
    let temp = TestPackage::new("session_outside_file");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    );
    temp.write("tools/task.sifr", "def main():\n    pass\n");
    let session = session(temp.path(), CargoLockMode::Normal);

    let diagnostic = session
        .plan_check(
            Some(&temp.path().join("tools/task.sifr")),
            &CargoFeatureSelection::default(),
            &CargoPackageSelection::default(),
        )
        .expect_err("outside source root should fail");

    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_EXPLICIT_FILE_OUTSIDE_SOURCE_ROOT
    );
}

#[test]
fn package_session_accepts_explicit_file_under_any_legacy_source_root() {
    let temp = TestPackage::new("session_legacy_roots_file");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroots = [\"examples/workloads/src\", \".\"]\n",
    );
    temp.write("demos/app.sifr", "def main():\n    pass\n");
    let session = session(temp.path(), CargoLockMode::Normal);

    let plan = session
        .plan_check(
            Some(&temp.path().join("demos/app.sifr")),
            &CargoFeatureSelection::default(),
            &CargoPackageSelection::default(),
        )
        .expect("file under second source root should be accepted");

    assert_eq!(plan.operation.operation, PackageOperation::Check);
}

#[test]
fn package_session_rejects_nested_script_expansion() {
    let temp = TestPackage::new("session_script_recursion");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[scripts]\ndev = { command = \"run\", args = [\"other\"] }\nother = { command = \"check\", args = [] }\n",
    );
    let session = session(temp.path(), CargoLockMode::Normal);

    let diagnostic = session
        .plan_run(&PackageRunRequest {
            script: Some("dev".to_string()),
            ..PackageRunRequest::default()
        })
        .expect_err("nested scripts should fail");

    assert_eq!(diagnostic.code, DiagnosticCode::PACKAGE_SCRIPT_RECURSION);
}

#[test]
fn manifest_parses_scripts_and_cargo_compatible_dependency_sections() {
    let temp = TestPackage::new("session_manifest");
    temp.write_package_manifest(
        "[package]\nname = \"demo_app\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[scripts]\ncheck-all = { command = \"check\", args = [\"--workspace\"] }\n\n[dependencies]\ndemo_json = \"1\"\n\n[dev-dependencies]\ndemo_test = { version = \"1\", package = \"sifr-demo-test\" }\n",
    );

    let manifest = SifrManifest::load(
        &crate::CargoPackageId("path+file:///tmp/demo#demo@0.1.0".to_string()),
        &temp.path().join("sifr.toml"),
    )
    .expect("manifest parses");

    assert_eq!(manifest.scripts["check-all"].command, "check");
    assert!(manifest.dependencies.contains_key("demo_json"));
    assert!(manifest.dev_dependencies.contains_key("demo_test"));
}

#[test]
fn cargo_failure_redaction_preserves_public_context_and_retires_0105() {
    let redacted = redact_cargo_stderr(
        "https://crates.io/api/v1/crates https://user:token@private.example.com/pkg ghs_secret cargo:token=abc password=hunter2 Zm9vYmFy",
    );

    assert!(redacted.contains("https://crates.io/api/v1/crates"));
    assert!(redacted.contains("https://[redacted host]/pkg"));
    assert!(redacted.contains("Zm9vYmFy"));
    assert!(!redacted.contains("private.example.com"));
    assert!(!redacted.contains("hunter2"));

    let diagnostic = map_cargo_failure(CargoAction::Fetch, &redacted);
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::PACKAGE_CARGO_COMMAND_FAILED
    );
    assert!(!active_registry_entries().any(|entry| {
        entry.id == "SIFR-PACKAGE-0105" && entry.state == DiagnosticState::Active
    }));
}

fn session(path: &Path, lock_mode: CargoLockMode) -> PackageSession {
    PackageSession::discover(PackageSessionOptions {
        current_dir: path.to_path_buf(),
        lock_mode,
    })
    .expect("session discovers")
}

struct TestPackage {
    path: PathBuf,
}

impl TestPackage {
    fn new(name: &str) -> Self {
        let unique = format!(
            "sifr_pkg_session_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        fs::create_dir_all(&path).expect("create temp package");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write_package_manifest(&self, contents: &str) {
        self.write("sifr.toml", contents);
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, contents).expect("write file");
    }
}

impl Drop for TestPackage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
