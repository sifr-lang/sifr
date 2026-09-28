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
    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("math.sqrt")
    );
    assert!(hover(&mut session, &uri).to_string().contains("math.sqrt"));
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
    session
        .generations
        .observe(Some("textDocument/didClose"))
        .expect("superseding close");
    DiagnosticsController::publish_document(
        &server,
        &mut session,
        &uri,
        DiagnosticsMode::OpenFiles,
    )
    .expect("stale generation stays pending");
    assert!(published(&client).is_empty());
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
fn changed_external_input_during_publication_retries_current_diagnostics() {
    let (mut session, temp, uri) = open_fixture(SOURCE);
    session
        .python_declaration_snapshot(&uri)
        .expect("prime status");
    session
        .python_declarations
        .inject_external_change_before_verification(
            temp.path().join("sifr.python-bindings.json"),
            Some(
                b"{\"schema_version\":1,\"environment_digest\":\"stale\",\"bindings\":[]}\n"
                    .to_vec(),
            ),
        );
    let (server, client) = Connection::memory();
    let current = publish(&mut session, &server, &client, &uri);
    assert!(has_code(
        current["diagnostics"].as_array().expect("diagnostics"),
        "SIFR-PYCONV-0001"
    ));
    assert!(session.take_next_diagnostic_job().is_none());
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
    let before_external_builds = session.python_declarations.analysis_plan_builds();
    let artifact = first.path().join("sifr.python-bindings.json");
    std::fs::write(
        &artifact,
        "{\"schema_version\":1,\"environment_digest\":\"stale\",\"bindings\":[]}\n",
    )
    .expect("change first-root artifact");
    let event = WatcherEvent::from_protocol(
        &url::Url::from_file_path(&artifact)
            .expect("artifact URI")
            .to_string(),
        1,
    )
    .expect("watcher event");
    session.record_watcher_file_events(&[event]);
    DiagnosticsController::publish_all(&server, &mut session).expect("publish external change");
    let external_publications = published(&client);
    let first_external = external_publications
        .iter()
        .find(|params| params["uri"] == first_uri)
        .expect("first external publication");
    let second_external = external_publications
        .iter()
        .find(|params| params["uri"] == second_uri)
        .expect("second external publication");
    assert!(has_code(
        first_external["diagnostics"]
            .as_array()
            .expect("first diagnostics"),
        "SIFR-PYCONV-0001"
    ));
    assert!(!has_code(
        second_external["diagnostics"]
            .as_array()
            .expect("second diagnostics"),
        "SIFR-PYCONV-0001"
    ));
    assert!(session.python_declarations.analysis_plan_builds() > before_external_builds);
    let after_first_rebuild = session.python_declarations.analysis_plan_builds();
    assert!(
        completion(&mut session, &second_uri)
            .to_string()
            .contains("verified")
    );
    assert_eq!(
        session.python_declarations.analysis_plan_builds(),
        after_first_rebuild
    );
    assert_eq!(
        session.external_input_generation(second.path()),
        second_generation
    );

    let manifest = first.path().join("sifr.toml");
    session
        .python_declarations
        .inject_external_change_before_verification(manifest, None);
    let stale = session
        .python_declaration_snapshot(&first_uri)
        .expect_err("old package-root result must be rejected");
    assert_eq!(stale.code(), lsp_server::ErrorCode::ContentModified as i32);
    assert!(!session.python_declarations.has_entry(first.path()));
    let current = session
        .python_declaration_snapshot(&first_uri)
        .expect("new root status");
    assert!(!current.insights.is_empty());
    assert_eq!(
        session.external_input_generation(second.path()),
        second_generation
    );
    assert!(session.python_declarations.has_entry(second.path()));
    let current_publication = publish(&mut session, &server, &client, &first_uri);
    assert_eq!(current_publication["version"], 2);
}

