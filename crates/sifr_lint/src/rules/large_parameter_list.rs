use crate::suppression::ParserAwareSuppressions;
use crate::{
    line_index_for_byte, lint_rule_diagnostic, rule_enabled, with_rule_severity,
    LintDiagnosticSpec, LintOptions, SuppressionComplexity,
};
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_ir::{HirClass, HirFunction, HirModule};
use std::path::Path;

const RULE: &str = "large-parameter-list";
const PARAMETER_LIMIT: usize = 5;

pub(crate) fn lint(
    module: &HirModule,
    source: &str,
    file: Option<&Path>,
    options: &LintOptions,
    suppressions: &mut ParserAwareSuppressions,
) -> Vec<RenderedDiagnostic> {
    if !rule_enabled(RULE, file, options) {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    for function in module
        .functions
        .iter()
        .chain(module.classes.iter().flat_map(class_methods))
    {
        if function.params.len() <= PARAMETER_LIMIT {
            continue;
        }
        let byte_start = find_function_start(source, &function.name);
        if suppressions.mark_suppressed(
            line_index_for_byte(source, byte_start),
            RULE,
            SuppressionComplexity::StatementRange,
        ) {
            continue;
        }
        diagnostics.push(with_rule_severity(
            lint_rule_diagnostic(LintDiagnosticSpec {
                code: DiagnosticCode::LINT_LARGE_PARAMETER_LIST,
                message: format!(
                    "function '{}' has {} parameters, exceeding the policy limit of {}",
                    function.name,
                    function.params.len(),
                    PARAMETER_LIMIT
                ),
                message_template:
                    "function '{function}' has {count} parameters, exceeding the policy limit of {limit}",
                rule: RULE,
                file,
                source,
                byte_start,
                byte_end: byte_start.saturating_add(3),
                label: "large parameter list",
                help: Some("consider grouping related data into a named type"),
                extra_args: vec![
                    ("function", function.name.clone()),
                    ("count", function.params.len().to_string()),
                    ("limit", PARAMETER_LIMIT.to_string()),
                ],
            }),
            RULE,
            options,
        ));
    }
    diagnostics
}

fn class_methods(class: &HirClass) -> impl Iterator<Item = &HirFunction> {
    class.methods.iter()
}

fn find_function_start(source: &str, name: &str) -> u32 {
    let needle = format!("def {name}");
    source
        .find(&needle)
        .and_then(|offset| u32::try_from(offset).ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_parameter_list_reports_hir_function() {
        let source =
            "def configure(a: int, b: int, c: int, d: int, e: int, f: int) -> int:\n    return a\n";
        let input = sifr_frontend::FrontendInput {
            path: sifr_frontend::SourcePath::new("main.sifr"),
            source: sifr_frontend::SourceText::new(source),
            mode: sifr_frontend::FrontendMode::SingleFile,
        };
        let mut context = sifr_frontend::FrontendContext::load_single_file(input).unwrap();
        let module = context.module_graph().entrypoint;
        let lowered = context.hir_module_view(module).into_value().hir;
        let mut suppressions = ParserAwareSuppressions::new(source, false);
        let diagnostics = lint(
            &lowered,
            source,
            None,
            &LintOptions::default(),
            &mut suppressions,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "SIFR-LINT-0007");
    }
}
