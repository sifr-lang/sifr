use crate::diagnostics::{write_stderr_line, RenderedDiagnostic};
use ruff_text_size::TextRange;
use sifr_diagnostics::{
    DiagnosticArg, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, SourceMap, SourceSpan,
};
use sifr_hir::{lower_module_with_externals, ExternalDefs, LoweringError, LoweringResult};
use sifr_python_ast::Stmt;

#[derive(Default)]
pub(crate) struct FrontendModuleDiagnostics {
    pub(crate) reveal_types: Vec<String>,
    pub(crate) warnings: Vec<String>,
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
                    lowering_error_to_diagnostic(
                        module_name,
                        diagnostic_style,
                        source_context,
                        error,
                    )
                })
                .collect();
            return Err(diagnostics);
        }
    };
    Ok(result)
}

fn lowering_error_to_diagnostic(
    module_name: &str,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
    error: LoweringError,
) -> RenderedDiagnostic {
    let code = lowering_error_code_or_internal(&error);
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
        return diagnostic_with_source_range(message, code, context, range);
    }
    crate::diagnostics::diagnostic_with_code(message, code)
}

pub(crate) fn lowering_error_code_or_internal(error: &LoweringError) -> DiagnosticCode {
    error
        .code
        .unwrap_or(DiagnosticCode::INTERNAL_COMPILER_PANIC)
}

fn diagnostic_with_source_range(
    message: String,
    code: DiagnosticCode,
    source_context: FrontendSourceContext<'_>,
    range: TextRange,
) -> RenderedDiagnostic {
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(source_context.display_path, source_context.source);
    let span = SourceSpan::new(source_id, range);
    let diagnostic = DiagnosticBuilder::source(code, code.declared_severity(), span)
        .message_template("{message}")
        .arg("message", DiagnosticArg::String(message))
        .build();
    let mut sink = DiagnosticSink::new();
    sink.emit_error(diagnostic);
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

pub(crate) fn emit_frontend_diagnostics(lowering_result: &LoweringResult) {
    for diag in &lowering_result.reveal_types {
        write_stderr_line(diag);
    }
    for warning in &lowering_result.warnings {
        write_stderr_line(warning);
    }
}

#[cfg(test)]
mod tests {
    use super::{lowering_error_to_diagnostic, FrontendDiagnosticStyle};
    use sifr_diagnostics::DiagnosticCode;
    use sifr_hir::LoweringError;

    fn lowering_error(code: Option<DiagnosticCode>, message: &str) -> LoweringError {
        LoweringError {
            code,
            message: message.to_string(),
            primary_range: None,
            line: None,
            col: None,
        }
    }

    #[test]
    fn coded_lowering_error_uses_active_diagnostic_code() {
        let error = lowering_error(Some(DiagnosticCode::TYPE_MISMATCH), "expected int, got str");

        let diagnostic =
            lowering_error_to_diagnostic("main", FrontendDiagnosticStyle::Bare, None, error);

        assert_eq!(diagnostic.code, "SIFR-TYPE-0002");
        assert_eq!(diagnostic.url, "https://sifr.sh/docs/errors/SIFR-TYPE-0002");
    }

    #[test]
    fn codeless_lowering_error_is_internal_compiler_diagnostic() {
        let error = lowering_error(None, "expected int, got str");

        let diagnostic = lowering_error_to_diagnostic(
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