#[test]
fn nested_manifest_removal_reassigns_to_ancestor_and_publishes_current_diagnostics() {
    let (mut session, temp, uri) = open_fixture(SOURCE);
    let ancestor = temp.path();
    let nested = ancestor.join("src");
    let nested_path = nested.join("main.sifr");
    let nested_manifest = nested.join("sifr.toml");
    std::fs::write(
        &nested_manifest,
        "[package]\nname = \"nested-python\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \".\"\n\n[python]\nvenv = \"../.venv\"\npyproject = \"../pyproject.toml\"\nlock = \"../uv.lock\"\n\n[trust]\npython = [\"builtins\", \"math\"]\n",
    )
    .expect("write nested manifest");
    session.observe_external_inputs_for_path(&nested_path);
    let warm = completion(&mut session, &uri);
    assert!(warm.to_string().contains("math.sqrt"));
    assert_eq!(
        session
            .external_input_identity_for_path(&nested_path)
            .package_root,
        Some(nested.clone())
    );
    assert!(session.python_declarations.has_entry(&nested));

    let mut manifest = std::fs::read_to_string(&nested_manifest).expect("read nested manifest");
    manifest.push_str("# advance nested generation\n");
    std::fs::write(&nested_manifest, manifest).expect("change nested manifest");
    session.observe_external_inputs_for_path(&nested_path);
    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("math.sqrt")
    );

    let invalid = SOURCE.replace("math.sqrt", "math.pi");
    session
        .change_compacted(&uri, Some(2), &[json!({"text": invalid})])
        .expect("change nested source");
    session
        .python_declarations
        .inject_external_change_before_verification(nested_manifest, None);
    let stale = session
        .python_declaration_snapshot(&uri)
        .expect_err("nested result must be rejected after manifest deletion");
    assert_eq!(stale.code(), lsp_server::ErrorCode::ContentModified as i32);
    assert_eq!(session.external_input_generation(&nested), None);
    assert!(session.external_input_generation(ancestor).is_some());

    let current = session
        .python_declaration_snapshot(&uri)
        .expect("ancestor-owned status must complete");
    assert!(!current.insights.is_empty());
    let identity = session.external_input_identity_for_path(&nested_path);
    assert_eq!(identity.package_root.as_deref(), Some(ancestor));
    assert_eq!(
        identity.generation,
        session.external_input_generation(ancestor).unwrap_or(0)
    );
    assert!(!session.python_declarations.has_entry(&nested));
    assert!(session.python_declarations.has_entry(ancestor));

    let (server, client) = Connection::memory();
    let publication = publish(&mut session, &server, &client, &uri);
    assert_eq!(publication["version"], 2);
    assert!(has_code(
        publication["diagnostics"]
            .as_array()
            .expect("current diagnostics"),
        "SIFR-PYCALL-0001"
    ));
    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("math.pi")
    );
    assert!(hover(&mut session, &uri).to_string().contains("math.pi"));
}

#[test]
fn unstable_external_input_returns_status_and_publishes_diagnostics() {
    let (mut session, temp, uri) = open_fixture(SOURCE);
    let artifact = temp.path().join(sifr_package::PYTHON_BINDINGS_FILE);
    std::fs::create_dir(&artifact).expect("make binding artifact unreadable as a file");
    let before = session.external_input_generation(temp.path()).unwrap_or(0);
    let invalid = SOURCE.replace("math.sqrt", "math.pi");
    session
        .change_compacted(&uri, Some(2), &[json!({"text": invalid})])
        .expect("change source while external input is unstable");

    assert!(
        completion(&mut session, &uri)
            .to_string()
            .contains("math.pi")
    );
    assert!(hover(&mut session, &uri).to_string().contains("math.pi"));
    assert!(session.external_input_generation(temp.path()).unwrap_or(0) > before);

    let (server, client) = Connection::memory();
    let publication = publish(&mut session, &server, &client, &uri);
    assert_eq!(publication["version"], 2);
    assert!(has_code(
        publication["diagnostics"]
            .as_array()
            .expect("current diagnostics"),
        "SIFR-PYCALL-0001"
    ));
    assert!(session.take_next_diagnostic_job().is_none());

    session
        .python_declarations
        .inject_external_change_before_verification(
            temp.path().join(sifr_package::PYTHON_CERTIFICATIONS_FILE),
            Some(b"{}\n".to_vec()),
        );
    let stale = session
        .python_declaration_snapshot(&uri)
        .expect_err("changed unstable inputs must reject stale work");
    assert_eq!(stale.code(), lsp_server::ErrorCode::ContentModified as i32);
    session
        .python_declaration_snapshot(&uri)
        .expect("unchanged unstable input must complete on retry");
}
