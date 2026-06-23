use crate::session::Session;
use serde_json::json;

#[test]
fn definition_request_for_stdlib_import_returns_sysroot_uri() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/definition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 24 }
        }),
    )
    .expect("definition should answer");
    let locations = response
        .as_array()
        .expect("definition response should be an array");

    assert_eq!(locations.len(), 1);
    let target_uri = locations[0]
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .expect("definition location should include uri");
    assert!(
        target_uri.ends_with("/stdlib/sifr/random.sifr"),
        "unexpected stdlib definition uri: {target_uri}"
    );
}

#[test]
fn definition_request_for_stdlib_call_returns_sysroot_uri() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/definition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 4, "character": 17 }
        }),
    )
    .expect("definition should answer");
    let locations = response
        .as_array()
        .expect("definition response should be an array");

    assert_eq!(locations.len(), 1);
    let target_uri = locations[0]
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .expect("definition location should include uri");
    assert!(
        target_uri.ends_with("/stdlib/sifr/random.sifr"),
        "unexpected stdlib call definition uri: {target_uri}"
    );
}

#[test]
fn type_definition_request_for_stdlib_import_returns_sysroot_uri() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/typeDefinition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 24 }
        }),
    )
    .expect("type definition should answer");
    let locations = response
        .as_array()
        .expect("type definition response should be an array");

    assert_eq!(locations.len(), 1);
    let target_uri = locations[0]
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .expect("type definition location should include uri");
    assert!(
        target_uri.ends_with("/stdlib/sifr/random.sifr"),
        "unexpected stdlib type definition uri: {target_uri}"
    );
}

#[test]
fn hover_request_for_stdlib_call_reflects_installed_signature() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/hover",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 4, "character": 17 }
        }),
    )
    .expect("hover should answer");
    let contents = response
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("hover response should contain markdown contents");

    assert!(contents.contains("randint"));
    assert!(contents.contains("minimum: int"));
    assert!(contents.contains("maximum: int"));
    assert!(contents.contains("Result[int, ValueError]"));
}

#[test]
fn completion_request_includes_public_stdlib_symbols_not_private_modules() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/completion",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 4, "character": 17 }
        }),
    )
    .expect("completion should answer");
    let items = response
        .get("items")
        .and_then(serde_json::Value::as_array)
        .expect("completion response should include items");
    let labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();

    assert!(labels.contains(&"randint"));
    assert!(!labels.iter().any(|label| label.starts_with("_sifr")));
}

#[test]
fn sysroot_request_reports_same_root_as_analysis_tooling() {
    let mut session = Session::new();
    let expected = sifr_analysis::tooling_sysroot_status().expect("sysroot status should resolve");

    let response = crate::requests::handle(&mut session, "sifr/sysroot", json!({}))
        .expect("sysroot request should answer");

    assert_eq!(
        response.get("ok").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        response.get("root").and_then(serde_json::Value::as_str),
        Some(expected.root.to_string_lossy().as_ref())
    );
    assert_eq!(
        response
            .get("toolchainId")
            .and_then(serde_json::Value::as_str),
        Some(expected.toolchain_id.as_str())
    );
}

fn open_stdlib_import_fixture() -> (Session, tempfile::TempDir, String) {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("main.sifr");
    let source = "\
from sifr.random import randint

def main() -> int | None:
    try:
        x: int = randint(1, 2)
        return x
    except ValueError:
        return None
";
    std::fs::write(&path, source).expect("write source");
    let uri = url::Url::from_file_path(&path)
        .expect("file uri")
        .to_string();
    let mut session = Session::new();
    session
        .open_document(
            uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            source.to_string(),
        )
        .expect("open document");
    (session, temp, uri)
}
