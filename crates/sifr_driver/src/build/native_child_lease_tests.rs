use super::*;
use crate::build::cargo_resolution::{CargoResolutionPolicy, prepare_cargo_resolution};
use crate::build::native_storage::{NativeFamily, publication_lock};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::process::{Child, Stdio};
use std::time::{Duration, Instant};

struct ChildGroup(Child, bool);
impl ChildGroup {
    fn spawn(command: &mut Command) -> Self {
        command
            .process_group(0)
            .stdin(Stdio::null())
            .stdout(Stdio::null());
        Self(command.spawn().expect("owned child"), false)
    }
    #[allow(unsafe_code)]
    fn stop(&mut self) {
        if self.1 {
            return;
        }
        // SAFETY: this guard owns the process group created immediately above.
        unsafe {
            libc::kill(-(self.0.id() as i32), libc::SIGKILL);
        }
        self.0.wait().expect("reap owned child");
        self.1 = true;
    }
}
impl Drop for ChildGroup {
    fn drop(&mut self) {
        self.stop();
    }
}

fn isolated(name: &str) -> bool {
    const CHILD: &str = "SIFR_PRODUCTION_LEASE_CONTROL_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        return false;
    }
    let root = tempfile::Builder::new()
        .permissions(std::fs::Permissions::from_mode(0o700))
        .prefix("sifr-production-lease-")
        .tempdir()
        .unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args([
            &format!("build::native_child_leases::tests::{name}"),
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(CHILD, name)
        .env("SIFR_CACHE_DIR", root.path().join("cache"));
    let output =
        crate::process_execution::output_with_deadline(&mut command, Duration::from_secs(120))
            .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"));
    true
}

fn lock_files(root: &Path) -> Vec<File> {
    std::fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(entry.unwrap().path())
                .unwrap()
        })
        .collect()
}

fn assert_busy(locks: &[File]) {
    for lock in locks {
        assert!(
            matches!(lock.try_lock(), Err(std::fs::TryLockError::WouldBlock)),
            "intended child lost its lease"
        );
    }
}

fn assert_released(locks: &[File]) {
    for lock in locks {
        let start = Instant::now();
        loop {
            match lock.try_lock() {
                Ok(()) => {
                    lock.unlock().unwrap();
                    break;
                }
                Err(std::fs::TryLockError::WouldBlock)
                    if start.elapsed() < Duration::from_secs(1) =>
                {
                    std::thread::sleep(Duration::from_millis(5))
                }
                other => panic!("unrelated live child retained lease: {other:?}"),
            }
        }
    }
}

