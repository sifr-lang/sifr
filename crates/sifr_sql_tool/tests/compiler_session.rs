#![cfg(feature = "__compiler_fixture")]
use sifr_sql_tool::compiler_session::CompilerSession;
use std::process::Command;
use std::time::{Duration, Instant};

const SOURCE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn start(mode: &str, deadline: Duration) -> Result<CompilerSession, String> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sifr-sql-compiler-fixture"));
    command.arg(mode);
    CompilerSession::spawn_admitted(&mut command, SOURCE, deadline)
}

#[test]
fn process_session_reuses_one_worker_and_requires_clean_shutdown() {
    let mut session = start("serve", Duration::from_secs(5)).unwrap();
    for _ in 0..3 {
        let declarations = session.migration("fixture source").unwrap();
        assert_eq!(declarations[0].id.as_str(), "fixture");
    }
    // Idle time belongs to the caller; only exchanges have a deadline.
    std::thread::sleep(Duration::from_millis(20));
    session.finish().unwrap();
}

#[test]
fn worker_selection_mismatch_death_and_silent_handshake_are_failures() {
    for mode in ["wrong-identity", "death", "silent", "descendant-pipe"] {
        let start_time = Instant::now();
        assert!(start(mode, Duration::from_millis(500)).is_err(), "{mode}");
        assert!(start_time.elapsed() < Duration::from_secs(5), "{mode}");
    }
}

#[test]
fn request_timeout_closes_the_worker_and_session() {
    let mut session = start("hang-request", Duration::from_millis(500)).unwrap();
    let started = Instant::now();
    assert!(
        session
            .migration("source")
            .unwrap_err()
            .contains("deadline")
    );
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(session.migration("source").unwrap_err().contains("closed"));
}

#[test]
fn worker_diagnostic_survives_and_prevents_reuse() {
    let mut session = start("diagnostic", Duration::from_secs(5)).unwrap();
    assert_eq!(
        session.migration("source").unwrap_err(),
        "line 3: test diagnostic\nline 4: test detail"
    );
    assert!(session.migration("source").unwrap_err().contains("closed"));
}

#[test]
fn shutdown_acknowledgement_does_not_hide_failed_or_hanging_exit() {
    for mode in ["bad-exit", "hang-exit"] {
        let session = start(mode, Duration::from_millis(500)).unwrap();
        let started = Instant::now();
        assert!(session.finish().is_err(), "{mode}");
        assert!(started.elapsed() < Duration::from_secs(5), "{mode}");
    }
}

#[test]
fn zero_deadline_fails_before_spawning() {
    assert!(start("serve", Duration::ZERO).is_err());
}
