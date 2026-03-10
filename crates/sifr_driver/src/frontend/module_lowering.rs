use crate::diagnostics::{write_stderr_line, CompileError, CompilePhase};
use sifr_hir::{lower_module_with_externals, ExternalDefs, LoweringResult};
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
                .map(|e| CompileError {
                    message: match diagnostic_style {
                        FrontendDiagnosticStyle::Bare => e.message,
                        FrontendDiagnosticStyle::ModulePrefixed => {
                            format!("[{}] {}", module_name, e.message)
                        }
                    },
                    phase: CompilePhase::TypeCheck,
                })
                .collect();
            return Err(compile_errors);
        }
    };
    Ok(result)
}

pub(crate) fn emit_frontend_diagnostics(lowering_result: &LoweringResult) {
    for diag in &lowering_result.reveal_types {
        write_stderr_line(diag);
    }
    for warning in &lowering_result.warnings {
        write_stderr_line(warning);
    }
}
