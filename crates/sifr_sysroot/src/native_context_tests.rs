use super::native_context::{NativeBuildContext, NativeToolchain};
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);

// Serialize executable fixture publication with child spawning. Concurrent
// forks can briefly inherit another test's writable script descriptor and
// make Linux reject execution with ETXTBSY even after the writer closes it.
static EXECUTABLE_FIXTURES: std::sync::Mutex<()> = std::sync::Mutex::new(());
struct Fixture {
    root: PathBuf,
    _guard: std::sync::MutexGuard<'static, ()>,
}
impl Fixture {
    fn new() -> Self {
        let guard = EXECUTABLE_FIXTURES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let root = std::env::temp_dir().join(format!(
            "sifr-dx2-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self {
            root,
            _guard: guard,
        }
    }
    fn tool(&self, name: &str, body: &str) -> PathBuf {
        let path = self.root.join(name);
        if path.exists() {
            return path;
        }
        fs::write(
            &path,
            format!(
                "#!/bin/sh
{body}
"
            ),
        )
        .unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }
    fn tools(&self) -> NativeToolchain {
        let cargo = self.tool(
            "cargo",
            r#"if [ "$1" = -vV ]; then echo 'cargo fixture'; else printf '%s' "$RUSTC"; fi"#,
        );
        let rustc = self.tool(
            "rustc",
            "printf 'rustc fixture\\nhost: x86_64-fixture-linux\\n'",
        );
        NativeToolchain::from_executables(&self.root, cargo, rustc, Some("fixture-explicit".into()))
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn native_context_pins_tools_across_temporary_project_selection() {
    let fixture = Fixture::new();
    let tools = fixture.tools();
    let temporary = fixture.root.join("temporary");
    fs::create_dir(&temporary).unwrap();
    fs::write(
        temporary.join("rust-toolchain.toml"),
        r#"[toolchain]
channel = "unavailable"
"#,
    )
    .unwrap();
    let mut command = tools.cargo_command().unwrap();
    assert_eq!(command.get_current_dir(), Some(fixture.root.as_path()));
    let result = command
        .arg("--manifest-path")
        .arg(temporary.join("Cargo.toml"))
        .output()
        .unwrap();
    assert!(result.status.success());
    assert_eq!(
        String::from_utf8(result.stdout).unwrap(),
        fixture.root.join("rustc").to_str().unwrap()
    );
}
#[test]
fn native_context_rejects_unavailable_selected_tools() {
    let fixture = Fixture::new();
    assert!(
        NativeToolchain::from_executables(
            &fixture.root,
            fixture.root.join("missing-cargo"),
            fixture.root.join("missing-rustc"),
            None
        )
        .is_err()
    );
}
#[test]
fn native_context_identity_is_deterministic_and_config_secrets_are_redacted() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.root.join(".cargo")).unwrap();
    fs::write(
        fixture.root.join(".cargo/config.toml"),
        r#"[env]
DX_SECRET = "private-test-secret"
"#,
    )
    .unwrap();
    let first = fixture.tools();
    let second = fixture.tools();
    assert_eq!(
        first.cargo_command().unwrap().get_args().count(),
        0,
        "implicit config must not be merged twice"
    );
    assert_eq!(first.identity(), second.identity());
    assert!(!format!("{first:?}").contains("private-test-secret"));
    fs::write(
        fixture.root.join(".cargo/config.toml"),
        r#"[build]
rustflags = ["--cfg=changed"]
"#,
    )
    .unwrap();
    assert_ne!(first.identity(), fixture.tools().identity());
    assert!(first.cargo_command().is_err());
}

#[test]
fn native_context_records_effective_target_and_rejects_new_configuration() {
    let fixture = Fixture::new();
    let original = fixture.tools();
    fs::create_dir(fixture.root.join(".cargo")).unwrap();
    fs::write(
        fixture.root.join(".cargo/config.toml"),
        "[build]\ntarget = \"aarch64-fixture-linux\"\n",
    )
    .unwrap();
    assert!(original.cargo_command().is_err());
    let changed = fixture.tools();
    assert_eq!(changed.target(), "aarch64-fixture-linux");
    assert_ne!(changed.identity(), original.identity());
}
#[test]
fn native_context_digests_executable_content_not_only_version_text() {
    let fixture = Fixture::new();
    let original = fixture.tools();
    let cargo = fixture.root.join("cargo");
    let mut bytes = fs::read(&cargo).unwrap();
    bytes.extend_from_slice(b"\n# behaviorally distinct implementation\n");
    fs::write(&cargo, bytes).unwrap();
    let changed = NativeToolchain::from_executables(
        &fixture.root,
        cargo,
        fixture.root.join("rustc"),
        Some("fixture-explicit".into()),
    )
    .unwrap();
    assert_eq!(changed.cargo_version(), original.cargo_version());
    assert_ne!(changed.identity(), original.identity());
}

#[test]
fn declared_environment_absent_empty_and_literal_unset_have_distinct_build_ids() {
    let fixture = Fixture::new();
    let toolchain = fixture.tools();
    let name = "SIFR_N03_DECLARED_ENV_TEST";
    let build_id = |value: Option<&str>| {
        let toolchain = toolchain
            .clone()
            .with_test_execution_environment(name, value)
            .with_declared_environment([name.to_owned()]);
        NativeBuildContext {
            target: toolchain.target().to_owned(),
            toolchain,
            profile: "dev".into(),
            flags_id: "flags".into(),
            features_id: "features".into(),
            resolution_id: "resolution".into(),
            python_loader_id: None,
            trust_policy_id: "trust".into(),
            destination: fixture.root.join("binary"),
        }
        .identity()
    };
    let absent = build_id(None);
    let empty = build_id(Some(""));
    let literal_unset = build_id(Some("<unset>"));
    assert_ne!(absent, empty);
    assert_ne!(absent, literal_unset);
    assert_ne!(empty, literal_unset);
    assert_eq!(literal_unset, build_id(Some("<unset>")));
}

#[test]
fn dx10_b05_profile_overrides_preserve_boundaries() {
    let fixture = Fixture::new();
    let manifest = fixture.root.join("generated.toml");
    fs::write(&manifest, "[workspace]\n").unwrap();
    fs::create_dir_all(fixture.root.join(".cargo")).unwrap();
    let config = fixture.root.join(".cargo/config.toml");
    for source in [
        "[profile.dev]\noverflow-checks = false\n",
        "[profile.dev]\npanic = \"abort\"\n",
        "[build]\nrustflags = [\"-C\", \"overflow-checks=off\"]\n",
    ] {
        fs::write(&config, source).unwrap();
        assert!(
            fixture
                .tools()
                .validate_application_profile("dev", &manifest)
                .is_err()
        );
    }
    fs::write(&config, "[profile.release]\npanic = \"abort\"\n").unwrap();
    assert!(
        fixture
            .tools()
            .validate_application_profile("dev", &manifest)
            .is_ok()
    );
    assert!(
        fixture
            .tools()
            .validate_application_profile("release", &manifest)
            .is_err()
    );
    fs::write(
        &config,
        "[profile.dev]\noverflow-checks = true\npanic = \"unwind\"\n",
    )
    .unwrap();
    assert!(
        fixture
            .tools()
            .validate_application_profile("dev", &manifest)
            .is_ok()
    );
}

#[test]
fn dx10_b05_generated_manifest_owns_profile_authority() {
    let fixture = Fixture::new();
    let manifest = fixture.root.join("generated.toml");
    fs::write(
        &manifest,
        "[workspace]\n[profile.dev]\npanic = \"unwind\"\noverflow-checks = true\n",
    )
    .unwrap();
    for unrelated in [
        "[workspace]\n[profile.dev]\npanic = \"abort\"\noverflow-checks = false\n",
        "this is not valid TOML [",
    ] {
        fs::write(fixture.root.join("Cargo.toml"), unrelated).unwrap();
        assert!(
            fixture
                .tools()
                .validate_application_profile("dev", &manifest)
                .is_ok()
        );
    }
    fs::write(&manifest, "[workspace]\n[profile.dev]\npanic = \"abort\"\n").unwrap();
    assert!(
        fixture
            .tools()
            .validate_application_profile("dev", &manifest)
            .is_err()
    );
    // No generated root may silently inherit an enclosing workspace's policy.
    fs::write(&manifest, "[package]\nname = \"member\"\n").unwrap();
    assert!(
        fixture
            .tools()
            .validate_application_profile("dev", &manifest)
            .is_err()
    );
}

fn launch_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.tool(
        "cargo",
        r#"if [ "$1" = -vV ]; then echo 'cargo selected'; else "$RUSTC" --version; fi"#,
    );
    fixture.tool(
        "rustc",
        r#"if [ "$1" != -vV ]; then
    [ "$SIFR_NATIVE_TEST_BOOTSTRAP" = selected ] || exit 92
fi
printf 'rustc selected\nhost: x86_64-fixture-linux\n'"#,
    );
    fixture.tool(
        "rustup",
        r#"case "$1" in
show) echo 'fixture-selected (overridden)' ;;
which)
    [ "$2" = --toolchain ] && [ "$3" = fixture-selected ] || exit 93
    printf '%s/%s\n' "$SIFR_NATIVE_TEST_ROOT" "$4" ;;
