use crate::python_interop_direct::{
    python_interop_function_body, python_interop_method_body,
    python_interop_method_body_with_retained_errors,
};
use crate::{generate_rust, render_stmts};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirExpr, HirFunction, HirModule, HirParam, HirStmt, MethodKind, PythonCallbackConcurrency,
    PythonCallbackDeclaration, PythonCallbackDispatch, PythonCallbackLifetime, PythonCleanupPolicy,
    PythonInteropDeclaration, PythonInteropDecoratorKind, PythonInteropEffect,
    PythonInteropParameter, PythonParameterKind, PythonTargetPath,
};
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashMap;

#[test]
fn typed_async_function_emits_owned_frame_schema_cancellation_and_await() {
    let optional_string = Type::Union(vec![Type::Str, Type::None]);
    let function = function(
        "collect",
        vec![
            param("value", Type::Int, ParamConvention::own()),
            param(
                "rest",
                Type::List(Box::new(Type::Int)),
                ParamConvention::borrow(),
            ),
            param("label", optional_string, ParamConvention::borrow()),
            param(
                "extra",
                Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                ParamConvention::borrow(),
            ),
        ],
        Type::Dict(
            Box::new(Type::Str),
            Box::new(Type::Tuple(vec![Type::Int, Type::Str])),
        ),
        declaration(
            vec!["pkg", "collect"],
            vec![
                shape("value", PythonParameterKind::Positional, false),
                shape("rest", PythonParameterKind::PositionalVariadic, false),
                shape("label", PythonParameterKind::KeywordOnly, true),
                shape("extra", PythonParameterKind::KeywordVariadic, false),
            ],
        ),
    );

    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default())
            .expect("typed async wrapper should lower"),
    );
    assert!(rendered.contains("async_from_int"), "{rendered}");
    assert!(rendered.contains("for __sifr_python_value_1 in rest.iter()"));
    assert!(rendered.contains("if let Some(__sifr_python_value_2) = label"));
    assert!(rendered.contains("for (__sifr_python_key_3, __sifr_python_value_3) in extra.iter()"));
    assert!(rendered.contains("PythonAsyncRequest::function"));
    assert!(rendered.contains("PythonAsyncType::Dict"));
    assert!(rendered.contains("PythonAsyncType::Tuple"));
    assert!(rendered.contains("__sifr_current_task_cancellation()"));
    assert!(rendered.contains("submit_async_declaration"));
    assert!(rendered.contains(".await"));
    assert!(rendered.contains("async_dict_items"));
    assert!(rendered.contains("async_tuple_items"));
    assert!(!rendered.contains("resolve_target"));
    assert!(!rendered.contains("call_object_owned"));
}

#[test]
fn local_object_record_uses_async_record_conversion_not_sealed_handle() {
    let object = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Object".to_string(),
        fields: vec![("value".to_string(), Type::Int)],
        methods: Vec::new(),
        parent_class: None,
    };
    let function = function(
        "echo",
        vec![param("value", object.clone(), ParamConvention::borrow())],
        object,
        declaration(
            vec!["builtins", "id"],
            vec![shape("value", PythonParameterKind::Positional, false)],
        ),
    );

    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default())
            .expect("local Object record should lower through record conversion"),
    );
    assert!(rendered.contains("async_from_record_results"), "{rendered}");
    assert!(rendered.contains("PythonAsyncType::Record"), "{rendered}");
    assert!(rendered.contains("async_record_field"), "{rendered}");
    assert!(!rendered.contains("async_from_object"), "{rendered}");
    assert!(!rendered.contains("async_to_object"), "{rendered}");
}

