use super::Session;
use serde_json::{Value, json};

#[test]
fn sql_virtual_document_features_share_the_analysis_snapshot() {
    let source = "def query(user_id: int) -> Template:\n    return t\"SELECT users.name FROM users WHERE users.id = {user_id} LIMIT 1\"\n";
    let (mut session, _temp, uri) = open_fixture(source);
    let completion = request(&mut session, "textDocument/completion", &uri, 1, 22);
    let hover = request(&mut session, "textDocument/hover", &uri, 1, 22);
    let references = request(&mut session, "textDocument/references", &uri, 1, 22);
    let semantic = crate::requests::handle(
        &mut session,
        "textDocument/semanticTokens/full",
        json!({ "textDocument": { "uri": uri } }),
    )
    .expect("semantic tokens");
    let inlay = crate::requests::handle(
        &mut session,
        "textDocument/inlayHint",
        json!({
            "textDocument": { "uri": uri },
            "range": {
                "start": { "line": 0, "character": 0 },
                "end": { "line": 2, "character": 0 }
            }
        }),
    )
    .expect("inlay hints");
    let rename = crate::requests::handle(
        &mut session,
        "textDocument/rename",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 1, "character": 22 },
            "newName": "accounts"
        }),
    )
    .expect("rename");
    let formatting = crate::requests::handle(
        &mut session,
        "textDocument/formatting",
        json!({
            "textDocument": { "uri": uri },
            "options": { "tabSize": 4, "insertSpaces": true }
        }),
    )
    .expect("formatting");
    let code_actions = crate::requests::handle(
        &mut session,
        "textDocument/codeAction",
        json!({
            "textDocument": { "uri": uri },
            "range": {
                "start": { "line": 1, "character": 20 },
                "end": { "line": 1, "character": 24 }
            },
            "context": {
                "diagnostics": [{
                    "range": {
                        "start": { "line": 1, "character": 20 },
                        "end": { "line": 1, "character": 24 }
                    },
                    "severity": 1,
                    "code": "SIFR-SQL-POSTGRESQL-0005",
                    "source": "sifr",
                    "message": "type mismatch"
                }]
            }
        }),
    )
    .expect("SQL code actions");

    let labels = completion
        .pointer("/items")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .filter(|label| matches!(*label, "SELECT" | "FROM" | "WHERE"))
        .collect::<Vec<_>>();
    let summary = json!({
        "completion": labels,
        "hover_has_cardinality": hover.to_string().contains("zero-or-one"),
        "reference_count": references.as_array().map_or(0, Vec::len),
        "semantic_token_data": semantic.pointer("/data").and_then(Value::as_array).map_or(0, Vec::len),
        "inlay_hint_count": inlay.as_array().map_or(0, Vec::len),
        "rename_edits_present": rename.pointer("/changes").is_some(),
        "format_preserves_hole": formatting.as_array().is_some_and(|edits| edits.is_empty())
            || formatting.to_string().contains("{user_id}"),
        "structured_cast_fix": code_actions.to_string().contains("quickfix.sifr.sql.cast"),
    });
    insta::assert_snapshot!(serde_json::to_string_pretty(&summary).expect("summary JSON"), @r###"
    {
      "completion": [
        "FROM",
        "SELECT",
        "WHERE"
      ],
      "format_preserves_hole": true,
      "hover_has_cardinality": true,
      "inlay_hint_count": 3,
      "reference_count": 3,
      "rename_edits_present": true,
      "semantic_token_data": 135,
      "structured_cast_fix": true
    }
    "###);
}

fn request(session: &mut Session, method: &str, uri: &str, line: u64, character: u64) -> Value {
    crate::requests::handle(
        session,
        method,
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character },
            "context": { "includeDeclaration": true }
        }),
    )
    .unwrap_or_else(|error| panic!("{method} failed: {error:?}"))
}

fn open_fixture(source: &str) -> (Session, tempfile::TempDir, String) {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("main.sifr");
    std::fs::write(&path, source).expect("write fixture");
    let uri = url::Url::from_file_path(&path)
        .expect("file URI")
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
