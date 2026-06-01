use crate::completion::{rank_completion_candidates, CompletionCandidate};
use crate::editor::{line_end_insert_range, EditorFacts, EditorToken};
use crate::queries::{
    CodeAction, CodeActionContext, CodeActionData, CompletionItem, CompletionItems,
    DeferredCodeAction, DiagnosticClass, DiagnosticExplanation, DiagnosticId, DocumentHighlight,
    DocumentSymbol, FileDiagnostics, FileTextEdits, FoldingRange, FormatOptions,
    GeneratedRustPreview, HoverInfo, InlayHint, Location, RenameTarget, SelectionRange,
    SemanticToken, SignatureHelp, SymbolName, SymbolQuery, TestCommand, TestCommandKind, TestItem,
    TestItemId, TypeHierarchyItem, TypeHierarchyItemId, WorkspaceEdit, WorkspaceSymbol,
};
use crate::snapshot::{
    AnalysisError, AnalysisErrorKind, AnalysisQueryKind, AnalysisQueryResult, AnalysisRevision,
    AnalysisSnapshot, QueryMetadata,
};
use crate::symbols::SymbolIndex;
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{
    DocumentVersion, FileId, FrontendInput, InvalidationReport, ModuleId, ProjectRoot, SourceText,
    UpdatedDocumentInfo, WorkspaceSession, WorkspaceSnapshot,
};
use sifr_syntax::TextPosition;
use std::collections::BTreeMap;

pub(super) type QueryResult<T> = Result<AnalysisQueryResult<T>, AnalysisError>;

pub struct AnalysisHost {
    pub(super) session: WorkspaceSession,
    pub(super) file_to_module: BTreeMap<FileId, ModuleId>,
    pub(super) symbol_index: Option<SymbolIndex>,
    pub(super) last_invalidation: Option<InvalidationReport>,
    pub(super) current_revision: AnalysisRevision,
}

impl AnalysisHost {
    pub fn open_project(root: &ProjectRoot) -> Result<Self, Vec<RenderedDiagnostic>> {
        let session = WorkspaceSession::open_project(root.clone())?;
        Self::new(session)
    }

    pub fn open_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>> {
        let session = WorkspaceSession::open_single_file(input)?;
        Self::new(session)
    }

    pub(super) fn new(mut session: WorkspaceSession) -> Result<Self, Vec<RenderedDiagnostic>> {
        let snapshot = session.snapshot();
        let Some(current_revision) = revision_from_workspace_snapshot(&snapshot) else {
            return Err(Vec::new());
        };
        let mut host = Self {
            session,
            file_to_module: BTreeMap::new(),
            symbol_index: None,
            last_invalidation: None,
            current_revision,
        };
        host.refresh_file_map();
        Ok(host)
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
        let current_version = self.context()?.document_version_for_file(file);
        if let Some(current) = current_version {
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
        let previous_revision = self.current_revision.graph;
        let report = {
            let context = self.context_mut()?;
            context
                .update_module_source(module, text, Some(version))
                .map_err(|diagnostics| frontend_diagnostics(&diagnostics))?
        };
        self.refresh_file_map();
        self.refresh_current_revision();
        self.session
            .record_dirty_scope(report.dirty_scope_report.clone());
        self.symbol_index = None;
        let report = InvalidationReport {
            previous_revision,
            next_revision: self.current_revision.graph,
            invalidated_modules: report.invalidated_modules,
            invalidated_queries: report.invalidated_queries,
            dirty_scope_report: report.dirty_scope_report,
            updated_documents: vec![UpdatedDocumentInfo {
                file,
                old_version: current_version,
                new_version: Some(version),
                text_changed: report
                    .updated_documents
                    .first()
                    .is_some_and(|document| document.text_changed),
            }],
        };
        self.last_invalidation = Some(report.clone());
        Ok(report)
    }

    pub fn snapshot(&mut self) -> AnalysisSnapshot {
        AnalysisSnapshot::new(self.session.snapshot(), self.current_revision)
    }

    #[must_use]
    pub fn is_snapshot_current(&self, snapshot: &AnalysisSnapshot) -> bool {
        snapshot.revision() == self.current_revision
            && snapshot.workspace().revision == self.session.revision()
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
            .context_mut()?
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
        if let Some(policy) = context
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.class == DiagnosticClass::Policy)
        {
            if let Some(insert_range) = line_end_insert_range(&source, range) {
                let rule = policy.rule_id.as_deref().unwrap_or("trailing-whitespace");
                actions.push(CodeAction {
                    title: format!("Suppress {rule} policy diagnostic"),
                    kind: "quickfix.sifr.suppress".to_string(),
                    edit: Some(WorkspaceEdit {
                        edits: vec![FileTextEdits {
                            file,
                            edits: vec![sifr_format::TextEdit {
                                range: insert_range,
                                replacement: format!("  # sifr: ignore[{rule}]"),
                            }],
                        }],
                    }),
                    data: None,
                });
            }
        }
        actions.extend(self.safe_fix_actions(file, range, context)?);
        Ok(self.result(AnalysisQueryKind::CodeActions, actions))
    }

