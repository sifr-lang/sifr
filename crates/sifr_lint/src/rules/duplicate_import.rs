use crate::suppression::ParserAwareSuppressions;
use crate::{
    line_index_for_byte, lint_rule_diagnostic, rule_enabled, with_rule_severity,
    LintDiagnosticSpec, LintOptions, SuppressionComplexity,
};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_python_ast::Stmt;
use std::collections::BTreeSet;
use std::path::Path;

const RULE: &str = "duplicate-import";

pub(crate) fn lint(
    suite: &[Stmt],
    source: &str,
    file: Option<&Path>,
    options: &LintOptions,
    suppressions: &mut ParserAwareSuppressions,
) -> Vec<RenderedDiagnostic> {
    if !rule_enabled(RULE, file, options) {
        return Vec::new();
    }
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    let mut context = DuplicateImportContext {
        diagnostics: &mut diagnostics,
        suppressions,
        source,
        file,
        options,
        seen: &mut seen,
    };
    for stmt in suite {
        match stmt {
            Stmt::Import(import) => {
                for alias in &import.names {
                    context.push(
                        alias.name.as_str().to_string(),
                        alias.range.start().to_u32(),
                        alias.range.end().to_u32(),
                    );
                }
            }
            Stmt::ImportFrom(import) => {
                let module = import
                    .module
                    .as_ref()
                    .map_or_else(String::new, |module| module.as_str().to_string());
                for alias in &import.names {
                    context.push(
                        format!("{module}.{}", alias.name.as_str()),
                        alias.range.start().to_u32(),
                        alias.range.end().to_u32(),
                    );
                }
            }
            _ => {}
        }
    }
    diagnostics
}

struct DuplicateImportContext<'a> {
    diagnostics: &'a mut Vec<RenderedDiagnostic>,
    suppressions: &'a mut ParserAwareSuppressions,
    source: &'a str,
    file: Option<&'a Path>,
    options: &'a LintOptions,
    seen: &'a mut BTreeSet<String>,
}

impl DuplicateImportContext<'_> {
    fn push(&mut self, import: String, byte_start: u32, byte_end: u32) {
        if self.seen.insert(import.clone()) {
            return;
        }
        if self.suppressions.mark_suppressed(
            line_index_for_byte(self.source, byte_start),
            RULE,
            SuppressionComplexity::SymbolWorkspace,
        ) {
            return;
        }
        self.diagnostics.push(with_rule_severity(
            lint_rule_diagnostic(LintDiagnosticSpec {
                code: DiagnosticCode::LINT_DUPLICATE_IMPORT,
                message: format!("duplicate import of '{import}'"),
                message_template: "duplicate import of '{import}'",
                rule: RULE,
                file: self.file,
                source: self.source,
                byte_start,
                byte_end,
                label: "duplicate import",
                help: Some("remove one of the duplicate import declarations"),
                extra_args: vec![("import", import)],
            }),
            RULE,
            self.options,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_import_reports_repeated_import() {
        let source = "import math\nimport math\n";
        let parsed = sifr_syntax::parse_module(source, None).unwrap();
        let mut suppressions = ParserAwareSuppressions::new(source, false);
        let diagnostics = lint(
            parsed.suite(),
            source,
            None,
            &LintOptions::default(),
            &mut suppressions,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "SIFR-LINT-0008");
    }
}
