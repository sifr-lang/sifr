use super::compile_stdlib_uncached;

#[test]
fn collections_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.collections")
        .expect("_sifr.collections should generate private Rust code");

    for name in [
        "new_set",
        "set_from_list",
        "set_add",
        "set_contains",
        "set_remove",
        "set_len",
        "set_union",
        "set_intersection",
        "defaultdict_new",
        "defaultdict_get",
        "defaultdict_set",
    ] {
        assert!(
            private_code.contains(&format!("sifr_stdlib::collections::{name}(")),
            "{name} should lower through _sifr.collections private Rust interop declarations"
        );
    }
    assert!(private_code.contains("sifr_runtime::interop::SifrIntBridge::from(item)"));
    assert!(private_code.contains("sifr_runtime::interop::SifrIntBridge::from(value)"));
    assert!(private_code.contains(".to_i64_saturating()"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.collections")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.collections")
        .is_some_and(|deps| deps.contains("_sifr.collections")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.collections")
        .is_some_and(|names| {
            [
                "new_set",
                "set_from_list",
                "set_add",
                "set_contains",
                "set_remove",
                "set_len",
                "set_union",
                "set_intersection",
                "defaultdict_new",
                "defaultdict_get",
                "defaultdict_set",
            ]
            .into_iter()
            .all(|name| !names.contains(name))
        }));
    let exports = compiled
        .defs
        .functions
        .get("sifr.collections")
        .expect("sifr.collections exports should be collected");
    for name in [
        "new_set",
        "set_from_list",
        "set_add",
        "set_contains",
        "set_remove",
        "set_len",
        "set_union",
        "set_intersection",
        "defaultdict_new",
        "defaultdict_get",
        "defaultdict_set",
    ] {
        assert!(
            exports.contains_key(name),
            "sifr.collections should export public {name} wrapper"
        );
        assert!(
            !exports.contains_key(&format!("_{name}_impl")),
            "_{name}_impl should stay a private _sifr.collections bridge helper"
        );
        assert!(
            !exports.contains_key(&format!("{name}_impl")),
            "{name}_impl should not leak as a public collections helper"
        );
    }
}
