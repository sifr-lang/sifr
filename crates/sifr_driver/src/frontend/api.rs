use crate::build::{compile_single_file_entrypoint_with_metadata, compile_single_file_frontend};
use crate::diagnostics::{CompileError, CompilePhase, CompileResult, CompileResultFull};
use crate::frontend::module_lowering::emit_frontend_diagnostics;
use crate::stdlib::StdlibCompiled;
use sifr_hir::LoweringResult;
use sifr_python_ast::Stmt;
use sifr_python_parser::parse_module;

pub(crate) struct FrontendCompiled {
    pub(crate) stdlib: StdlibCompiled,
    pub(crate) lowering_result: LoweringResult,
}

pub fn parse_source(source: &str) -> Result<Vec<Stmt>, Vec<CompileError>> {
    match parse_module(source) {
        Ok(parsed) => {
            if !parsed.is_valid() {
                let errors: Vec<CompileError> = parsed
                    .errors()
                    .iter()
                    .map(|e| CompileError {
                        message: format!("{e}"),
                        phase: CompilePhase::Parse,
                    })
                    .collect();
                return Err(errors);
            }
            Ok(parsed.into_suite())
        }
        Err(e) => Err(vec![CompileError {
            message: format!("failed to parse: {e}"),
            phase: CompilePhase::Parse,
        }]),
    }
}

fn compile_frontend(source: &str) -> Result<FrontendCompiled, Vec<CompileError>> {
    compile_single_file_frontend(source)
}

pub fn lower_source(source: &str) -> Result<LoweringResult, Vec<CompileError>> {
    compile_frontend(source).map(|frontend| frontend.lowering_result)
}

pub fn type_check_source(source: &str) -> Vec<CompileError> {
    match lower_source(source) {
        Ok(lowering_result) => {
            emit_frontend_diagnostics(&lowering_result);
            vec![]
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
        required_crates: codegen_result.required_crates,
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

pub fn check(source: &str) -> Vec<CompileError> {
    type_check_source(source)
}