    pub fn safe_fix_all_action(&mut self, file: FileId) -> QueryResult<WorkspaceEdit> {
        let source = self.source_text(file)?;
        let fixed = sifr_lint::fix_source(&source, None, &sifr_lint::LintOptions::default());
        Ok(self.result(
            AnalysisQueryKind::CodeActions,
            WorkspaceEdit {
                edits: fixed_source_edits(file, &source, &fixed.fixed_source),
            },
        ))
    }

    fn safe_fix_actions(
        &mut self,
        file: FileId,
        range: TextRange,
        context: &CodeActionContext,
    ) -> Result<Vec<CodeAction>, AnalysisError> {
        let source = self.source_text(file)?;
        let lint = sifr_lint::lint_source(&source, None, &sifr_lint::LintOptions::default());
        let fixes = sifr_lint::collect_fixes(
            &lint.diagnostics,
            &sifr_lint::FixOptions::from(&sifr_lint::LintOptions::default()),
        );
        let mut actions = Vec::new();
        for fix in fixes {
            if !context.diagnostics.iter().any(|diagnostic| {
                diagnostic.class == DiagnosticClass::Policy
                    && diagnostic.rule_id.as_deref() == Some(fix.rule_id.as_str())
            }) {
                continue;
            }
            let edits = fix
                .edits
                .iter()
                .map(source_edit_to_text_edit)
                .collect::<Vec<_>>();
            if edits.is_empty() || !edits.iter().any(|edit| ranges_overlap(edit.range, range)) {
                continue;
            }
            actions.push(CodeAction {
                title: format!("Apply safe fix for {}", fix.rule_id),
                kind: "quickfix.sifr.applySafeFix".to_string(),
                edit: Some(WorkspaceEdit {
                    edits: vec![FileTextEdits { file, edits }],
                }),
                data: None,
            });
        }
        if !actions.is_empty() {
            actions.push(CodeAction {
                title: "Fix all safe Sifr policy diagnostics".to_string(),
                kind: "source.fixAll.sifr".to_string(),
                edit: None,
                data: Some(CodeActionData {
                    action: DeferredCodeAction::FixAllSafePolicy,
                    file,
                    expected_version: self
                        .context()?
                        .document_version_for_file(file)
                        .map(DocumentVersion::as_i64),
                }),
            });
        }
        Ok(actions)
    }

    pub fn format_document(
        &mut self,
        file: FileId,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        let source = self.source_text(file)?;
        let path = self.context()?.path_for_file(file);
        let result = sifr_format::format_source(&source, path, options)
            .map_err(|diagnostics| frontend_diagnostics(&diagnostics))?;
        let edits = if result.formatted == source {
            Vec::new()
        } else {
            vec![sifr_format::TextEdit {
                range: full_range(&source)?,
                replacement: result.formatted,
            }]
        };
        Ok(self.result(AnalysisQueryKind::FormatDocument, edits))
    }

