//! Real native entrypoints and canonical compiler through the production sandbox.
use super::{Project, compiler_worker, host_tool_sandbox};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn binary(family: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_BIN_EXE_sifr"))
        .parent()
        .expect("compiler target directory")
        .join(format!("sifr-sql-{family}"));
    assert!(
        path.is_file(),
        "prepare the SQL native executables before this contract: {}",
        path.display()
    );
    path
}
fn direct(project: &Project, family: &str, args: &[&str], compiler: &Path) -> Output {
    Command::new(binary(family))
        .args(args)
        .current_dir(project.root.path())
        .env("SIFR_SQL_COMPILER", compiler)
        .env(
            "SIFR_SYSROOT",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        )
        .env_remove("SIFR_TOOL_NAMESPACE")
        .env_remove("SIFR_TOOL_SQL_COMPILER")
        .env_remove("SIFR_TOOL_CAPABILITIES")
        .env_remove("SIFR_SQL_DATABASE_URL")
        .env_remove("SIFR_SQL_DATABASE_PATH")
        .output()
        .expect("real native command")
}
fn confined(
    project: &Project,
    family: &str,
    args: &[&str],
    compiler: &Path,
    semantic_grant: bool,
) -> Result<host_tool_sandbox::SandboxedToolOutput, String> {
    let executable = binary(family);
    let args = args
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let mut capabilities = BTreeSet::from(["project-write".to_string()]);
    if semantic_grant {
        capabilities.insert("sql-compiler".to_string());
    }
    let executable_hash = sifr_sysroot::sha256_file(&executable).expect("native binary identity");
    host_tool_sandbox::run_sandboxed_tool(&host_tool_sandbox::SandboxedToolRequest {
        executable: &executable,
        compiler_executable: compiler,
        args: &args,
        workspace_root: project.root.path(),
        namespace: "sql",
        capabilities: &capabilities,
        // This boundary test directly supplies trusted executables. Full
        // package/lock admission remains exercised by host_tool_cli.rs.
        package_checksum: "fixture-package",
        lockfile_fingerprint: "fixture-lock",
        executable_hash: &executable_hash,
        stream_output: false,
    })
}
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(root: &Path, path: &Path, output: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(path).expect("artifact directory") {
            let path = entry.expect("artifact entry").path();
            if path.is_dir() {
                collect(root, &path, output);
            } else {
                output.insert(
                    path.strip_prefix(root).expect("artifact path").to_owned(),
                    std::fs::read(path).expect("artifact bytes"),
                );
            }
        }
    }
    let mut output = BTreeMap::new();
    collect(root, root, &mut output);
    output
}
fn compiler() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_sifr"))
}

#[test]
fn native_schema_commands_match_direct_execution_through_the_real_sandbox() {
    for (family, version) in [("sqlite", "3.53.4"), ("mysql", "8.4"), ("postgresql", "18")] {
        let project = compiler_worker::project(family, version);
        let args = ["schema", "build", "--profile", "app"];
        let expected = direct(&project, family, &args, compiler());
        assert!(
            expected.status.success(),
            "{}",
            String::from_utf8_lossy(&expected.stderr)
        );
        let artifacts = project.root.path().join(".sifr/sql/app");
        let expected_files = files(&artifacts);
        let actual = confined(&project, family, &args, compiler(), true).expect("confined command");
        assert!(
            actual.status.success(),
            "{}",
            String::from_utf8_lossy(&actual.stderr)
        );
        assert_eq!(actual.stdout, expected.stdout);
        assert_eq!(actual.stderr, expected.stderr);
        assert_eq!(files(&artifacts), expected_files);
    }
}

#[test]
fn semantic_requests_reject_missing_grants_and_wrong_compiler_executables() {
    let project = compiler_worker::project("sqlite", "3.53.4");
    let args = ["schema", "build", "--profile", "app"];
    let denied = confined(&project, "sqlite", &args, compiler(), false).expect("capability denial");
    assert!(!denied.status.success());
    assert!(String::from_utf8_lossy(&denied.stderr).contains("sql-compiler capability"));
    let wrong =
        confined(&project, "sqlite", &args, &binary("sqlite"), true).expect("wrong worker exit");
    assert!(!wrong.status.success());
    assert!(String::from_utf8_lossy(&wrong.stderr).contains("SQL compiler"));
    assert!(!project.root.path().join(".sifr/sql/app").exists());
}

