use crate::diagnostics::RenderedDiagnostic;
use ruff_text_size::TextRange;
use sifr_diagnostics::{
    DiagnosticArg, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, SourceMap, SourceSpan,
};
use sifr_hir::{
    lower_module_with_externals, ExternalDefs, HirDiagnostic, LoweringResult,
    LoweringWarningDiagnostic, RevealTypeDiagnostic,
};
use sifr_python_ast::Stmt;

#[derive(Default)]
pub(crate) struct FrontendModuleDiagnostics {
    pub(crate) reveal_types: Vec<RevealTypeDiagnostic>,
    pub(crate) rendered_reveal_types: Vec<RenderedDiagnostic>,
    pub(crate) warnings: Vec<LoweringWarningDiagnostic>,
    pub(crate) rendered_warnings: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Copy)]
pub(crate) enum FrontendDiagnosticStyle {
    Bare,
    ModulePrefixed,
}

#[derive(Clone, Copy)]
pub(crate) struct FrontendSourceContext<'a> {
    pub(crate) display_path: &'a str,
    pub(crate) source: &'a str,
}

#[cfg(test)]
pub(crate) fn lower_frontend_module(
    module_name: &str,
    stmts: &[Stmt],
    external_defs: &ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    lower_frontend_module_with_source(module_name, stmts, external_defs, diagnostic_style, None)
}

pub(crate) fn lower_frontend_module_with_source(
    module_name: &str,
    stmts: &[Stmt],
    external_defs: &ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    let result = match lower_module_with_externals(stmts, external_defs) {
        Ok(result) => result,
        Err(errors) => {
            let diagnostics: Vec<RenderedDiagnostic> = errors
                .into_iter()
                .map(|error| {
                    hir_diagnostic_to_rendered(module_name, diagnostic_style, source_context, error)
                })
                .collect();
            return Err(diagnostics);
        }
    };
    Ok(result)
}

fn hir_diagnostic_to_rendered(
    module_name: &str,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
    error: HirDiagnostic,
) -> RenderedDiagnostic {
    let code = hir_diagnostic_code_or_internal(&error);
    let uncoded = error.code.is_none();
    let primary_range = error.primary_range;
    let message = match diagnostic_style {
        FrontendDiagnosticStyle::Bare => error.message,
        FrontendDiagnosticStyle::ModulePrefixed => {
            format!("[{}] {}", module_name, error.message)
        }
    };
    let message = if uncoded {
        format!(
            "internal compiler error: HIR lowering emitted a diagnostic without canonical code: {message}"
        )
    } else {
        message
    };
    if let (Some(context), Some(range)) = (source_context, primary_range) {
        return diagnostic_with_source_range(
            code,
            context,
            range,
            "{message}",
            &[("message", DiagnosticArg::String(message.clone()))],
        );
    }
    crate::diagnostics::diagnostic_with_code(message, code)
}

pub(crate) fn hir_diagnostic_code_or_internal(error: &HirDiagnostic) -> DiagnosticCode {
    error
        .code
        .unwrap_or(DiagnosticCode::INTERNAL_COMPILER_PANIC)
}

