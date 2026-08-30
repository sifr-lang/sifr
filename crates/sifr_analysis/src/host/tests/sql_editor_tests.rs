use super::*;

#[test]
fn sql_templates_route_through_virtual_document_editor_queries() {
    let source = "def query(user_id: int) -> Template:\n    return t\"SELECT users.name FROM users WHERE users.id = {user_id} LIMIT 1\"\n";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let users_position = TextPosition {
        line: 1,
        character: 22,
    };

    let completion = host
        .completion(file, &users_position)
        .expect("SQL completion")
        .into_value();
    assert!(completion.items.iter().any(|item| item.label == "SELECT"));

    let hover = host
        .hover(file, &users_position)
        .expect("SQL hover")
        .into_value()
        .expect("hover value");
    assert!(hover.contents.contains("SQL"));
    assert!(hover.contents.contains("Cardinality: `zero-or-one`"));

    let references = host
        .references(file, &users_position)
        .expect("SQL references")
        .into_value();
    assert!(references.len() >= 2);

    let rename = host
        .rename(file, &users_position, &SymbolName("accounts".to_string()))
        .expect("SQL rename")
        .into_value();
    assert!(rename.edits.iter().any(|edits| edits.edits.len() >= 2));

    let tokens = host
        .semantic_tokens(file, None)
        .expect("SQL semantic tokens")
        .into_value();
    assert!(tokens.iter().any(|token| {
        token.token_type == "keyword" && token.modifiers.iter().any(|value| value == "sql")
    }));

    let hints = host
        .inlay_hints(file, None)
        .expect("SQL inlay hints")
        .into_value();
    assert!(hints.iter().any(|hint| hint.label.contains("$1")));
    assert!(hints.iter().any(|hint| hint.label.contains("zero-or-one")));
}
