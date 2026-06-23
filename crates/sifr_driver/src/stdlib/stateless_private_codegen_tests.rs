use super::compile_stdlib_uncached;

#[test]
fn platform_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.platform")
        .expect("_sifr.platform should generate private Rust code");

    assert!(private_code.contains("sifr_stdlib::platform::platform_system()"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.platform")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.platform")
        .is_some_and(|deps| deps.contains("_sifr.platform")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.platform")
        .is_some_and(|names| !names.contains("platform_system")));
}

#[test]
fn html_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.html")
        .expect("_sifr.html should generate private Rust code");

    assert!(private_code.contains("sifr_stdlib::html::html_escape(s)"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.html")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.html")
        .is_some_and(|deps| deps.contains("_sifr.html")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.html")
        .is_some_and(|names| !names.contains("html_escape")));
}
