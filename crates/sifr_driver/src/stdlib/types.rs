use sifr_codegen::StdlibCode;
use sifr_lowering::ExternalDefs;
use sifr_sysroot::ResolvedSysroot;
use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct StdlibCompiled {
    pub(crate) defs: ExternalDefs,
    pub(crate) code: StdlibCode,
    pub(crate) interop: StdlibRustInterop,
}

#[derive(Clone, Default)]
pub(crate) struct StdlibRustInterop {
    pub(crate) plan: sifr_codegen::InteropBuildPlan,
    pub(crate) module_sources: HashMap<String, StdlibRustInteropModuleSource>,
    pub(crate) sysroot: Option<ResolvedSysroot>,
}

#[derive(Clone)]
pub(crate) struct StdlibRustInteropModuleSource {
    pub(crate) source: String,
    pub(crate) display_path: String,
}
