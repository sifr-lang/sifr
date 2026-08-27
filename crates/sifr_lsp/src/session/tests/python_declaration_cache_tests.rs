use super::python_declaration_tests::{SOURCE, open_fixture, request};
use crate::request_queue::CancellationTarget;
use lsp_server::RequestId;
use serde_json::json;

#[test]
fn debug_cache_stats_reports_python_declaration_work() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);
    let _ = request(&mut session, "textDocument/completion", &uri, 6, 15);
    let stats = crate::requests::handle(&mut session, "sifr/debugCacheStats", json!({}))
        .expect("debug cache stats request");

    assert_eq!(stats.pointer("/pythonDeclarations/hits"), Some(&json!(0)));
    assert_eq!(stats.pointer("/pythonDeclarations/misses"), Some(&json!(1)));
    assert_eq!(
        stats.pointer("/pythonDeclarations/externalFingerprintRuns"),
        Some(&json!(1))
    );
    assert_eq!(
        stats.pointer("/pythonDeclarations/snapshotBuilds"),
        Some(&json!(1))
    );
}

#[test]
fn cancelled_python_declaration_request_stops_before_probe() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);
    let id = RequestId::from(150);
    session
        .enqueue_request(
            &id,
            "textDocument/completion",
            crate::scheduler::WorkLane::LatencySensitive,
        )
        .expect("enqueue request");
    let scheduled = session.start_next_request().expect("start request");
    session
        .begin_request_execution(scheduled.id())
        .expect("begin request");
    assert_eq!(session.cancel_request(&id), CancellationTarget::InFlight);

    let error = session
        .python_declaration_snapshot(&uri)
        .expect_err("cancelled request must stop");
    assert!(error.message().contains("cancelled"));
    assert_eq!(session.python_declarations.probe_runs(), 0);
    session.finish_request(&id);
}
