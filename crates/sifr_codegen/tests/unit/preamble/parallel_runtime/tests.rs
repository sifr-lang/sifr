use super::parallel_runtime_rust_code;

#[test]
fn rayon_items_enter_the_shared_silent_panic_boundary() {
    let runtime = parallel_runtime_rust_code();

    assert_eq!(
        runtime
            .matches("__sifr_panic_boundary.catch_unwind(|| worker(item))")
            .count(),
        4
    );
    assert!(!runtime.contains("sifr_runtime::interop::catch_unwind_silently(|| worker(item))"));
    assert!(
        !runtime.contains(
            "std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| worker(item)))"
        )
    );
}
