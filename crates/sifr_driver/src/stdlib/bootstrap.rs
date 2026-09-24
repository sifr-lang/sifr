use crate::diagnostics::RenderedDiagnostic;
use crate::stdlib::types::StdlibCompiled;
use sifr_codegen::StdlibCode;
#[cfg(test)]
use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::ExternalDefs;
#[cfg(test)]
use sifr_lowering::lower_module_sysroot_public_stdlib_with_externals;
#[cfg(test)]
use sifr_lowering::{HirFunction, HirParam};
#[cfg(test)]
use sifr_stdlib_manifest::{LoadedStdlibSource, LoadedStdlibSourceKind};
#[cfg(test)]
use sifr_sysroot::ResolvedSysroot;
#[cfg(test)]
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::HashMap;

#[cfg(test)]
use sifr_compiler_services::stdlib::{
    collect_public_constant_integer_value_exports, function_type_from_hir,
    function_type_from_params,
};
pub(crate) use sifr_compiler_services::stdlib::{signature_params, stdlib_class_template};
#[cfg(test)]
use sifr_stdlib_manifest::load_stdlib_tooling_sources_from_sysroot;
#[cfg(test)]
use sifr_syntax::parse_module_raw;
pub(crate) fn compile_stdlib(
    compiler: &crate::CompilerContext,
) -> Result<std::sync::Arc<StdlibCompiled>, Vec<RenderedDiagnostic>> {
    let provider = compiler.metadata_provider()?;
    let mut defs = ExternalDefs::default();
    defs.provider = Some(provider.clone());
    Ok(std::sync::Arc::new(StdlibCompiled {
        defs,
        code: StdlibCode::default(),
        metadata_features: HashMap::new(),
        interop: crate::stdlib::StdlibRustInterop {
            sysroot: Some(compiler.sysroot()?.clone()),
            ..Default::default()
        },
        provider: Some(provider),
    }))
}

pub fn external_defs(
    compiler: &crate::CompilerContext,
) -> Result<ExternalDefs, Vec<RenderedDiagnostic>> {
    Ok(compile_stdlib(compiler)?.defs.clone())
}

#[cfg(test)]
pub(crate) fn compile_stdlib_uncached() -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    compile_stdlib_for_context(&crate::CompilerContext::for_test())
}

#[cfg(test)]
fn compile_stdlib_for_context(
    compiler: &crate::CompilerContext,
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let sysroot = compiler.sysroot()?.clone();
    let sources = load_stdlib_tooling_sources_from_sysroot(&sysroot).map_err(|error| {
        vec![crate::diagnostics::diagnostic_with_code(
            format!("Sifr stdlib source inventory is invalid: {error}"),
            DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
        )]
    })?;
    compile_stdlib_sources_with_sysroot(&sources, sysroot)
}

#[cfg(test)]
pub(crate) fn compile_stdlib_sources_with_sysroot(
    sources: &[LoadedStdlibSource],
    sysroot: ResolvedSysroot,
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    let compiled =
        sifr_compiler_services::stdlib::compile_stdlib_sources_with_sysroot(sources, sysroot)?;
    Ok(StdlibCompiled {
        provider: None,
        defs: compiled.defs,
        metadata_features: compiled.metadata_features,
        code: compiled.code,
        interop: compiled.interop,
    })
}
#[cfg(test)]
#[path = "bootstrap_template_tests.rs"]
mod template_tests;
#[cfg(test)]
#[path = "bootstrap_tests.rs"]
mod tests;
/// Explicit complete inventory view for tests whose assertion enumerates the inventory.
/// Ordinary test compilation uses the same lazy entrypoint as installed consumers.
#[cfg(test)]
pub(crate) fn metadata_inventory_for_test(
    compiler: &crate::CompilerContext,
) -> Result<std::sync::Arc<StdlibCompiled>, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    let provider = compiler.metadata_provider()?;
    let names = provider.modules.keys().cloned().collect::<Vec<_>>();
    provider
        .materialize(&names, compiler.sysroot()?)
        .map(std::sync::Arc::new)
        .map_err(|e| {
            vec![crate::diagnostics::diagnostic_with_code(
                e.to_string(),
                sifr_diagnostics::DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]
        })
}
