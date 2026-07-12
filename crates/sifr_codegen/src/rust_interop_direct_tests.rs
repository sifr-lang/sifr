use crate::rust_interop_direct::{rust_interop_function_body, rust_interop_method_body};
use crate::{generate_rust_with_metadata, render_expr, RustStmt};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirModule, HirParam, MethodKind,
    RustInteropAbiRequirements, RustInteropArgument, RustInteropDeclaration,
    RustInteropDecoratorKind, RustInteropEffect, RustInteropValue, RustTargetPath,
};
use sifr_type_system::{FixedIntType, ParamConvention, Type};
use std::collections::HashMap;

#[test]
fn rust_interop_function_body_emits_package_bridge_root() {
    let func = HirFunction {
        name: "digest".to_string(),
        params: vec![HirParam {
            name: "data".to_string(),
            ty: Type::Bytes,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Bytes,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["bridge", "hash", "digest"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        rust_interop_function_body(&func).expect("bridge interop metadata should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("bridge interop should lower to a return expression");
    };

    assert_eq!(render_expr(expr), "bridge::hash::digest(data)");
}

#[test]
fn rust_interop_method_body_emits_self_handle_call() {
    let func = HirFunction {
        name: "encode".to_string(),
        params: vec![HirParam {
            name: "text".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::List(Box::new(Type::FixedInt(FixedIntType::U32))),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["Self", "encode"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        rust_interop_method_body(&func).expect("Self interop metadata should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("Self interop should lower to a return expression");
    };

    assert_eq!(render_expr(expr), "self._handle.encode(text)");
}

#[test]
fn emitted_direct_result_none_interop_does_not_append_ok_tail() {
    let error_ty = Type::Class {
        name: "ZipError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let module = HirModule {
        functions: vec![HirFunction {
            name: "zip_close".to_string(),
            params: vec![HirParam {
                name: "path".to_string(),
                ty: Type::Str,
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: Type::Result(Box::new(Type::None), Box::new(error_ty)),
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                &["sifr_stdlib", "zip", "zip_close"],
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        classes: vec![zip_error_class()],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module).rust_source;

    assert!(generated.contains("sifr_stdlib::zip::zip_close(path)"));
    assert!(!generated.contains("return Ok(());"), "{generated}");
}

#[test]
fn emitted_opaque_class_is_a_sealed_runtime_handle_alias() {
    let mut object = zip_error_class();
    object.name = "Object".to_string();
    object.fields.clear();
    object.is_error_type = false;
    object.parent_class = Some("NonSend".to_string());
    object.rust_interop = vec![RustInteropDeclaration {
        kind: RustInteropDecoratorKind::Opaque,
        target: None,
        arguments: vec![RustInteropArgument {
            name: Some("type".to_string()),
            value: RustInteropValue::TargetPath(RustTargetPath {
                segments: ["sifr_runtime", "python", "ForeignObject"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                span: TextRange::default(),
            }),
            span: TextRange::default(),
        }],
        span: TextRange::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements {
            opaque_handle: true,
            ..RustInteropAbiRequirements::default()
        },
    }];
    let module = HirModule {
        functions: Vec::new(),
        classes: vec![object],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };

    let generated = generate_rust_with_metadata(&module).rust_source;

    assert!(generated.contains(
        "type Object = sifr_runtime::interop::Handle<sifr_runtime::python::ForeignObject>;"
    ));
    assert!(!generated.contains("struct Object"));
}

#[test]
fn rust_interop_function_body_maps_python_error_fields_without_parent_metadata() {
    let python_error = Type::Class {
        name: "PythonError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("kind".to_string(), Type::Str),
            ("exception_type".to_string(), Type::Str),
            ("traceback".to_string(), Type::Str),
            ("context".to_string(), Type::Str),
        ],
        methods: Vec::new(),
        parent_class: None,
    };
    let func = HirFunction {
        name: "py_from_none".to_string(),
        params: Vec::new(),
        return_type: Type::Result(
            Box::new(Type::Tuple(vec![Type::Int, Type::Int])),
            Box::new(python_error),
        ),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "python", "py_from_none"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        rust_interop_function_body(&func).expect("Python primitive interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("Python primitive interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "sifr_stdlib::python::py_from_none().map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(|__sifr_bridge_error| PythonError { message: __sifr_bridge_error.message.to_string(), kind: __sifr_bridge_error.kind.to_string(), exception_type: __sifr_bridge_error.exception_type.to_string(), traceback: __sifr_bridge_error.traceback.to_string(), context: __sifr_bridge_error.context.to_string(), __sifr_python_error: Some(__sifr_bridge_error) })"
    );
}

#[test]
fn rust_interop_function_body_adapts_sealed_python_object_callback_parameter() {
    let python_error = Type::Class {
        name: "PythonError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("kind".to_string(), Type::Str),
            ("exception_type".to_string(), Type::Str),
            ("traceback".to_string(), Type::Str),
            ("context".to_string(), Type::Str),
        ],
        methods: Vec::new(),
        parent_class: None,
    };
    let object = Type::Class {
        name: "Object".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };
    let func = HirFunction {
        name: "py_local_callback".to_string(),
        params: vec![HirParam {
            name: "handler".to_string(),
            ty: Type::Callable(
                vec![object.clone()],
                vec![ParamConvention::borrow()],
                Box::new(Type::Result(
                    Box::new(object),
                    Box::new(python_error.clone()),
                )),
            ),
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Result(
            Box::new(Type::Tuple(vec![
                Type::Int,
                Type::Int,
                Type::Int,
                Type::Int,
                Type::Str,
            ])),
            Box::new(python_error),
        ),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "python", "py_local_callback"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        rust_interop_function_body(&func).expect("Python callback interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("Python callback interop should lower to a return expression");
    };
    let rendered = render_expr(expr);

    assert!(rendered.contains("sifr_stdlib::python::py_local_callback(move |__sifr_callback_arg|"));
    assert!(rendered.contains("handler(&__sifr_callback_arg)"));
    assert!(rendered.contains("Ok(__sifr_callback_result)"));
    assert!(!rendered.contains("__sifr_callback_arg.0"));
    assert!(rendered.contains("sifr_stdlib::python::PythonError"));
    assert!(rendered.contains("PythonError { message: __sifr_bridge_error.message.to_string()"));
}

#[test]
fn rust_interop_function_body_converts_python_int_dict_return() {
    let python_error = Type::Class {
        name: "PythonError".to_string(),
        fields: vec![
            ("message".to_string(), Type::Str),
            ("kind".to_string(), Type::Str),
            ("exception_type".to_string(), Type::Str),
            ("traceback".to_string(), Type::Str),
            ("context".to_string(), Type::Str),
        ],
        methods: Vec::new(),
        parent_class: None,
    };
    let func = HirFunction {
        name: "py_copy_dict_str_int".to_string(),
        params: vec![
            HirParam {
                name: "handle".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            },
            HirParam {
                name: "token".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            },
        ],
        return_type: Type::Result(
            Box::new(Type::Dict(Box::new(Type::Str), Box::new(Type::Int))),
            Box::new(python_error),
        ),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "python", "py_copy_dict_str_int"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        rust_interop_function_body(&func).expect("Python dict copy interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("Python dict copy interop should lower to a return expression");
    };
    let rendered = render_expr(expr);

    assert!(rendered.contains("py_copy_dict_str_int"));
    assert!(rendered.contains("__sifr_bridge_value.to_i64_saturating()"));
    assert!(rendered.contains("collect::<HashMap<_, _>>()"));
}

fn zip_error_class() -> HirClass {
    HirClass {
        name: "ZipError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        is_hashable: false,
        is_error_type: true,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: Some("Error".to_string()),
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    }
}

fn declaration(kind: RustInteropDecoratorKind, segments: &[&str]) -> RustInteropDeclaration {
    RustInteropDeclaration {
        kind,
        target: Some(RustTargetPath {
            segments: segments
                .iter()
                .map(|segment| (*segment).to_string())
                .collect(),
            span: TextRange::default(),
        }),
        arguments: Vec::new(),
        span: TextRange::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements::default(),
    }
}
