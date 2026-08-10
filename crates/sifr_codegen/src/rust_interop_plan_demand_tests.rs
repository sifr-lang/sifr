use super::{interop_build_plan_for_named_modules, rust_interop_plan_tests::class};
use sifr_ir::{HirClassKind, HirModule};
use sifr_type_system::Type;

#[test]
fn ordinary_modules_skip_structural_identity_collection() {
    let module = HirModule {
        functions: Vec::new(),
        classes: vec![class(
            "Payload",
            HirClassKind::Regular,
            vec![("value".to_string(), Type::Int)],
        )],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

    assert_eq!(plan.rust.structural_identity_algorithm_version, None);
    assert!(plan.rust.structural_shape_identities.is_empty());
    assert!(plan
        .cache_key_fragment()
        .contains("rust.structural_shape_identities=0"));
}
