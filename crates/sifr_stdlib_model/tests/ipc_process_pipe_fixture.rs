#![cfg(unix)]
#![allow(clippy::expect_used)]

use sifr_stdlib_model::{
    read_frame, write_frame, IpcConnectionConfig, IpcConnectionState, IpcEnvelope,
    IpcMalformedKind, IpcShutdownMode, IpcTerminationReason, IpcWireSchema,
    IPC_DEFAULT_MAX_FRAME_BYTES,
};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Mutex, MutexGuard};

static WORKER_STARTUP_LOCK: Mutex<()> = Mutex::new(());

fn sample_schema() -> IpcWireSchema {
    IpcWireSchema {
        name: "demo.worker.Echo".to_string(),
        version: 1,
        hash: 0x4733_c89f_b23a_40ec_b5f3_bcda_99fb_34da_u128.to_be_bytes(),
        compatible_version_min: 1,
        compatible_version_max: 1,
    }
}

struct WorkerProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    startup_lock: Option<MutexGuard<'static, ()>>,
}

impl WorkerProcess {
    fn spawn() -> Self {
        let startup_lock = WORKER_STARTUP_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let repo_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .expect("stdlib model crate lives under crates/sifr_stdlib_model");
        let target_dir = repo_root
            .join("target")
            .join("ipc_process_pipe_fixture_worker");
        let mut child =
            Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
                .arg("run")
                .arg("--quiet")
                .arg("--manifest-path")
                .arg(manifest_dir.join("Cargo.toml"))
                .arg("--target-dir")
                .arg(target_dir)
                .arg("--features")
                .arg("__test_fixture")
                .arg("--bin")
                .arg("sifr-stdlib-ipc-pipe-fixture-worker")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("spawn IPC fixture worker");
        let stdin = child.stdin.take().expect("worker stdin is piped");
        let stdout = child.stdout.take().expect("worker stdout is piped");
        Self {
            child,
            stdin,
            stdout,
            startup_lock: Some(startup_lock),
        }
    }

    fn release_startup_lock(&mut self) {
        drop(self.startup_lock.take());
    }

