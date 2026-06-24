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

#[test]
fn calendar_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.calendar")
        .expect("_sifr.calendar should generate private Rust code");

    assert!(private_code.contains(
        "sifr_stdlib::calendar::calendar_isleap(sifr_runtime::interop::SifrIntBridge::from(year))"
    ));
    assert!(private_code.contains(
        "sifr_stdlib::calendar::calendar_monthrange(sifr_runtime::interop::SifrIntBridge::from(year), sifr_runtime::interop::SifrIntBridge::from(month)).into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating()).collect()"
    ));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.calendar")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.calendar")
        .is_some_and(|deps| deps.contains("_sifr.calendar")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.calendar")
        .is_some_and(|names| !names.contains("calendar_isleap")));
}

#[test]
fn uuid_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.uuid")
        .expect("_sifr.uuid should generate private Rust code");

    assert!(private_code.contains("sifr_stdlib::uuid::uuid4()"));
    assert!(private_code.contains("sifr_stdlib::uuid::uuid3_text(namespace, name)"));
    assert!(private_code.contains("sifr_stdlib::uuid::uuid5_text(namespace, name)"));
    assert!(compiled
        .code
        .intrinsic_names
        .get("_sifr.uuid")
        .is_some_and(std::collections::HashSet::is_empty));
    assert!(compiled
        .code
        .transitive_deps
        .get("sifr.uuid")
        .is_some_and(|deps| deps.contains("_sifr.uuid")));
    assert!(compiled
        .code
        .intrinsic_names
        .get("sifr.uuid")
        .is_some_and(|names| !names.contains("uuid4")));
}