fn diagnostic_with_source_range(
    code: DiagnosticCode,
    source_context: FrontendSourceContext<'_>,
    range: TextRange,
    message_template: &'static str,
    args: &[(&'static str, DiagnosticArg)],
) -> RenderedDiagnostic {
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(source_context.display_path, source_context.source);
    let span = SourceSpan::new(source_id, range);
    let mut builder = DiagnosticBuilder::source(code, code.declared_severity(), span)
        .message_template(message_template);
    for (name, value) in args {
        builder = builder.arg(name, value.clone());
    }
    let diagnostic = builder.build();
    let mut sink = DiagnosticSink::new();
    if code.declared_severity() == sifr_diagnostics::Severity::Error {
        let _ = sink.emit_error(diagnostic);
    } else {
        sink.emit(diagnostic);
    }
    match sifr_diagnostics::render::render_sink(&sink, &source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        Ok(_) => crate::diagnostics::diagnostic_with_code(
            "internal compiler error: frontend diagnostic renderer emitted an unexpected diagnostic count",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
        Err(error) => crate::diagnostics::diagnostic_with_code(
            format!("internal compiler error: invalid frontend diagnostic span: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

#[must_use]
pub(crate) fn reveal_type_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    reveal_types: &[RevealTypeDiagnostic],
) -> Vec<RenderedDiagnostic> {
    reveal_types
        .iter()
        .map(|diagnostic| reveal_type_diagnostic(source_context, diagnostic))
        .collect()
}

#[must_use]
pub(crate) fn warning_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    warnings: &[LoweringWarningDiagnostic],
) -> Vec<RenderedDiagnostic> {
    warnings
        .iter()
        .map(|diagnostic| warning_diagnostic(source_context, diagnostic))
        .collect()
}

fn warning_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &LoweringWarningDiagnostic,
) -> RenderedDiagnostic {
    let (code, message, message_template, args, primary_range) = match diagnostic {
        LoweringWarningDiagnostic::ArithmeticOverflowRisk {
            operation,
            primary_range,
        } => (
            DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK,
            format!("integer {operation} may overflow at runtime"),
            "integer {operation} may overflow at runtime",
            vec![("operation", DiagnosticArg::String(operation.clone()))],
            *primary_range,
        ),
        LoweringWarningDiagnostic::UnreachableStatement { primary_range } => (
            DiagnosticCode::FLOW_UNREACHABLE_STATEMENT,
            "unreachable statement ignored".to_string(),
            "unreachable statement ignored",
            Vec::new(),
            *primary_range,
        ),
        LoweringWarningDiagnostic::BigIntTransitionAlias { primary_range } => (
            DiagnosticCode::INT_BIGINT_TRANSITION_ALIAS,
            "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values".to_string(),
            "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values",
            Vec::new(),
            *primary_range,
        ),
    };
    if let (Some(context), Some(range)) = (source_context, primary_range) {
        return diagnostic_with_source_range(code, context, range, message_template, &args);
    }
    rendered_spanless_diagnostic(code, message, message_template, &args)
}

fn reveal_type_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &RevealTypeDiagnostic,
) -> RenderedDiagnostic {
    let code = DiagnosticCode::TYPE_REVEAL_TYPE;
    let message = format!("revealed type is {}", diagnostic.revealed_type);
    let args = [(
        "revealed_type",
        DiagnosticArg::String(diagnostic.revealed_type.clone()),
    )];
    if let (Some(context), Some(range)) = (source_context, diagnostic.primary_range) {
        return diagnostic_with_source_range(
            code,
            context,
            range,
            "revealed type is {revealed_type}",
            &args,
        );
    }
    rendered_spanless_diagnostic(code, message, "revealed type is {revealed_type}", &args)
}

fn rendered_spanless_diagnostic(
    code: DiagnosticCode,
    message: String,
    message_template: &'static str,
    args: &[(&'static str, DiagnosticArg)],
) -> RenderedDiagnostic {
    let mut rendered_args = std::collections::BTreeMap::new();
    for (name, value) in args {
        rendered_args.insert((*name).to_string(), value.clone());
    }
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: message_template.to_string(),
        args: rendered_args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{hir_diagnostic_to_rendered, FrontendDiagnosticStyle};
    use sifr_diagnostics::DiagnosticCode;
    use sifr_hir::HirDiagnostic;

    fn hir_diagnostic(code: Option<DiagnosticCode>, message: &str) -> HirDiagnostic {
        HirDiagnostic {
            code,
            message: message.to_string(),
            primary_range: None,
            line: None,
            col: None,
        }
    }

    #[test]
    fn coded_hir_diagnostic_uses_active_diagnostic_code() {
        let error = hir_diagnostic(Some(DiagnosticCode::TYPE_MISMATCH), "expected int, got str");

        let diagnostic =
            hir_diagnostic_to_rendered("main", FrontendDiagnosticStyle::Bare, None, error);

        assert_eq!(diagnostic.code, "SIFR-TYPE-0002");
        assert_eq!(diagnostic.url, "https://sifr.sh/docs/errors/SIFR-TYPE-0002");
    }

    #[test]
    fn codeless_hir_diagnostic_is_internal_compiler_diagnostic() {
        let error = hir_diagnostic(None, "expected int, got str");

        let diagnostic = hir_diagnostic_to_rendered(
            "main",
            FrontendDiagnosticStyle::ModulePrefixed,
            None,
            error,
        );

        assert_eq!(
            diagnostic.message,
            "internal compiler error: HIR lowering emitted a diagnostic without canonical code: [main] expected int, got str"
        );
        assert_eq!(diagnostic.code, "SIFR-INTERNAL-0001");
    }
}
