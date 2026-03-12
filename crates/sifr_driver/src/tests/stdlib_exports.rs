use crate::compile_stdlib;

#[test]
fn stdlib_heapq_exports_allowlisted_private_max_heap_helpers() {
    let compiled = compile_stdlib().expect("stdlib should compile");
    let heapq_functions = compiled
        .defs
        .functions
        .get("sifr.heapq")
        .expect("sifr.heapq exports should exist");

    for name in ["_heapify_max", "_heappop_max", "_heapreplace_max"] {
        assert!(
            heapq_functions.contains_key(name),
            "expected sifr.heapq export '{name}' to be visible for compat imports"
        );
    }
}
