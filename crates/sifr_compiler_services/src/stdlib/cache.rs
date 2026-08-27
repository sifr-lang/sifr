use crate::stdlib::StdlibCompiled;
use sifr_diagnostics::RenderedDiagnostic;
use std::sync::OnceLock;

pub(super) static STDLIB_COMPILED_CACHE: OnceLock<Result<StdlibCompiled, Vec<RenderedDiagnostic>>> =
    OnceLock::new();

pub(crate) fn get_or_init_stdlib_cache(
    cache: &OnceLock<Result<StdlibCompiled, Vec<RenderedDiagnostic>>>,
    build: impl FnOnce() -> Result<StdlibCompiled, Vec<RenderedDiagnostic>>,
) -> Result<StdlibCompiled, Vec<RenderedDiagnostic>> {
    cache.get_or_init(build).clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::compile_stdlib_uncached;
    use sifr_diagnostics::DiagnosticCode;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_get_or_init_stdlib_cache_reuses_successful_compilation() {
        let cache: OnceLock<Result<StdlibCompiled, Vec<RenderedDiagnostic>>> = OnceLock::new();
        let build_calls = AtomicUsize::new(0);

        let first = get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            compile_stdlib_uncached()
        })
        .expect("initial stdlib compilation should succeed");
        let second = get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            panic!("stdlib cache should not rebuild on second lookup");
        })
        .expect("cached stdlib compilation should be reused");

        assert_eq!(build_calls.load(Ordering::SeqCst), 1);
        assert_eq!(first.defs.functions.len(), second.defs.functions.len());
        assert_eq!(
            first.code.module_rust_code.len(),
            second.code.module_rust_code.len()
        );
    }

    #[test]
    fn test_get_or_init_stdlib_cache_reuses_error_without_fallback_rebuild() {
        let cache: OnceLock<Result<StdlibCompiled, Vec<RenderedDiagnostic>>> = OnceLock::new();
        let build_calls = AtomicUsize::new(0);

        let first = match get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            Err(vec![crate::diagnostics::diagnostic_with_code(
                "sentinel stdlib cache error",
                DiagnosticCode::STDLIB_CACHE_FAILURE,
            )])
        }) {
            Ok(_) => panic!("sentinel error should be cached"),
            Err(errors) => errors,
        };
        let second = match get_or_init_stdlib_cache(&cache, || {
            build_calls.fetch_add(1, Ordering::SeqCst);
            compile_stdlib_uncached()
        }) {
            Ok(_) => panic!("cached error should be reused"),
            Err(errors) => errors,
        };

        assert_eq!(build_calls.load(Ordering::SeqCst), 1);
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].message, "sentinel stdlib cache error");
    }
}
