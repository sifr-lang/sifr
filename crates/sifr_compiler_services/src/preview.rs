use crate::diagnostics::{GeneratedSourceMapFile, render_codegen_error, run_codegen_with_boundary};
use crate::stdlib::compile_stdlib;
use sifr_diagnostics::RenderedDiagnostic;
use sifr_frontend::{
    FrontendContext, FrontendDiagnosticStyle, FrontendInput, FrontendMode, SourceOrigin,
    SourcePath, SourceText,
};
use sifr_lowering::LoweringOptions;
use sifr_stdlib_manifest::StdlibFeature;
use std::collections::HashSet;

pub struct CompilerPreview {
    pub rust_source: String,
    pub generated_source_map: Vec<GeneratedSourceMapFile>,
    pub used_stdlib_modules: HashSet<String>,
    pub required_features: HashSet<StdlibFeature>,
    pub interop: sifr_codegen::InteropBuildPlan,
    pub lowering_stats: sifr_codegen::LoweringStats,
}

pub fn compile_source_preview(source: &str) -> Result<CompilerPreview, Vec<RenderedDiagnostic>> {
    let stdlib = compile_stdlib()?;
    let mut context = FrontendContext::load_single_file_with_external_defs(
        FrontendInput {
            path: SourcePath::new("main.sifr"),
            source: SourceText::new(source),
            mode: FrontendMode::SingleFile,
        },
        stdlib.defs,
    )?;
    let mut product =
        context.compile_project(FrontendDiagnosticStyle::Bare, &LoweringOptions::default())?;
    let lowering = product.lowering_results.remove("main").ok_or_else(|| {
        vec![crate::diagnostics::diagnostic_with_code(
            "compiler preview is missing the main lowering result",
            sifr_diagnostics::DiagnosticCode::INTERNAL_COMPILER_PANIC,
        )]
    })?;
    let static_programs = lowering.specialization_outputs.clone();
    let mut generated = run_codegen_with_boundary(
        "internal compiler panic during generated Rust preview",
        || {
            sifr_codegen::generate_rust_with_stdlib_for_module(
                &lowering.module,
                &stdlib.code,
                Some("main"),
            )
        },
    )
    .map_err(|error| vec![*error])?
    .map_err(|error| vec![render_codegen_error(&error)])?;
    generated.static_programs = static_programs;
    if generated
        .interop
        .rust
        .structural_identity_algorithm_version
        .is_some()
    {
        generated.static_program_structural_owners =
            sifr_codegen::structural_static_program_owners(&lowering.module);
    }
    let static_source = sifr_codegen::emit_static_specialization_programs(
        &generated.static_programs,
        &generated.static_program_structural_owners,
    )
    .map_err(|error| vec![render_codegen_error(&error)])?;
    prepend_static_specialization(&mut generated.rust_source, &static_source);
    let source_map = generated_source_map_files(&generated.rust_source);
    Ok(CompilerPreview {
        rust_source: generated.rust_source,
        generated_source_map: source_map,
        used_stdlib_modules: generated.used_stdlib_modules,
        required_features: generated.required_features,
        interop: generated.interop,
        lowering_stats: generated.lowering_stats,
    })
}

fn prepend_static_specialization(rust_source: &mut String, static_source: &str) {
    if !static_source.is_empty() {
        *rust_source = format!("{static_source}\n{rust_source}");
    }
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

#[cfg(test)]
mod tests {
    use super::prepend_static_specialization;

    #[test]
    fn preview_static_specialization_is_a_pinned_source_prefix() {
        let mut source = "fn main() {}\n".to_string();

        prepend_static_specialization(&mut source, "static DATA: i64 = 1;");

        assert_eq!(source, "static DATA: i64 = 1;\nfn main() {}\n");
    }
}
