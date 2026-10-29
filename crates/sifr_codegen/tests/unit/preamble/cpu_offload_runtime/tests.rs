use super::*;

#[test]
fn worker_panic_boundary_uses_the_shared_runtime_hook() {
    let items = build_worker_panic_hook_items();
    assert_eq!(items.len(), 2);
    assert!(matches!(
        &items[0],
        RustItem::Use(path)
            if path == &["sifr_runtime", "interop", "SilentPanicBoundary"]
    ));
    let Some(RustItem::Fn { body, .. }) = items
        .iter()
        .find(|item| matches!(item, RustItem::Fn { .. }))
    else {
        panic!("worker panic boundary must be a function");
    };
    let rendered = format!("{body:?}");

    assert!(rendered.contains("SilentPanicBoundary::enter"));
    assert!(rendered.contains("__sifr_panic_boundary.catch_unwind"));
    assert!(!rendered.contains("take_hook"));
    assert!(!rendered.contains("set_hook"));
}
