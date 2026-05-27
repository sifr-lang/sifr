use crate::{
    line_index_for_byte, lint_rule_diagnostic, rule_enabled, with_rule_severity,
    LintDiagnosticSpec, LintOptions, SuppressionComplexity,
};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_syntax::SyntaxToken;
use std::path::Path;

const RULE: &str = "todo-comment";

pub(crate) fn lint(
    tokens: &[SyntaxToken],
    source: &str,
    file: Option<&Path>,
    options: &LintOptions,
    suppressions: &mut crate::ParserAwareSuppressions,
) -> Vec<RenderedDiagnostic> {
    if !rule_enabled(RULE, file, options) {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for token in tokens
        .iter()
        .filter(|token| token.kind.as_str() == "Comment")
    {
        let byte_start = token.range.start().to_u32();
        let byte_end = token.range.end().to_u32();
        let Ok(start) = usize::try_from(byte_start) else {
            continue;
        };
        let Ok(end) = usize::try_from(byte_end) else {
            continue;
        };
        let Some(comment) = source.get(start..end) else {
            continue;
        };
        let Some((marker, marker_offset)) = tracked_marker(comment) else {
            continue;
        };
        let marker_start = byte_start.saturating_add(u32::try_from(marker_offset).unwrap_or(0));
        if suppressions.mark_suppressed(
            line_index_for_byte(source, marker_start),
            RULE,
            SuppressionComplexity::PhysicalLine,
        ) {
            continue;
        }
        diagnostics.push(with_rule_severity(
            lint_rule_diagnostic(LintDiagnosticSpec {
                code: DiagnosticCode::LINT_TODO_COMMENT,
                message: format!("comment contains tracked task marker '{marker}'"),
                message_template: "comment contains tracked task marker '{marker}'",
                rule: RULE,
                file,
                source,
                byte_start: marker_start,
                byte_end: marker_start.saturating_add(u32::try_from(marker.len()).unwrap_or(1)),
                label: "tracked task marker",
                help: Some("track the work item externally or remove the marker"),
                extra_args: vec![("marker", marker.to_string())],
            }),
            RULE,
            options,
        ));
    }
    diagnostics
}

fn tracked_marker(line: &str) -> Option<(&'static str, usize)> {
    ["TODO", "FIXME"]
        .into_iter()
        .find_map(|marker| line.find(marker).map(|marker_start| (marker, marker_start)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_comment_reports_tracked_marker() {
        let source = "# TODO: wire docs\n";
        let parsed = sifr_syntax::parse_module(source, None).unwrap();
        let mut suppressions = crate::ParserAwareSuppressions::new(source, false);
        let diagnostics = lint(
            parsed.tokens(),
            source,
            None,
            &LintOptions::default(),
            &mut suppressions,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "SIFR-LINT-0005");
    }

    #[test]
    fn todo_comment_ignores_marker_inside_string_literal() {
        let source = "def main():\n    marker: str = \"# TODO\"\n";
        let parsed = sifr_syntax::parse_module(source, None).unwrap();
        let mut suppressions = crate::ParserAwareSuppressions::new(source, false);
        let diagnostics = lint(
            parsed.tokens(),
            source,
            None,
            &LintOptions::default(),
            &mut suppressions,
        );
        assert!(diagnostics.is_empty());
    }
}