#[test]
fn recursive_factory_emits_loop_thread_schema_and_owned_opaque_result() {
    let payload = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Payload".to_string(),
        fields: vec![
            ("name".to_string(), Type::Str),
            ("scores".to_string(), Type::List(Box::new(Type::Int))),
        ],
        methods: Vec::new(),
        parent_class: None,
    };
    let client = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Client".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };
    let mut opaque = HashMap::new();
    opaque.insert("Client".to_string(), opaque_declaration());
    let function = function(
        "make_client",
        vec![param("payload", payload, ParamConvention::borrow())],
        client,
        declaration(
            vec!["pkg", "make_client"],
            vec![shape("payload", PythonParameterKind::Positional, false)],
        ),
    );

    let rendered = render_stmts(
        &python_interop_function_body(&function, &opaque).expect("factory should lower"),
    );
    assert!(rendered.contains("async_from_record_results"), "{rendered}");
    assert!(rendered.contains("async_from_list_results"), "{rendered}");
    assert!(rendered.contains("PythonAsyncType::Opaque"), "{rendered}");
    assert!(rendered.contains("\"pkg\".to_string()"), "{rendered}");
    assert!(rendered.contains("\"Client\".to_string()"), "{rendered}");
    assert!(rendered.contains("async_to_object"), "{rendered}");
    assert!(rendered.contains("Client::__sifr_from_python_object("));
}

#[test]
fn borrowed_and_consuming_async_methods_select_distinct_identity_transfers() {
    let borrowed = method_function(false);
    let consuming = method_function(true);

    let borrowed = render_stmts(
        &python_interop_method_body(&borrowed, &Default::default(), None)
            .expect("borrowed method should lower"),
    );
    assert!(borrowed.contains("PythonAsyncRequest::borrowed_method"));
    assert!(borrowed.contains("&self.__sifr_python_object"));

    let consuming = render_stmts(
        &python_interop_method_body(&consuming, &Default::default(), None)
            .expect("consuming method should lower"),
    );
    assert!(consuming.contains("PythonAsyncRequest::owned_method"));
    assert!(consuming.contains("self.__sifr_python_object"));
    assert!(!consuming.contains("&self.__sifr_python_object"));
}

#[test]
fn async_close_selects_identity_finalization_only_for_complete_class_contract() {
    let mut close_declaration = declaration(vec!["Self", "aclose"], Vec::new());
    close_declaration.consumes_receiver = true;
    let close = function("aclose", Vec::new(), Type::None, close_declaration);
    let mut owner = opaque_declaration();
    owner.cleanup = Some(PythonCleanupPolicy::AsyncClose);

    let rendered = render_stmts(
        &python_interop_method_body(&close, &Default::default(), Some(&owner))
            .expect("valid async close should lower"),
    );
    assert!(rendered.contains("PythonAsyncRequest::semantic_close_method"));
    assert!(rendered.contains("self.__sifr_python_object"));
    assert!(!rendered.contains("PythonAsyncRequest::owned_method"));
    for required in [
        "retained_callback_finalization_scope",
        "scope.child().clone()",
        "submit_async_declaration_with_callbacks",
        "finish_retained_callback_finalization",
    ] {
        assert!(
            rendered.contains(required),
            "missing {required}\n{rendered}"
        );
    }
    let claim = rendered
        .find("retained_callback_finalization_scope")
        .expect("owner-close cancellation claim");
    let request = rendered
        .find("PythonAsyncRequest::semantic_close_method")
        .expect("semantic owner-close request");
    let close = rendered
        .find("submit_async_declaration_with_callbacks")
        .expect("async owner close submission");
    let resume = rendered
        .find("finish_retained_callback_finalization")
        .expect("native cancellation resumption");
    assert!(
        claim < request && request < close && close < resume,
        "{rendered}"
    );

    let borrowed = method_function(false);
    let borrowed = render_stmts(
        &python_interop_method_body(&borrowed, &Default::default(), Some(&owner))
            .expect("borrowed methods on async-close classes remain ordinary typed requests"),
    );
    assert!(borrowed.contains("PythonAsyncRequest::borrowed_method"));
    assert!(!borrowed.contains("PythonAsyncRequest::semantic_close_method"));

    let wrong_method = method_function(true);
    assert!(python_interop_method_body(&wrong_method, &Default::default(), Some(&owner)).is_none());
}