#[test]
fn production_policy_retains_only_its_cargo_child() {
    if isolated("production_policy_retains_only_its_cargo_child") {
        return;
    }
    let cache = crate::cache_storage::root();
    let family = NativeFamily::acquire("child-control", "sources", "environment", "trust").unwrap();
    let project = family.project(&cache, "lease_control").unwrap();
    let publication = publication_lock(&project).unwrap();
    std::fs::create_dir_all(project.join("src")).unwrap();
    std::fs::write(
        project.join("Cargo.toml"),
        "[package]\nname='lease_control'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(project.join("src/main.rs"), "fn main() {}\n").unwrap();
    let authority = cache.join("authority.lock");
    std::fs::write(&authority, "version = 4\n").unwrap();
    let mut base = CargoResolutionPolicy::normal();
    base.lock_mode = sifr_package::CargoLockMode::Locked;
    base.cargo_vendor_mode = sifr_stdlib_manifest::CargoVendorMode::PackageOwned;
    base.authoritative_locks.push(authority);
    let policy = base
        .with_lease(family.lease())
        .unwrap()
        .with_lease(publication.lease())
        .unwrap();
    assert_eq!(
        base.normal_seed_cache_fragment(),
        policy.normal_seed_cache_fragment()
    );
    let prepared = prepare_cargo_resolution(&project, &policy, &[]).unwrap();
    let ready = prepared.command_policy(&policy).unwrap();
    assert_eq!(
        ready.child_leases.files.len(),
        3,
        "family, publication, resolution"
    );
    let mut locks = lock_files(&cache.join("native/families/.locks"));
    locks.extend(lock_files(&cache.join("native/publications/.locks")));
    locks.extend(lock_files(
        &cache.join("native/artifacts/cargo_resolution/.locks"),
    ));
    assert_eq!(locks.len(), 3);
    let mut unrelated = ChildGroup::spawn(Command::new("sleep").arg("30"));
    let mut command = ready.cargo_command().unwrap();
    command.args([
        "--config",
        "alias.sifr-lease-control='!sleep'",
        "sifr-lease-control",
        "30",
    ]);
    let mut intended = ChildGroup::spawn(&mut command);
    drop(command);
    drop(ready);
    drop(prepared);
    drop(policy);
    drop(base);
    drop(publication);
    drop(family);
    // Cargo itself and its actual shell-alias descendant retain the owned leases.
    std::thread::sleep(Duration::from_millis(150));
    assert!(
        intended.0.try_wait().unwrap().is_none(),
        "Cargo control exited prematurely"
    );
    assert_busy(&locks);
    intended.stop();
    assert!(unrelated.0.try_wait().unwrap().is_none());
    assert_released(&locks);
}

#[test]
fn native_policy_failed_spawn_releases_after_command_disposal() {
    if isolated("native_policy_failed_spawn_releases_after_command_disposal") {
        return;
    }
    let cache = crate::cache_storage::root();
    let family = NativeFamily::acquire("failed-spawn", "sources", "environment", "trust").unwrap();
    let policy = CargoResolutionPolicy::normal()
        .with_lease(family.lease())
        .unwrap();
    let locks = lock_files(&cache.join("native/families/.locks"));
    let mut command = Command::new(cache.join("absent-executable"));
    policy.child_leases.bind(&mut command).unwrap();
    drop(policy);
    drop(family);
    assert!(command.spawn().is_err());
    assert_busy(&locks);
    drop(command);
    assert_released(&locks);
}

#[test]
fn generated_test_releases_family_before_artifact_execution() {
    if isolated("generated_test_releases_family_before_artifact_execution") {
        return;
    }
    let scope = crate::cache_storage::root().join("input");
    crate::cache_storage::directory(&scope).unwrap();
    let project = crate::test_runner::GeneratedTestRunnerProject {
        application_profile: crate::ApplicationProfile::Test,
        cargo_resolution: CargoResolutionPolicy::normal(),
        interop: sifr_codegen::InteropBuildPlan::default(),
        cache_scope: scope,
        support_module_names: Vec::new(), support_rust_files: Default::default(),
        bridge_rust_files: Default::default(), all_stdlib_modules: Default::default(),
        all_required_features: Default::default(),
        all_rust_code: r#"#[test]
fn finalized_artifact_has_no_mutable_family_owner() {
    let cache = std::path::PathBuf::from(std::env::var_os("SIFR_CACHE_DIR").unwrap());
    let locks = std::fs::read_dir(cache.join("native/families/.locks")).unwrap();
    let mut count = 0;
    for entry in locks {
        let file = std::fs::OpenOptions::new().read(true).write(true).open(entry.unwrap().path()).unwrap();
        file.try_lock().expect("native family must be released before test execution");
        count += 1;
    }
    assert_eq!(count, 1);
    let executable = std::env::current_exe().unwrap();
    let root = executable.ancestors().find(|p| p.join("artifact_cache.json").is_file()).unwrap();
    let lease = std::fs::File::open(root.parent().unwrap().join(".locks").join(root.file_name().unwrap())).unwrap();
    assert!(matches!(lease.try_lock(), Err(std::fs::TryLockError::WouldBlock)), "executed artifact must stay leased");
}"#.into(),
    };
    let outcome = crate::test_runner::execute_test_runner_project(&project).unwrap();
    assert!(outcome.success);
}
