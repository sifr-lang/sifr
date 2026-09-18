use crate::document_store::DiagnosticsMode;
use crate::session::Session;
use serde_json::json;

#[test]
fn dx11_syntax_does_not_load_metadata_and_late_semantics_see_latest_overlay() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("main.sifr");
    std::fs::write(&path, "def main():\n    value: int = 1\n").unwrap();
    let uri = url::Url::from_file_path(&path).unwrap().to_string();
    let mut session = Session::new();
    let mut settings = session.store().settings().clone();
    settings.diagnostics_mode = DiagnosticsMode::Off;
    session.store_mut().apply_settings(settings);
    session
        .open_document(
            uri.clone(),
            "sifr",
            Some(1),
            "def main():\n    value: int = 1\n".into(),
        )
        .unwrap();
    for method in [
        "textDocument/formatting",
        "textDocument/foldingRange",
        "textDocument/selectionRange",
    ] {
        crate::requests::handle(
            &mut session,
            method,
            json!({"textDocument":{"uri":uri},"positions":[{"line":1,"character":5}],"options":{}}),
        )
        .unwrap();
        assert!(session.compiler_context().metadata_stats().is_none());
    }
    assert!(
        crate::diagnostics::document_diagnostics(&mut session, &uri)
            .unwrap()
            .is_empty()
    );
    session
        .change_compacted(
            &uri,
            Some(2),
            &[json!({"text":"def main():\n    value: int = \"bad\"\n"})],
        )
        .unwrap();
    assert!(
        !crate::diagnostics::document_diagnostics(&mut session, &uri)
            .unwrap()
            .is_empty()
    );
    assert!(session.close_document(&uri));
    assert!(session.compiler_context().metadata_stats().is_none());
}

#[test]
fn dx11_invalid_incremental_batch_leaves_source_and_version_unchanged() {
    let temp = tempfile::tempdir().unwrap();
    let uri = url::Url::from_file_path(temp.path().join("main.sifr"))
        .unwrap()
        .to_string();
    let mut session = Session::new();
    session
        .store_mut()
        .open(uri.clone(), "sifr", Some(1), "original".into())
        .unwrap();
    let result = session.change_compacted(&uri, Some(2), &[
        json!({"text":"replacement"}),
        json!({"range":{"start":{"line":99,"character":0},"end":{"line":99,"character":0}},"text":"x"})]);
    assert!(result.is_err());
    assert_eq!(session.store().document(&uri).unwrap().text(), "original");
    assert_eq!(session.store().document(&uri).unwrap().version(), Some(1));
}

#[cfg(unix)]
#[test]
fn dx11_nonexistent_overlay_alias_has_one_physical_owner() {
    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("real");
    let alias = temp.path().join("alias");
    std::fs::create_dir(&real).unwrap();
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    let real_uri = url::Url::from_file_path(real.join("new.sifr"))
        .unwrap()
        .to_string();
    let alias_uri = url::Url::from_file_path(alias.join("new.sifr"))
        .unwrap()
        .to_string();
    let mut store = crate::document_store::DocumentStore::new();
    store
        .open(real_uri.clone(), "sifr", Some(1), "first".into())
        .unwrap();
    assert!(
        store
            .open(alias_uri.clone(), "sifr", Some(1), "second".into())
            .is_err()
    );
    assert_eq!(store.document(&real_uri).unwrap().text(), "first");
    store.close(&real_uri);
    store
        .open(alias_uri.clone(), "sifr", Some(1), "second".into())
        .unwrap();
    assert_eq!(
        store.document(&alias_uri).unwrap().path(),
        real.join("new.sifr")
    );
}

