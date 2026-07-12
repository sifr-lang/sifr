use super::*;
use crate::rust_interop_bridge_contract::RustBridgeTypeKind;
use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirParam, MethodKind, RustInteropAbiRequirements,
    RustInteropArgument, RustInteropDecoratorKind, RustInteropEffect, RustInteropValue,
    RustTargetPath,
};
use sifr_type_system::{ParamConvention, Type};

#[test]
fn interop_build_plan_collects_function_class_and_method_declarations() {
    let module = HirModule {
        functions: vec![function_with_declaration(
            "hash",
            RustInteropDecoratorKind::Function,
            "bridge.hash.digest",
        )],
        classes: vec![HirClass {
            name: "Consumer".to_string(),
            fields: Vec::new(),
            methods: vec![function_with_declaration(
                "poll",
                RustInteropDecoratorKind::Function,
                "Self.poll",
            )],
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::Regular,
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: None,
            type_params: Vec::new(),
            enum_variants: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Opaque,
                "bridge.kafka.Consumer",
            )],
        }],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);

    assert_eq!(plan.rust.declarations.len(), 3);
    assert!(plan.rust.declarations.iter().any(|entry| {
        entry.module_name.as_deref() == Some("main")
            && matches!(entry.owner, RustInteropOwner::Function { ref name } if name == "hash")
    }));
    assert!(plan.rust.declarations.iter().any(|entry| {
        matches!(entry.owner, RustInteropOwner::Class { ref name } if name == "Consumer")
            && entry.declaration.kind == RustInteropDecoratorKind::Opaque
    }));
    assert!(plan.rust.declarations.iter().any(|entry| {
        matches!(
            entry.owner,
            RustInteropOwner::Method {
                ref class_name,
                ref name,
            } if class_name == "Consumer" && name == "poll"
        )
    }));
    let cache_key_fragment = plan.cache_key_fragment();
    assert!(cache_key_fragment.contains("owner=function:hash"));
    assert!(cache_key_fragment.contains("kind=opaque"));
    assert!(cache_key_fragment.contains("target=bridge.kafka.Consumer@0..0"));
}

#[test]
fn interop_build_plan_cache_key_records_integer_list_arguments() {
    let mut function = function_with_declaration(
        "tensor",
        RustInteropDecoratorKind::View,
        "bridge.tensor.view",
    );
    function.rust_interop[0]
        .arguments
        .push(RustInteropArgument {
            name: Some("shape".to_string()),
            value: RustInteropValue::IntegerList(vec![2, 3]),
            span: Default::default(),
        });
    let module = module_with(vec![function], Vec::new());

    let cache_key_fragment =
        interop_build_plan_for_named_modules([(Some("main"), &module)]).cache_key_fragment();

    assert!(cache_key_fragment.contains(";arg=shape:int-list:2,3@0..0"));
}

