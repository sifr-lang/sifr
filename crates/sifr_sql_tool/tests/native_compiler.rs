#![cfg(all(unix, feature = "__compiler_fixture"))]
use sifr_process::OwnedChild;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

fn command(mode: &str, root: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sifr-sql-compiler-fixture"));
    command
        .arg(mode)
        .current_dir(root)
        .env_remove("SIFR_TOOL_NAMESPACE")
        .env_remove("SIFR_TOOL_SQL_COMPILER")
        .env_remove("SIFR_TOOL_CAPABILITIES")
        .env_remove("SIFR_SQL_COMPILER")
        .env_remove("SIFR_FIXTURE_WORKER_PID")
        .env_remove("SIFR_FIXTURE_READY")
        .env(
            "SIFR_SYSROOT",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    command
}
fn wait(child: &mut OwnedChild) -> ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status;
        }
        assert!(Instant::now() < deadline, "fixture did not terminate");
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn unused_controller_never_selects_or_launches_a_compiler() {
    let root = tempfile::tempdir().unwrap();
    let mut command = command("native-unused", root.path());
    command.env("SIFR_SQL_COMPILER", "/missing/explicit/compiler");
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    assert!(wait(&mut child).success());
}
#[test]
fn inherited_channel_requires_capability_and_an_anonymous_socket() {
    let root = tempfile::tempdir().unwrap();
    for (mode, grant) in [
        ("stdin-v1", ""),
        ("wrong-version", "sql-compiler"),
        ("stdin-v1", "sql-compiler"),
    ] {
        let mut command = command("native-once", root.path());
        command
            .env("SIFR_TOOL_NAMESPACE", "sql")
            .env("SIFR_TOOL_SQL_COMPILER", mode)
            .env("SIFR_TOOL_CAPABILITIES", grant);
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        assert!(!wait(&mut child).success(), "{mode}/{grant}");
    }
    let mut command = command("native-once", root.path());
    command.env("SIFR_TOOL_NAMESPACE", "sql").env(
        "SIFR_SQL_COMPILER",
        env!("CARGO_BIN_EXE_sifr-sql-compiler-fixture"),
    );
    let mut child = OwnedChild::spawn(&mut command).unwrap();
    assert!(
        !wait(&mut child).success(),
        "namespace must not fall back to direct execution"
    );
}
#[test]
fn inherited_channel_roundtrip_and_shutdown_use_one_owned_worker() {
    let root = tempfile::tempdir().unwrap();
    let (client, server) = UnixStream::pair().unwrap();
    let mut worker = command("serve", root.path());
    worker
        .stdin(Stdio::from(OwnedFd::from(server.try_clone().unwrap())))
        .stdout(Stdio::from(OwnedFd::from(server)));
    let mut worker = OwnedChild::spawn(&mut worker).unwrap();
    let mut native = command("native-once", root.path());
    native
        .env("SIFR_TOOL_NAMESPACE", "sql")
        .env("SIFR_TOOL_SQL_COMPILER", "stdin-v1")
        .env("SIFR_TOOL_CAPABILITIES", "sql-compiler")
        .stdin(Stdio::from(OwnedFd::from(client)));
    let mut native = OwnedChild::spawn(&mut native).unwrap();
    assert!(wait(&mut native).success());
    assert!(wait(&mut worker).success());
}
#[test]
fn direct_worker_finishes_and_idle_cancellation_reaps_it() {
    let root = tempfile::tempdir().unwrap();
    let mut native = command("native-once", root.path());
    native.env(
        "SIFR_SQL_COMPILER",
        env!("CARGO_BIN_EXE_sifr-sql-compiler-fixture"),
    );
    let mut native = OwnedChild::spawn(&mut native).unwrap();
    assert!(wait(&mut native).success());
    for signal in ["TERM", "INT"] {
        let ready = root.path().join(format!("ready-{signal}"));
        let worker_pid = root.path().join(format!("worker-{signal}"));
        let mut native = command("native-idle", root.path());
        native
            .env(
                "SIFR_SQL_COMPILER",
                env!("CARGO_BIN_EXE_sifr-sql-compiler-fixture"),
            )
            .env("SIFR_FIXTURE_READY", &ready)
            .env("SIFR_FIXTURE_WORKER_PID", &worker_pid);
        let mut native = OwnedChild::spawn(&mut native).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() {
            assert!(
                native.try_wait().unwrap().is_none(),
                "native fixture failed before ready"
            );
            assert!(
                Instant::now() < deadline,
                "native fixture did not become idle"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        let worker_pid = std::fs::read_to_string(worker_pid).unwrap();
        assert!(
            Command::new("kill")
                .args([&format!("-{signal}"), &native.id().to_string()])
                .status()
                .unwrap()
                .success()
        );
        assert_eq!(
            wait(&mut native).code(),
            Some(if signal == "TERM" { 143 } else { 130 })
        );
        assert!(
            !Command::new("kill")
                .args(["-0", &worker_pid])
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success(),
            "owned compiler survived native cancellation"
        );
    }
}
