use crate::completion::{rank_completion_candidates, CompletionCandidate};
use crate::editor::{line_end_insert_range, EditorFacts, EditorToken};
use crate::queries::{
    CodeAction, CodeActionContext, CompletionItem, CompletionItems, DiagnosticExplanation,
    DiagnosticId, DocumentHighlight, DocumentSymbol, FileDiagnostics, FileTextEdits, FoldingRange,
    FormatOptions, GeneratedRustPreview, HoverInfo, InlayHint, Location, RenameTarget,
    SelectionRange, SemanticToken, SignatureHelp, SymbolName, SymbolQuery, TestCommand,
    TestCommandKind, TestItem, TestItemId, TypeHierarchyItem, TypeHierarchyItemId, WorkspaceEdit,
    WorkspaceSymbol,
};
use crate::snapshot::{
    AnalysisError, AnalysisErrorKind, AnalysisQueryKind, AnalysisQueryResult, AnalysisRevision,
    AnalysisSnapshot, QueryMetadata,
};
use crate::symbols::SymbolIndex;
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{
    DocumentVersion, FileId, FrontendContext, FrontendInput, InvalidationReport, ModuleId,
    ProjectRoot, SourceText,
};
use sifr_syntax::TextPosition;
use std::collections::BTreeMap;

type QueryResult<T> = Result<AnalysisQueryResult<T>, AnalysisError>;

pub struct AnalysisHost {
    context: FrontendContext,
    file_to_module: BTreeMap<FileId, ModuleId>,
    symbol_index: Option<SymbolIndex>,
    last_invalidation: Option<InvalidationReport>,
}

impl AnalysisHost {
    pub fn open_project(root: &ProjectRoot) -> Result<Self, Vec<RenderedDiagnostic>> {
        let context = FrontendContext::load_project(root)?;
        Ok(Self::new(context))
    }

    pub fn open_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>> {
        let context = FrontendContext::load_single_file(input)?;
        Ok(Self::new(context))
    }

    fn new(context: FrontendContext) -> Self {
        let mut host = Self {
            context,
            file_to_module: BTreeMap::new(),
            symbol_index: None,
            last_invalidation: None,
        };
        host.refresh_file_map();
        host
    }

    pub fn update_document(
        &mut self,
        file: FileId,
        version: DocumentVersion,
        text: SourceText,
    ) -> Result<InvalidationReport, AnalysisError> {
        let Some(module) = self.file_to_module.get(&file).copied() else {
            return Err(unknown_file(file));
        };
        if let Some(current) = self.context.document_version_for_file(file) {
            if version <= current {
                return Err(AnalysisError::new(
                    AnalysisErrorKind::StaleDocumentVersion,
                    format!(
                        "stale document version {} for file {}; current version is {}",
                        version.as_i64(),
                        file.as_u32(),
                        current.as_i64()
                    ),
                ));
            }
        }
        let report = self
            .context
            .update_module_source(module, text, Some(version))
            .map_err(|diagnostics| frontend_diagnostics(&diagnostics))?;
        self.refresh_file_map();
        self.symbol_index = None;
        self.last_invalidation = Some(report.clone());
        Ok(report)
    }

    #[must_use]
    pub fn snapshot(&self) -> AnalysisSnapshot {
        AnalysisSnapshot::new(self.current_revision())
    }

    #[must_use]
    pub fn last_invalidation(&self) -> Option<&InvalidationReport> {
        self.last_invalidation.as_ref()
    }

    #[must_use]
    pub fn files(&self) -> Vec<FileId> {
        self.file_to_module.keys().copied().collect()
    }

    pub fn diagnostics(&mut self, file: FileId) -> QueryResult<Vec<RenderedDiagnostic>> {
        let module = self.module_for_file(file)?;
        let mut diagnostics = self
            .context
            .diagnostics_for_module(module)
            .into_value()
            .diagnostics;
        diagnostics.extend(self.lint_diagnostics(file)?);
        Ok(self.result(AnalysisQueryKind::Diagnostics, diagnostics))
    }

