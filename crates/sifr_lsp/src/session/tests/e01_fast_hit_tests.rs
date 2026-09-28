use crate::diagnostics::{DiagnosticsController, document_diagnostics};
use crate::document_store::DiagnosticsMode;
use crate::request_queue::CancellationTarget;
use crate::session::Session;
use crate::session::watcher_events::WatcherEvent;
use lsp_server::{Connection, Message, RequestId};
use serde_json::{Value, json};

use super::python_declaration_tests::{SOURCE, open_fixture};

fn completion(session: &mut Session, uri: &str) -> Value {
    crate::requests::handle(
        session,
        "textDocument/completion",
        json!({"textDocument":{"uri":uri},"position":{"line":6,"character":15}}),
    )
    .expect("completion")
}

fn hover(session: &mut Session, uri: &str) -> Value {
    crate::requests::handle(
        session,
        "textDocument/hover",
        json!({"textDocument":{"uri":uri},"position":{"line":6,"character":13}}),
    )
    .expect("hover")
}

fn has_code(diagnostics: &[Value], code: &str) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.get("code").and_then(Value::as_str) == Some(code))
}

fn published(client: &Connection) -> Vec<Value> {
    client
        .receiver
        .try_iter()
        .filter_map(|message| match message {
            Message::Notification(notification)
                if notification.method == "textDocument/publishDiagnostics" =>
            {
                Some(notification.params)
            }
            _ => None,
        })
        .collect()
}

fn publish(session: &mut Session, server: &Connection, client: &Connection, uri: &str) -> Value {
    DiagnosticsController::publish_document(server, session, uri, DiagnosticsMode::OpenFiles)
        .expect("publish diagnostics");
    published(client)
        .into_iter()
        .find(|params| params.get("uri").and_then(Value::as_str) == Some(uri))
        .expect("current document publication")
}

#[test]
fn verified_unchanged_request_reuses_python_status() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);
    let first = completion(&mut session, &uri);
    assert!(first.to_string().contains("verified"));
    let builds = session.python_declarations.analysis_plan_builds();
    assert_eq!(builds, 1);
    assert_eq!(session.python_declarations.probe_runs(), 1);

    let second = completion(&mut session, &uri);
    assert_eq!(second, first);
    assert!(hover(&mut session, &uri).to_string().contains("verified"));
    assert!(
        document_diagnostics(&mut session, &uri)
            .expect("warm diagnostics")
            .is_empty()
    );
    assert_eq!(session.python_declarations.analysis_plan_builds(), builds);
    assert_eq!(session.python_declarations.probe_runs(), 1);
}

#[test]
fn source_config_and_external_changes_recompute_requests_and_publish_current_diagnostics() {
    let (mut session, temp, uri) = open_fixture(SOURCE);
    let (server, client) = Connection::memory();
    assert!(
        !publish(&mut session, &server, &client, &uri)["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .iter()
            .any(|item| item["code"] == "SIFR-PYCALL-0001")
    );
    let initial = session.python_declarations.analysis_plan_builds();

    let invalid = SOURCE.replace("math.sqrt", "math.pi");
    session
        .change_compacted(&uri, Some(2), &[json!({"text":invalid})])
        .expect("change Python target");
    let changed = publish(&mut session, &server, &client, &uri);
    assert_eq!(changed["version"], 2);
    assert!(has_code(
        changed["diagnostics"]
            .as_array()
            .expect("diagnostics array"),
        "SIFR-PYCALL-0001"
    ));
    assert!(session.python_declarations.analysis_plan_builds() > initial);
    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("math.pi")
    );
    assert!(hover(&mut session, &uri).to_string().contains("math.pi"));

    session
        .change_compacted(&uri, Some(3), &[json!({"text":SOURCE})])
        .expect("restore Python target");
    let restored = publish(&mut session, &server, &client, &uri);
    assert_eq!(restored["version"], 3);
    assert!(!has_code(
        restored["diagnostics"]
            .as_array()
            .expect("diagnostics array"),
        "SIFR-PYCALL-0001"
    ));

    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("verified")
    );
    assert!(hover(&mut session, &uri).to_string().contains("verified"));
    let before_config = session.python_declarations.analysis_plan_builds();
    let manifest = temp.path().join("sifr.toml");
    let mut contents = std::fs::read_to_string(&manifest).expect("read config");
    contents.push_str("# changed configuration\n");
    std::fs::write(&manifest, contents).expect("change config");
    let configured = publish(&mut session, &server, &client, &uri);
    assert_eq!(configured["version"], 3);
    assert!(session.python_declarations.analysis_plan_builds() > before_config);
    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("verified")
    );
    assert!(hover(&mut session, &uri).to_string().contains("verified"));

    let artifact = temp.path().join("sifr.python-bindings.json");
    std::fs::write(
        &artifact,
        "{\"schema_version\":1,\"environment_digest\":\"stale\",\"bindings\":[]}\n",
    )
    .expect("change binding artifact");
    let event = WatcherEvent::from_protocol(
        &url::Url::from_file_path(&artifact)
            .expect("artifact URI")
            .to_string(),
        1,
    )
    .expect("watcher event");
    session.record_watcher_file_events(&[event]);
    let external = publish(&mut session, &server, &client, &uri);
    assert!(has_code(
        external["diagnostics"]
            .as_array()
            .expect("diagnostics array"),
        "SIFR-PYCONV-0001"
    ));
    let _ = completion(&mut session, &uri);
    let _ = hover(&mut session, &uri);
}

