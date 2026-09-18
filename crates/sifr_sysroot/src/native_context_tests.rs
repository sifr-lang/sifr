use super::native_context::NativeToolchain;
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
