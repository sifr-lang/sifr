use sifr_codegen::StdlibCode;
use sifr_lowering::ExternalDefs;
use sifr_sysroot::ResolvedSysroot;
use std::collections::HashMap;

#[derive(Clone)]
pub struct StdlibCompiled {
    pub defs: ExternalDefs,
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
