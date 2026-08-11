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
        receiver: None,
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
fn rust_interop_function_body_emits_owned_threadsafe_callback_policy() {
    let callback_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "CallbackError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let func = HirFunction {
        name: "subscribe".to_string(),
        params: vec![HirParam {
            name: "handler".to_string(),
            ty: Type::Callable(
                vec![Type::Str],
                vec![ParamConvention::borrow()],
                Box::new(Type::Result(Box::new(Type::None), Box::new(callback_error))),
            ),
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![
            declaration(
                RustInteropDecoratorKind::Function,
                &["bridge", "events", "subscribe"],
            ),
            callback_policy_declaration(),
        ],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = rust_interop_function_body(&func).expect("thread-safe callback body");
    let [RustStmt::Expr(expr)] = body.as_slice() else {
        panic!("thread-safe callback should lower to a direct expression");
    };
    let rendered = render_expr(expr);

    assert!(rendered.contains("ThreadsafeCallbackBridge::new"));
    assert!(rendered.contains("CallbackBackpressure::Bounded(2usize)"));
    assert!(rendered.contains("CallbackOverflow::Error"));
    assert!(rendered.contains("CallbackShutdown::Drain"));
    assert!(rendered.contains("handler(&__sifr_callback_arg_0)"));
    assert!(rendered.contains("Err(__sifr_callback_error) => Err("));
}

#[test]
fn rust_interop_method_callback_parameter_is_send_sync_static() {
    let method = HirFunction {
        name: "subscribe".to_string(),
        params: vec![HirParam {
            name: "handler".to_string(),
            ty: Type::Callable(
                vec![Type::Str],
                vec![ParamConvention::borrow()],
                Box::new(Type::None),
            ),
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::None,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![
            declaration(
                RustInteropDecoratorKind::Function,
                &["bridge", "events", "subscribe"],
            ),
            callback_policy_declaration(),
        ],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let class = HirClass {
        name: "Registrar".to_string(),
        identity: None,
        fields: Vec::new(),
        field_defaults: Vec::new(),
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
        type_params: Vec::new(),
        enum_variants: Vec::new(),
        rust_interop: Vec::new(),
    };
    let emitter = crate::RustEmitter::new();
    let rendered = crate::render_type(&emitter.lower_class_method_param_type(
        &class,
        &method,
        "handler",
        &method.params[0].ty,
        method.params[0].convention,
    ));

    assert!(rendered.contains("+ Send + Sync + 'static"), "{rendered}");
}

#[test]
fn rust_interop_method_body_emits_self_handle_call() {
    let error_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "EncodeError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let func = HirFunction {
        name: "encode".to_string(),
        params: vec![HirParam {
            name: "text".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Result(
            Box::new(Type::List(Box::new(Type::FixedInt(FixedIntType::U32)))),
            Box::new(error_ty),
        ),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["Self", "encode"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = rust_interop_method_body(&func, false)
        .expect("Self interop metadata should lower to a body");
    let [RustStmt::Let { value, .. }, RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("Self interop should bind the checked handle before its return expression");
    };

    let checked_receiver = render_expr(value);
    let rendered = render_expr(expr);
    assert!(
        checked_receiver.contains("self.inner_ref().map_err"),
        "{checked_receiver}"
    );
    assert!(
        rendered.contains("__sifr_opaque_inner.encode(text)"),
        "{rendered}"
    );
    assert!(!rendered.contains("self._handle"), "{rendered}");
}

#[test]
fn non_opaque_rust_bound_method_does_not_inject_receiver_argument() {
    let func = HirFunction {
        name: "encode".to_string(),
        params: vec![HirParam {
            name: "text".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Str,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["bridge", "encode"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = rust_interop_method_body(&func, false)
        .expect("ordinary Rust-bound method should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("ordinary Rust-bound method should lower to a return expression");
    };

    assert_eq!(render_expr(expr), "bridge::encode(text)");
}

#[test]
fn emitted_non_opaque_owned_rust_method_preserves_owned_receiver_shape() {
    let source = r"
class BridgeError(Error):
    message: str

class Plain:
    @rust(bridge.consume, panic=trusted_no_panic)
    def consume(own self) -> Result[str, BridgeError]:
        ...
";
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let generated = generate_rust_with_metadata(&lowered.module).rust_source;

    assert!(
        generated.contains("fn consume(self) -> Result<String, BridgeError>"),
        "{generated}"
    );
    assert!(generated.contains("bridge::consume()"), "{generated}");
    assert!(!generated.contains("bridge::consume(self)"), "{generated}");
}

#[test]
fn emitted_opaque_non_close_methods_forward_borrowed_handle_receivers() {
    let source = r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource, close=close)
class Resource:
    @rust(bridge.resources.ping, panic=trusted_no_panic)
    def ping(self) -> Result[str, ResourceError]:
        ...

    @rust(bridge.resources.close, panic=trusted_no_panic)
    def close(own self) -> Result[None, ResourceError]:
        ...
";
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let generated = generate_rust_with_metadata(&lowered.module).rust_source;

    assert!(
        generated.contains("fn ping(&self) -> Result<String, ResourceError>;"),
        "{generated}"
    );
    assert!(
        generated.contains("bridge::resources::ping(self)"),
        "{generated}"
    );
}

#[test]
fn emitted_opaque_self_method_checks_handle_state_without_panicking() {
    let source = r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource)
class Resource:
    @rust(Self.ping, panic=trusted_no_panic)
    def ping(self) -> Result[str, ResourceError | RustPanicError]:
        ...
";
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let generated = generate_rust_with_metadata(&lowered.module).rust_source;

    assert!(
        generated.contains("self.inner_ref().map_err"),
        "{generated}"
    );
    assert!(
        generated.contains("HandleStateError::Closed"),
        "{generated}"
    );
    assert!(
        generated.contains("HandleStateError::Poisoned"),
        "{generated}"
    );
    assert!(!generated.contains("self._handle"), "{generated}");
}

#[test]
fn emitted_opaque_self_method_maps_poison_to_plain_declared_error() {
    let source = r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource)
class Resource:
    @rust(Self.ping, panic=trusted_no_panic)
    def ping(self) -> Result[str, ResourceError]:
        ...
";
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let generated = generate_rust_with_metadata(&lowered.module).rust_source;

    assert!(
        generated.contains("HandleStateError::Poisoned(__sifr_stored_panic)"),
        "{generated}"
    );
    assert!(
        generated.contains("message: __sifr_stored_panic.to_string()"),
        "{generated}"
    );
    assert!(!generated.contains("__SifrRustPanicError"), "{generated}");
}

#[test]
fn emitted_sync_opaque_self_method_checks_state_outside_panic_boundary() {
    let source = r"
class ResourceError(Error):
    message: str

@rust.opaque(type=bridge.resources.Resource)
class Resource:
    @rust(Self.ping)
    def ping(self) -> Result[str, ResourceError | RustPanicError]:
        ...
";
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let generated = generate_rust_with_metadata(&lowered.module).rust_source;

    assert!(
        generated
            .contains("let __sifr_opaque_inner = self.inner_ref().map_err(|__sifr_handle_error|"),
        "{generated}"
    );
    assert!(
        generated.contains("catch_rust_panic(|| __sifr_opaque_inner.ping())"),
        "{generated}"
    );
    assert!(
        !generated.contains("catch_rust_panic(|| self.inner_ref()"),
        "{generated}"
    );
}

#[test]
fn emitted_opaque_async_close_method_routes_owned_handle_to_bridge() {
    let source = r"
class ResourceError(Error):
    message: str

@rust.opaque(
    type=bridge.resources.Resource,
    close=async_close,
    borrow=exclusive,
    thread_affinity=tokio_current_thread,
)
class Resource:
    @rust(bridge.resources.aclose, panic=trusted_no_panic)
    async def aclose(own self) -> Result[None, ResourceError]:
        ...
";
    let parsed = sifr_python_parser::parse_module(source).expect("source should parse");
    let lowered = sifr_lowering::lower_module(parsed.suite()).expect("source should lower");
    let generated = generate_rust_with_metadata(&lowered.module).rust_source;

    assert!(
        generated.contains(
            "trait __SifrOpaqueResourceMethods {\n    async fn aclose(self) -> Result<(), ResourceError>;"
        ),
        "{generated}"
    );
    assert!(
        !generated.contains("pub trait __SifrOpaqueResourceMethods"),
        "{generated}"
    );
    assert!(
        generated.contains(
            "bridge::resources::aclose(self).await.map(|__sifr_bridge_ok| __sifr_bridge_ok)"
        ),
        "{generated}"
    );
}

#[test]
fn emitted_direct_result_none_interop_does_not_append_ok_tail() {
    let error_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
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
            receiver: None,
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

    assert!(generated.contains("::sifr_stdlib::zip::zip_close(path)"));
    assert!(!generated.contains("return Ok(());"), "{generated}");
}

#[test]
fn emitted_python_object_class_does_not_claim_a_source_spellable_rust_name() {
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
        consumes_receiver: false,
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

    assert!(!generated.contains("type __SifrPythonObject"));
    assert!(!generated.contains("type Object"));
    assert!(!generated.contains("struct Object"));
}

#[test]
fn rust_interop_function_body_maps_python_error_fields_without_parent_metadata() {
    let python_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
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
        receiver: None,
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
        "::sifr_stdlib::python::py_from_none().map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(|__sifr_bridge_error| PythonError { message: __sifr_bridge_error.message.to_string(), kind: __sifr_bridge_error.kind.to_string(), exception_type: __sifr_bridge_error.exception_type.to_string(), traceback: __sifr_bridge_error.traceback.to_string(), context: __sifr_bridge_error.context.to_string(), __sifr_python_error: Some(__sifr_bridge_error) })"
    );
}

#[test]
fn rust_interop_function_body_adapts_sealed_python_object_callback_parameter() {
    let python_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
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
        identity: Some("_sifr.python.Object".to_string()),
        type_args: Vec::new(),
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
        receiver: None,
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

    assert!(
        rendered.contains("::sifr_stdlib::python::py_local_callback(move |__sifr_callback_arg|")
    );
    assert!(rendered.contains("handler(&__sifr_callback_arg)"));
    assert!(rendered.contains("Ok(__sifr_callback_result)"));
    assert!(!rendered.contains("__sifr_callback_arg.0"));
    assert!(rendered.contains("::sifr_stdlib::python::PythonError"));
    assert!(rendered.contains("PythonError { message: __sifr_bridge_error.message.to_string()"));
}

#[test]
fn rust_interop_function_body_converts_python_int_dict_return() {
    let python_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
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
        receiver: None,
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
    assert!(rendered.contains("collect::<::std::collections::HashMap<_, _>>()"));
}

fn zip_error_class() -> HirClass {
    HirClass {
        name: "ZipError".to_string(),
        identity: None,
        fields: vec![("message".to_string(), Type::Str)],
        field_defaults: Vec::new(),
        declaration_metadata: Vec::new(),
        methods: Vec::new(),
        is_hashable: false,
        is_error_type: true,
        kind: HirClassKind::Regular,
        operator_impls: Vec::new(),
        newtype_inner: None,
        implements_protocols: Vec::new(),
        parent_class: Some("Error".to_string()),
        parent_type: None,
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
        consumes_receiver: false,
    }
}

fn callback_policy_declaration() -> RustInteropDeclaration {
    RustInteropDeclaration {
        kind: RustInteropDecoratorKind::Callback,
        target: None,
        arguments: vec![
            RustInteropArgument {
                name: Some("backpressure".to_string()),
                value: RustInteropValue::PolicyCall {
                    name: "bounded".to_string(),
                    argument: Box::new(RustInteropValue::Integer(2)),
                    span: TextRange::default(),
                },
                span: TextRange::default(),
            },
            RustInteropArgument {
                name: Some("overflow".to_string()),
                value: RustInteropValue::Symbol("error".to_string()),
                span: TextRange::default(),
            },
            RustInteropArgument {
                name: Some("shutdown".to_string()),
                value: RustInteropValue::Symbol("drain".to_string()),
                span: TextRange::default(),
            },
        ],
        span: TextRange::default(),
        effect: RustInteropEffect::Sync,
        abi_requirements: RustInteropAbiRequirements::default(),
        consumes_receiver: false,
    }
}
