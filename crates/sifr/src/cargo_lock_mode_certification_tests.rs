use crate::check_and_package_commands::{cmd_check, package_entrypoint_for_file};
use crate::cli_model_and_entrypoint::{
    Cli, Commands, DiagnosticFormat, EXIT_SUCCESS, EXIT_USER_DIAGNOSTIC,
};
use crate::diagnostic_rendering_and_run::{cmd_build, cmd_run};
use clap::Parser;
use sifr_diagnostics::DiagnosticCode;
use sifr_package::{CargoLockMode, PackageSession, PackageSessionOptions};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

const SCENARIO_ROOT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../verification/areas/rust_interop/fixtures/cargo_locked_offline/examples/locked_offline_cache"
);

static CWD_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct Scenario {
    root: PathBuf,
}

impl Scenario {
    fn copy(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "sifr_cargo_lock_mode_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        copy_tree(Path::new(SCENARIO_ROOT), &root);
        Self { root }
    }

    fn empty(name: &str) -> Self {
        let scenario = Self::copy(name);
        std::fs::remove_dir_all(&scenario.root).expect("copied scenario should be replaceable");
        std::fs::create_dir_all(&scenario.root).expect("empty scenario root should be created");
        scenario
    }
}

impl Drop for Scenario {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

struct CurrentDirGuard {
    previous: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl CurrentDirGuard {
    fn enter(path: &Path) -> Self {
        let lock = CWD_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("cargo lock-mode test cwd lock should not be poisoned");
        let previous = std::env::current_dir().expect("test cwd should exist");
        std::env::set_current_dir(path).expect("scenario cwd should be selected");
        Self {
            previous,
            _lock: lock,
        }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.previous).expect("test cwd should be restored");
    }
}

#[test]
fn build_lock_flags_parse_and_normalize_without_collapsing_frozen() {
    for (flags, expected) in [
        (vec![], CargoLockMode::Normal),
        (vec!["--locked"], CargoLockMode::Locked),
        (vec!["--offline"], CargoLockMode::Offline),
        (vec!["--frozen"], CargoLockMode::Frozen),
        (vec!["--locked", "--offline"], CargoLockMode::Frozen),
    ] {
        let mut args = vec!["sifr", "build", "main.sifr"];
        args.extend(flags);
        let cli = Cli::try_parse_from(args).expect("build lock flags should parse");
        let Commands::Build {
            locked,
            offline,
            frozen,
            ..
        } = cli.command.expect("build command should be present")
        else {
            panic!("expected parsed build command");
        };
        assert_eq!(
            crate::cli_lock_modes::lock_mode_from_flags(locked, offline, frozen),
            expected
        );
    }
}

#[test]
fn constrained_modes_reject_manifestless_check_build_and_run() {
    let scenario = Scenario::empty("manifestless");
    std::fs::write(
        scenario.root.join("main.sifr"),
        "def main() -> None:\n    pass\n",
    )
    .expect("manifestless source should be written");
    let _cwd = CurrentDirGuard::enter(&scenario.root);
    for (command, operation) in [
        (
            "check",
            Box::new(|| {
                cmd_check(
                    Some(Path::new("main.sifr")),
                    None,
                    &sifr_package::CargoPackageSelection::default(),
                    CargoLockMode::Frozen,
                    DiagnosticFormat::Compact,
                )
            }) as Box<dyn Fn() -> i32>,
        ),
        (
            "build",
            Box::new(|| {
                cmd_build(
                    Path::new("main.sifr"),
                    Path::new("build"),
                    CargoLockMode::Frozen,
                    true,
                    DiagnosticFormat::Compact,
                )
            }),
        ),
        (
            "run",
            Box::new(|| {
                cmd_run(
                    Some("main.sifr"),
                    None,
                    None,
                    &[],
                    &[],
                    CargoLockMode::Frozen,
                    DiagnosticFormat::Compact,
                )
            }),
        ),
    ] {
        let (exit_code, diagnostics) = crate::diagnostic_test_sink::capture(operation);
        assert_eq!(exit_code, EXIT_USER_DIAGNOSTIC);
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("{command} must emit one diagnostic: {diagnostics:#?}");
        };
        assert_eq!(diagnostic.code, DiagnosticCode::RUST_CARGO_METADATA.code());
        assert!(
            diagnostic
                .message
                .contains(&format!("sifr {command} --frozen requires")),
            "{command} must reject the manifestless lock mode: {diagnostic:#?}"
        );
    }
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn test_locked_offline_sifr_commands_and_warm_cache() {
    let scenario = Scenario::copy("positive");
    let _cwd = CurrentDirGuard::enter(&scenario.root);
    let source = Path::new("src/main.sifr");
    let source_lock = Path::new("Cargo.lock");
    let source_lock_before =
        std::fs::read(source_lock).expect("checked-in scenario lock should be readable");

    let session = PackageSession::discover(PackageSessionOptions {
        current_dir: scenario.root.clone(),
        lock_mode: CargoLockMode::Frozen,
    })
    .expect("frozen package session should resolve");
    let entrypoint = package_entrypoint_for_file(
        source,
        &session,
        CargoLockMode::Frozen,
        DiagnosticFormat::Human,
        false,
    )
    .expect("package entrypoint resolution should not render an error")
    .expect("scenario must resolve as a package entrypoint");
    let (cold, mut cargo_invocations) = sifr_driver::capture_cargo_invocations(|| {
        sifr_driver::build_cached_package_project(&entrypoint)
    });
    let cold = cold.expect("cold frozen prepared build should succeed");
    assert!(!cold.build_report().cache_hit());
    let (warm, warm_invocations) = sifr_driver::capture_cargo_invocations(|| {
        sifr_driver::build_cached_package_project(&entrypoint)
    });
    let warm = warm.expect("warm frozen build should succeed");
    assert!(warm.build_report().cache_hit());
    assert_eq!(cold.binary_path(), warm.binary_path());
    assert!(
        warm_invocations.is_empty(),
        "a warm binary cache hit must not launch Cargo: {warm_invocations:#?}"
    );

    for mode in [
        CargoLockMode::Frozen,
        CargoLockMode::Locked,
        CargoLockMode::Offline,
    ] {
        let (check_exit, captured) = sifr_driver::capture_cargo_invocations(|| {
            cmd_check(
                Some(source),
                None,
                &sifr_package::CargoPackageSelection::default(),
                mode,
                DiagnosticFormat::Human,
            )
        });
        cargo_invocations.extend(captured);
        assert_eq!(
            check_exit,
            EXIT_SUCCESS,
            "sifr check must preserve {} mode",
            mode.as_str()
        );
        let output = scenario.root.join(format!("build-{}", mode.as_str()));
        let (build_exit, captured) = sifr_driver::capture_cargo_invocations(|| {
            cmd_build(source, &output, mode, true, DiagnosticFormat::Human)
        });
        cargo_invocations.extend(captured);
        assert_eq!(
            build_exit,
            EXIT_SUCCESS,
            "sifr build must preserve {} mode",
            mode.as_str()
        );
        assert!(output
            .join("sifr_output/target/release/sifr_output")
            .is_file());
        let (run_exit, captured) = sifr_driver::capture_cargo_invocations(|| {
            cmd_run(
                Some("src/main.sifr"),
                None,
                None,
                &[],
                &[],
                mode,
                DiagnosticFormat::Human,
            )
        });
        cargo_invocations.extend(captured);
        assert_eq!(
            run_exit,
            EXIT_SUCCESS,
            "sifr run must preserve {} mode",
            mode.as_str()
        );
    }
    assert_cargo_invocations_preserve_lock_modes(&cargo_invocations);

    assert_eq!(
        std::fs::read(source_lock).expect("scenario lock should remain readable"),
        source_lock_before,
        "check/build/run must not mutate the authoritative package lock"
    );
}

fn assert_cargo_invocations_preserve_lock_modes(invocations: &[sifr_driver::CargoInvocation]) {
    assert!(
        !invocations.is_empty(),
        "the certification path must observe Cargo subprocesses"
    );
    for invocation in invocations {
        let requested = invocation
            .lock_mode
            .cargo_arg()
            .expect("the certification sink records only constrained modes");
        match invocation.phase {
            "package-metadata" | "final-build" => assert!(
                invocation.args.iter().any(|arg| arg == requested),
                "{} must preserve {requested}: {invocation:#?}",
                invocation.phase
            ),
            "rust-probe" => assert!(
                invocation.args.iter().any(|arg| arg == requested)
                    && invocation.args.iter().any(|arg| arg == "--frozen"),
                "a constrained Rust probe must preserve {requested} and frozen strength: \
                 {invocation:#?}"
            ),
            "resolution" => {
                assert!(
                    invocation.args.iter().any(|arg| arg == "metadata"),
                    "prepared resolution must use Cargo metadata: {invocation:#?}"
                );
                assert_eq!(
                    invocation.args.iter().any(|arg| arg == "--offline"),
                    invocation.lock_mode.is_network_disallowed(),
                    "prepared resolution must preserve the requested network policy: \
                     {invocation:#?}"
                );
            }
            phase => panic!("unclassified Cargo invocation phase `{phase}`: {invocation:#?}"),
        }
    }

    for mode in [
        CargoLockMode::Frozen,
        CargoLockMode::Locked,
        CargoLockMode::Offline,
    ] {
        let expected = mode
            .cargo_arg()
            .expect("certified non-normal mode must have a Cargo argument");
        let matching = invocations
            .iter()
            .filter(|invocation| invocation.lock_mode == mode && invocation.phase == "final-build")
            .collect::<Vec<_>>();
        assert!(
            !matching.is_empty(),
            "final-build must execute at least once in {} mode: {invocations:#?}",
            mode.as_str()
        );
        assert!(
            matching
                .iter()
                .all(|invocation| invocation.args.iter().any(|arg| arg == expected)),
            "every final-build must preserve {expected}: {matching:#?}"
        );
        let metadata = invocations
            .iter()
            .filter(|invocation| {
                invocation.lock_mode == mode && invocation.phase == "package-metadata"
            })
            .collect::<Vec<_>>();
        assert!(
            !metadata.is_empty(),
            "package metadata must execute at least once in {} mode: {invocations:#?}",
            mode.as_str()
        );
    }
    assert!(
        invocations
            .iter()
            .any(|invocation| invocation.phase == "rust-probe"),
        "the cold constrained path must execute Rust probes"
    );
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
#[doc = "sifr-evidence: executes-compiler-diagnostic"]
fn test_lockfile_feature_and_frozen_drift_rejected_without_network() {
    for case in [
        DriftCase::MissingLock,
        DriftCase::StaleVersion,
        DriftCase::Checksum,
        DriftCase::Source,
        DriftCase::Feature,
    ] {
        assert_drift_rejected(case);
    }
}

#[derive(Clone, Copy, Debug)]
enum DriftCase {
    MissingLock,
    StaleVersion,
    Checksum,
    Source,
    Feature,
}

fn assert_drift_rejected(case: DriftCase) {
    let scenario = Scenario::copy(&format!("negative_{case:?}"));
    let _cwd = CurrentDirGuard::enter(&scenario.root);
    move_rust_declarations_to_imported_module(&scenario.root);
    let lock_path = scenario.root.join("Cargo.lock");
    let manifest_path = scenario.root.join("rust/locked_bridge/Cargo.toml");
    match case {
        DriftCase::MissingLock => {
            std::fs::remove_file(&lock_path).expect("negative case should remove copied lock");
        }
        DriftCase::StaleVersion => replace_file(
            &lock_path,
            "version = \"2.14.0\"",
            "version = \"2.99.0\"",
        ),
        DriftCase::Checksum => replace_file(
            &lock_path,
            "d466e9454f08e4a911e14806c24e16fba1b4c121d1ea474396f396069cf949d9",
            "0466e9454f08e4a911e14806c24e16fba1b4c121d1ea474396f396069cf949d9",
        ),
        DriftCase::Source => replace_file(
            &lock_path,
            "registry+https://github.com/rust-lang/crates.io-index",
            "git+https://invalid.example/indexmap?rev=deadbeef#deadbeef",
        ),
        DriftCase::Feature => replace_file(
            &manifest_path,
            "indexmap = { version = \"=2.14.0\", default-features = false }",
            "indexmap = { version = \"=2.14.0\", default-features = false, features = [\"serde\"] }",
        ),
    }
    let lock_before = std::fs::read(&lock_path).ok();
    let (exit_code, diagnostics) = crate::diagnostic_test_sink::capture(|| {
        cmd_check(
            Some(Path::new("src/main.sifr")),
            None,
            &sifr_package::CargoPackageSelection::default(),
            CargoLockMode::Frozen,
            DiagnosticFormat::Compact,
        )
    });
    assert_eq!(
        exit_code, EXIT_USER_DIAGNOSTIC,
        "{case:?} must fail through the Sifr check command"
    );
    let [diagnostic] = diagnostics.as_slice() else {
        panic!("{case:?} must emit exactly one CLI diagnostic: {diagnostics:#?}");
    };
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::RUST_CARGO_METADATA.code(),
        "{case:?} imported Rust declaration must map to the stable Rust Cargo diagnostic"
    );
    assert!(
        diagnostic.message.contains(case.expected_reason()),
        "{case:?} diagnostic must report its Cargo failure reason: {diagnostic:#?}"
    );
    assert_eq!(
        std::fs::read(&lock_path).ok(),
        lock_before,
        "{case:?} must not write the lockfile in frozen mode"
    );
}

impl DriftCase {
    const fn expected_reason(self) -> &'static str {
        match self {
            Self::MissingLock => "missing lockfile",
            Self::Checksum => "checksum drift",
            Self::StaleVersion => "stale selected version",
            Self::Source => "dependency source drift",
            Self::Feature => "requested feature selection drift",
        }
    }
}