#[test]
fn interop_build_plan_records_bridge_signature_and_generated_types() {
    let token_ty = Type::Class {
        name: "Token".to_string(),
        fields: vec![("text".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: None,
    };
    let error_ty = Type::Class {
        name: "HashError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "hash".to_string(),
            params: vec![HirParam {
                name: "input".to_string(),
                ty: Type::Bytes,
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: Type::Result(Box::new(token_ty), Box::new(error_ty)),
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                "bridge.hash.digest",
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);
    let signature = &plan.rust.bridge_contracts.signatures[0];

    assert_eq!(
        signature.params[0].ty.rust_borrowed_type.as_deref(),
        Some("&[u8]")
    );
    assert_eq!(
        signature.return_type.rust_return_type.as_deref(),
        Some("Result<crate::__sifr_bridge::main::TokenBridge, crate::__sifr_bridge::main::HashErrorBridge>")
    );
    let bridge_type_names = plan
        .rust
        .bridge_contracts
        .generated_types
        .iter()
        .map(|bridge_type| bridge_type.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(bridge_type_names, vec!["HashErrorBridge", "TokenBridge"]);
    assert!(plan
        .cache_key_fragment()
        .contains("rust.generated_bridge_types=2"));
}

#[test]
fn interop_build_plan_accepts_tuple_result_with_error_class_flag() {
    let python_error_ty = Type::Class {
        name: "PythonError".to_string(),
        fields: python_error_fields(),
        methods: Vec::new(),
        parent_class: None,
    };
    let mut python_error_class = class("PythonError", HirClassKind::Regular, python_error_fields());
    python_error_class.is_error_type = true;
    let module = HirModule {
        functions: vec![HirFunction {
            name: "py_from_none".to_string(),
            params: Vec::new(),
            return_type: Type::Result(
                Box::new(Type::Tuple(vec![Type::Int, Type::Int])),
                Box::new(python_error_ty),
            ),
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                "sifr_stdlib.python.py_from_none",
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        classes: vec![python_error_class],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    };

    let plan = interop_build_plan_for_named_modules([(Some("_sifr.python"), &module)]);
    let signature = &plan.rust.bridge_contracts.signatures[0];

    assert_eq!(signature.return_type.kind, RustBridgeTypeKind::Result);
    assert_eq!(
        signature.return_type.rust_return_type.as_deref(),
        Some("Result<(i64, i64), crate::__sifr_bridge::_sifr_python::PythonErrorBridge>")
    );
    let python_error_bridge = plan
        .rust
        .bridge_contracts
        .generated_types
        .iter()
        .find(|bridge_type| bridge_type.name == "PythonErrorBridge")
        .expect("PythonError should generate an error bridge type");
    assert_eq!(
        python_error_bridge.kind,
        crate::rust_interop_bridge_contract::RustGeneratedBridgeTypeKind::Error
    );
}

#[test]
fn interop_bridge_resolves_imported_opaque_type_to_declared_rust_target() {
    let object_ty = Type::Class {
        name: "Object".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };
    let mut object_class = class("Object", HirClassKind::Regular, Vec::new());
    object_class.parent_class = Some("NonSend".to_string());
    object_class.rust_interop = vec![opaque_declaration("sifr_runtime.python.ForeignObject")];
    let declarations = module_with(Vec::new(), vec![object_class]);
    let consumer = module_with(
        vec![HirFunction {
            name: "identity".to_string(),
            params: vec![HirParam {
                name: "value".to_string(),
                ty: object_ty.clone(),
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: object_ty,
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                "bridge.python.identity",
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        Vec::new(),
    );

    let plan = interop_build_plan_for_named_modules([
        (Some("sifr.python_core"), &declarations),
        (Some("sifr.python"), &consumer),
    ]);
    let signature = &plan.rust.bridge_contracts.signatures[0];

    assert_eq!(
        signature.params[0].ty.kind,
        RustBridgeTypeKind::OpaqueHandle
    );
    assert_eq!(
        signature.params[0].ty.rust_borrowed_type.as_deref(),
        Some("&sifr_runtime::interop::Handle<sifr_runtime::python::ForeignObject>")
    );
    assert_eq!(
        signature.return_type.rust_return_type.as_deref(),
        Some("sifr_runtime::interop::Handle<sifr_runtime::python::ForeignObject>")
    );
    assert!(plan.rust.bridge_contracts.generated_types.is_empty());
}

#[test]
fn interop_bridge_callable_params_require_callback_contract() {
    let mut function = HirFunction {
        name: "subscribe".to_string(),
        params: vec![HirParam {
            name: "callback".to_string(),
            ty: Type::Callable(
                vec![Type::Int],
                vec![ParamConvention::borrow()],
                Box::new(Type::None),
            ),
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            "bridge.events.subscribe",
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let module_without_callback = module_with(vec![function.clone()], Vec::new());
    let plan_without_callback =
        interop_build_plan_for_named_modules([(Some("main"), &module_without_callback)]);

    assert_eq!(
        plan_without_callback.rust.bridge_contracts.signatures[0].params[0]
            .ty
            .kind,
        RustBridgeTypeKind::Unsupported
    );

    function.rust_interop.push(callback_declaration());
    let module_with_callback = module_with(vec![function], Vec::new());
    let plan_with_callback =
        interop_build_plan_for_named_modules([(Some("main"), &module_with_callback)]);
    let signature = &plan_with_callback.rust.bridge_contracts.signatures[0];

    assert_eq!(plan_with_callback.rust.bridge_contracts.signatures.len(), 1);
    assert_eq!(signature.params[0].ty.kind, RustBridgeTypeKind::Callback);
    assert_eq!(
        signature.params[0].ty.rust_borrowed_type.as_deref(),
        Some("&sifr_runtime::interop::ThreadsafeCallbackBridge")
    );
}

#[test]
fn interop_bridge_generated_field_paths_use_declaring_module() {
    let token_ty = Type::Class {
        name: "Token".to_string(),
        fields: vec![("text".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: None,
    };
    let wrapper_ty = Type::Class {
        name: "Wrapper".to_string(),
        fields: vec![("token".to_string(), token_ty.clone())],
        methods: Vec::new(),
        parent_class: None,
    };
    let models = module_with(
        Vec::new(),
        vec![class("Token", HirClassKind::Regular, Vec::new())],
    );
    let api = module_with(
        vec![HirFunction {
            name: "wrap".to_string(),
            params: Vec::new(),
            return_type: wrapper_ty,
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                "bridge.tokens.wrap",
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        vec![class("Wrapper", HirClassKind::Regular, Vec::new())],
    );

    let plan =
        interop_build_plan_for_named_modules([(Some("models"), &models), (Some("api"), &api)]);
    let wrapper = plan
        .rust
        .bridge_contracts
        .generated_types
        .iter()
        .find(|bridge_type| bridge_type.name == "WrapperBridge")
        .expect("wrapper bridge type is recorded");
    assert_eq!(wrapper.module_name.as_deref(), Some("api"));
    assert_eq!(
        wrapper.fields[0].rust_type,
        "crate::__sifr_bridge::models::TokenBridge"
    );
    assert!(plan
        .rust
        .bridge_contracts
        .generated_types
        .iter()
        .any(|bridge_type| {
            bridge_type.module_name.as_deref() == Some("models")
                && bridge_type.name == "TokenBridge"
        }));
}

#[test]
fn interop_bridge_rejects_enum_discriminants_outside_repr_u32() {
    let bad_enum = Type::Enum {
        name: "Status".to_string(),
        variants: vec![("Broken".to_string(), Some(-1))],
    };
    let module = module_with(
        vec![HirFunction {
            name: "status".to_string(),
            params: Vec::new(),
            return_type: bad_enum,
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                "bridge.status.current",
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        vec![class("Status", HirClassKind::Enum, Vec::new())],
    );

    let plan = interop_build_plan_for_named_modules([(Some("main"), &module)]);
    let return_type = &plan.rust.bridge_contracts.signatures[0].return_type;
    assert_eq!(return_type.kind, RustBridgeTypeKind::Unsupported);
    assert!(matches!(
        return_type.unsupported_reason.as_deref(),
        Some(reason) if reason.contains("repr(u32) range")
    ));
    assert!(plan.rust.bridge_contracts.generated_types.is_empty());
}

fn function_with_declaration(
    name: &str,
    kind: RustInteropDecoratorKind,
    target: &str,
) -> HirFunction {
    HirFunction {
        name: name.to_string(),
        params: Vec::new(),
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(kind, target)],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}

fn class(name: &str, kind: HirClassKind, fields: Vec<(String, Type)>) -> HirClass {
    HirClass {
        name: name.to_string(),
        fields,
        methods: Vec::new(),
        is_hashable: false,
        is_error_type: false,
        kind,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: None,
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    }
}

fn python_error_fields() -> Vec<(String, Type)> {
    vec![
        ("message".to_string(), Type::Str),
        ("kind".to_string(), Type::Str),
        ("exception_type".to_string(), Type::Str),
        ("traceback".to_string(), Type::Str),
        ("context".to_string(), Type::Str),
    ]
}

fn module_with(functions: Vec<HirFunction>, classes: Vec<HirClass>) -> HirModule {
    HirModule {
        functions,
        classes,
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: std::collections::HashMap::new(),
        type_param_bounds: std::collections::HashMap::new(),
    }
}

fn declaration(kind: RustInteropDecoratorKind, target: &str) -> RustInteropDeclaration {
    RustInteropDeclaration {
        kind,
        target: Some(RustTargetPath {
            segments: target.split('.').map(str::to_string).collect(),
            span: Default::default(),
        }),
        arguments: Vec::new(),
        span: Default::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements::default(),
    }
}

fn opaque_declaration(target: &str) -> RustInteropDeclaration {
    RustInteropDeclaration {
        kind: RustInteropDecoratorKind::Opaque,
        target: None,
        arguments: vec![RustInteropArgument {
            name: Some("type".to_string()),
            value: RustInteropValue::TargetPath(RustTargetPath {
                segments: target.split('.').map(str::to_string).collect(),
                span: Default::default(),
            }),
            span: Default::default(),
        }],
        span: Default::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements {
            opaque_handle: true,
            ..RustInteropAbiRequirements::default()
        },
    }
}

fn callback_declaration() -> RustInteropDeclaration {
    RustInteropDeclaration {
        kind: RustInteropDecoratorKind::Callback,
        target: None,
        arguments: Vec::new(),
        span: Default::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements::default(),
    }
}