run)
    [ "$2" = fixture-selected ] || exit 94
    shift 2
    export SIFR_NATIVE_TEST_BOOTSTRAP=selected
    exec "$@" ;;
*) exit 95 ;;
esac"#,
    );
    fixture
}

fn launch_child(fixture: &Fixture, mode: &str) -> std::process::Output {
    let mut paths = vec![fixture.root.clone()];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "native_context_tests::native_launch_subprocess_entry",
            "--nocapture",
        ])
        .env("SIFR_NATIVE_TEST_ROOT", &fixture.root)
        .env("SIFR_NATIVE_TEST_MODE", mode)
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env(
            "SIFR_RUST_TOOLCHAIN",
            if mode == "missing" {
                "unavailable"
            } else {
                "fixture-selected"
            },
        )
        .env_remove("SIFR_CARGO")
        .env_remove("SIFR_RUSTC")
        .env_remove("RUSTC")
        .env_remove("CARGO_BUILD_RUSTC")
        .env_remove("SIFR_NATIVE_TEST_BOOTSTRAP")
        .current_dir(&fixture.root);
    if mode == "paired" {
        command
            .env("SIFR_CARGO", fixture.root.join("cargo"))
            .env("SIFR_RUSTC", fixture.root.join("rustc"));
    } else if mode == "direct" {
        command
            .env("PATH", &fixture.root)
            .env_remove("SIFR_RUST_TOOLCHAIN")
            .env_remove("RUSTUP_TOOLCHAIN");
    }
    command.output().unwrap()
}

