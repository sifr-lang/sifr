use super::*;
use crate::queries::SymbolQuery;
use crate::snapshot::{AnalysisErrorKind, AnalysisQueryKind};
use crate::{
    CodeActionContext, DiagnosticId, DocumentVersion, FormatOptions, FrontendInput, ProjectRoot,
    SourceText, SymbolName, TestItemId, TextPosition, TypeHierarchyItemId,
};
use sifr_diagnostics::DiagnosticArg;
use sifr_frontend::{FrontendMode, SourcePath};

fn single_file_input(source: &str) -> FrontendInput {
    FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    }
}

#[test]
fn single_file_session_updates_versions_and_invalidates_symbols() {
    let mut host = AnalysisHost::open_single_file(single_file_input(
        "def main():\n    value: int = 1\n    return value\n",
    ))
    .expect("single-file analysis host should load");
    let file = host.files()[0];

    let before = host
        .document_symbols(file)
        .expect("document symbols should query")
        .into_value();
    assert!(before.iter().any(|symbol| symbol.name == "main"));

    let report = host
        .update_document(
            file,
            DocumentVersion::new(2),
            SourceText::new("def renamed():\n    return 2\n"),
        )
        .expect("newer document version should update");
    assert_eq!(
        report.updated_documents[0].new_version,
        Some(DocumentVersion::new(2))
    );

    let after = host
        .document_symbols(file)
        .expect("document symbols should refresh after update")
        .into_value();
    assert!(after.iter().any(|symbol| symbol.name == "renamed"));
    assert!(!after.iter().any(|symbol| symbol.name == "main"));
}

#[test]
fn stale_document_version_is_rejected() {
    let mut host = AnalysisHost::open_single_file(single_file_input("def main():\n    return 1\n"))
        .expect("single-file analysis host should load");
    let file = host.files()[0];
    host.update_document(
        file,
        DocumentVersion::new(3),
        SourceText::new("def main():\n    return 2\n"),
    )
    .expect("newer version should update");

    let error = host
        .update_document(
            file,
            DocumentVersion::new(2),
            SourceText::new("def main():\n    return 3\n"),
        )
        .expect_err("older version should be rejected");

    assert_eq!(error.kind, AnalysisErrorKind::StaleDocumentVersion);
}

#[test]
fn stale_snapshot_is_rejected_after_update() {
    let mut host = AnalysisHost::open_single_file(single_file_input("def main():\n    return 1\n"))
        .expect("single-file analysis host should load");
    let file = host.files()[0];
    let snapshot = host.snapshot();

    host.update_document(
        file,
        DocumentVersion::new(1),
        SourceText::new("def main():\n    return 2\n"),
    )
    .expect("document update should invalidate snapshot");

    let error = snapshot
        .diagnostics(&mut host, file)
        .expect_err("stale snapshot should not answer queries");

    assert_eq!(error.kind, AnalysisErrorKind::StaleSnapshot);
}

