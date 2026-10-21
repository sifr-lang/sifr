//! Complete checked stdlib construction shared by semantic and native consumers.
mod bootstrap;
mod re_exports;
pub use bootstrap::*;
use sifr_stdlib_manifest::LoadedStdlibSource;
use std::{collections::HashSet, sync::Arc};
struct SemanticObserver;
impl<'a> CheckedModuleObserver<'a> for SemanticObserver {
    fn retain_declaration(&mut self, _: &'a LoadedStdlibSource, _: &Arc<sifr_ir::HirModule>) {}
    fn materialize(
        &mut self,
        _: &'a LoadedStdlibSource,
        _: &sifr_ir::HirModule,
        _: &HashSet<String>,
    ) -> Result<(), Vec<sifr_diagnostics::RenderedDiagnostic>> {
        Ok(())
    }
}
pub fn check_sources(
    sources: &[LoadedStdlibSource],
) -> Result<CheckedStdlib, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    check_stdlib_sources(sources, &mut SemanticObserver)
}
