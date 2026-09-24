mod bootstrap;
mod interop;
mod re_exports;
pub use bootstrap::{
    collect_public_constant_integer_value_exports, compile_stdlib_sources_with_sysroot,
    function_type_from_hir, function_type_from_params, signature_params, stdlib_class_template,
};
use sifr_codegen::StdlibCode;
use sifr_lowering::ExternalDefs;
use sifr_sysroot::ResolvedSysroot;
use std::collections::HashMap;
/// Source bootstrap result. Provider selection is owned by the metadata reader.
pub struct SourceStdlibCompiled {
    pub defs: ExternalDefs,
    pub metadata_features:
        HashMap<String, std::collections::BTreeSet<sifr_stdlib_manifest::StdlibFeature>>,
    pub code: StdlibCode,
    pub interop: StdlibRustInterop,
}
#[derive(Clone, Default)]
pub struct StdlibRustInterop {
    pub plan: sifr_codegen::InteropBuildPlan,
    pub module_sources: HashMap<String, StdlibRustInteropModuleSource>,
    pub sysroot: Option<ResolvedSysroot>,
}
#[derive(Clone)]
pub struct StdlibRustInteropModuleSource {
    pub source: String,
    pub display_path: String,
}

pub mod tooling;

pub fn external_defs(
    compiler: &crate::CompilerContext,
) -> Result<ExternalDefs, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    let provider = compiler.metadata_provider()?;
    let mut defs = ExternalDefs::default();
    defs.provider = Some(provider);
    Ok(defs)
}
