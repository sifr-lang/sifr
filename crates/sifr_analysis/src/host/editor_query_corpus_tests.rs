use super::text_edits::full_range;
use super::*;
use crate::queries::{CodeActionContext, DiagnosticId, SymbolQuery};
use crate::{DocumentVersion, FormatOptions, ProjectRoot, SourceText, SymbolName, TextPosition};
use sifr_frontend::{FrontendInput, FrontendMode, SourcePath};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct Marker {
    file: String,
    name: String,
    line: u32,
    character: u32,
    queries: Vec<String>,
}

#[derive(Clone, Debug)]
struct FixtureFile {
    relative_path: String,
    source: String,
    markers: Vec<Marker>,
}

fn temp_project_dir(name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "sifr_analysis_editor_query_corpus_{name}_{}_{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    dir
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("verification/areas/developer_tooling/editor_query_corpus/multi_file")
}

fn load_fixture(relative_path: &str) -> FixtureFile {
    let text = std::fs::read_to_string(corpus_root().join(relative_path))
        .expect("editor corpus fixture should exist");
    let mut source = String::new();
    let mut markers = Vec::new();
    for line in text.lines() {
        if let Some(marker) = parse_marker(relative_path, line) {
            markers.push(marker);
        } else {
            source.push_str(line);
            source.push('\n');
        }
    }
    FixtureFile {
        relative_path: relative_path.to_string(),
        source,
        markers,
    }
}

fn parse_marker(relative_path: &str, line: &str) -> Option<Marker> {
    let payload = line.trim().strip_prefix("# @marker ")?;
    let mut name = None;
    let mut marker_line = None;
    let mut character = None;
    let mut queries = None;
    for part in payload.split_whitespace() {
        if let Some(value) = part.strip_prefix("line=") {
            marker_line = value.parse::<u32>().ok();
        } else if let Some(value) = part.strip_prefix("character=") {
            character = value.parse::<u32>().ok();
        } else if let Some(value) = part.strip_prefix("queries=") {
            queries = Some(value.split(',').map(str::to_string).collect());
        } else if name.is_none() {
            name = Some(part.to_string());
        }
    }
    Some(Marker {
        file: relative_path.to_string(),
        name: name?,
        line: marker_line?,
        character: character?,
        queries: queries?,
    })
}

fn write_fixtures(dir: &Path, fixtures: &[FixtureFile]) {
    for fixture in fixtures {
        let path = dir.join(&fixture.relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("fixture directory should be created");
        }
        std::fs::write(path, &fixture.source).expect("fixture source should be written");
    }
}

fn marker<'a>(fixtures: &'a [FixtureFile], name: &str) -> &'a Marker {
    fixtures
        .iter()
        .flat_map(|fixture| fixture.markers.iter())
        .find(|candidate| candidate.name == name)
        .expect("marker should exist")
}

fn position(marker: &Marker) -> TextPosition {
    TextPosition {
        line: marker.line,
        character: marker.character,
    }
}

fn has_query(marker: &Marker, query: &str) -> bool {
    marker.queries.iter().any(|candidate| candidate == query)
}

