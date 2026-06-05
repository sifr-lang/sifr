use crate::build::{compile_single_file_entrypoint_with_metadata, compile_single_file_frontend};
use crate::diagnostics::{CompileResult, CompileResultFull, RenderedDiagnostic};
use crate::stdlib::StdlibCompiled;
use sifr_frontend::{reveal_type_diagnostics, warning_diagnostics, FrontendSourceContext};
use sifr_hir::LoweringResult;
use sifr_python_ast::Stmt;

pub(crate) struct FrontendCompiled {
    pub(crate) stdlib: StdlibCompiled,
    pub(crate) lowering_result: LoweringResult,
}

pub fn parse_source(source: &str) -> Result<Vec<Stmt>, Vec<RenderedDiagnostic>> {
    sifr_frontend::parse_source(source, None)
}

fn compile_frontend(source: &str) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
    compile_single_file_frontend(source)
}

pub fn lower_source(source: &str) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    compile_frontend(source).map(|frontend| frontend.lowering_result)
}

pub fn type_check_source(source: &str) -> Vec<RenderedDiagnostic> {
    match lower_source(source) {
        Ok(lowering_result) => {
            let source_context = FrontendSourceContext {
                display_path: "main",
                source,
            };
            let mut diagnostics =
                warning_diagnostics(Some(source_context), &lowering_result.warnings);
            diagnostics.extend(reveal_type_diagnostics(
                Some(source_context),
                &lowering_result.reveal_types,
            ));
            diagnostics
        }
        Err(errors) => errors,
    }
}

pub fn compile_with_metadata(source: &str) -> CompileResultFull {
    let codegen_result = match compile_single_file_entrypoint_with_metadata(source) {
        Ok(result) => result,
        Err(errors) => return CompileResultFull::Errors { errors },
    };

    CompileResultFull::Success {
        rust_source: codegen_result.rust_source,
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_features: codegen_result.required_features,
        lowering_stats: codegen_result.lowering_stats,
    }
}

pub fn compile(source: &str) -> CompileResult {
    let result = compile_with_metadata(source);
    match result {
        CompileResultFull::Success { rust_source, .. } => CompileResult::Success { rust_source },
        CompileResultFull::Errors { errors } => CompileResult::Errors { errors },
    }
}

pub fn check(source: &str) -> Vec<RenderedDiagnostic> {
    type_check_source(source)
}
