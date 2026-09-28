use crate::session::tests::python_declaration_tests::{SOURCE, open_fixture};
use lsp_server::{Connection, Message};
use serde_json::{Value, json};

use super::super::{
    revalidate_and_publish_without_watchers, watcher_registration::WatcherRegistration,
};

#[test]
fn request_time_multi_root_external_change_publishes_only_current_root_status() {
    let (mut session, first, first_uri) = open_fixture(SOURCE);
    let (_other_session, second, second_uri) = open_fixture(SOURCE);
    session
        .open_document(second_uri.clone(), "sifr", Some(1), SOURCE.to_string())
        .expect("open second root");
    session
        .python_declaration_snapshot(&first_uri)
        .expect("first warm status");
    session
        .python_declaration_snapshot(&second_uri)
        .expect("second warm status");
    let second_generation = session.external_input_generation(second.path());
    let before = session.python_declarations.analysis_plan_builds();

    std::fs::write(
        first.path().join("sifr.python-bindings.json"),
        r#"{"schema_version":1,"environment_digest":"stale","bindings":[]}"#,
    )
    .expect("change first external input");
    let unsupported = WatcherRegistration::new(&json!({"capabilities": {}}));
    let (server, client) = Connection::memory();
    revalidate_and_publish_without_watchers(&mut session, &unsupported, &server);
    let publications = client
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
        .collect::<Vec<_>>();
    let first_current = publications
        .iter()
        .find(|params| params["uri"] == first_uri)
        .expect("first publication");
    let second_current = publications
        .iter()
        .find(|params| params["uri"] == second_uri)
        .expect("second publication");
    let has_code = |params: &Value, code: &str| {
        params["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .iter()
            .any(|item| item["code"] == code)
    };
    assert!(has_code(first_current, "SIFR-PYCONV-0001"));
    assert!(!has_code(second_current, "SIFR-PYCONV-0001"));
    assert_eq!(first_current["version"], 1);
    assert_eq!(second_current["version"], 1);
    assert_eq!(
        session.external_input_generation(second.path()),
        second_generation
    );
    assert_eq!(
        session.python_declarations.analysis_plan_builds(),
        before + 1
    );

    for uri in [&first_uri, &second_uri] {
        let completion = crate::requests::handle(
            &mut session,
            "textDocument/completion",
            json!({"textDocument":{"uri":uri},"position":{"line":6,"character":15}}),
        )
        .expect("current completion");
        assert!(completion.to_string().contains("sqrt"));
        let hover = crate::requests::handle(
            &mut session,
            "textDocument/hover",
            json!({"textDocument":{"uri":uri},"position":{"line":6,"character":13}}),
        )
        .expect("current hover");
        assert!(hover.to_string().contains("math.sqrt"));
    }
    assert_eq!(
        session.python_declarations.analysis_plan_builds(),
        before + 1
    );
}