    pub fn workspace_diagnostics(&mut self) -> QueryResult<Vec<FileDiagnostics>> {
        let mut files = Vec::new();
        for file in self.file_to_module.keys().copied().collect::<Vec<_>>() {
            files.push(FileDiagnostics {
                file: Some(file),
                diagnostics: self.diagnostics(file)?.into_value(),
            });
        }
        Ok(self.result(AnalysisQueryKind::WorkspaceDiagnostics, files))
    }

    pub fn completion(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<CompletionItems> {
        let query = self
            .editor_facts(file)?
            .identifier_at_position(position)
            .map(|token| token.text.clone())
            .unwrap_or_default();
        let candidates = self
            .symbol_index()?
            .workspace_symbols("")
            .into_iter()
            .map(|symbol| CompletionCandidate {
                label: symbol.name,
                kind: symbol.kind,
                detail: symbol.container_name,
            })
            .collect();
        let ranked = rank_completion_candidates(&query, candidates);
        let items = ranked
            .candidates
            .into_iter()
            .map(|candidate| CompletionItem {
                label: candidate.label,
                kind: candidate.kind,
                detail: candidate.detail,
            })
            .collect();
        Ok(self.result(AnalysisQueryKind::Completion, CompletionItems { items }))
    }

    pub fn hover(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<HoverInfo>> {
        let facts = self.editor_facts(file)?;
        let hover = facts.token_at_position(position).map(|token| HoverInfo {
            contents: format!("{} ({})", token.text, token.kind),
        });
        Ok(self.result(AnalysisQueryKind::Hover, hover))
    }

    pub fn signature_help(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<SignatureHelp>> {
        let facts = self.editor_facts(file)?;
        let help = call_identifier_before_position(&facts, position).map(|label| SignatureHelp {
            label,
            active_parameter: Some(0),
        });
        Ok(self.result(AnalysisQueryKind::SignatureHelp, help))
    }

    pub fn definition(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        let locations = self.locations_for_identifier_at(file, position, true)?;
        Ok(self.result(AnalysisQueryKind::Definition, locations))
    }

    pub fn declaration(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        let locations = self.locations_for_identifier_at(file, position, true)?;
        Ok(self.result(AnalysisQueryKind::Declaration, locations))
    }

    pub fn type_definition(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        let locations = self.locations_for_identifier_at(file, position, true)?;
        Ok(self.result(AnalysisQueryKind::TypeDefinition, locations))
    }

    pub fn references(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<Location>> {
        let locations = self.locations_for_identifier_at(file, position, false)?;
        Ok(self.result(AnalysisQueryKind::References, locations))
    }

    pub fn prepare_rename(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<RenameTarget>> {
        let facts = self.editor_facts(file)?;
        let Some(token) = facts.identifier_at_position(position) else {
            return Ok(self.result(AnalysisQueryKind::PrepareRename, None));
        };
        self.prepare_rename_symbol(&token.text)
    }

    pub fn prepare_rename_symbol(&mut self, name: &str) -> QueryResult<Option<RenameTarget>> {
        let target = self
            .symbol_index()?
            .unique_symbol_named(name)
            .map(|symbol| RenameTarget { symbol });
        Ok(self.result(AnalysisQueryKind::PrepareRename, target))
    }

    pub fn rename(
        &mut self,
        file: FileId,
        position: &TextPosition,
        new_name: &SymbolName,
    ) -> QueryResult<WorkspaceEdit> {
        let facts = self.editor_facts(file)?;
        let edits = if let Some(token) = facts.identifier_at_position(position) {
            self.reference_edits(&token.text, &new_name.0)?
        } else {
            Vec::new()
        };
        Ok(self.result(AnalysisQueryKind::Rename, WorkspaceEdit { edits }))
    }

    pub fn document_symbols(&mut self, file: FileId) -> QueryResult<Vec<DocumentSymbol>> {
        self.module_for_file(file)?;
        let facts = self.editor_facts(file)?;
        let mut symbols = self.symbol_index()?.document_symbols(file);
        for symbol in &mut symbols {
            symbol.range = facts
                .tokens_named(&symbol.name)
                .first()
                .map(|token| token.range);
        }
        Ok(self.result(AnalysisQueryKind::DocumentSymbols, symbols))
    }

    pub fn workspace_symbols(&mut self, query: &SymbolQuery) -> QueryResult<Vec<WorkspaceSymbol>> {
        let symbols = self.symbol_index()?.workspace_symbols(&query.query);
        Ok(self.result(AnalysisQueryKind::WorkspaceSymbols, symbols))
    }

    pub fn semantic_tokens(
        &mut self,
        file: FileId,
        range: Option<TextRange>,
    ) -> QueryResult<Vec<SemanticToken>> {
        let tokens = self.editor_facts(file)?.semantic_tokens(range);
        Ok(self.result(AnalysisQueryKind::SemanticTokens, tokens))
    }

    pub fn inlay_hints(
        &mut self,
        file: FileId,
        range: Option<TextRange>,
    ) -> QueryResult<Vec<InlayHint>> {
        let hints = self.editor_facts(file)?.inlay_hints(range);
        Ok(self.result(AnalysisQueryKind::InlayHints, hints))
    }

    pub fn document_highlights(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Vec<DocumentHighlight>> {
        let facts = self.editor_facts(file)?;
        let highlights = facts
            .identifier_at_position(position)
            .map(|token| {
                facts
                    .tokens_named(&token.text)
                    .into_iter()
                    .map(|token| DocumentHighlight { range: token.range })
                    .collect()
            })
            .unwrap_or_default();
        Ok(self.result(AnalysisQueryKind::DocumentHighlights, highlights))
    }

    pub fn folding_ranges(&mut self, file: FileId) -> QueryResult<Vec<FoldingRange>> {
        let ranges = self.editor_facts(file)?.folding_ranges();
        Ok(self.result(AnalysisQueryKind::FoldingRanges, ranges))
    }

    pub fn selection_ranges(
        &mut self,
        file: FileId,
        positions: &[TextPosition],
    ) -> QueryResult<Vec<SelectionRange>> {
        let ranges = self.editor_facts(file)?.selection_ranges(positions);
        Ok(self.result(AnalysisQueryKind::SelectionRanges, ranges))
    }

    pub fn prepare_type_hierarchy(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> QueryResult<Option<TypeHierarchyItem>> {
        let facts = self.editor_facts(file)?;
        let item = facts.identifier_at_position(position).and_then(|token| {
            token
                .text
                .chars()
                .next()
                .is_some_and(char::is_uppercase)
                .then(|| TypeHierarchyItem {
                    id: TypeHierarchyItemId(format!("{}:{}", file.as_u32(), token.text)),
                    name: token.text.clone(),
                    kind: "type".to_string(),
                    location: Location {
                        file,
                        range: Some(token.range),
                    },
                })
        });
        Ok(self.result(AnalysisQueryKind::PrepareTypeHierarchy, item))
    }

    pub fn type_hierarchy_supertypes(
        &mut self,
        _item: TypeHierarchyItemId,
    ) -> QueryResult<Vec<TypeHierarchyItem>> {
        Ok(self.result(AnalysisQueryKind::TypeHierarchySupertypes, Vec::new()))
    }

    pub fn type_hierarchy_subtypes(
        &mut self,
        _item: TypeHierarchyItemId,
    ) -> QueryResult<Vec<TypeHierarchyItem>> {
        Ok(self.result(AnalysisQueryKind::TypeHierarchySubtypes, Vec::new()))
    }

    pub fn code_actions(
        &mut self,
        file: FileId,
        range: TextRange,
        context: &CodeActionContext,
    ) -> QueryResult<Vec<CodeAction>> {
        let source = self.source_text(file)?;
        let mut actions = Vec::new();
        if context
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.0.starts_with("SIFR-LINT-"))
        {
            if let Some(insert_range) = line_end_insert_range(&source, range) {
                actions.push(CodeAction {
                    title: "Suppress trailing-whitespace policy diagnostic".to_string(),
                    kind: "quickfix.sifr.suppress".to_string(),
                    edit: Some(WorkspaceEdit {
                        edits: vec![FileTextEdits {
                            file,
                            edits: vec![sifr_format::TextEdit {
                                range: insert_range,
                                replacement: "  # sifr: ignore[trailing-whitespace]".to_string(),
                            }],
                        }],
                    }),
                });
            }
        }
        Ok(self.result(AnalysisQueryKind::CodeActions, actions))
    }

    pub fn format_document(
        &mut self,
        file: FileId,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        let source = self.source_text(file)?;
        let path = self.context.path_for_file(file);
        let edits = sifr_format::format_range(&source, full_range(&source)?, path, options)
            .map_err(|diagnostics| frontend_diagnostics(&diagnostics))?;
        Ok(self.result(AnalysisQueryKind::FormatDocument, edits))
    }

    pub fn format_range(
        &mut self,
        file: FileId,
        range: TextRange,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        let source = self.source_text(file)?;
        let path = self.context.path_for_file(file);
        let edits = sifr_format::format_range(&source, range, path, options)
            .map_err(|diagnostics| frontend_diagnostics(&diagnostics))?;
        Ok(self.result(AnalysisQueryKind::FormatRange, edits))
    }

    pub fn generated_rust_preview(
        &mut self,
        file: FileId,
        range: Option<TextRange>,
    ) -> QueryResult<GeneratedRustPreview> {
        self.module_for_file(file)?;
        let source = self.source_text(file)?;
        let rust = match sifr_driver::compile_with_metadata(&source) {
            sifr_driver::CompileResultFull::Success { rust_source, .. } => Some(rust_source),
            sifr_driver::CompileResultFull::Errors { errors } => {
                return Ok(self.result(
                    AnalysisQueryKind::GeneratedRustPreview,
                    GeneratedRustPreview {
                        file,
                        range,
                        rust: None,
                        unavailable_reason: Some(format!(
                            "generated Rust preview unavailable because compilation produced {} diagnostic(s)",
                            errors.len()
                        )),
                    },
                ));
            }
        };
        Ok(self.result(
            AnalysisQueryKind::GeneratedRustPreview,
            GeneratedRustPreview {
                file,
                range,
                rust,
                unavailable_reason: None,
            },
        ))
    }

    pub fn explain_diagnostic(
        &mut self,
        diagnostic: &DiagnosticId,
    ) -> QueryResult<DiagnosticExplanation> {
        let found = self
            .workspace_diagnostics()?
            .into_value()
            .into_iter()
            .flat_map(|file| file.diagnostics)
            .find(|rendered| rendered.code == diagnostic.0);
        let explanation = DiagnosticExplanation {
            unavailable_reason: found
                .is_none()
                .then(|| "diagnostic not found in current workspace snapshot".to_string()),
            diagnostic: found,
        };
        Ok(self.result(AnalysisQueryKind::ExplainDiagnostic, explanation))
    }

    pub fn discover_tests(&mut self) -> QueryResult<Vec<TestItem>> {
        let tests = self
            .file_to_module
            .keys()
            .map(|file| TestItem {
                id: TestItemId(format!("check:{}", file.as_u32())),
                label: format!("Check file {}", file.as_u32()),
                file: Some(*file),
            })
            .collect();
        Ok(self.result(AnalysisQueryKind::DiscoverTests, tests))
    }

    pub fn test_command(&mut self, test: TestItemId) -> QueryResult<TestCommand> {
        Ok(self.result(
            AnalysisQueryKind::TestCommand,
            TestCommand {
                kind: TestCommandKind::Run,
                args: vec![test.0],
            },
        ))
    }

    fn symbol_index(&mut self) -> Result<&SymbolIndex, AnalysisError> {
        let revision = self.current_revision();
        let needs_refresh = self
            .symbol_index
            .as_ref()
            .is_none_or(|index| index.revision() != revision);
        if needs_refresh {
            let graph = self.context.module_graph();
            let analysis = self.context.analysis_for_project();
            let query_revision = AnalysisRevision {
                graph: analysis.metadata().graph_revision,
                source: analysis.metadata().source_revision,
            };
            self.symbol_index = Some(SymbolIndex::build(query_revision, &graph, analysis.value()));
        }
        self.symbol_index.as_ref().ok_or_else(|| {
            AnalysisError::new(
                AnalysisErrorKind::UnknownSymbol,
                "symbol index was not constructed",
            )
        })
    }

    fn lint_diagnostics(&self, file: FileId) -> Result<Vec<RenderedDiagnostic>, AnalysisError> {
        let source = self.source_text(file)?;
        let path = self.context.path_for_file(file);
        Ok(sifr_lint::lint_source(&source, path, &sifr_lint::LintOptions::default()).diagnostics)
    }

    fn editor_facts(&mut self, file: FileId) -> Result<EditorFacts, AnalysisError> {
        let module = self.module_for_file(file)?;
        let source = self.source_text(file)?;
        let parsed = self.context.parse_module(module).into_value().parsed;
        let tokens = parsed
            .tokens()
            .iter()
            .filter_map(|token| {
                let start = usize::try_from(token.range.start().to_u32()).ok()?;
                let end = usize::try_from(token.range.end().to_u32()).ok()?;
                let text = source.get(start..end)?.to_string();
                Some(EditorToken {
                    kind: token.kind.as_str().to_string(),
                    text,
                    range: token.range,
                })
            })
            .collect();
        Ok(EditorFacts { source, tokens })
    }

    fn locations_for_identifier_at(
        &mut self,
        file: FileId,
        position: &TextPosition,
        first_only: bool,
    ) -> Result<Vec<Location>, AnalysisError> {
        let facts = self.editor_facts(file)?;
        let Some(token) = facts.identifier_at_position(position) else {
            return Ok(Vec::new());
        };
        self.locations_for_name(&token.text, first_only)
    }

    fn locations_for_name(
        &mut self,
        name: &str,
        first_only: bool,
    ) -> Result<Vec<Location>, AnalysisError> {
        let mut locations = Vec::new();
        for file in self.file_to_module.keys().copied().collect::<Vec<_>>() {
            let facts = self.editor_facts(file)?;
            for token in facts.tokens_named(name) {
                locations.push(Location {
                    file,
                    range: Some(token.range),
                });
                if first_only {
                    return Ok(locations);
                }
            }
        }
        Ok(locations)
    }

    fn reference_edits(
        &mut self,
        old_name: &str,
        new_name: &str,
    ) -> Result<Vec<FileTextEdits>, AnalysisError> {
        let mut edits = Vec::new();
        for file in self.file_to_module.keys().copied().collect::<Vec<_>>() {
            let facts = self.editor_facts(file)?;
            let file_edits = facts
                .tokens_named(old_name)
                .into_iter()
                .map(|token| sifr_format::TextEdit {
                    range: token.range,
                    replacement: new_name.to_string(),
                })
                .collect::<Vec<_>>();
            if !file_edits.is_empty() {
                edits.push(FileTextEdits {
                    file,
                    edits: file_edits,
                });
            }
        }
        Ok(edits)
    }

    fn source_text(&self, file: FileId) -> Result<String, AnalysisError> {
        self.context
            .source_text_for_file(file)
            .map(str::to_owned)
            .ok_or_else(|| unknown_file(file))
    }

    fn module_for_file(&self, file: FileId) -> Result<ModuleId, AnalysisError> {
        self.file_to_module
            .get(&file)
            .copied()
            .ok_or_else(|| unknown_file(file))
    }

    fn refresh_file_map(&mut self) {
        self.file_to_module = self
            .context
            .module_graph()
            .modules
            .into_iter()
            .map(|module| (module.file, module.id))
            .collect();
    }

    fn current_revision(&self) -> AnalysisRevision {
        AnalysisRevision {
            graph: self.context.module_graph().revision,
            source: self.context.source_map().revision,
        }
    }

    fn metadata(&self, query: AnalysisQueryKind) -> QueryMetadata {
        QueryMetadata {
            query,
            revision: self.current_revision(),
        }
    }

    fn result<T>(&self, query: AnalysisQueryKind, value: T) -> AnalysisQueryResult<T> {
        AnalysisQueryResult::new(value, self.metadata(query))
    }
}

impl AnalysisSnapshot {
    pub fn diagnostics(
        &self,
        host: &mut AnalysisHost,
        file: FileId,
    ) -> QueryResult<Vec<RenderedDiagnostic>> {
        host.ensure_snapshot_current(self)?;
        host.diagnostics(file)
    }

    pub fn workspace_symbols(
        &self,
        host: &mut AnalysisHost,
        query: &SymbolQuery,
    ) -> QueryResult<Vec<WorkspaceSymbol>> {
        host.ensure_snapshot_current(self)?;
        host.workspace_symbols(query)
    }
}

impl AnalysisHost {
    fn ensure_snapshot_current(&self, snapshot: &AnalysisSnapshot) -> Result<(), AnalysisError> {
        let current = self.current_revision();
        if snapshot.revision() == current {
            return Ok(());
        }
        Err(AnalysisError::new(
            AnalysisErrorKind::StaleSnapshot,
            format!(
                "analysis snapshot is stale: captured graph/source {}:{}, current graph/source {}:{}",
                snapshot.revision().graph.as_u64(),
                snapshot.revision().source.as_u64(),
                current.graph.as_u64(),
                current.source.as_u64()
            ),
        ))
    }
}

fn full_range(source: &str) -> Result<TextRange, AnalysisError> {
    let end = u32::try_from(source.len()).map_err(|_| {
        AnalysisError::new(
            AnalysisErrorKind::InvalidFormatRange,
            "source is too large to format through TextRange",
        )
    })?;
    Ok(TextRange::new(TextSize::new(0), TextSize::new(end)))
}

fn unknown_file(file: FileId) -> AnalysisError {
    AnalysisError::new(
        AnalysisErrorKind::UnknownFile,
        format!("unknown file id {}", file.as_u32()),
    )
}

fn frontend_diagnostics(diagnostics: &[RenderedDiagnostic]) -> AnalysisError {
    let message = diagnostics
        .first()
        .map(|diagnostic| diagnostic.message.clone())
        .unwrap_or_else(|| "frontend query failed without diagnostics".to_string());
    AnalysisError::new(AnalysisErrorKind::FrontendDiagnostic, message)
}

fn call_identifier_before_position(facts: &EditorFacts, position: &TextPosition) -> Option<String> {
    let token = facts.token_at_position(position)?;
    if token.text != "(" {
        return None;
    }
    facts
        .tokens
        .iter()
        .rev()
        .find(|candidate| {
            candidate.range.end() <= token.range.start()
                && crate::editor::is_identifier_token(candidate)
        })
        .map(|candidate| format!("{}(...)", candidate.text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::SymbolQuery;
    use crate::snapshot::{AnalysisErrorKind, AnalysisQueryKind};
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
        let mut host =
            AnalysisHost::open_single_file(single_file_input("def main():\n    return 1\n"))
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
        let mut host =
            AnalysisHost::open_single_file(single_file_input("def main():\n    return 1\n"))
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
            host.explain_diagnostic(&DiagnosticId("SIFR-LINT-0004".to_string()))
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
                    diagnostics: vec![DiagnosticId("SIFR-LINT-0004".to_string())],
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

        let explanation = host
            .explain_diagnostic(&DiagnosticId("SIFR-NOPE-0000".to_string()))
            .expect("explain diagnostic should query")
            .into_value();
        assert!(explanation.diagnostic.is_none());
        assert!(explanation.unavailable_reason.is_some());
    }
}
