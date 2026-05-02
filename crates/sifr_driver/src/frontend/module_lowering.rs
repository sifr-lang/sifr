use crate::diagnostics::{write_stderr_line, RenderedDiagnostic};
use sifr_diagnostics::DiagnosticCode;
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

pub(crate) fn lower_frontend_module(
    module_name: &str,
    stmts: &[Stmt],
    external_defs: &ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    let result = match lower_module_with_externals(stmts, external_defs) {
        Ok(result) => result,
        Err(errors) => {
            let diagnostics: Vec<RenderedDiagnostic> = errors
                .into_iter()
                .map(|error| lowering_error_to_diagnostic(module_name, diagnostic_style, error))
                .collect();
            return Err(diagnostics);
        }
    };
    Ok(result)
}

fn lowering_error_to_diagnostic(
    module_name: &str,
    diagnostic_style: FrontendDiagnosticStyle,
    error: LoweringError,
) -> RenderedDiagnostic {
    let code = lowering_error_code_or_internal(&error);
    let uncoded = error.code.is_none();
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
    crate::diagnostics::diagnostic_with_code(message, code)
}

pub(crate) fn lowering_error_code_or_internal(error: &LoweringError) -> DiagnosticCode {
    error
        .code
        .unwrap_or(DiagnosticCode::INTERNAL_COMPILER_PANIC)
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

        let diagnostic = lowering_error_to_diagnostic("main", FrontendDiagnosticStyle::Bare, error);

        assert_eq!(diagnostic.code, "SIFR-TYPE-0002");
        assert_eq!(diagnostic.url, "https://sifr.sh/docs/errors/SIFR-TYPE-0002");
    }

    #[test]
    fn codeless_lowering_error_is_internal_compiler_diagnostic() {
        let error = lowering_error(None, "expected int, got str");

        let diagnostic =
            lowering_error_to_diagnostic("main", FrontendDiagnosticStyle::ModulePrefixed, error);

        assert_eq!(
            diagnostic.message,
            "internal compiler error: HIR lowering emitted a diagnostic without canonical code: [main] expected int, got str"
        );
        assert_eq!(diagnostic.code, "SIFR-INTERNAL-0001");
    }
}
