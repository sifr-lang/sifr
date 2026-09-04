use crate::{replace_parallel_runtime_items, replace_sync_channel_runtime_items};

#[test]
fn sync_runtime_replacement_keeps_only_the_demanded_public_operation_path() {
    let rendered = replace_sync_channel_runtime_items("fn channel<T: Clone>() {}\n");

    assert_eq!(rendered.matches("fn channel<").count(), 1);
    assert!(!rendered.contains("fn bounded_channel<"));
}

#[test]
fn parallel_runtime_replacement_keeps_only_the_demanded_public_operation_path() {
    let rendered = replace_parallel_runtime_items(
        "fn map() { __sifr_parallel_map(Vec::<i64>::new(), |value| value); }\n",
        &std::collections::HashSet::new(),
    );

    assert_eq!(rendered.matches("fn map").count(), 1);
    assert!(!rendered.contains("fn try_map"));
    assert!(rendered.contains("fn __sifr_parallel_map"));
    assert!(!rendered.contains("fn __sifr_parallel_try_map"));
    assert!(!rendered.contains("struct Pool"));
}

#[test]
fn parallel_runtime_replacement_accepts_typed_external_demand() {
    let demanded = std::collections::HashSet::from(["__sifr_pool_map".to_string()]);
    let rendered = replace_parallel_runtime_items("fn map() {}\n", &demanded);

    assert!(rendered.contains("fn __sifr_pool_map"));
    assert!(!rendered.contains("fn __sifr_pool_try_map"));
    assert!(rendered.contains("struct Pool"));
}