fn move_rust_declarations_to_imported_module(root: &Path) {
    std::fs::write(
        root.join("src/bridge.sifr"),
        r#"@rust(locked_bridge.cached_hash, panic=trusted_no_panic)
def cached_hash(input: bytes) -> uint32: ...

@rust(locked_bridge.lockfile_generation, panic=trusted_no_panic)
def lockfile_generation() -> uint32: ...
"#,
    )
    .expect("imported Rust bridge module should be written");
    std::fs::write(
        root.join("src/main.sifr"),
        r#"from bridge import cached_hash, lockfile_generation

def main() -> None:
    print(lockfile_generation())
    print(cached_hash(b"sifr-rust-interop"))
"#,
    )
    .expect("entry module without Rust decorators should be written");
}

fn replace_file(path: &Path, before: &str, after: &str) {
    let source = std::fs::read_to_string(path).expect("mutation source should be readable");
    let mutated = source.replacen(before, after, 1);
    assert_ne!(mutated, source, "negative mutation must change its target");
    std::fs::write(path, mutated).expect("negative mutation should be installed");
}

fn copy_tree(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).expect("scenario destination should be created");
    for entry in std::fs::read_dir(source).expect("scenario source should be readable") {
        let entry = entry.expect("scenario entry should be readable");
        let target = destination.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("scenario file should be copied");
        }
    }
}
