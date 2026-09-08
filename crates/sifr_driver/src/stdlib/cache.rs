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
    let _b50_cache = sifr_ir::b50_phase_diagnostic::span(sifr_ir::b50_phase_diagnostic::Phase::Cache);
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
    let _b50_cache = sifr_ir::b50_phase_diagnostic::span(sifr_ir::b50_phase_diagnostic::Phase::Cache);
    cache
        .get_or_init(|| build().map(Arc::new))
        .as_ref()
        .map(|compiled| project(compiled))
        .map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::compile_stdlib_uncached;
    use sifr_diagnostics::DiagnosticCode;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn stdlib_cache_shared_bundle_outlives_cache_and_keeps_definitions_isolated() {
        let retained = {
            let cache = OnceLock::new();
            let first = get_or_init_stdlib_cache(&cache, compile_stdlib_uncached)
                .expect("complete bootstrap");
            let second = get_or_init_stdlib_cache(&cache, || panic!("no rebuild"))
                .expect("shared bootstrap");
            assert!(Arc::ptr_eq(&first, &second));
            let mut local = project_stdlib_cache(
                &cache,
                || panic!("no rebuild"),
                |compiled| compiled.defs.clone(),
            )
            .unwrap();
            local.functions.remove("sifr.calendar");
            assert!(first.defs.functions.contains_key("sifr.calendar"));
            assert!(second.defs.functions.contains_key("sifr.calendar"));
            first
        };
        // The compilation handle owns the whole checked bundle after the cache dies.
        let source = "from sifr.calendar import isleap\ndef main():\n    assert isleap(2024)\n";
        let parsed = sifr_syntax::parse_module_raw(source, None).unwrap();
        let lowered =
            sifr_lowering::lower_module_with_externals(parsed.suite(), &retained.defs).unwrap();
        let generated = sifr_codegen::generate_rust_with_stdlib(&lowered.module, &retained.code);
        assert!(
            generated
                .interop
                .stdlib_demand
                .declarations
                .iter()
                .any(|declaration| declaration.module_name.as_deref() == Some("_sifr.calendar"),)
        );
        assert!(
            retained
                .interop
                .module_sources
                .contains_key("_sifr.calendar")
        );
        assert!(retained.interop.sysroot.is_some());
    }

    #[test]
    fn stdlib_interop_startup_defs_projection_preserves_cached_errors_and_isolation() {
        let cache = OnceLock::new();
        let calls = AtomicUsize::new(0);
        let defs = project_stdlib_cache(
            &cache,
            || {
                calls.fetch_add(1, Ordering::SeqCst);
                compile_stdlib_uncached()
            },
            |compiled| compiled.defs.clone(),
        )
        .expect("definitions initialize the same cache");
        assert!(defs.functions.contains_key("sifr.calendar"));
        let original = cache.get().unwrap().as_ref().unwrap();
        let inventory = std::sync::Arc::clone(&original.code.hir_modules);
        let sysroot = original.interop.sysroot.as_ref().unwrap().clone();
        let a = "from sifr.calendar import isleap\ndef main():\n    assert isleap(2024)\n";
        let b = "from sifr.python import from_bool\ndef main():\n    _ = from_bool(True)\n";
        let mut plans = std::collections::HashMap::new();
        for source in [a, b, b, a] {
            let definitions =
                project_stdlib_cache(&cache, || panic!("no rebuild"), |c| c.defs.clone()).unwrap();
            let parsed = sifr_syntax::parse_module_raw(source, None).unwrap();
            let lowered =
                sifr_lowering::lower_module_with_externals(parsed.suite(), &definitions).unwrap();
            let compiled =
                get_or_init_stdlib_cache(&cache, || panic!("projection must preserve full cache"))
                    .unwrap();
            assert!(std::sync::Arc::ptr_eq(
                &inventory,
                &compiled.code.hir_modules
            ));
            assert_eq!(compiled.interop.sysroot.as_ref().unwrap(), &sysroot);
            let selected = sifr_codegen::generate_rust_with_stdlib(&lowered.module, &compiled.code)
                .interop
                .stdlib_demand;
            assert!(
                selected
                    .declarations
                    .iter()
                    .any(|d| d.module_name.as_deref()
                        == Some(if source == a {
                            "_sifr.calendar"
                        } else {
                            "_sifr.python"
                        }))
            );
            if source == a {
                assert!(
                    selected
                        .declarations
                        .iter()
                        .all(|d| d.module_name.as_deref() != Some("_sifr.python"))
                );
            }
            if let Some(previous) = plans.insert(source, selected.clone()) {
                assert_eq!(selected, previous);
            }
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let failed = OnceLock::new();
        let first = project_stdlib_cache(
            &failed,
            || {
                Err(vec![crate::diagnostics::diagnostic_with_code(
                    "sentinel projection failure",
                    DiagnosticCode::STDLIB_CACHE_FAILURE,
                )])
            },
            |c| c.defs.clone(),
        )
        .err()
        .unwrap();
        let second = get_or_init_stdlib_cache(&failed, || panic!("cached failure cannot rebuild"))
            .err()
            .unwrap();
        let third = project_stdlib_cache(
            &failed,
            || panic!("same cached failure"),
            |c| c.defs.clone(),
        )
        .err()
        .unwrap();
        assert_eq!(first, second);
        assert_eq!(second, third);
    }

    #[test]
    fn test_get_or_init_stdlib_cache_reuses_successful_compilation() {
        let cache: OnceLock<Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>>> = OnceLock::new();
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
        let cache: OnceLock<Result<Arc<StdlibCompiled>, Vec<RenderedDiagnostic>>> = OnceLock::new();
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