    fn finish(self) {
        drop(self.stdin);
        drop(self.stdout);
        let output = self
            .child
            .wait_with_output()
            .expect("wait for IPC fixture worker");
        assert!(
            output.status.success(),
            "worker failed: status={:?}, stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn connect(worker: &mut WorkerProcess) -> IpcConnectionState {
    let mut connection = IpcConnectionState::new(IpcConnectionConfig::new(sample_schema()))
        .expect("sample connection config is valid");
    let hello = connection
        .begin_parent_handshake()
        .expect("initialized parent can begin handshake");
    write_frame(&mut worker.stdin, &hello, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("write hello to worker stdin");
    let ready = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read worker bootstrap frame")
        .expect("worker emits bootstrap frame");
    worker.release_startup_lock();
    connection
        .accept_worker_bootstrap(&ready)
        .expect("worker ready is accepted");
    connection
}

#[test]
fn unix_child_process_pipes_complete_run_and_shutdown() {
    let mut worker = WorkerProcess::spawn();
    let mut connection = connect(&mut worker);
    let run = IpcEnvelope::Run {
        request_id: 7,
        payload: b"hello".to_vec(),
    };

    connection
        .apply_established_frame(&run)
        .expect("parent reserves outbound run");
    write_frame(&mut worker.stdin, &run, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("write run to worker stdin");

    let started = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read started frame")
        .expect("worker emits started frame");
    assert_eq!(started, IpcEnvelope::Started { request_id: 7 });
    connection
        .apply_established_frame(&started)
        .expect("parent accepts started frame");

    let completed = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read completed frame")
        .expect("worker emits completed frame");
    assert_eq!(
        completed,
        IpcEnvelope::Completed {
            request_id: 7,
            payload: b"hello".to_vec(),
        }
    );
    connection
        .apply_established_frame(&completed)
        .expect("parent accepts completed frame");
    assert_eq!(connection.in_flight_len(), 0);

    shutdown(&mut worker, &mut connection, IpcTerminationReason::Shutdown);
    worker.finish();
}

#[test]
fn unix_child_process_pipes_cancel_in_flight_request() {
    let mut worker = WorkerProcess::spawn();
    let mut connection = connect(&mut worker);
    let run = IpcEnvelope::Run {
        request_id: 11,
        payload: b"hold".to_vec(),
    };

    connection
        .apply_established_frame(&run)
        .expect("parent reserves held run");
    write_frame(&mut worker.stdin, &run, IPC_DEFAULT_MAX_FRAME_BYTES).expect("write held run");
    let started = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read started frame")
        .expect("worker starts held run");
    connection
        .apply_established_frame(&started)
        .expect("parent accepts started frame");

    let cancel = IpcEnvelope::Cancel { request_id: 11 };
    connection
        .apply_established_frame(&cancel)
        .expect("parent accepts outbound cancel");
    write_frame(&mut worker.stdin, &cancel, IPC_DEFAULT_MAX_FRAME_BYTES).expect("write cancel");
    let failed = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read failed frame")
        .expect("worker emits cancellation failure");
    assert_eq!(
        failed,
        IpcEnvelope::Failed {
            request_id: 11,
            error: b"cancelled".to_vec(),
        }
    );
    connection
        .apply_established_frame(&failed)
        .expect("parent accepts failed frame");
    assert_eq!(connection.in_flight_len(), 0);

    shutdown(&mut worker, &mut connection, IpcTerminationReason::Shutdown);
    worker.finish();
}

#[test]
fn unix_child_process_pipes_report_backpressure_full() {
    let mut worker = WorkerProcess::spawn();
    let mut connection = connect(&mut worker);
    let first = IpcEnvelope::Run {
        request_id: 13,
        payload: b"hold".to_vec(),
    };

    connection
        .apply_established_frame(&first)
        .expect("parent reserves held run");
    write_frame(&mut worker.stdin, &first, IPC_DEFAULT_MAX_FRAME_BYTES).expect("write held run");
    let started = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read started frame")
        .expect("worker starts held run");
    assert_eq!(started, IpcEnvelope::Started { request_id: 13 });
    connection
        .apply_established_frame(&started)
        .expect("parent accepts started frame");

    let second = IpcEnvelope::Run {
        request_id: 14,
        payload: b"second".to_vec(),
    };
    write_frame(&mut worker.stdin, &second, IPC_DEFAULT_MAX_FRAME_BYTES).expect("write second run");
    let malformed = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read backpressure frame")
        .expect("worker emits backpressure frame");
    assert_eq!(
        malformed,
        IpcConnectionState::protocol_error_frame(IpcMalformedKind::RequestId, "backpressure_full")
    );
    connection
        .apply_established_frame(&malformed)
        .expect("parent closes on backpressure protocol error");
    assert_eq!(
        connection.phase(),
        sifr_stdlib_model::IpcConnectionPhase::Closed
    );

    worker.finish();
}

#[test]
fn unix_child_process_pipes_report_unsupported_payload() {
    let mut worker = WorkerProcess::spawn();
    let mut connection = connect(&mut worker);
    let run = IpcEnvelope::Run {
        request_id: 17,
        payload: b"unsupported:sifr.process.Child".to_vec(),
    };

    connection
        .apply_established_frame(&run)
        .expect("parent reserves unsupported payload run");
    write_frame(&mut worker.stdin, &run, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("write unsupported payload run");
    let unsupported = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read unsupported-payload frame")
        .expect("worker emits unsupported-payload frame");
    assert_eq!(
        unsupported,
        IpcEnvelope::UnsupportedPayload {
            type_name: "sifr.process.Child".to_string(),
        }
    );
    connection
        .apply_established_frame(&unsupported)
        .expect("parent closes on unsupported payload");
    assert_eq!(
        connection.phase(),
        sifr_stdlib_model::IpcConnectionPhase::Closed
    );

    worker.finish();
}

#[test]
fn unix_child_process_pipes_report_malformed_frame() {
    let mut worker = WorkerProcess::spawn();
    let mut connection = connect(&mut worker);

    worker
        .stdin
        .write_all(&3_u32.to_le_bytes())
        .expect("write malformed frame length");
    worker
        .stdin
        .write_all(&[0xaa])
        .expect("write truncated payload");
    worker.stdin.flush().expect("flush malformed frame");
    drop(worker.stdin);

    let malformed = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read malformed-frame report")
        .expect("worker emits malformed-frame report");
    assert_eq!(
        malformed,
        IpcConnectionState::protocol_error_frame(IpcMalformedKind::Truncated, "truncated")
    );
    connection
        .apply_established_frame(&malformed)
        .expect("parent closes on protocol error frame");
    assert_eq!(
        connection.phase(),
        sifr_stdlib_model::IpcConnectionPhase::Closed
    );

    let output = worker
        .child
        .wait_with_output()
        .expect("wait for malformed worker");
    assert!(
        output.status.success(),
        "worker failed: status={:?}, stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn shutdown(
    worker: &mut WorkerProcess,
    connection: &mut IpcConnectionState,
    expected_reason: IpcTerminationReason,
) {
    let shutdown = IpcEnvelope::Shutdown {
        mode: IpcShutdownMode::Drain,
    };
    connection
        .apply_established_frame(&shutdown)
        .expect("parent enters draining state");
    write_frame(&mut worker.stdin, &shutdown, IPC_DEFAULT_MAX_FRAME_BYTES).expect("write shutdown");
    let terminating = read_frame(&mut worker.stdout, IPC_DEFAULT_MAX_FRAME_BYTES)
        .expect("read terminating frame")
        .expect("worker emits terminating frame");
    assert_eq!(
        terminating,
        IpcEnvelope::Terminating {
            reason: expected_reason
        }
    );
    connection
        .apply_established_frame(&terminating)
        .expect("parent accepts terminating frame");
}
