use crate::diagnostics::{DiagnosticsController, document_diagnostics};
use crate::document_store::DiagnosticsMode;
use crate::request_queue::CancellationTarget;
use crate::session::Session;
use lsp_server::{Connection, Message, RequestId};
use serde_json::{Value, json};

const SEED: u64 = 0x4830_3168_5055_424c;

fn publications(client: &Connection) -> Vec<Value> {
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
        .expect("publication");
    publications(client)
        .into_iter()
        .find(|params| params["uri"] == uri)
        .expect("publication for current URI")
}

#[test]
fn request_publication() {
    let temp = tempfile::tempdir().expect("temporary project");
    let path = temp.path().join("main.sifr");
    let uri = url::Url::from_file_path(&path)
        .expect("file URI")
        .to_string();
    let mut source = "def main() -> int:\n    return 1\n".to_string();
    std::fs::write(&path, &source).expect("initial source");
    let mut session = Session::new();
    session
        .open_document(
            uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            source.clone(),
        )
        .expect("open incremental document");
    let (server, client) = Connection::memory();
    let mut state = SEED;
    let mut saw_error = false;
    let mut saw_recovery = false;
    let mut saw_span = false;

    for step in 0..32 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let operation = if step < 4 { step } else { state as usize % 4 };
        source = match operation {
            0 => format!("def main() -> int:\n    return {}\n", state % 71),
            1 => format!(
                "def main() -> int:\n    result: str = {}\n    return result\n",
                state % 71
            ),
            2 => "def main() -> int:\n    return 1\n".to_string(),
            _ => "def main() -> int:\n    return missing\n".to_string(),
        };
        std::fs::write(&path, &source).expect("write exact disk snapshot");
        let version = step as i32 + 2;
        session
            .change_compacted(&uri, Some(version), &[json!({"text":source})])
            .expect("incremental edit");

        if step == 5 {
            let id = RequestId::from(901);
            session
                .enqueue_request(
                    &id,
                    "textDocument/diagnostic",
                    crate::scheduler::WorkLane::LatencySensitive,
                )
                .expect("enqueue");
            let scheduled = session.start_next_request().expect("scheduled");
            session
                .begin_request_execution(scheduled.id())
                .expect("begin request");
            assert_eq!(session.cancel_request(&id), CancellationTarget::InFlight);
            let error = document_diagnostics(&mut session, &uri)
                .expect_err("cancelled request must not produce diagnostics");
            assert_eq!(error.code(), lsp_server::ErrorCode::RequestCanceled as i32);
            session.finish_request(&id);
        }

        let actual = publish(&mut session, &server, &client, &uri);
        let mut fresh = Session::new();
        fresh
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(version),
                source.clone(),
            )
            .expect("open fresh document at identical snapshot");
        let (fresh_server, fresh_client) = Connection::memory();
        let expected = publish(&mut fresh, &fresh_server, &fresh_client, &uri);
        assert_eq!(
            actual["version"], expected["version"],
            "seed={SEED:#x} step={step} operation={operation} source={source:?}"
        );
        assert_eq!(
            actual["diagnostics"], expected["diagnostics"],
            "seed={SEED:#x} step={step} operation={operation} source={source:?}"
        );
        let diagnostics = actual["diagnostics"].as_array().expect("diagnostics array");
        saw_error |= !diagnostics.is_empty();
        saw_recovery |= saw_error && diagnostics.is_empty();
        saw_span |= diagnostics.iter().any(|item| item.get("range").is_some());
    }
    assert!(saw_error && saw_recovery && saw_span);
    project_structural_publication_equivalence();
}