#[test]
fn sqlite_provision_and_cleanup_keep_database_ownership_inside_the_sandbox() {
    let project = compiler_worker::project("sqlite", "3.53.4");
    let output = confined(
        &project,
        "sqlite",
        &["test", "provision", "--profile", "app"],
        compiler(),
        true,
    )
    .expect("confined provisioning");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest = sifr_sql_contract::TestConnectionManifest::from_json(
        std::str::from_utf8(&output.stdout).expect("manifest UTF-8"),
    )
    .expect("connection manifest");
    let sifr_sql_contract::ProvisionedConnection::File { path } = manifest.connection else {
        panic!("SQLite must provision an owned file");
    };
    assert!(Path::new(&path).starts_with(project.root.path()));
    let connection = rusqlite::Connection::open(&path).expect("provisioned database");
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .expect("integrity");
    assert_eq!(integrity, "ok");
    drop(connection);
    let cleanup = confined(
        &project,
        "sqlite",
        &[
            "test",
            "cleanup",
            "--resource-id",
            &manifest.cleanup.resource_id,
        ],
        Path::new("/missing/compiler-must-not-start"),
        true,
    )
    .expect("lazy cleanup");
    assert!(
        cleanup.status.success(),
        "{}",
        String::from_utf8_lossy(&cleanup.stderr)
    );
    assert!(!Path::new(&path).exists());
}

#[test]
fn reading_a_missing_plan_is_lazy_even_with_a_granted_compiler_channel() {
    let project = compiler_worker::project("sqlite", "3.53.4");
    let args = ["migration", "plan", "--profile", "app"];
    let missing = Path::new("/missing/compiler-must-not-start");
    let expected = direct(&project, "sqlite", &args, missing);
    let actual = confined(&project, "sqlite", &args, missing, true).expect("lazy plan read");
    assert_eq!(actual.status.code(), expected.status.code());
    assert_eq!(actual.stdout, expected.stdout);
    assert_eq!(actual.stderr, expected.stderr);
    assert!(String::from_utf8_lossy(&actual.stderr).contains("migration plan"));
}

#[test]
fn lazy_compiler_rejects_lock_drift_before_the_first_request() {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let project = compiler_worker::project("sqlite", "3.53.4");
    let root = project.root.path();
    let mut command = Command::new(binary("sqlite"));
    command
        .args(["schema", "build", "--profile", "app"])
        .current_dir(root)
        .env("SIFR_TOOL_NAMESPACE", "sql")
        .env("SIFR_TOOL_CAPABILITIES", "project-write,sql-compiler")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut service =
        super::host_tool_compiler::LazyCompiler::attach(&mut command, root, compiler())
            .expect("admit the invocation before the tool starts");
    let lock = root.join("Cargo.lock");
    let mut changed = std::fs::read(&lock).expect("initial lock");
    changed.extend_from_slice(b"\n# changed after parent admission\n");
    std::fs::write(&lock, changed).expect("lock drift before first request");
    let mut child = sifr_process::OwnedChild::spawn(&mut command).expect("native client");
    drop(command);
    let deadline = Instant::now() + Duration::from_secs(120);
    let mut rejected = None;
    let status = loop {
        if let Err(message) = service.poll() {
            rejected = Some(message);
            child.terminate();
            break child.wait().expect("reap rejected client");
        }
        if let Some(status) = child.try_wait().expect("client status") {
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "compiler admission test exceeded its deadline"
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("stderr")
        .read_to_string(&mut stderr)
        .expect("diagnostic");
    service.finish(status.success()).expect("worker cleanup");
    assert!(
        !status.success(),
        "compiler accepted a lock changed after caller admission"
    );
    let diagnostic = rejected.as_deref().unwrap_or(&stderr);
    assert!(diagnostic.contains("changed"), "{diagnostic}");
    assert!(
        !root.join(".sifr/sql/app").exists(),
        "rejected inputs must not publish artifacts"
    );
}
