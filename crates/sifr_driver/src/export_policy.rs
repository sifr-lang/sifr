pub(crate) fn should_export_callable(module_name: &str, callable_name: &str) -> bool {
    if module_name == "sifr.math"
        && matches!(callable_name, "dist_impl" | "fsum_impl" | "sumprod_impl")
    {
        return false;
    }
    !callable_name.starts_with('_')
        || matches!(
            (module_name, callable_name),
            (
                "sifr.heapq",
                "_heapify_max" | "_heappop_max" | "_heapreplace_max"
            )
        )
}