#[test]
fn async_owner_methods_observe_typed_retained_callback_failures() {
    let mut close_declaration = declaration(vec!["Self", "aclose"], Vec::new());
    close_declaration.consumes_receiver = true;
    let close = function("aclose", Vec::new(), Type::None, close_declaration);
    let mut owner = opaque_declaration();
    owner.cleanup = Some(PythonCleanupPolicy::AsyncClose);
    let errors = vec![Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "HandlerError".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: None,
    }];

    for method in [&close, &method_function(false)] {
        let rendered = render_stmts(
            &python_interop_method_body_with_retained_errors(
                method,
                &Default::default(),
                Some(&owner),
                &Default::default(),
                &errors,
            )
            .expect("async owner method should lower"),
        );
        assert!(
            rendered.contains("__sifr_python_callbacks.owner()"),
            "{rendered}"
        );
        assert!(
            rendered.contains("__sifr_python_callback_failure_0.clone()"),
            "{rendered}"
        );
        assert!(rendered.contains("take_if_owner_first"), "{rendered}");
        assert!(
            rendered.contains("attach_callback_failure_evidence"),
            "{rendered}"
        );
    }
}

#[test]
fn resolved_bridge_target_stays_structured_in_typed_request() {
    let function = function(
        "value",
        Vec::new(),
        Type::Int,
        declaration(
            vec!["__sifr_bridge__", "p_abc123", "adapter", "value"],
            Vec::new(),
        ),
    );
    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default())
            .expect("bridge wrapper should lower"),
    );
    for segment in ["__sifr_bridge__", "p_abc123", "adapter", "value"] {
        assert!(rendered.contains(&format!("\"{segment}\".to_string()")));
    }
    assert!(rendered.contains("Vec<::sifr_runtime::python::PythonAsyncValue> = Vec::new()"));
    assert!(
        rendered.contains("Vec<(String, ::sifr_runtime::python::PythonAsyncValue)> = Vec::new()")
    );
}

#[test]
fn zero_argument_record_wrapper_emits_concrete_frames_and_borrowed_field_names() {
    let response = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Response".to_string(),
        fields: vec![
            ("status".to_string(), Type::Int),
            ("message".to_string(), Type::Str),
        ],
        methods: Vec::new(),
        parent_class: None,
    };
    let function = function(
        "response",
        Vec::new(),
        response,
        declaration(vec!["pkg", "response"], Vec::new()),
    );
    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default())
            .expect("zero-argument record wrapper should lower"),
    );

    assert!(rendered.contains("Vec<::sifr_runtime::python::PythonAsyncValue> = Vec::new()"));
    assert!(rendered.contains("&\"status\".to_string()"));
    assert!(rendered.contains("&\"message\".to_string()"));
}

#[test]
fn standalone_typed_wrapper_emits_cancellation_carrier_preamble() {
    let wrapper = function(
        "value",
        Vec::new(),
        Type::Int,
        declaration(vec!["pkg", "value"], Vec::new()),
    );
    let module = HirModule {
        functions: vec![wrapper],
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };

    let rendered = generate_rust(&module);
    assert!(rendered.contains("fn __sifr_current_task_cancellation"));
    assert!(rendered.contains("submit_async_declaration"));
}