#[test]
fn cancelled_and_reopened_requests_cannot_publish_stale_results() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);
    session
        .python_declaration_snapshot(&uri)
        .expect("prime warm status");
    let id = RequestId::from(801);
    session
        .enqueue_request(
            &id,
            "textDocument/completion",
            crate::scheduler::WorkLane::LatencySensitive,
        )
        .expect("queue request");
    let scheduled = session.start_next_request().expect("scheduled request");
    session
        .begin_request_execution(scheduled.id())
        .expect("begin request");
    assert_eq!(session.cancel_request(&id), CancellationTarget::InFlight);
    assert!(session.python_declaration_snapshot(&uri).is_err());
    assert!(document_diagnostics(&mut session, &uri).is_err());
    session.finish_request(&id);

    let (server, client) = Connection::memory();
    let previous = session
        .generations
        .observe(Some("textDocument/didChange"))
        .expect("generation");
    session.generation = previous;
    session
        .change_compacted(
            &uri,
            Some(2),
            &[json!({"text":SOURCE.replace("math.sqrt", "math.pi")})],
        )
        .expect("old edit");
    session
        .schedule_document_diagnostics(&uri)
        .expect("schedule old diagnostics");
    assert!(session.close_document(&uri));
    session
        .open_document(uri.clone(), "sifr", Some(1), SOURCE.to_string())
        .expect("reopen current document");
    let latest = session
        .generations
        .observe(Some("textDocument/didOpen"))
        .expect("new generation");
    session.generation = latest;
    let current = publish(&mut session, &server, &client, &uri);
    assert_eq!(current["version"], 1);
    assert!(!has_code(
        current["diagnostics"]
            .as_array()
            .expect("diagnostics array"),
        "SIFR-PYCALL-0001"
    ));
    assert!(published(&client).is_empty());
    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("verified")
    );
    assert!(hover(&mut session, &uri).to_string().contains("verified"));
}

#[test]
fn multi_root_requests_keep_generation_and_publication_isolated() {
    let (mut session, first, first_uri) = open_fixture(SOURCE);
    let (_other_session, second, second_uri) = open_fixture(SOURCE);
    session
        .open_document(second_uri.clone(), "sifr", Some(1), SOURCE.to_string())
        .expect("open second root");
    session
        .python_declaration_snapshot(&first_uri)
        .expect("first status");
    session
        .python_declaration_snapshot(&second_uri)
        .expect("second status");
    let second_generation = session.external_input_generation(second.path());
    let second_builds = session.python_declarations.analysis_plan_builds();

    session
        .change_compacted(
            &first_uri,
            Some(2),
            &[json!({"text":SOURCE.replace("math.sqrt", "math.pi")})],
        )
        .expect("change first root");
    assert!(session.python_declarations.has_entry(second.path()));
    assert!(
        completion(&mut session, &second_uri)
            .to_string()
            .contains("verified")
    );
    assert_eq!(
        session.python_declarations.analysis_plan_builds(),
        second_builds
    );
    assert_eq!(
        session.external_input_generation(second.path()),
        second_generation
    );

    let (server, client) = Connection::memory();
    DiagnosticsController::publish_all(&server, &mut session).expect("publish both roots");
    let publications = published(&client);
    let first_diagnostics = publications
        .iter()
        .find(|params| params["uri"] == first_uri)
        .expect("first publication");
    let second_diagnostics = publications
        .iter()
        .find(|params| params["uri"] == second_uri)
        .expect("second publication");
    assert!(has_code(
        first_diagnostics["diagnostics"]
            .as_array()
            .expect("first diagnostics"),
        "SIFR-PYCALL-0001"
    ));
    assert!(!has_code(
        second_diagnostics["diagnostics"]
            .as_array()
            .expect("second diagnostics"),
        "SIFR-PYCALL-0001"
    ));
    assert_eq!(first_diagnostics["version"], 2);
    assert_eq!(second_diagnostics["version"], 1);
    assert_eq!(
        session.external_input_generation(second.path()),
        second_generation
    );
    assert!(first.path() != second.path());
}
