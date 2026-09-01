use crate::build::{compile_single_file_entrypoint_with_metadata, compile_single_file_frontend};
use crate::diagnostics::{
    CompileResult, CompileResultFull, GeneratedSourceMapFile, RenderedDiagnostic,
};
use crate::stdlib::StdlibCompiled;
use sifr_frontend::{
    FrontendSourceContext, SourceOrigin, reveal_type_diagnostics, warning_diagnostics,
};
use sifr_lowering::LoweringResult;
use sifr_python_ast::Suite;

pub(crate) struct FrontendCompiled {
    pub(crate) stdlib: StdlibCompiled,
    pub(crate) lowering_result: LoweringResult,
}

pub fn parse_source(source: &str) -> Result<Suite, Vec<RenderedDiagnostic>> {
    sifr_frontend::parse_source(source, None)
}

fn compile_frontend(source: &str) -> Result<FrontendCompiled, Vec<RenderedDiagnostic>> {
    compile_single_file_frontend(source)
}

pub fn lower_source(source: &str) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    compile_frontend(source).map(|frontend| frontend.lowering_result)
}

pub fn compile_sql_migration_source(
    source: &str,
) -> Result<Vec<sifr_frontend::MigrationSourceDeclaration>, Vec<RenderedDiagnostic>> {
    let lowered = lower_source(source)?;
    sifr_frontend::sql_migration_declarations(&lowered.module).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            error.message,
            sifr_diagnostics::DiagnosticCode::SQL_PROVIDER_CONTRACT,
        )]
    })
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
        generated_source_map: generated_source_map_files(&codegen_result.rust_source),
        rust_source: codegen_result.rust_source,
        used_stdlib_modules: codegen_result.used_stdlib_modules,
        required_features: codegen_result.required_features,
        interop: codegen_result.interop,
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

fn generated_source_map_files(rust_source: &str) -> Vec<GeneratedSourceMapFile> {
    let mut files = Vec::new();
    if let Some(source) = generated_support_source(rust_source) {
        files.push(GeneratedSourceMapFile {
            path: "src/main.rs#stdlib-preamble".to_string(),
            origin: SourceOrigin::GeneratedSupport,
            source,
        });
    }
    files.push(GeneratedSourceMapFile {
        path: "src/main.rs".to_string(),
        origin: SourceOrigin::CompilerSynthetic,
        source: rust_source.to_string(),
    });
    files
}

fn generated_support_source(rust_source: &str) -> Option<String> {
    let start = rust_source.find("// --- stdlib:")?;
    let tail = &rust_source[start..];
    let end = tail.find("\n// --- end stdlib ---")?;
    let support = tail[..end].trim_end();
    (!support.is_empty()).then(|| format!("{support}\n"))
}
