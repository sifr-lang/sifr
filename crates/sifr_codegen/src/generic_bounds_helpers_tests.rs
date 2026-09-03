use super::*;
use crate::{RustParam, RustType, Visibility};
use sifr_ir::{HirClassKind, HirExpr, HirModule, HirParam, MethodKind};
use sifr_type_system::ParamConvention;

#[test]
fn generic_type_rendering_preserves_compiler_owned_class_names() {
    let emitter = RustEmitter::new();
    for (identity, expected) in [
        ("sifr.io.FileHandle", "__SifrIoFileHandle"),
        ("sifr.io.TextFileHandle", "__SifrIoTextFileHandle"),
    ] {
        let ty = Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: identity.rsplit('.').next().expect("class name").to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        };
        assert_eq!(emitter.render_rust_type_with_generics(&ty), expected);
    }
}

#[test]
fn canonical_generic_class_rendering_preserves_concrete_arguments() {
    let mut emitter = RustEmitter::new();
    emitter.generic_classes.insert("NullContext".to_string());
    let ty = Type::Class {
        identity: Some("sifr.resource.NullContext".to_string()),
        type_args: vec![Type::Int],
        name: "NullContext".to_string(),
        fields: vec![("value".to_string(), Type::Int)],
        methods: Vec::new(),
        parent_class: None,
    };
    let canonical = sifr_type_system::stdlib_class_rust_name("sifr.resource", "NullContext");

    assert_eq!(
        emitter.render_rust_type_with_generics(&ty),
        format!("{canonical}<SifrInt>")
    );
}

#[test]
fn nested_generic_classes_keep_emitter_owned_arguments() {
    let mut emitter = RustEmitter::new();
    emitter.generic_classes.insert("Counter".to_string());
    emitter
        .generic_class_params
        .insert("Counter".to_string(), vec!["T".to_string()]);
    let counter = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Counter".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    };

    assert_eq!(
        emitter.render_rust_type_with_generics(&Type::List(Box::new(counter))),
        "Vec<Counter<T>>"
    );
    assert_eq!(
        emitter.render_rust_type_with_generics(&Type::Result(
            Box::new(Type::Int),
            Box::new(Type::Never),
        )),
        "Result<SifrInt, ::std::convert::Infallible>"
    );
}

#[test]
fn same_basename_class_method_inherits_only_its_module_function_bounds() {
    let type_var = Type::TypeVar("T".to_string());
    let display = HirFunction {
        name: "display".to_string(),
        params: vec![HirParam {
            name: "value".to_string(),
            ty: type_var.clone(),
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::None,
        body: vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "print".to_string(),
                args: vec![HirExpr::Name {
                    name: "value".to_string(),
                    binding_id: None,
                    ty: type_var.clone(),
                }],
                mutable_arg_places: Vec::new(),
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec!["T".to_string()],
    };
    let class_type = Type::TypeVar("Item".to_string());
    let method = HirFunction {
        name: "display".to_string(),
        params: Vec::new(),
        return_type: Type::None,
        body: vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "display".to_string(),
                args: vec![HirExpr::Name {
                    name: "value".to_string(),
                    binding_id: None,
                    ty: class_type,
                }],
                mutable_arg_places: Vec::new(),
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let class = HirClass {
        name: "Boxed".to_string(),
        identity: None,
        fields: Vec::new(),
        field_defaults: Vec::new(),
        field_default_identities: Vec::new(),
        declaration_metadata: Vec::new(),
        methods: vec![method.clone()],
        is_hashable: false,
        is_error_type: false,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: None,
        parent_type: None,
        type_params: vec!["Item".to_string()],
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    };
    let module = HirModule {
        functions: vec![display.clone()],
        classes: vec![class.clone()],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };
    let mut emitter = RustEmitter::new();
    emitter.function_type_param_bounds = RustEmitter::closed_function_type_param_bounds(&module);
    emitter
        .module_generic_functions
        .insert(display.name.clone(), display);
    let item = RustItem::Fn {
        name: "display".to_string(),
        visibility: Visibility::Private,
        type_params: Vec::new(),
        params: Vec::<RustParam>::new(),
        ret: Some(RustType::Unit),
        body: Vec::new(),
        is_async: false,
    };

    let bounds = emitter.class_method_type_param_bounds(&class, &[(&method, item)]);

    assert_eq!(
        bounds["display"]["Item"],
        HashSet::from(["std::fmt::Display".to_string()])
    );
}
