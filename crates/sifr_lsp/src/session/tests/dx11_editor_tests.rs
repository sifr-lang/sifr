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