#[test]
fn dx11_superseded_diagnostics_remain_pending_until_latest_input_is_applied() {
    let temp = tempfile::tempdir().unwrap();
    let uri = url::Url::from_file_path(temp.path().join("main.sifr"))
        .unwrap()
        .to_string();
    let mut session = Session::new();
    session
        .open_document(
            uri.clone(),
            "sifr",
            Some(1),
            "def main():\n    value: int = 1\n".into(),
        )
        .unwrap();
    let (server, client) = lsp_server::Connection::memory();
    session.generation = session
        .generations
        .observe(Some("textDocument/didOpen"))
        .unwrap();
    let latest = session
        .generations
        .observe(Some("textDocument/didSave"))
        .unwrap();
    crate::diagnostics::DiagnosticsController::publish_document(
        &server,
        &mut session,
        &uri,
        DiagnosticsMode::OpenFiles,
    )
    .unwrap();
    assert!(client.receiver.try_recv().is_err());
    session.generation = latest;
    crate::diagnostics::DiagnosticsController::publish_document(
        &server,
        &mut session,
        &uri,
        DiagnosticsMode::OpenFiles,
    )
    .unwrap();
    let lsp_server::Message::Notification(notification) = client.receiver.try_recv().unwrap()
    else {
        panic!("expected diagnostics");
    };
    assert_eq!(notification.params["uri"], uri);
    assert_eq!(notification.params["version"], 1);
    assert!(client.receiver.try_recv().is_err());
    assert!(session.take_next_diagnostic_job().is_none());
}

#[test]
fn dx11_stale_close_clear_cannot_overwrite_reopened_document() {
    let temp = tempfile::tempdir().unwrap();
    let uri = url::Url::from_file_path(temp.path().join("main.sifr"))
        .unwrap()
        .to_string();
    let mut session = Session::new();
    let (server, client) = lsp_server::Connection::memory();
    session.generation = session
        .generations
        .observe(Some("textDocument/didClose"))
        .unwrap();
    session.diagnostic_clears.insert(uri.clone());
    let reopened = session
        .generations
        .observe(Some("textDocument/didOpen"))
        .unwrap();
    crate::diagnostics::DiagnosticsController::flush_clears(&server, &mut session).unwrap();
    assert!(client.receiver.try_recv().is_err());
    assert!(session.diagnostic_clears.contains(&uri));
    session
        .store_mut()
        .open(uri.clone(), "sifr", Some(1), "new owner".into())
        .unwrap();
    session.generation = reopened;
    crate::diagnostics::DiagnosticsController::flush_clears(&server, &mut session).unwrap();
    assert!(client.receiver.try_recv().is_err());
    assert!(session.diagnostic_clears.is_empty());
    session.store_mut().close(&uri);
    session.diagnostic_clears.insert(uri.clone());
    crate::diagnostics::DiagnosticsController::flush_clears(&server, &mut session).unwrap();
    let lsp_server::Message::Notification(clear) = client.receiver.try_recv().unwrap() else {
        panic!("expected clear");
    };
    assert_eq!(clear.params, json!({"uri":uri,"diagnostics":[]}));
}

#[test]
fn dx11_incremental_push_reconciles_multiple_documents_without_workspace_progress() {
    let temp = tempfile::tempdir().unwrap();
    let mut session = Session::new();
    session.set_work_done_progress_enabled(true);
    let mut uris = Vec::new();
    for name in ["first.sifr", "second.sifr"] {
        let uri = url::Url::from_file_path(temp.path().join(name))
            .unwrap()
            .to_string();
        session
            .open_document(
                uri.clone(),
                "sifr",
                Some(1),
                "def main():\n    value: int = 1\n".into(),
            )
            .unwrap();
        uris.push(uri);
    }
    let (server, client) = lsp_server::Connection::memory();
    crate::notifications::handle(
        &mut session,
        &server,
        "textDocument/didChange",
        json!({"textDocument":{"uri":uris[0],"version":2},
               "contentChanges":[{"text":"def main():\n    value: int = 2\n"}]}),
    )
    .unwrap();
    let incremental = client.receiver.try_iter().collect::<Vec<_>>();
    assert_eq!(incremental.len(), 2);
    for message in incremental {
        let lsp_server::Message::Notification(notification) = message else {
            panic!("expected diagnostics");
        };
        assert_eq!(notification.method, "textDocument/publishDiagnostics");
    }
    crate::diagnostics::DiagnosticsController::publish_all(&server, &mut session).unwrap();
    let workspace = client.receiver.try_iter().collect::<Vec<_>>();
    assert_eq!(
        workspace
            .iter()
            .filter(|message| matches!(message,
        lsp_server::Message::Notification(notification) if notification.method == "$/progress"))
            .count(),
        2
    );
}