#[test]
fn asyncio_callback_emits_owned_loop_factory_async_handler_and_async_drain() {
    let handler_type = Type::AsyncCallable(
        vec![Type::Int],
        vec![ParamConvention::own()],
        Box::new(Type::Int),
    );
    let mut declaration = declaration(
        vec!["pkg", "apply"],
        vec![shape("handler", PythonParameterKind::Positional, false)],
    );
    declaration.callbacks.push(PythonCallbackDeclaration {
        parameter_name: "handler".to_string(),
        span: TextRange::default(),
        lifetime: PythonCallbackLifetime::Call,
        dispatch: PythonCallbackDispatch::Asyncio,
        concurrency: Some(PythonCallbackConcurrency::Serial),
        argument_types: vec![Type::Int],
        argument_conventions: vec![ParamConvention::own()],
        success_type: Type::Int,
        handler_error_type: None,
        is_async: true,
        owner_class: None,
        owner_cleanup: None,
    });
    let wrapper = function(
        "apply",
        vec![param("handler", handler_type, ParamConvention::borrow())],
        Type::Int,
        declaration,
    );

    let rendered = render_stmts(
        &python_interop_function_body(&wrapper, &Default::default())
            .expect("asyncio callback wrapper should lower"),
    );
    assert!(
        rendered.contains("asyncio_callback_scoped_with_owner"),
        "{rendered}"
    );
    assert!(
        rendered.contains("AsyncioCallbackConcurrency::Serial"),
        "{rendered}"
    );
    assert!(
        rendered.contains("__SIFR_TASK_CANCELLATION.scope"),
        "{rendered}"
    );
    assert!(rendered.contains("close_call_scope().await"), "{rendered}");
    assert!(
        rendered.contains("reconcile_callback_outcome"),
        "{rendered}"
    );
    for required in [
        "retained_callback_finalization_scope",
        "scope.child().clone()",
        "finish_retained_callback_finalization",
    ] {
        assert!(
            rendered.contains(required),
            "missing {required}\n{rendered}"
        );
    }
    let request = rendered
        .find("submit_async_declaration")
        .expect("async declaration request");
    let drain = rendered
        .find("close_call_scope().await")
        .expect("asyncio callback drain");
    let resume = rendered
        .find("finish_retained_callback_finalization")
        .expect("native cancellation resumption");
    assert!(request < drain && drain < resume, "{rendered}");
}

#[test]
fn foreign_callback_in_async_wrapper_uses_nonblocking_drain() {
    let handler_type = Type::Callable(
        vec![Type::Int],
        vec![ParamConvention::own()],
        Box::new(Type::Int),
    );
    let mut declaration = declaration(
        vec!["pkg", "apply"],
        vec![shape("handler", PythonParameterKind::Positional, false)],
    );
    declaration.callbacks.push(PythonCallbackDeclaration {
        parameter_name: "handler".to_string(),
        span: TextRange::default(),
        lifetime: PythonCallbackLifetime::Call,
        dispatch: PythonCallbackDispatch::Foreign,
        concurrency: Some(PythonCallbackConcurrency::Serial),
        argument_types: vec![Type::Int],
        argument_conventions: vec![ParamConvention::own()],
        success_type: Type::Int,
        handler_error_type: None,
        is_async: false,
        owner_class: None,
        owner_cleanup: None,
    });
    let wrapper = function(
        "apply",
        vec![param("handler", handler_type, ParamConvention::borrow())],
        Type::Int,
        declaration,
    );

    let rendered = render_stmts(
        &python_interop_function_body(&wrapper, &Default::default())
            .expect("foreign callback wrapper should lower"),
    );
    assert!(
        rendered.contains("close_call_scope_async().await"),
        "{rendered}"
    );
    assert!(!rendered.contains("close_call_scope();"), "{rendered}");
    for required in [
        "retained_callback_finalization_scope",
        "scope.child().clone()",
        "finish_retained_callback_finalization",
    ] {
        assert!(
            rendered.contains(required),
            "missing {required}\n{rendered}"
        );
    }
    let request = rendered
        .find("submit_async_declaration")
        .expect("async declaration request");
    let drain = rendered
        .find("close_call_scope_async().await")
        .expect("foreign callback drain");
    let resume = rendered
        .find("finish_retained_callback_finalization")
        .expect("native cancellation resumption");
    assert!(request < drain && drain < resume, "{rendered}");
}

