use crate::diagnostics::RenderedDiagnostic;
use crate::stdlib::StdlibCompiled;
use std::sync::{Arc, OnceLock};

pub(super) static STDLIB_COMPILED_CACHE: OnceLock<
    Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>>,
> = OnceLock::new();

pub(crate) fn get_or_init_stdlib_cache(
    cache: &OnceLock<Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>>>,
    build: impl FnOnce() -> Result<StdlibCompiled, Vec<RenderedDiagnostic>>,
) -> Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>> {
    cache
        .get_or_init(|| build().map(Arc::new))
        .as_ref()
        .map(Arc::clone)
        .map_err(Clone::clone)
}

pub(crate) fn project_stdlib_cache<T>(
    cache: &OnceLock<Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>>>,
    build: impl FnOnce() -> Result<StdlibCompiled, Vec<RenderedDiagnostic>>,
    project: impl FnOnce(&StdlibCompiled) -> T,
) -> Result<T, Vec<RenderedDiagnostic>> {
    cache
        .get_or_init(|| build().map(Arc::new))
        .as_ref()
        .map(|compiled| project(compiled))
        .map_err(Clone::clone)
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
