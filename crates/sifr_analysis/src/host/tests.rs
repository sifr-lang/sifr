use super::text_edits::full_range;
use super::*;
use crate::queries::SymbolQuery;
use crate::snapshot::{AnalysisErrorKind, AnalysisQueryKind};
use crate::{
    CodeActionContext, DiagnosticId, DocumentVersion, FormatOptions, FrontendInput, ProjectRoot,
    SourceText, SymbolBucketKind, SymbolBucketReadinessState, SymbolName, TestItemId, TextPosition,
    TypeHierarchyItemId,
};
use sifr_diagnostics::DiagnosticArg;
use sifr_frontend::{FrontendMode, SourcePath, WorkspaceTracePhase};

fn single_file_input(source: &str) -> FrontendInput {
    FrontendInput {
        path: SourcePath::new("main.sifr"),
        source: SourceText::new(source),
        mode: FrontendMode::SingleFile,
    }
}

fn temp_project_dir(name: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time should move forward")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "sifr_analysis_{name}_{}_{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    dir
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
    assert!(host.is_snapshot_current(&snapshot));

    host.update_document(
        file,
        DocumentVersion::new(1),
        SourceText::new("def main():\n    return 2\n"),
    )
    .expect("document update should invalidate snapshot");
    assert!(!host.is_snapshot_current(&snapshot));

    let error = snapshot
        .diagnostics(&mut host, file)
        .expect_err("stale snapshot should not answer queries");

    assert_eq!(error.kind, AnalysisErrorKind::StaleSnapshot);
    let debug = host.debug_snapshot();
    assert!(
        debug
            .trace
            .events
            .iter()
            .any(|event| event.phase == WorkspaceTracePhase::StaleRejection
                && event.detail.contains("captured_workspace"))
    );
}

#[test]
fn dependency_sensitive_invalidation_is_explained_in_trace() {
    let dir = temp_project_dir("dependency_trace");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main() -> int:\n    return value()\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    let root = ProjectRoot {
        root: SourcePath::new(dir.clone()),
        entrypoint: SourcePath::new(dir.join("main.sifr")),
    };
    let mut host = AnalysisHost::open_project(&root).expect("project host should load");
    let helper = host
        .document_file_for_path(&dir.join("helper.sifr"))
        .expect("helper file should be known");

    let report = host
        .update_document(
            helper,
            DocumentVersion::new(2),
            SourceText::new("def value() -> str:\n    return \"changed\"\n"),
        )
        .expect("export signature update should invalidate");

    assert!(matches!(
        report.dirty_scope_report.scope,
        sifr_frontend::WorkspaceDirtyScope::ReverseDependencies { .. }
    ));
    let debug = host.debug_snapshot();
    assert!(
        debug
            .trace
            .events
            .iter()
            .any(|event| event.phase == WorkspaceTracePhase::Invalidation
                && event.detail.contains("ReverseDependencies")
                && event.detail.contains("ExportSignatureChanged"))
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn analysis_snapshot_carries_workspace_state_and_query_metadata() {
    let source = "def main():\n    return 1\n";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let snapshot = host.snapshot();
    let snapshot_id = snapshot.workspace_snapshot_id();
    let position = TextPosition {
        line: 0,
        character: 0,
    };
    let range = full_range(source).expect("source should fit in range");

    assert!(snapshot.workspace().source_map.is_some());
    assert!(snapshot.workspace().module_graph.is_some());
    assert_eq!(
        snapshot.workspace().dirty_scope_report.scope,
        sifr_frontend::WorkspaceDirtyScope::None
    );

    let diagnostics = snapshot
        .diagnostics(&mut host, file)
        .expect("snapshot query should run");
    assert_eq!(
        diagnostics.metadata().workspace_snapshot_id,
        Some(snapshot_id)
    );
    assert_eq!(diagnostics.metadata().revision, snapshot.revision());

    for metadata in [
        snapshot
            .document_symbols(&mut host, file)
            .expect("snapshot document symbols should run")
            .metadata(),
        snapshot
            .workspace_symbols(&mut host, &SymbolQuery::default())
            .expect("snapshot workspace symbols should run")
            .metadata(),
        snapshot
            .completion(&mut host, file, &position)
            .expect("snapshot completion should run")
            .metadata(),
        snapshot
            .code_actions(&mut host, file, range, &CodeActionContext::default())
            .expect("snapshot code actions should run")
            .metadata(),
        snapshot
            .format_document(&mut host, file, FormatOptions::default())
            .expect("snapshot formatting should run")
            .metadata(),
        snapshot
            .generated_rust_preview(&mut host, file)
            .expect("snapshot generated Rust preview should run")
            .metadata(),
    ] {
        assert_eq!(metadata.workspace_snapshot_id, Some(snapshot_id));
        assert_eq!(metadata.revision, snapshot.revision());
    }

    let next_snapshot = host.snapshot();
    assert!(next_snapshot.workspace_snapshot_id().as_u64() > snapshot_id.as_u64());
}

#[test]
fn completion_query_includes_rust_interop_policy_candidates() {
    let source = "@rust.callback(\n    \n)\ndef main():\n    return 1\n";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");
    let file = host.files()[0];
    let completions = host
        .completion(
            file,
            &TextPosition {
                line: 1,
                character: 4,
            },
        )
        .expect("completion should query")
        .into_value();
    let labels = completions
        .items
        .into_iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert!(labels.contains(&"backpressure".to_string()));
    assert!(labels.contains(&"overflow".to_string()));
    assert!(labels.contains(&"shutdown".to_string()));
    assert!(!labels.contains(&"lifetime".to_string()));
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
    assert!(
        first
            .value()
            .iter()
            .any(|symbol| symbol.name == "helper_value")
    );
}

#[test]
fn project_symbol_index_refreshes_dirty_module_buckets_only() {
    let dir = std::env::temp_dir().join(format!(
        "sifr_analysis_bucket_refresh_{}_{}",
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
    let snapshot = host.snapshot();
    let main_file = snapshot
        .workspace()
        .source_map
        .as_ref()
        .expect("source map should exist")
        .files
        .iter()
        .find(|file| file.source.as_str().contains("def main"))
        .map(|file| file.id)
        .expect("main file should be indexed");

    let before = host
        .workspace_symbols(&SymbolQuery::default())
        .expect("workspace symbols should build the index");
    let helper_before = before
        .value()
        .iter()
        .find(|symbol| symbol.name == "helper_value")
        .cloned()
        .expect("helper symbol should exist before edit");
    let readiness_before = host
        .symbol_bucket_readiness()
        .expect("bucket readiness should be available");
    assert!(readiness_before.iter().any(|bucket| {
        bucket.id.kind == SymbolBucketKind::Workspace
            && bucket.state == SymbolBucketReadinessState::Exact
            && bucket.entry_count > 0
    }));
    assert!(readiness_before.iter().any(|bucket| {
        bucket.id.kind == SymbolBucketKind::Stdlib
            && bucket.state == SymbolBucketReadinessState::Exact
            && bucket.entry_count > 0
    }));
    assert_eq!(
        host.workspace_import_symbols(&SymbolQuery {
            query: "helper_value".to_string(),
        })
        .expect("import symbols should query bucketed imports")
        .metadata()
        .query,
        AnalysisQueryKind::WorkspaceSymbols
    );

    host.update_document(
        main_file,
        DocumentVersion::new(2),
        SourceText::new(
            "from helper import helper_value\n\ndef renamed():\n    return helper_value\n",
        ),
    )
    .expect("main module edit should update");

    let after = host
        .workspace_symbols(&SymbolQuery::default())
        .expect("workspace symbols should reuse clean buckets after edit");
    let helper_after = after
        .value()
        .iter()
        .find(|symbol| symbol.name == "helper_value")
        .cloned()
        .expect("helper symbol should remain after unrelated edit");
    let readiness_after = host
        .symbol_bucket_readiness()
        .expect("bucket readiness should remain available");

    assert_eq!(helper_before, helper_after);
    assert_eq!(readiness_before.len(), readiness_after.len());
    assert!(after.value().iter().any(|symbol| symbol.name == "renamed"));
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
        host.selection_ranges(file, std::slice::from_ref(&position))
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
        host.generated_rust_preview(file)
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
            .selection_ranges(file, std::slice::from_ref(&value_position))
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
        .generated_rust_preview(file)
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
    let source = "# TODO: follow up\ndef configure(a: int, b: int, c: int, d: int, e: int, f: int) -> int:\n    return a\n\ndef main() -> int:\n    return configure(1, 2, 3, 4, 5, 6)\n";
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
    assert!(
        analysis_codes.iter().any(|code| code == "SIFR-LINT-0007"),
        "analysis must run the HIR-backed rule against its canonical HIR"
    );
}

#[test]
fn workspace_diagnostic_order_is_stable_across_repeated_queries() {
    let source = "# TODO: follow up\ndef main():\n    return 1  \n";
    let mut host =
        AnalysisHost::open_single_file(single_file_input(source)).expect("host should load");

    let first = host
        .workspace_diagnostics()
        .expect("workspace diagnostics should query")
        .into_value();
    let second = host
        .workspace_diagnostics()
        .expect("workspace diagnostics should query again")
        .into_value();

    assert_eq!(first, second);
    assert!(first.iter().any(|file| !file.diagnostics.is_empty()));
}

#[test]
fn workspace_diagnostic_order_is_stable_under_parallel_readers() {
    let dir = temp_project_dir("parallel_diagnostic_order");
    std::fs::write(
        dir.join("main.sifr"),
        "# TODO: main follow up\ndef main():\n    return 1  \n",
    )
    .expect("main source should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "# TODO: helper follow up\ndef helper() -> int:\n    return 2  \n",
    )
    .expect("helper source should be written");
    let root = ProjectRoot {
        root: SourcePath::new(dir.clone()),
        entrypoint: SourcePath::new(dir.join("main.sifr")),
    };

    let mut host = AnalysisHost::open_project(&root).expect("project host should load");
    let expected = host
        .workspace_diagnostics()
        .expect("workspace diagnostics should query")
        .into_value();
    let snapshot = host.snapshot();
    let shared_host = std::sync::Arc::new(std::sync::Mutex::new(host));
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));

    let handles = (0..8)
        .map(|_| {
            let snapshot = snapshot.clone();
            let shared_host = std::sync::Arc::clone(&shared_host);
            let barrier = std::sync::Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                let mut host = shared_host
                    .lock()
                    .expect("shared host should not be poisoned");
                snapshot
                    .workspace_diagnostics(&mut host)
                    .expect("shared snapshot diagnostics should query")
                    .into_value()
            })
        })
        .collect::<Vec<_>>();

    for handle in handles {
        let diagnostics = handle.join().expect("parallel reader should not panic");
        assert_eq!(diagnostics, expected);
    }
    assert!(expected.iter().any(|file| !file.diagnostics.is_empty()));

    let _ = std::fs::remove_dir_all(dir);
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
