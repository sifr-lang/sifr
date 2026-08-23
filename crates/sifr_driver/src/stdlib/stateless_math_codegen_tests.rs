use super::compile_stdlib_uncached;

#[test]
fn math_private_declarations_codegen_through_sifr_stdlib() {
    let compiled = compile_stdlib_uncached().expect("stdlib should compile");
    let private_code = compiled
        .code
        .module_rust_code
        .get("_sifr.math")
        .expect("_sifr.math should generate private Rust code");

    assert!(private_code.rust.contains("::sifr_stdlib::math::sqrt(x)"));
    assert!(
        private_code
            .rust
            .contains("::sifr_stdlib::math::pow_val(x, y)")
    );
    assert!(private_code.rust.contains("::sifr_stdlib::math::floor(x)"));
    assert!(private_code.rust.contains("::sifr_stdlib::math::frexp(x)"));
    assert!(
        private_code
            .rust
            .contains("const PI: f64 = 3.141592653589793_f64;")
    );
    assert!(
        private_code
            .rust
            .contains("const E: f64 = 2.718281828459045_f64;")
    );
    assert!(
        private_code
            .rust
            .contains("const TAU: f64 = 6.283185307179586_f64;")
    );
    assert!(
        private_code
            .rust
            .contains("const INF: f64 = f64::INFINITY;")
    );
    assert!(private_code.rust.contains("const NAN: f64 = f64::NAN;"));
    assert!(!private_code.rust.contains("std::f64::consts::PI"));
    assert!(
        compiled
            .code
            .transitive_deps
            .get("sifr.math")
            .is_some_and(|deps| deps.contains("_sifr.math"))
    );
    let private_constant_mappings = compiled
        .code
        .module_constants
        .get("_sifr.math")
        .expect("_sifr.math should expose compiled constant mappings");
    let public_constant_mappings = compiled
        .code
        .module_constants
        .get("sifr.math")
        .expect("sifr.math should re-export compiled constant mappings");
    for name in ["pi", "e", "tau", "inf", "nan"] {
        assert!(
            private_constant_mappings.contains_key(name),
            "_sifr.math should map {name}"
        );
        assert!(
            public_constant_mappings.contains_key(name),
            "sifr.math should map {name}"
        );
    }
    let public_constants = compiled
        .defs
        .constants
        .get("sifr.math")
        .expect("sifr.math should export public constants");
    assert!(public_constants.contains_key("pi"));
    assert!(public_constants.contains_key("tau"));
    assert!(public_constants.contains_key("inf"));
    assert!(public_constants.contains_key("nan"));
    let public_functions = compiled
        .defs
        .functions
        .get("sifr.math")
        .expect("sifr.math should export public functions");
    for name in ["dist", "fsum", "sumprod"] {
        let function = public_functions
            .get(name)
            .unwrap_or_else(|| panic!("sifr.math should export {name}"));
        assert!(
            function.params.iter().all(
                |(_, _, convention)| *convention == sifr_type_system::ParamConvention::borrow()
            ),
            "{name} should keep read-only public list parameters"
        );
    }
    for name in ["dist_impl", "fsum_impl", "sumprod_impl"] {
        assert!(
            !public_functions.contains_key(name),
            "{name} should stay an internal aggregate bridge helper"
        );
    }
}
