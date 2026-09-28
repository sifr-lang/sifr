use super::implementation::{AnalysisHost, QueryResult};
use super::text_edits::{fixed_source_edits, ranges_overlap, source_edit_to_text_edit};
use crate::editor::line_end_insert_range;
use crate::queries::{
    CodeAction, CodeActionContext, CodeActionData, DeferredCodeAction, DiagnosticClass,
    DiagnosticId, FileTextEdits, WorkspaceEdit,
};
use crate::snapshot::{AnalysisError, AnalysisQueryKind};
use ruff_text_size::{TextRange, TextSize};
use sifr_diagnostics::{DiagnosticArg, RenderedDiagnostic};
use sifr_frontend::{DocumentVersion, FileId};

impl AnalysisHost {
    pub fn code_actions(
        &mut self,
        file: FileId,
        range: TextRange,
        context: &CodeActionContext,
    ) -> QueryResult<Vec<CodeAction>> {
        let source = self.source_text(file)?;
        let mut actions = Vec::new();
        let diagnostics = if context
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.class == DiagnosticClass::Policy)
        {
            self.lint_diagnostics(file)?
        } else {
            Vec::new()
        };
        let suppression = context.diagnostics.iter().find_map(|policy| {
            if policy.class != DiagnosticClass::Policy {
                return None;
            }
            let rule = policy.rule_id.as_deref()?;
            diagnostics.iter().find_map(|current| {
                current
                    .spans
                    .iter()
                    .find_map(|span| {
                        suppression_insert_range(&source, range, policy, current, span)
                    })
                    .map(|insert_range| (rule, insert_range))
            })
        });
        if let Some((rule, insert_range)) = suppression {
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
        actions.extend(self.safe_fix_actions(file, range, context, &diagnostics)?);
        actions.extend(self.sql_code_actions(file, range, context)?);
        Ok(self.result(AnalysisQueryKind::CodeActions, actions))
    }

    pub fn safe_fix_all_action(&mut self, file: FileId) -> QueryResult<WorkspaceEdit> {
        let source = self.source_text(file)?;
        let diagnostics = self.lint_diagnostics(file)?;
        let path = self
            .context()?
            .path_for_file(file)
            .map(std::path::Path::to_path_buf);
        let fixed = sifr_lint::fix_source_from_diagnostics(
            &source,
            path.as_deref(),
            &sifr_lint::LintOptions::default(),
            &diagnostics,
        );
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
        diagnostics: &[RenderedDiagnostic],
    ) -> Result<Vec<CodeAction>, AnalysisError> {
        let fixes = sifr_lint::collect_fixes(
            diagnostics,
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
}

fn suppression_insert_range(
    source: &str,
    requested: TextRange,
    policy: &DiagnosticId,
    current: &RenderedDiagnostic,
    span: &sifr_diagnostics::DiagnosticSpan,
) -> Option<TextRange> {
    if current.code != policy.code || !span.is_primary {
        return None;
    }
    let (Some(DiagnosticArg::String(current_rule)), Some(requested_rule)) =
        (current.args.get("rule"), policy.rule_id.as_deref())
    else {
        return None;
    };
    if current_rule.as_str() != requested_rule {
        return None;
    }
    let requested_start = usize::try_from(requested.start().to_u32()).ok()?;
    let requested_end = usize::try_from(requested.end().to_u32()).ok()?;
    let span_start = usize::try_from(span.byte_start).ok()?;
    source.get(..requested_start)?;
    source.get(..requested_end)?;
    source.get(..span_start)?;
    let first_line = source[..requested_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let last_line = source[..requested_end]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let span_line = source[..span_start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    if span_line < first_line || span_line > last_line {
        return None;
    }
    let location = TextSize::new(span.byte_start);
    line_end_insert_range(source, TextRange::new(location, location))
}
