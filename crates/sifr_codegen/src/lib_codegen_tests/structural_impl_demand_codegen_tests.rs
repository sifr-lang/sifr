use crate::{generate_rust, generate_rust_multi};
use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirModule, MethodKind, RustInteropAbiRequirements,
    RustInteropDeclaration, RustInteropDecoratorKind, RustInteropEffect,
};
use sifr_type_system::Type;

#[test]
fn ordinary_class_codegen_skips_structural_impls() {
    let module = module(Vec::new(), vec![payload_class()]);

    let generated = generate_rust(&module);

    assert!(!generated.contains("StructuralType"));
    assert!(!generated.contains("StructuralConstruct"));
    assert!(!generated.contains("StructuralProject"));
}

#[test]
fn project_structural_demand_enables_implicit_classes_across_modules() {
    let models = module(Vec::new(), vec![payload_class()]);
    let api = module(vec![structural_function()], Vec::new());

    let generated = generate_rust_multi(&[("models", &models), ("api", &api)]);
    let models_rust = generated.get("models").expect("models module is generated");

    assert!(models_rust.contains("StructuralType"));
    assert!(models_rust.contains("StructuralConstruct"));
    assert!(models_rust.contains("StructuralProject"));
}

fn module(functions: Vec<HirFunction>, classes: Vec<HirClass>) -> HirModule {
    HirModule {
        functions,
        classes,
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    }
}

fn payload_class() -> HirClass {
    HirClass {
        name: "Payload".to_string(),
        identity: None,
        fields: vec![("value".to_string(), Type::Int)],
        field_defaults: Vec::new(),
        declaration_metadata: Vec::new(),
        methods: Vec::new(),
        is_hashable: false,
        is_error_type: false,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: None,
        parent_type: None,
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    }
}

fn structural_function() -> HirFunction {
    HirFunction {
        name: "construct".to_string(),
        params: Vec::new(),
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![RustInteropDeclaration {
            kind: RustInteropDecoratorKind::Structural,
            target: None,
            arguments: Vec::new(),
            span: Default::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
            consumes_receiver: false,
        }],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}
