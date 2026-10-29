use super::*;
use ruff_text_size::TextRange;
use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirImport, MethodKind, RustInteropAbiRequirements,
    RustInteropDeclaration, RustInteropDecoratorKind, RustInteropEffect,
};

#[test]
fn local_imports_bring_opaque_extension_traits_into_scope() {
    let declaration = |kind| RustInteropDeclaration {
        kind,
        target: None,
        arguments: Vec::new(),
        span: TextRange::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements::default(),
        consumes_receiver: false,
    };
    let provider = HirModule {
        functions: Vec::new(),
        classes: vec![HirClass {
            name: "Resource".to_string(),
            identity: None,
            fields: Vec::new(),
            field_defaults: Vec::new(),
            field_default_identities: Vec::new(),
            declaration_metadata: Vec::new(),
            methods: vec![HirFunction {
                name: "close".to_string(),
                params: Vec::new(),
                return_type: sifr_type_system::Type::None,
                body: Vec::new(),
                is_async: false,
                method_kind: MethodKind::Regular,
                receiver: None,
                decorators: Vec::new(),
                rust_interop: vec![declaration(RustInteropDecoratorKind::Function)],
                python_interop: Vec::new(),
                compiler_intrinsic: None,
                type_params: Vec::new(),
            }],
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
            rust_interop: vec![declaration(RustInteropDecoratorKind::Opaque)],
        }],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };
    let consumer = HirModule {
        functions: Vec::new(),
        classes: Vec::new(),
        imports: vec![HirImport {
            module: "resources".to_string(),
            names: vec!["Resource".to_string()],
            aliases: Vec::new(),
        }],
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };
    let modules = HashMap::from([("resources", &provider), ("main", &consumer)]);

    let imports = render_local_module_imports(&consumer, &modules, &StdlibCode::default());

    assert!(
        imports.contains("use crate::resources::Resource;"),
        "{imports}"
    );
    assert!(
        imports.contains("use crate::resources::__SifrOpaqueResourceMethods as _;"),
        "{imports}"
    );
}