    pub fn format_range(
        &mut self,
        file: FileId,
        range: TextRange,
        options: FormatOptions,
    ) -> QueryResult<Vec<sifr_format::TextEdit>> {
        let source = self.source_text(file)?;
        let path = self.context()?.path_for_file(file);
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
            .find(|rendered| rendered.code == diagnostic.code);
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
        let revision = self.current_revision;
        let needs_refresh = self
            .symbol_index
            .as_ref()
            .is_none_or(|index| index.revision() != revision);
        if needs_refresh {
            let graph = self.context()?.module_graph();
            let analysis = self.context_mut()?.analysis_for_project();
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
        let path = self.context()?.path_for_file(file);
        Ok(sifr_lint::lint_source(&source, path, &sifr_lint::LintOptions::default()).diagnostics)
    }

    fn editor_facts(&mut self, file: FileId) -> Result<EditorFacts, AnalysisError> {
        let module = self.module_for_file(file)?;
        let source = self.source_text(file)?;
        let parsed = self.context_mut()?.parse_module(module).into_value().parsed;
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
        self.context()?
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

    fn context(&self) -> Result<&sifr_frontend::FrontendContext, AnalysisError> {
        self.session.context().ok_or_else(|| {
            AnalysisError::new(
                AnalysisErrorKind::FrontendDiagnostic,
                "analysis workspace session has not loaded frontend state",
            )
        })
    }

    fn context_mut(&mut self) -> Result<&mut sifr_frontend::FrontendContext, AnalysisError> {
        self.session.context_mut().ok_or_else(|| {
            AnalysisError::new(
                AnalysisErrorKind::FrontendDiagnostic,
                "analysis workspace session has not loaded frontend state",
            )
        })
    }

    pub(super) fn refresh_file_map(&mut self) {
        if let Some(context) = self.session.context() {
            self.file_to_module = context
                .module_graph()
                .modules
                .into_iter()
                .map(|module| (module.file, module.id))
                .collect();
        }
    }

    pub(super) fn refresh_current_revision(&mut self) {
        if let Some(context) = self.session.context() {
            self.current_revision = AnalysisRevision {
                graph: context.module_graph().revision,
                source: context.source_map().revision,
            };
        }
    }

    fn metadata(&self, query: AnalysisQueryKind) -> QueryMetadata {
        QueryMetadata {
            query,
            revision: self.current_revision,
            workspace_snapshot_id: None,
        }
    }

    fn result<T>(&self, query: AnalysisQueryKind, value: T) -> AnalysisQueryResult<T> {
        AnalysisQueryResult::new(value, self.metadata(query))
    }
}

impl AnalysisHost {
    pub(super) fn ensure_snapshot_current(
        &self,
        snapshot: &AnalysisSnapshot,
    ) -> Result<(), AnalysisError> {
        let current = self.current_revision;
        if self.is_snapshot_current(snapshot) {
            return Ok(());
        }
        Err(AnalysisError::new(
            AnalysisErrorKind::StaleSnapshot,
            format!(
                "analysis snapshot is stale: captured workspace {} graph/source {}:{}, current workspace {} graph/source {}:{}",
                snapshot.workspace().revision.as_u64(),
                snapshot.revision().graph.as_u64(),
                snapshot.revision().source.as_u64(),
                self.session.revision().as_u64(),
                current.graph.as_u64(),
                current.source.as_u64()
            ),
        ))
    }
}

fn revision_from_workspace_snapshot(snapshot: &WorkspaceSnapshot) -> Option<AnalysisRevision> {
    Some(AnalysisRevision {
        graph: snapshot.module_graph.as_ref()?.revision,
        source: snapshot.source_map.as_ref()?.revision,
    })
}

pub(super) fn full_range(source: &str) -> Result<TextRange, AnalysisError> {
    let end = u32::try_from(source.len()).map_err(|_| {
        AnalysisError::new(
            AnalysisErrorKind::InvalidFormatRange,
            "source is too large to format through TextRange",
        )
    })?;
    Ok(TextRange::new(TextSize::new(0), TextSize::new(end)))
}

fn source_edit_to_text_edit(edit: &sifr_lint::SourceEdit) -> sifr_format::TextEdit {
    sifr_format::TextEdit {
        range: TextRange::new(TextSize::new(edit.byte_start), TextSize::new(edit.byte_end)),
        replacement: edit.replacement.clone(),
    }
}

fn fixed_source_edits(file: FileId, source: &str, fixed: &str) -> Vec<FileTextEdits> {
    if source == fixed {
        Vec::new()
    } else {
        vec![FileTextEdits {
            file,
            edits: vec![sifr_format::TextEdit {
                range: full_range(source)
                    .unwrap_or_else(|_| TextRange::new(TextSize::new(0), TextSize::new(u32::MAX))),
                replacement: fixed.to_string(),
            }],
        }]
    }
}

fn ranges_overlap(left: TextRange, right: TextRange) -> bool {
    left.start() < right.end() && right.start() < left.end()
}

pub(super) fn unknown_file(file: FileId) -> AnalysisError {
    AnalysisError::new(
        AnalysisErrorKind::UnknownFile,
        format!("unknown file id {}", file.as_u32()),
    )
}

pub(super) fn frontend_diagnostics(diagnostics: &[RenderedDiagnostic]) -> AnalysisError {
    let message = diagnostics
        .first()
        .map(|diagnostic| diagnostic.message.clone())
        .unwrap_or_else(|| "frontend query failed without diagnostics".to_string());
    AnalysisError::new(AnalysisErrorKind::FrontendDiagnostic, message)
}

pub(super) fn call_identifier_before_position(
    facts: &EditorFacts,
    position: &TextPosition,
) -> Option<String> {
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
