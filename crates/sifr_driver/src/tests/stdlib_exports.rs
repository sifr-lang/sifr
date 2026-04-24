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

#[test]
fn stdlib_dsu_exports_union_find_class() {
    let compiled = compile_stdlib().expect("stdlib should compile");
    let dsu_classes = compiled
        .defs
        .classes
        .get("sifr.dsu")
        .expect("sifr.dsu exports should exist");

    assert!(
        dsu_classes.contains_key("UnionFind"),
        "expected sifr.dsu export 'UnionFind' to be visible for stdlib imports"
    );
}

#[test]
fn stdlib_trie_exports_trie_class() {
    let compiled = compile_stdlib().expect("stdlib should compile");
    let trie_classes = compiled
        .defs
        .classes
        .get("sifr.trie")
        .expect("sifr.trie exports should exist");

    assert!(
        trie_classes.contains_key("Trie"),
        "expected sifr.trie export 'Trie' to be visible for stdlib imports"
    );
}