#[test]
fn receiver_asyncio_registration_has_terminal_provisional_rollback() {
    let handler_type = Type::AsyncCallable(
        vec![Type::Int],
        vec![ParamConvention::own()],
        Box::new(Type::Int),
    );
    let mut declaration = declaration(
        vec!["Self", "install"],
        vec![shape("handler", PythonParameterKind::Positional, false)],
    );
    declaration.callbacks.push(PythonCallbackDeclaration {
        parameter_name: "handler".to_string(),
        span: TextRange::default(),
        lifetime: PythonCallbackLifetime::Receiver,
        dispatch: PythonCallbackDispatch::Asyncio,
        concurrency: Some(PythonCallbackConcurrency::Parallel),
        argument_types: vec![Type::Int],
        argument_conventions: vec![ParamConvention::own()],
        success_type: Type::Int,
        handler_error_type: None,
        is_async: true,
        owner_class: Some("Client".to_string()),
        owner_cleanup: Some(PythonCleanupPolicy::AsyncClose),
    });
    let method = function(
        "install",
        vec![param("handler", handler_type, ParamConvention::borrow())],
        Type::None,
        declaration,
    );
    let mut owner = opaque_declaration();
    owner.cleanup = Some(PythonCleanupPolicy::AsyncClose);

    let rendered = render_stmts(
        &python_interop_method_body(&method, &Default::default(), Some(&owner))
            .expect("receiver callback method should lower"),
    );
    for required in [
        "let mut __sifr_provisional_callback_0 = None",
        "__sifr_provisional_callback_0 = Some(",
        "rollback_provisional().await",
        "receiver-callback-registration",
        "retained_callback_finalization_scope",
        "scope.child().clone()",
        "finish_retained_callback_finalization",
    ] {
        assert!(
            rendered.contains(required),
            "missing {required}\n{rendered}"
        );
    }
    let request = rendered
        .find("submit_async_declaration")
        .expect("registration request");
    let rollback = rendered
        .find("rollback_provisional().await")
        .expect("terminal rollback");
    let resume = rendered
        .find("finish_retained_callback_finalization")
        .expect("native cancellation resumption");
    assert!(request < rollback && rollback < resume, "{rendered}");
    syn::parse_file(&format!(
        "async fn generated() -> Result<(), PythonError> {{ {rendered} }}"
    ))
    .expect("provisional receiver wrapper should be valid Rust syntax");
}

#[test]
fn retained_asyncio_result_masks_native_cancellation_until_rollback_finishes() {
    let handler_type = Type::AsyncCallable(
        vec![Type::Int],
        vec![ParamConvention::own()],
        Box::new(Type::Int),
    );
    let mut declaration = declaration(
        vec!["pkg", "subscribe"],
        vec![shape("handler", PythonParameterKind::Positional, false)],
    );
    declaration.callbacks.push(PythonCallbackDeclaration {
        parameter_name: "handler".to_string(),
        span: TextRange::default(),
        lifetime: PythonCallbackLifetime::Result,
        dispatch: PythonCallbackDispatch::Asyncio,
        concurrency: Some(PythonCallbackConcurrency::Parallel),
        argument_types: vec![Type::Int],
        argument_conventions: vec![ParamConvention::own()],
        success_type: Type::Int,
        handler_error_type: None,
        is_async: true,
        owner_class: Some("Subscription".to_string()),
        owner_cleanup: Some(PythonCleanupPolicy::AsyncClose),
    });
    let owner_type = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Subscription".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };
    let wrapper = function(
        "subscribe",
        vec![param("handler", handler_type, ParamConvention::borrow())],
        owner_type,
        declaration,
    );
    let mut owner = opaque_declaration();
    owner.cleanup = Some(PythonCleanupPolicy::AsyncClose);
    owner.target = Some(PythonTargetPath {
        segments: vec!["pkg".to_string(), "Subscription".to_string()],
        span: TextRange::default(),
    });
    let opaque_classes = HashMap::from([("Subscription".to_string(), owner)]);

    let rendered = render_stmts(
        &python_interop_function_body(&wrapper, &opaque_classes)
            .expect("retained asyncio wrapper should lower"),
    );
    for required in [
        "retained_callback_finalization_scope",
        "scope.child().clone()",
        "finalize_retained_callbacks",
        "finish_retained_callback_finalization",
    ] {
        assert!(
            rendered.contains(required),
            "missing {required}\n{rendered}"
        );
    }
    let rollback = rendered
        .find("finalize_retained_callbacks")
        .expect("awaited rollback");
    let resume = rendered
        .find("finish_retained_callback_finalization")
        .expect("native cancellation resumption");
    assert!(rollback < resume, "{rendered}");
}

