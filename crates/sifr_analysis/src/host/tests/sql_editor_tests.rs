use super::*;

#[test]
fn sql_templates_route_through_virtual_document_editor_queries() {
    let source = "@app.query\ndef query(user_id: int) -> Template:\n    return t\"SELECT users.name FROM users WHERE users.id = {user_id} LIMIT 1\"\n";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let users_position = TextPosition {
        line: 2,
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

#[test]
fn sql_profile_load_failure_preserves_non_sql_package_analysis() {
    let dir = temp_project_dir("sql_profile_failure_isolation");
    std::fs::create_dir_all(dir.join("src")).expect("source directory");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"editor-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde = \"1\"\n",
    )
    .expect("Cargo manifest");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"src\"\n").expect("Sifr manifest");
    let entrypoint = dir.join("src/main.sifr");
    let source = "def answer() -> int:\n    return 42\n";
    std::fs::write(&entrypoint, source).expect("entrypoint");
    let root = ProjectRoot {
        root: SourcePath::new(dir.clone()),
        entrypoint: SourcePath::new(entrypoint.clone()),
    };
    let mut host = AnalysisHost::open_project_with_overlays(
        &root,
        vec![(
            SourcePath::new(entrypoint),
            Some("file:///editor-app/src/main.sifr".to_string()),
            DocumentVersion::new(1),
            SourceText::new(source),
        )],
    )
    .expect("SQL initialization must not abort the analysis host");
    let file = host.files()[0];
    let symbols = host
        .document_symbols(file)
        .expect("ordinary analysis should remain available")
        .into_value();
    assert!(symbols.iter().any(|symbol| symbol.name == "answer"));
    let diagnostics = host
        .diagnostics(file)
        .expect("diagnostics should query")
        .into_value();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("cargo metadata")
                || diagnostic.message.contains("SQL editor package graph")
        }),
        "diagnostics={:?}",
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn sql_profile_load_failure_preserves_direct_open_project_analysis() {
    let dir = temp_project_dir("sql_profile_failure_direct_open");
    std::fs::create_dir_all(dir.join("src")).expect("source directory");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"editor-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nserde = \"1\"\n",
    )
    .expect("Cargo manifest");
    std::fs::write(dir.join("sifr.toml"), "[source]\nroot = \"src\"\n").expect("Sifr manifest");
    let entrypoint = dir.join("src/main.sifr");
    std::fs::write(&entrypoint, "def answer() -> int:\n    return 42\n").expect("entrypoint");
    let root = ProjectRoot {
        root: SourcePath::new(dir.clone()),
        entrypoint: SourcePath::new(entrypoint),
    };
    let mut host = AnalysisHost::open_project(&root)
        .expect("SQL initialization must not abort direct project analysis");
    let file = host.files()[0];
    assert!(
        host.document_symbols(file)
            .expect("ordinary symbols")
            .into_value()
            .iter()
            .any(|symbol| symbol.name == "answer")
    );
    assert!(
        host.diagnostics(file)
            .expect("initialization diagnostics")
            .into_value()
            .iter()
            .any(|diagnostic| diagnostic.message.contains("cargo metadata")
                || diagnostic.message.contains("SQL editor package graph"))
    );
    let _ = std::fs::remove_dir_all(dir);
}
