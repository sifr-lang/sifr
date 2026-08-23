use crate::suppression::ParserAwareSuppressions;
use crate::{
    LintDiagnosticSpec, LintOptions, SuppressionComplexity, line_index_for_byte,
    lint_rule_diagnostic, rule_enabled, with_rule_severity,
};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{Expr, Stmt};
use std::path::Path;

const RULE: &str = "boolean-positional-argument";

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
    let mut visitor = BooleanArgumentVisitor {
        source,
        file,
        options,
        suppressions,
        diagnostics: Vec::new(),
        _scope: std::marker::PhantomData,
    };
    visitor.visit_body(suite);
    visitor.diagnostics
}

struct BooleanArgumentVisitor<'a, 's> {
    source: &'a str,
    file: Option<&'a Path>,
    options: &'a LintOptions,
    suppressions: &'a mut ParserAwareSuppressions,
    diagnostics: Vec<RenderedDiagnostic>,
    _scope: std::marker::PhantomData<&'s ()>,
}

impl<'s> BooleanArgumentVisitor<'_, 's> {
    fn visit_body(&mut self, suite: &'s [Stmt]) {
        for stmt in suite {
            self.visit_stmt(stmt);
        }
    }
}

impl<'s> Visitor<'s> for BooleanArgumentVisitor<'_, 's> {
    fn visit_expr(&mut self, expr: &'s Expr) {
        if let Expr::Call(call) = expr {
            for argument in &call.arguments.args {
                if let Expr::BooleanLiteral(boolean) = argument {
                    let byte_start = boolean.range.start().to_u32();
                    let diagnostic_line = line_index_for_byte(self.source, byte_start);
                    if self.suppressions.mark_suppressed(
                        diagnostic_line,
                        RULE,
                        SuppressionComplexity::SingleNode,
                    ) {
                        continue;
                    }
                    self.diagnostics.push(with_rule_severity(
                        lint_rule_diagnostic(LintDiagnosticSpec {
                            code: DiagnosticCode::LINT_BOOLEAN_POSITIONAL_ARGUMENT,
                            message: "boolean literal passed as a positional argument".to_string(),
                            message_template: "boolean literal passed as a positional argument",
                            rule: RULE,
                            file: self.file,
                            source: self.source,
                            byte_start,
                            byte_end: boolean.range.end().to_u32(),
                            label: "positional boolean literal",
                            help: Some("prefer a named argument to document the boolean meaning"),
                            extra_args: Vec::new(),
                        }),
                        RULE,
                        self.options,
                    ));
                }
            }
        }
        visitor::walk_expr(self, expr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boolean_positional_argument_reports_literal_call_arg() {
        let source = "def main():\n    configure(True)\n";
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
        assert_eq!(diagnostics[0].code, "SIFR-LINT-0006");
    }
}
