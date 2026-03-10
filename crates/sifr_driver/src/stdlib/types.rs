use sifr_codegen::StdlibCode;
use sifr_hir::ExternalDefs;

#[derive(Clone)]
pub(crate) struct StdlibCompiled {
    pub(crate) defs: ExternalDefs,
    pub(crate) code: StdlibCode,
}