#[test]
fn native_launcher_bootstraps_selected_tools_and_nested_rustc() {
    let fixture = launch_fixture();
    let output = launch_child(&fixture, "selected");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn native_launcher_identity_covers_launcher_bytes() {
    let fixture = launch_fixture();
    let first = launch_child(&fixture, "selected");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stdout)
    );
    let launcher = fixture.root.join("rustup");
    let mut bytes = fs::read(&launcher).unwrap();
    bytes.extend_from_slice(b"\n# distinct launcher implementation\n");
    fs::write(launcher, bytes).unwrap();
    let second = launch_child(&fixture, "selected");
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stdout)
    );
    let identity = |bytes: &[u8]| {
        String::from_utf8_lossy(bytes)
            .lines()
            .find_map(|line| line.split_once("NATIVE_LAUNCH_ID=").map(|(_, value)| value))
            .unwrap()
            .to_owned()
    };
    assert_ne!(identity(&first.stdout), identity(&second.stdout));
}

#[test]
fn native_launcher_rejects_missing_selection_without_direct_fallback() {
    let fixture = launch_fixture();
    let output = launch_child(&fixture, "missing");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn native_launcher_keeps_explicit_pairs_and_non_rustup_tools_direct() {
    for mode in ["paired", "direct"] {
        let fixture = Fixture::new();
        fixture.tool(
            "cargo",
            r#"if [ "$1" = -vV ]; then echo 'cargo direct'; else "$RUSTC" --version; fi"#,
        );
        fixture.tool(
            "rustc",
            r#"[ -z "$SIFR_NATIVE_TEST_BOOTSTRAP" ] || exit 96
printf 'rustc selected\nhost: x86_64-fixture-linux\n'"#,
        );
        if mode == "paired" {
            fixture.tool("rustup", "exit 97");
        }
        let output = launch_child(&fixture, mode);
        assert!(
            output.status.success(),
            "{mode}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn native_launch_subprocess_entry() {
    let Some(root) = std::env::var_os("SIFR_NATIVE_TEST_ROOT") else {
        return;
    };
    let root = PathBuf::from(root);
    let tools = NativeToolchain::resolve_at(&root);
    if std::env::var("SIFR_NATIVE_TEST_MODE").unwrap() == "missing" {
        assert!(tools.is_err());
        return;
    }
    let tools = tools.unwrap();
    assert_eq!(tools.cargo_path(), root.join("cargo"));
    assert_eq!(tools.rustc_path(), root.join("rustc"));
    let result = tools
        .cargo_command()
        .unwrap()
        .arg("build")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).starts_with("rustc selected"));
    println!("NATIVE_LAUNCH_ID={}", tools.identity());
}