#[test]
fn marker_editor_corpus_covers_multifile_queries_and_stale_snapshots() {
    let fixtures = vec![load_fixture("main.sifr"), load_fixture("helper.sifr")];
    let dir = temp_project_dir("editor_corpus");
    write_fixtures(&dir, &fixtures);
    let mut host = AnalysisHost::open_project(&ProjectRoot {
        root: SourcePath::new(&dir),
        entrypoint: SourcePath::new(dir.join("main.sifr")),
    })
    .expect("editor corpus project should load");
    let main_file = host
        .document_file_for_path(&dir.join("main.sifr"))
        .expect("main fixture file should be known");
    let helper_file = host
        .document_file_for_path(&dir.join("helper.sifr"))
        .expect("helper fixture file should be known");
    let value = marker(&fixtures, "value_binding");
    assert_eq!(value.file, "main.sifr");
    let value_position = position(value);
    let main_source = fixtures
        .iter()
        .find(|fixture| fixture.relative_path == "main.sifr")
        .expect("main fixture should exist")
        .source
        .clone();
    let main_range = full_range(&main_source).expect("fixture should fit in a text range");

    assert!(has_query(value, "hover"));
    assert!(
        host.hover(main_file, &value_position)
            .expect("hover should query")
            .into_value()
            .is_some()
    );
    assert!(has_query(value, "completion"));
    assert!(
        !host
            .completion(main_file, &value_position)
            .expect("completion should query")
            .into_value()
            .items
            .is_empty()
    );
    assert!(has_query(value, "definition"));
    assert!(
        !host
            .definition(main_file, &value_position)
            .expect("definition should query")
            .into_value()
            .is_empty()
    );
    assert!(has_query(value, "references"));
    assert!(
        host.references(main_file, &value_position)
            .expect("references should query")
            .into_value()
            .len()
            >= 2
    );
    assert!(has_query(value, "rename"));
    assert!(
        !host
            .rename(
                main_file,
                &value_position,
                &SymbolName("renamed_value".to_string()),
            )
            .expect("rename should query")
            .into_value()
            .edits
            .is_empty()
    );
    assert!(has_query(value, "semantic_tokens"));
    assert!(
        !host
            .semantic_tokens(main_file, None)
            .expect("semantic tokens should query")
            .into_value()
            .is_empty()
    );
    let helper = marker(&fixtures, "helper_export");
    assert!(has_query(helper, "definition"));
    assert!(
        !host
            .definition(helper_file, &position(helper))
            .expect("helper definition should query")
            .into_value()
            .is_empty()
    );
    assert!(has_query(helper, "references"));
    let helper_references = host
        .references(helper_file, &position(helper))
        .expect("helper references should query")
        .into_value();
    assert!(
        helper_references
            .iter()
            .any(|location| location.file == main_file)
    );
    assert!(
        helper_references
            .iter()
            .any(|location| location.file == helper_file)
    );
    assert!(has_query(helper, "semantic_tokens"));
    assert!(
        !host
            .semantic_tokens(helper_file, None)
            .expect("helper semantic tokens should query")
            .into_value()
            .is_empty()
    );
    assert!(has_query(value, "formatting"));
    assert!(
        !host
            .format_document(main_file, FormatOptions::default())
            .expect("formatting should query")
            .into_value()
            .is_empty()
    );
    assert!(has_query(value, "code_actions"));
    assert!(
        !host
            .code_actions(
                main_file,
                main_range,
                &CodeActionContext {
                    diagnostics: vec![DiagnosticId::policy(
                        "SIFR-LINT-0004",
                        "trailing-whitespace",
                    )],
                },
            )
            .expect("code actions should query")
            .into_value()
            .is_empty()
    );

    let stale = marker(&fixtures, "stale_return");
    assert!(has_query(stale, "diagnostics"));
    assert!(
        host.diagnostics(main_file)
            .expect("diagnostics should query")
            .into_value()
            .iter()
            .any(|diagnostic| diagnostic.code == "SIFR-LINT-0004")
    );
    assert!(has_query(stale, "stale_snapshot"));
    let snapshot = host.snapshot();
    host.update_document(
        main_file,
        DocumentVersion::new(2),
        SourceText::new(main_source.replace("helper_value", "1")),
    )
    .expect("fixture update should stale the captured snapshot");
    let error = snapshot
        .hover(&mut host, main_file, &position(stale))
        .expect_err("old snapshot should reject editor queries");
    assert_eq!(
        error.kind,
        crate::snapshot::AnalysisErrorKind::StaleSnapshot
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_handles_are_internal_and_reject_wrong_snapshot_resolution() {
    let mut host = AnalysisHost::open_single_file(FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new("def main():\n    value: int = 1\n    return value  \n"),
        mode: FrontendMode::SingleFile,
    })
    .expect("single file host should load");
    let file = host.files()[0];
    let snapshot = host.snapshot();
    let position = TextPosition {
        line: 1,
        character: 6,
    };
    let range = full_range("def main():\n    value: int = 1\n    return value  \n")
        .expect("source should fit in range");
    let symbol = snapshot
        .workspace_symbols(&mut host, &SymbolQuery::default())
        .expect("workspace symbols should query")
        .into_value()
        .into_iter()
        .find(|symbol| symbol.name == "main")
        .expect("main symbol should exist");
    let symbol_handle = snapshot.symbol_handle(symbol.clone());
    let type_handle = snapshot.type_handle("int");
    let signature_handle = snapshot.signature_handle("main(...)");
    let diagnostic_handle = snapshot.diagnostic_handle("SIFR-LINT-0004", Some(file));
    let span_handle = snapshot.source_span_handle(file, range);

    assert_eq!(snapshot.resolve_symbol_handle(&symbol_handle), Ok(symbol));
    assert_eq!(snapshot.resolve_type_handle(&type_handle), Ok("int"));
    assert_eq!(
        snapshot.resolve_signature_handle(&signature_handle),
        Ok("main(...)")
    );
    assert_eq!(
        snapshot.resolve_diagnostic_handle(&diagnostic_handle),
        Ok(("SIFR-LINT-0004", Some(file)))
    );
    assert_eq!(
        snapshot
            .resolve_source_span_handle(&span_handle)
            .expect("source span handle should resolve")
            .file,
        file
    );

    host.update_document(
        file,
        DocumentVersion::new(2),
        SourceText::new("def main():\n    value: int = 2\n    return value\n"),
    )
    .expect("document update should advance snapshot identity");
    let next_snapshot = host.snapshot();
    assert!(next_snapshot.workspace_snapshot_id() != snapshot.workspace_snapshot_id());
    for stale_error in [
        next_snapshot.resolve_symbol_handle(&symbol_handle).err(),
        next_snapshot.resolve_type_handle(&type_handle).err(),
        next_snapshot
            .resolve_signature_handle(&signature_handle)
            .err(),
        next_snapshot
            .resolve_diagnostic_handle(&diagnostic_handle)
            .err(),
        next_snapshot.resolve_source_span_handle(&span_handle).err(),
    ] {
        assert_eq!(
            stale_error
                .expect("wrong snapshot should reject handle")
                .kind,
            crate::snapshot::AnalysisErrorKind::StaleSnapshot
        );
    }

    assert!(
        snapshot
            .hover(&mut host, file, &position)
            .expect_err("old analysis snapshot should also reject host query")
            .message
            .contains("stale")
    );
}
