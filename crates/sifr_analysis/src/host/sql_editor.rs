use super::implementation::AnalysisHost;
use crate::queries::{
    CodeAction, CodeActionContext, CompletionItem, CompletionItems, DocumentHighlight,
    FileTextEdits, HoverInfo, InlayHint, Location, SemanticToken, SymbolName, WorkspaceEdit,
};
use crate::snapshot::{AnalysisError, AnalysisErrorKind};
use ruff_text_size::{TextRange, TextSize};
use sifr_frontend::{FileId, SqlEditorDocumentView};
use sifr_syntax::{SourceText, TextPosition};
use std::fmt::Write as _;

impl AnalysisHost {
    pub(super) fn sql_completion(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> Result<Option<CompletionItems>, AnalysisError> {
        let Some((document, offset)) = self.sql_document_at(file, position)? else {
            return Ok(None);
        };
        let items = document
            .completion_symbols(offset)
            .into_iter()
            .map(|symbol| {
                let detail = symbol_detail(&symbol);
                let symbol_file = self.file_for_document(symbol.definition_document.as_deref());
                CompletionItem {
                    label: symbol.name,
                    kind: symbol.kind,
                    detail,
                    symbol_file,
                }
            })
            .collect();
        Ok(Some(CompletionItems { items }))
    }

    pub(super) fn sql_hover(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> Result<Option<HoverInfo>, AnalysisError> {
        let Some((document, offset)) = self.sql_document_at(file, position)? else {
            return Ok(None);
        };
        let Some(symbol) = document.symbol_at_source_offset(offset) else {
            return Ok(None);
        };
        let mut contents = format!("SQL {} `{}`", symbol.kind, symbol.name);
        if let Some(database_type) = &symbol.database_type {
            let _ = write!(contents, "\n\nDatabase type: `{database_type}`");
        }
        if let Some(sifr_type) = &symbol.sifr_type {
            let _ = write!(contents, "\n\nSifr type: `{sifr_type}`");
        }
        if let Some(nullable) = symbol.nullable {
            let _ = write!(contents, "\n\nNullable: `{nullable}`");
        }
        let _ = write!(contents, "\n\nCardinality: `{}`", document.cardinality);
        Ok(Some(HoverInfo {
            contents,
            symbol_name: symbol.name,
            symbol_file: self.file_for_document(symbol.definition_document.as_deref()),
        }))
    }

    pub(super) fn sql_locations(
        &mut self,
        file: FileId,
        position: &TextPosition,
        definitions_only: bool,
    ) -> Result<Option<Vec<Location>>, AnalysisError> {
        let Some((document, offset)) = self.sql_document_at(file, position)? else {
            return Ok(None);
        };
        let Some(symbol) = document.symbol_at_source_offset(offset) else {
            return Ok(Some(Vec::new()));
        };
        if definitions_only
            && let (Some(definition_file), Some(range)) = (
                self.file_for_document(symbol.definition_document.as_deref()),
                symbol.definition_range,
            )
        {
            return Ok(Some(vec![Location {
                file: definition_file,
                range: Some(range),
            }]));
        }
        let mut locations = Vec::new();
        for (candidate_file, candidate) in self.all_sql_documents()? {
            locations.extend(
                candidate
                    .source_ranges_for_symbol(&symbol.name)
                    .into_iter()
                    .map(|range| Location {
                        file: candidate_file,
                        range: Some(range),
                    }),
            );
        }
        if definitions_only {
            locations.truncate(1);
        }
        Ok(Some(locations))
    }

    pub(super) fn sql_rename(
        &mut self,
        file: FileId,
        position: &TextPosition,
        new_name: &SymbolName,
    ) -> Result<Option<WorkspaceEdit>, AnalysisError> {
        if !valid_sql_identifier(&new_name.0) {
            return Err(AnalysisError::new(
                AnalysisErrorKind::FrontendDiagnostic,
                "SQL rename requires one unquoted identifier",
            ));
        }
        let Some((document, offset)) = self.sql_document_at(file, position)? else {
            return Ok(None);
        };
        let Some(symbol) = document.symbol_at_source_offset(offset) else {
            return Ok(Some(WorkspaceEdit { edits: Vec::new() }));
        };
        let edits = self
            .all_sql_documents()?
            .into_iter()
            .filter_map(|(candidate_file, candidate)| {
                let edits = candidate
                    .source_ranges_for_symbol(&symbol.name)
                    .into_iter()
                    .map(|range| sifr_format::TextEdit {
                        range,
                        replacement: new_name.0.clone(),
                    })
                    .collect::<Vec<_>>();
                (!edits.is_empty()).then_some(FileTextEdits {
                    file: candidate_file,
                    edits,
                })
            })
            .collect();
        Ok(Some(WorkspaceEdit { edits }))
    }

    pub(super) fn sql_semantic_tokens(
        &mut self,
        file: FileId,
        range: Option<TextRange>,
    ) -> Result<Vec<SemanticToken>, AnalysisError> {
        let documents = self.sql_documents_for_file(file)?;
        Ok(documents
            .into_iter()
            .flat_map(|document| document.semantic_source_tokens())
            .filter(|(token_range, _)| {
                range.is_none_or(|range| ranges_overlap(range, *token_range))
            })
            .map(|(range, token_type)| SemanticToken {
                range,
                token_type: token_type.to_string(),
                modifiers: vec!["embedded".to_string(), "sql".to_string()],
            })
            .collect())
    }

    pub(super) fn sql_inlay_hints(
        &mut self,
        file: FileId,
        range: Option<TextRange>,
    ) -> Result<Vec<InlayHint>, AnalysisError> {
        let source = self.source_text(file)?;
        let source = SourceText::new(source);
        let mut hints = Vec::new();
        for document in self.sql_documents_for_file(file)? {
            for (index, parameter_range) in document.parameter_source_ranges() {
                if range.is_some_and(|range| !ranges_overlap(range, parameter_range)) {
                    continue;
                }
                let Some(position) = source.text_position(parameter_range.end()) else {
                    continue;
                };
                let parameter_type = document
                    .parameter_types
                    .get(index)
                    .map_or("unknown", String::as_str);
                hints.push(InlayHint {
                    position,
                    label: format!("${}: {parameter_type}", index + 1),
                });
            }
            if let Some(position) = source.text_position(document.template.source_range.end()) {
                let fields = document
                    .result_fields
                    .iter()
                    .map(|field| field.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                hints.push(InlayHint {
                    position,
                    label: if fields.is_empty() {
                        format!("SQL cardinality: {}", document.cardinality)
                    } else {
                        format!("SQL result: {{{fields}}}; {}", document.cardinality)
                    },
                });
            }
        }
        Ok(hints)
    }

    pub(super) fn sql_highlights(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> Result<Option<Vec<DocumentHighlight>>, AnalysisError> {
        let Some((document, offset)) = self.sql_document_at(file, position)? else {
            return Ok(None);
        };
        let Some(symbol) = document.symbol_at_source_offset(offset) else {
            return Ok(Some(Vec::new()));
        };
        Ok(Some(
            document
                .source_ranges_for_symbol(&symbol.name)
                .into_iter()
                .map(|range| DocumentHighlight { range })
                .collect(),
        ))
    }

    pub(super) fn sql_code_actions(
        &mut self,
        file: FileId,
        range: TextRange,
        context: &CodeActionContext,
    ) -> Result<Vec<CodeAction>, AnalysisError> {
        let mut actions = Vec::new();
        for document in self.sql_documents_for_file(file)? {
            if !document.contains_source_offset(range.start()) {
                continue;
            }
            for diagnostic in &context.diagnostics {
                for fix in document.fixes_for_diagnostic(&diagnostic.code, range) {
                    let Some(source_range) = document.source_range_for_fix(&fix) else {
                        continue;
                    };
                    let edit = fix.replacement.map(|replacement| WorkspaceEdit {
                        edits: vec![FileTextEdits {
                            file,
                            edits: vec![sifr_format::TextEdit {
                                range: source_range,
                                replacement,
                            }],
                        }],
                    });
                    actions.push(CodeAction {
                        title: fix.detail.map_or(fix.title.clone(), |detail| {
                            format!("{} — {detail}", fix.title)
                        }),
                        kind: format!("quickfix.sifr.sql.{}", fix_kind(fix.kind)),
                        edit,
                        data: None,
                    });
                }
            }
        }
        actions.sort_by(|left, right| (&left.title, &left.kind).cmp(&(&right.title, &right.kind)));
        actions.dedup_by(|left, right| left.title == right.title && left.kind == right.kind);
        Ok(actions)
    }

    pub(super) fn sql_document_at(
        &mut self,
        file: FileId,
        position: &TextPosition,
    ) -> Result<Option<(SqlEditorDocumentView, TextSize)>, AnalysisError> {
        let source = SourceText::new(self.source_text(file)?);
        let Some(offset) = source.byte_offset(position) else {
            return Ok(None);
        };
        Ok(self
            .sql_documents_for_file(file)?
            .into_iter()
            .find(|document| document.contains_source_offset(offset))
            .map(|document| (document, offset)))
    }

    fn sql_documents_for_file(
        &mut self,
        file: FileId,
    ) -> Result<Vec<SqlEditorDocumentView>, AnalysisError> {
        let Some(module) = self.file_to_module.get(&file).copied() else {
            return Ok(Vec::new());
        };
        Ok(self
            .context_mut()?
            .analysis_for_module(module)
            .into_value()
            .sql_documents)
    }

    fn all_sql_documents(&mut self) -> Result<Vec<(FileId, SqlEditorDocumentView)>, AnalysisError> {
        let files = self.file_to_module.keys().copied().collect::<Vec<_>>();
        let mut documents = Vec::new();
        for file in files {
            documents.extend(
                self.sql_documents_for_file(file)?
                    .into_iter()
                    .map(|document| (file, document)),
            );
        }
        Ok(documents)
    }

    fn file_for_document(&self, document: Option<&str>) -> Option<FileId> {
        let document = document?;
        self.context()
            .ok()?
            .source_map()
            .files
            .iter()
            .find_map(|file| {
                (file.module_name.as_deref() == Some(document)
                    || file.canonical_path.as_path().to_string_lossy() == document)
                    .then_some(file.id)
            })
    }
}

fn symbol_detail(symbol: &sifr_frontend::SqlEditorSymbol) -> Option<String> {
    let mut details = Vec::new();
    if let Some(database_type) = &symbol.database_type {
        details.push(format!("database {database_type}"));
    }
    if let Some(sifr_type) = &symbol.sifr_type {
        details.push(format!("Sifr {sifr_type}"));
    }
    if let Some(nullable) = symbol.nullable {
        details.push(if nullable { "nullable" } else { "not null" }.to_string());
    }
    (!details.is_empty()).then(|| details.join("; "))
}

fn fix_kind(kind: sifr_frontend::SqlEditorFixKind) -> &'static str {
    match kind {
        sifr_frontend::SqlEditorFixKind::Alias => "alias",
        sifr_frontend::SqlEditorFixKind::Cast => "cast",
        sifr_frontend::SqlEditorFixKind::MissingColumn => "missingColumn",
        sifr_frontend::SqlEditorFixKind::UnsafeCollection => "unsafeCollection",
        sifr_frontend::SqlEditorFixKind::MigrationImpact => "migrationImpact",
    }
}

fn valid_sql_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn ranges_overlap(left: TextRange, right: TextRange) -> bool {
    left.start() < right.end() && right.start() < left.end()
}