#[test]
fn project_symbol_index_is_stable_for_workspace_queries() {
    let dir = std::env::temp_dir().join(format!(
        "sifr_analysis_project_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import helper_value\n\ndef main():\n    return helper_value\n",
    )
    .expect("main should be written");
    std::fs::write(dir.join("helper.sifr"), "helper_value: int = 1\n")
        .expect("helper should be written");

    let mut host = AnalysisHost::open_project(&ProjectRoot {
        root: SourcePath::new(&dir),
        entrypoint: SourcePath::new(dir.join("main.sifr")),
    })
    .expect("project analysis host should load");

    let first = host
        .workspace_symbols(&SymbolQuery {
            query: "helper_value".to_string(),
        })
        .expect("workspace symbols should query");
    let second = host
        .workspace_symbols(&SymbolQuery {
            query: "helper_value".to_string(),
        })
        .expect("workspace symbols should query from the same revision");

    assert_eq!(first.value(), second.value());
    assert_eq!(first.metadata().query, AnalysisQueryKind::WorkspaceSymbols);
    assert!(first
        .value()
        .iter()
        .any(|symbol| symbol.name == "helper_value"));
}

#[test]
fn all_editor_query_methods_expose_current_revision_metadata() {
    let mut host = AnalysisHost::open_single_file(single_file_input(
        "def main():\n    value: int = 1\n    return value\n",
    ))
    .expect("single-file analysis host should load");
    let file = host.files()[0];
    let position = TextPosition {
        line: 0,
        character: 0,
    };
    let range = full_range("def main():\n    value: int = 1\n    return value\n")
        .expect("source should fit in range");

    assert_eq!(
        host.diagnostics(file)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::Diagnostics
    );
    assert_eq!(
        host.workspace_diagnostics()
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::WorkspaceDiagnostics
    );
    assert_eq!(
        host.completion(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::Completion
    );
    assert_eq!(
        host.hover(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::Hover
    );
    assert_eq!(
        host.signature_help(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::SignatureHelp
    );
    assert_eq!(
        host.definition(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::Definition
    );
    assert_eq!(
        host.declaration(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::Declaration
    );
    assert_eq!(
        host.type_definition(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::TypeDefinition
    );
    assert_eq!(
        host.references(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::References
    );
    assert_eq!(
        host.prepare_rename(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::PrepareRename
    );
    assert_eq!(
        host.rename(file, &position, &SymbolName("renamed".to_string()))
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::Rename
    );
    assert_eq!(
        host.document_symbols(file)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::DocumentSymbols
    );
    assert_eq!(
        host.workspace_symbols(&SymbolQuery::default())
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::WorkspaceSymbols
    );
    assert_eq!(
        host.semantic_tokens(file, None)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::SemanticTokens
    );
    assert_eq!(
        host.inlay_hints(file, None)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::InlayHints
    );
    assert_eq!(
        host.document_highlights(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::DocumentHighlights
    );
    assert_eq!(
        host.folding_ranges(file)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::FoldingRanges
    );
    assert_eq!(
        host.selection_ranges(file, &[position.clone()])
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::SelectionRanges
    );
    assert_eq!(
        host.prepare_type_hierarchy(file, &position)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::PrepareTypeHierarchy
    );
    assert_eq!(
        host.type_hierarchy_supertypes(TypeHierarchyItemId("type".to_string()))
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::TypeHierarchySupertypes
    );
    assert_eq!(
        host.type_hierarchy_subtypes(TypeHierarchyItemId("type".to_string()))
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::TypeHierarchySubtypes
    );
    assert_eq!(
        host.code_actions(file, range, &CodeActionContext::default())
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::CodeActions
    );
    assert_eq!(
        host.format_document(file, FormatOptions::default())
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::FormatDocument
    );
    assert_eq!(
        host.format_range(file, range, FormatOptions::default())
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::FormatRange
    );
    assert_eq!(
        host.generated_rust_preview(file, None)
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::GeneratedRustPreview
    );
    assert_eq!(
        host.explain_diagnostic(&DiagnosticId::policy(
            "SIFR-LINT-0004",
            "trailing-whitespace",
        ))
        .expect("query should run")
        .metadata()
        .query,
        AnalysisQueryKind::ExplainDiagnostic
    );
    assert_eq!(
        host.discover_tests()
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::DiscoverTests
    );
    assert_eq!(
        host.test_command(TestItemId("main".to_string()))
            .expect("query should run")
            .metadata()
            .query,
        AnalysisQueryKind::TestCommand
    );
}

#[test]
fn editor_queries_return_navigation_rename_tokens_and_generated_rust() {
    let source = "\
def helper(x: int) -> int:
    return x

def main():
    value: int = helper(1)
    return value
";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let value_position = TextPosition {
        line: 5,
        character: 11,
    };

    let hover = host
        .hover(file, &value_position)
        .expect("hover should query")
        .into_value();
    assert!(hover.is_some(), "hover should return token-backed contents");

    let definitions = host
        .definition(file, &value_position)
        .expect("definition should query")
        .into_value();
    assert!(
        !definitions.is_empty(),
        "definition should resolve through token identity"
    );

    let references = host
        .references(file, &value_position)
        .expect("references should query")
        .into_value();
    assert!(
        references.len() >= 2,
        "references should include declaration and use"
    );

    let rename = host
        .rename(
            file,
            &value_position,
            &SymbolName("renamed_value".to_string()),
        )
        .expect("rename should query")
        .into_value();
    assert!(
        rename
            .edits
            .iter()
            .any(|file_edits| !file_edits.edits.is_empty()),
        "rename should produce workspace edits"
    );

    let symbols = host
        .document_symbols(file)
        .expect("document symbols should query")
        .into_value();
    assert!(
        symbols
            .iter()
            .any(|symbol| symbol.name == "main" && symbol.range.is_some()),
        "document symbols should include token ranges"
    );

    assert!(
        !host
            .semantic_tokens(file, None)
            .expect("semantic tokens should query")
            .into_value()
            .is_empty(),
        "semantic tokens should be token backed"
    );
    assert!(
        !host
            .folding_ranges(file)
            .expect("folding ranges should query")
            .into_value()
            .is_empty(),
        "folding ranges should cover function bodies"
    );
    assert!(
        !host
            .selection_ranges(file, &[value_position.clone()])
            .expect("selection ranges should query")
            .into_value()
            .is_empty(),
        "selection ranges should include token/line/document ancestors"
    );
    assert!(
        !host
            .inlay_hints(file, None)
            .expect("inlay hints should query")
            .into_value()
            .is_empty(),
        "inlay hints should expose annotation-backed hints"
    );

    let preview = host
        .generated_rust_preview(file, None)
        .expect("generated Rust preview should query")
        .into_value();
    assert!(
        preview
            .rust
            .as_deref()
            .is_some_and(|rust| rust.contains("fn main")),
        "generated Rust preview should be compiler backed"
    );
}

#[test]
fn analysis_lint_diagnostics_match_lint_engine_for_policy_rules() {
    let source = "# TODO: follow up\ndef main():\n    configure(True)\n";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let analysis_codes = host
        .diagnostics(file)
        .expect("diagnostics should query")
        .into_value()
        .into_iter()
        .filter(|diagnostic| matches!(diagnostic.args.get("rule"), Some(DiagnosticArg::String(_))))
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    let engine_codes = sifr_lint::lint_source(source, None, &sifr_lint::LintOptions::default())
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert_eq!(analysis_codes, engine_codes);
}

#[test]
fn code_actions_offer_policy_suppression_and_explain_not_found_is_explicit() {
    let source = "def main():\n    return 1  \n";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let range = full_range(source).expect("source should fit in range");
    let actions = host
        .code_actions(
            file,
            range,
            &CodeActionContext {
                diagnostics: vec![DiagnosticId::policy(
                    "SIFR-LINT-0004",
                    "trailing-whitespace",
                )],
            },
        )
        .expect("code actions should query")
        .into_value();
    assert!(
        actions
            .iter()
            .any(|action| action.kind == "quickfix.sifr.suppress" && action.edit.is_some()),
        "lint diagnostics should offer explicit suppression edits"
    );
    assert!(
        actions
            .iter()
            .any(|action| action.kind == "quickfix.sifr.applySafeFix" && action.edit.is_some()),
        "safe policy diagnostics should offer explicit fix edits"
    );
    assert!(
        actions
            .iter()
            .any(|action| action.kind == "source.fixAll.sifr" && action.edit.is_none()),
        "safe policy diagnostics should offer deferred fix-all"
    );

    let hard_actions = host
        .code_actions(
            file,
            range,
            &CodeActionContext {
                diagnostics: vec![DiagnosticId::hard("SIFR-TYPE-0001")],
            },
        )
        .expect("hard diagnostic code actions should query")
        .into_value();
    assert!(
        hard_actions.is_empty(),
        "hard diagnostics must not offer policy suppression or fix actions"
    );

    let explanation = host
        .explain_diagnostic(&DiagnosticId::hard("SIFR-NOPE-0000"))
        .expect("explain diagnostic should query")
        .into_value();
    assert!(explanation.diagnostic.is_none());
    assert!(explanation.unavailable_reason.is_some());
}