fn project_structural_publication_equivalence() {
    let temp = tempfile::tempdir().expect("temporary project");
    let src = temp.path().join("src");
    std::fs::create_dir(&src).expect("source directory");
    std::fs::write(src.join("lib.rs"), "").expect("Cargo library marker");
    std::fs::write(
        temp.path().join("Cargo.toml"),
        r#"[package]
name = "h01h-equivalence"
version = "0.0.0"
edition = "2024"

[package.metadata.sifr]
manifest = "sifr.toml"

[workspace]
"#,
    )
    .expect("Cargo manifest");
    std::fs::write(
        temp.path().join("Cargo.lock"),
        r#"version = 4

[[package]]
name = "h01h-equivalence"
version = "0.0.0"
"#,
    )
    .expect("Cargo lock");
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"h01h-equivalence\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    )
    .expect("project manifest");
    let main_path = src.join("main.sifr");
    let helper_path = temp.path().join("extra.sifr");
    let renamed_path = temp.path().join("renamed.sifr");
    let mut main_source =
        "from extra import extra\n\ndef main() -> int:\n    return extra()\n".to_string();
    let helper_source = "def extra() -> int:\n    return 3\n";
    std::fs::write(&main_path, &main_source).expect("main source");
    let uri = url::Url::from_file_path(&main_path)
        .expect("main URI")
        .to_string();
    let helper_uri = url::Url::from_file_path(&helper_path)
        .expect("helper URI")
        .to_string();
    let renamed_uri = url::Url::from_file_path(&renamed_path)
        .expect("renamed URI")
        .to_string();
    let mut incremental = Session::new();
    incremental
        .open_document(
            uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            main_source.clone(),
        )
        .expect("open project main");
    let (server, client) = Connection::memory();
    let initial = publish(&mut incremental, &server, &client, &uri);
    assert!(
        initial["diagnostics"]
            .as_array()
            .expect("initial diagnostics")
            .iter()
            .any(|item| item["code"] == "SIFR-IMPORT-0002")
    );

    for (step, changes) in [
        (0, json!([{"uri":helper_uri,"type":1}])),
        (1, json!([{"uri":helper_uri,"type":3}])),
        (
            2,
            json!([
                {"uri":helper_uri,"type":3},
                {"uri":renamed_uri,"type":1}
            ]),
        ),
    ] {
        match step {
            0 => std::fs::write(&helper_path, helper_source).expect("add imported module"),
            1 => std::fs::remove_file(&helper_path).expect("delete imported module"),
            _ => {
                std::fs::write(&helper_path, helper_source).expect("source to rename");
                std::fs::rename(&helper_path, &renamed_path).expect("rename imported module");
                main_source =
                    "from renamed import extra\n\ndef main() -> int:\n    return extra()\n"
                        .to_string();
                std::fs::write(&main_path, &main_source).expect("update import");
                incremental
                    .change_compacted(&uri, Some(2), &[json!({"text":main_source})])
                    .expect("change import to renamed module");
            }
        }
        crate::notifications::handle(
            &mut incremental,
            &server,
            "workspace/didChangeWatchedFiles",
            json!({"changes":changes}),
        )
        .expect("watcher notification");
        let actual = publications(&client)
            .into_iter()
            .filter(|params| params["uri"] == uri)
            .last()
            .expect("watcher must publish current diagnostics");

        let mut fresh = Session::new();
        fresh
            .open_document(
                uri.clone(),
                crate::capabilities::LANGUAGE_ID,
                Some(if step == 2 { 2 } else { 1 }),
                main_source.clone(),
            )
            .expect("fresh project at identical snapshot");
        let (fresh_server, fresh_client) = Connection::memory();
        let expected = publish(&mut fresh, &fresh_server, &fresh_client, &uri);
        assert_eq!(
            actual["diagnostics"], expected["diagnostics"],
            "seed={SEED:#x} structural step={step} source={main_source:?}"
        );
        match step {
            0 | 2 => assert!(
                actual["diagnostics"]
                    .as_array()
                    .expect("diagnostics")
                    .is_empty(),
                "step={step} diagnostics={:?}",
                actual["diagnostics"]
            ),
            _ => assert!(
                actual["diagnostics"]
                    .as_array()
                    .expect("diagnostics")
                    .iter()
                    .any(|item| {
                        item["code"] == "SIFR-IMPORT-0002"
                            && item["range"]
                                == json!({
                                    "start": {"line": 0, "character": 0},
                                    "end": {"line": 0, "character": 23}
                                })
                    })
            ),
        }
    }
}
