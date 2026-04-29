use crate::diagnostics::{write_stderr_line, CompileError, CompilePhase};
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
) -> Result<LoweringResult, Vec<CompileError>> {
    let result = match lower_module_with_externals(stmts, external_defs) {
        Ok(result) => result,
        Err(errors) => {
            let compile_errors: Vec<CompileError> = errors
                .into_iter()
                .map(|error| lowering_error_to_compile_error(module_name, diagnostic_style, error))
                .collect();
            return Err(compile_errors);
        }
    };
    Ok(result)
}

fn lowering_error_to_compile_error(
    module_name: &str,
    diagnostic_style: FrontendDiagnosticStyle,
    error: LoweringError,
) -> CompileError {
    let message = match diagnostic_style {
        FrontendDiagnosticStyle::Bare => error.message,
        FrontendDiagnosticStyle::ModulePrefixed => {
            format!("[{}] {}", module_name, error.message)
        }
    };
    if let Some(code) = error.code {
        CompileError::with_code(message, CompilePhase::TypeCheck, code)
    } else {
        CompileError::new(message, CompilePhase::TypeCheck)
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
    use super::{lowering_error_to_compile_error, FrontendDiagnosticStyle};
    use sifr_diagnostics::DiagnosticCode;
    use sifr_hir::LoweringError;

    fn lowering_error(code: Option<DiagnosticCode>, message: &str) -> LoweringError {
        LoweringError {
            code,
            message: message.to_string(),
            line: None,
            col: None,
        }
    }

    #[test]
    fn coded_lowering_error_uses_active_diagnostic_code() {
        let error = lowering_error(Some(DiagnosticCode::TYPE_MISMATCH), "expected int, got str");

        let compile_error =
            lowering_error_to_compile_error("main", FrontendDiagnosticStyle::Bare, error);
        let diagnostic = compile_error.to_diagnostic();

        assert_eq!(compile_error.code, Some(DiagnosticCode::TYPE_MISMATCH));
        assert_eq!(diagnostic.code, "SIFR-TYPE-0002");
        assert_eq!(diagnostic.url, "https://sifr.sh/docs/errors/SIFR-TYPE-0002");
    }

    #[test]
    fn codeless_lowering_error_preserves_legacy_bridge() {
        let error = lowering_error(None, "expected int, got str");

        let compile_error =
            lowering_error_to_compile_error("main", FrontendDiagnosticStyle::ModulePrefixed, error);
        let diagnostic = compile_error.to_diagnostic();

        assert_eq!(compile_error.code, None);
        assert_eq!(compile_error.message, "[main] expected int, got str");
        assert_eq!(diagnostic.code, "SIFR-TYPE-0001");
    }
}
