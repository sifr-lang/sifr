//! Read-only editor compilation and saved-check reuse.
mod saved_checks;
mod sql_application_queries;
pub(crate) mod sql_typed_methods;
pub use sql_application_queries::compile_application_queries;
#[cfg(test)]
mod tests;

use crate::CompilerContext;
pub use saved_checks::{manifestless_inputs, restore_editor_checks, saved_check_policy};
use sifr_frontend::{
    FrontendDiagnosticStyle, FrontendProductInput, SourceOrigin, compile_frontend_product,
};
use sifr_lowering::LoweringOptions;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedSourceMapFile {
    pub path: String,
    pub origin: SourceOrigin,
    pub source: String,
}

pub enum PreviewResult {
    Success {
        rust: String,
        source_map_files: Vec<GeneratedSourceMapFile>,
    },
    Unavailable {
        diagnostic_count: usize,
    },
}

/// Compile one pinned source through the same C01 frontend product as the
/// driver, then project its generated Rust for the editor. No partial source
/// or source map escapes on any failed stage.
pub fn generated_rust_preview(compiler: &CompilerContext, source: &str) -> PreviewResult {
    let compiled = (|| {
        let provider = compiler.metadata_provider()?;
        let sysroot = compiler.sysroot()?;
        let suite = sifr_frontend::parse_source(source, None)?;
        let input = HashMap::from([(
            "main".to_string(),
            FrontendProductInput {
                suite: &suite,
                source,
                display_path: "main",
                source_backed: true,
            },
        )]);
        let mut defs = sifr_lowering::ExternalDefs::default();
        defs.provider = Some(provider.clone());
        let mut product = compile_frontend_product(
            &input,
            defs,
            FrontendDiagnosticStyle::Bare,
            &LoweringOptions::default(),
        )?;
        compile_application_queries(
            &mut product,
            &crate::sql_editor::PreparedSqlProfiles::default(),
        )?;
        let Some(main) = product.hir_modules.get("main") else {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                "frontend product has no main module",
                sifr_diagnostics::DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        let requested = sifr_codegen::stdlib_module_roots(main);
        let stdlib = provider.materialize(&requested, sysroot).map_err(|error| {
            vec![crate::diagnostics::diagnostic_with_code(
                error.to_string(),
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        })?;
        let generated = crate::diagnostics::run_codegen_with_boundary(
            "internal compiler panic during single-file code generation",
            || sifr_codegen::generate_rust_with_stdlib_for_module(main, &stdlib.code, Some("main")),
        )
        .map_err(|error| vec![*error])?;
        // A single-file preview has no application package Cargo owner. The
        // driver rejects direct Rust declarations at metadata resolution;
        // report the same failure before exposing any generated source.
        if !generated.interop.rust.declarations.is_empty() {
            return Err(vec![crate::diagnostics::diagnostic_with_code(
                "Rust interop declarations require a Sifr package Cargo context",
                sifr_diagnostics::DiagnosticCode::RUST_CARGO_METADATA,
            )]);
        }
        Ok::<_, Vec<sifr_diagnostics::RenderedDiagnostic>>(generated.rust_source)
    })();
    match compiled {
        Ok(rust) => PreviewResult::Success {
            source_map_files: generated_source_map_files(&rust),
            rust,
        },
        Err(errors) => PreviewResult::Unavailable {
            diagnostic_count: errors.len(),
        },
    }
}

pub fn generated_source_map_files(rust: &str) -> Vec<GeneratedSourceMapFile> {
    let mut files = Vec::new();
    if let Some(start) = rust.find("// --- stdlib:") {
        if let Some(end) = rust[start..].find("\n// --- end stdlib ---") {
            let source = rust[start..start + end].trim_end();
            if !source.is_empty() {
                files.push(GeneratedSourceMapFile {
                    path: "src/main.rs#stdlib-preamble".into(),
                    origin: SourceOrigin::GeneratedSupport,
                    source: format!("{source}\n"),
                });
            }
        }
    }
    files.push(GeneratedSourceMapFile {
        path: "src/main.rs".into(),
        origin: SourceOrigin::CompilerSynthetic,
        source: rust.into(),
    });
    files
}