#[test]
fn async_python_error_converts_to_an_active_error_supertype() {
    let wrapper = function(
        "value",
        Vec::new(),
        Type::Int,
        declaration(vec!["pkg", "value"], Vec::new()),
    );
    let main = HirFunction {
        name: "main".to_string(),
        params: Vec::new(),
        return_type: Type::Result(
            Box::new(Type::None),
            Box::new(Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "Error".to_string(),
                fields: vec![("message".to_string(), Type::Str)],
                methods: Vec::new(),
                parent_class: None,
            }),
        ),
        body: vec![HirStmt::Raise {
            value: HirExpr::Name {
                name: "python_error".to_string(),
                binding_id: None,
                ty: python_error_type(),
            },
        }],
        is_async: true,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let module = HirModule {
        functions: vec![wrapper, main],
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    };

    let rendered = generate_rust(&module);
    assert_eq!(
        rendered.matches("impl From<PythonError> for Error").count(),
        1
    );
    assert!(rendered.contains("Self::new(err.message)"));
    assert!(
        rendered.contains("return Err(python_error.into());"),
        "{rendered}"
    );
}

#[test]
fn async_main_is_scoped_under_root_cancellation_carrier() {
    let mut items = vec![crate::RustItem::Fn {
        name: "main".to_string(),
        visibility: crate::Visibility::Private,
        type_params: Vec::new(),
        params: Vec::new(),
        ret: None,
        body: vec![crate::RustStmt::Expr(crate::RustExpr::Tuple(Vec::new()))],
        is_async: true,
    }];
    crate::scope_async_main_cancellation(&mut items);
    let rendered = crate::render_items(&items);
    assert!(rendered.contains("let __sifr_root_cancellation"));
    assert!(rendered.contains("__SIFR_TASK_CANCELLATION.scope"));
    syn::parse_file(&rendered).expect("root-scoped async main should be valid Rust syntax");
}

fn function(
    name: &str,
    params: Vec<HirParam>,
    ok_type: Type,
    declaration: PythonInteropDeclaration,
) -> HirFunction {
    HirFunction {
        name: name.to_string(),
        params,
        return_type: Type::Result(Box::new(ok_type), Box::new(python_error_type())),
        body: Vec::new(),
        is_async: true,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    }
}

fn method_function(consumes_receiver: bool) -> HirFunction {
    let mut declaration = declaration(
        vec!["Self", "work"],
        vec![shape("amount", PythonParameterKind::Positional, false)],
    );
    declaration.consumes_receiver = consumes_receiver;
    function(
        "work",
        vec![param("amount", Type::Int, ParamConvention::own())],
        Type::Int,
        declaration,
    )
}

fn declaration(
    target: Vec<&str>,
    parameters: Vec<PythonInteropParameter>,
) -> PythonInteropDeclaration {
    PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Coroutine,
        target: Some(PythonTargetPath {
            segments: target.into_iter().map(str::to_string).collect(),
            span: TextRange::default(),
        }),
        span: TextRange::default(),
        effect: PythonInteropEffect::Async,
        cleanup: None,
        consumes_receiver: false,
        parameters,
        required_import_root: Some("pkg".to_string()),
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    }
}

fn opaque_declaration() -> PythonInteropDeclaration {
    PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Opaque,
        target: Some(PythonTargetPath {
            segments: vec!["pkg".to_string(), "Client".to_string()],
            span: TextRange::default(),
        }),
        span: TextRange::default(),
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters: Vec::new(),
        required_import_root: Some("pkg".to_string()),
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    }
}

fn python_error_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "PythonError".to_string(),
        fields: ["message", "kind", "exception_type", "traceback", "context"]
            .into_iter()
            .map(|name| (name.to_string(), Type::Str))
            .collect(),
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

fn param(name: &str, ty: Type, convention: ParamConvention) -> HirParam {
    HirParam {
        name: name.to_string(),
        ty,
        default: None,
        keyword_only: false,
        convention,
    }
}

fn shape(name: &str, kind: PythonParameterKind, omit: bool) -> PythonInteropParameter {
    PythonInteropParameter {
        name: name.to_string(),
        kind,
        has_default: omit,
        omit_when_absent: omit,
        span: TextRange::default(),
    }
}
