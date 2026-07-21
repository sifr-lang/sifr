use crate::python_interop_direct::{
    input_conversion, is_python_object, output_value_expr, python_interop_function_body,
    python_interop_function_body_with_retained_errors, python_interop_method_body,
    python_interop_method_body_with_retained_errors,
};
use crate::{generate_rust, render_stmts};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirClass, HirClassKind, HirFunction, HirModule, HirParam, MethodKind,
    PythonCallbackConcurrency, PythonCallbackDeclaration, PythonCallbackDispatch,
    PythonCallbackLifetime, PythonCleanupPolicy, PythonInteropDeclaration,
    PythonInteropDecoratorKind, PythonInteropEffect, PythonInteropParameter, PythonParameterKind,
    PythonTargetPath,
};
use sifr_type_system::{ParamConvention, Type};

struct PythonOpaqueAutoTraitProbe {
    _marker: std::marker::PhantomData<std::rc::Rc<()>>,
}

static_assertions::assert_not_impl_any!(PythonOpaqueAutoTraitProbe: Send, Sync);

#[test]
fn sync_wrapper_emits_complete_owned_argument_frame() {
    let error_type = Type::Class {
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
        parent_class: Some("Error".to_string()),
    };
    let optional_string = Type::Union(vec![Type::Str, Type::None]);
    let params = vec![
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
    ];
    let function = HirFunction {
        name: "collect".to_string(),
        params,
        return_type: Type::Result(Box::new(Type::Int), Box::new(error_type)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration()],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        python_interop_function_body(&function, &Default::default()).expect("wrapper should lower");
    let rendered = render_stmts(&body);
    assert!(rendered.contains("::sifr_runtime::python::resolve_target"));
    assert!(rendered.contains("for __sifr_python_value_1 in rest.iter()"));
    assert!(rendered.contains("if let Some(__sifr_python_value_2) = label"));
    assert!(rendered.contains("for (__sifr_python_key_3, __sifr_python_value_3) in extra.iter()"));
    assert!(rendered.contains("::sifr_runtime::python::call_object_owned"));
    assert!(rendered.contains("::sifr_runtime::python::to_int"));
}

#[test]
fn resolved_bridge_target_emits_reserved_runtime_lookup() {
    let mut declaration = declaration();
    declaration.target = Some(PythonTargetPath {
        segments: vec![
            "__sifr_bridge__".to_string(),
            "p_abc123".to_string(),
            "adapter".to_string(),
            "value".to_string(),
        ],
        span: TextRange::default(),
    });
    declaration.parameters.clear();
    declaration.required_import_root = None;
    let function = HirFunction {
        name: "value".to_string(),
        params: Vec::new(),
        return_type: Type::Result(Box::new(Type::Int), Box::new(python_error_type())),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default())
            .expect("resolved bridge wrapper should lower"),
    );

    assert!(rendered.contains("\"__sifr_bridge__\""));
    assert!(rendered.contains("\"p_abc123\""));
    assert!(rendered.contains("\"adapter\""));
}

#[test]
fn omittable_positional_parameters_are_forwarded_by_name_without_shifting() {
    let mut declaration = declaration();
    declaration.parameters = vec![
        shape("a", PythonParameterKind::Positional, false),
        shape("b", PythonParameterKind::Positional, true),
        shape("c", PythonParameterKind::Positional, false),
    ];
    let function = HirFunction {
        name: "collect".to_string(),
        params: vec![
            param("a", Type::Int, ParamConvention::own()),
            param("b", Type::Int, ParamConvention::own()),
            param("c", Type::Int, ParamConvention::own()),
        ],
        return_type: Type::Result(
            Box::new(Type::Int),
            Box::new(Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "PythonError".to_string(),
                fields: Vec::new(),
                methods: Vec::new(),
                parent_class: Some("Error".to_string()),
            }),
        ),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default()).expect("wrapper"),
    );
    assert!(rendered.contains("__sifr_python_args.push(__sifr_python_arg_0)"));
    assert!(
        rendered.contains("(\"b\".to_string(), __sifr_python_arg_1)"),
        "{rendered}"
    );
    assert!(
        rendered.contains("(\"c\".to_string(), __sifr_python_arg_2)"),
        "{rendered}"
    );
}

#[test]
fn recursive_wrapper_emits_list_dict_tuple_and_record_conversions() {
    let error_type = python_error_type();
    let record = Type::Class {
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
    let mut declaration = declaration();
    declaration.parameters = vec![shape("payload", PythonParameterKind::Positional, false)];
    assert!(
        input_conversion("payload", &record, &Default::default()).is_some(),
        "recursive input should lower"
    );
    let output_type = Type::Dict(
        Box::new(Type::Str),
        Box::new(Type::Tuple(vec![Type::Int, record.clone()])),
    );
    assert!(
        output_value_expr("value", &output_type, &error_type, &Default::default()).is_some(),
        "recursive output should lower"
    );
    let function = HirFunction {
        name: "round_trip".to_string(),
        params: vec![param("payload", record.clone(), ParamConvention::borrow())],
        return_type: Type::Result(Box::new(output_type), Box::new(error_type)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default())
            .expect("recursive wrapper should lower"),
    );
    assert!(rendered.contains("from_record_results"), "{rendered}");
    assert!(rendered.contains("from_list_results"), "{rendered}");
    assert!(rendered.contains("dict_str_items"), "{rendered}");
    assert!(rendered.contains("tuple_items"), "{rendered}");
    assert!(rendered.contains("record_field"), "{rendered}");
}

#[test]
fn object_basename_does_not_bypass_canonical_python_identity() {
    let local_object = Type::Class {
        identity: Some("main.Object".to_string()),
        type_args: Vec::new(),
        name: "Object".to_string(),
        fields: vec![("value".to_string(), Type::Int)],
        methods: Vec::new(),
        parent_class: None,
    };
    let canonical_object = Type::Class {
        identity: Some("_sifr.python.Object".to_string()),
        type_args: Vec::new(),
        name: "Object".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };

    let local = input_conversion("value", &local_object, &Default::default())
        .expect("same-named local record should use record conversion");
    let rendered_local = render_stmts(&[crate::RustStmt::Expr(local)]);

    assert!(
        rendered_local.contains("from_record_results"),
        "{rendered_local}"
    );
    assert!(
        !rendered_local.contains("temporary_argument_handle"),
        "{rendered_local}"
    );
    assert!(is_python_object(&canonical_object));
}

#[test]
fn consuming_opaque_close_emits_semantic_close_operation() {
    let error_type = python_error_type();
    let mut declaration = declaration();
    declaration.target = Some(PythonTargetPath {
        segments: vec!["Self".to_string(), "close".to_string()],
        span: TextRange::default(),
    });
    declaration.parameters.clear();
    declaration.consumes_receiver = true;
    let method = HirFunction {
        name: "close".to_string(),
        params: Vec::new(),
        return_type: Type::Result(Box::new(Type::None), Box::new(error_type)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let rendered = render_stmts(
        &python_interop_method_body(&method, &Default::default(), None)
            .expect("consuming close should lower"),
    );
    assert!(rendered.contains("semantic_close_with_callbacks(self.__sifr_python_object"));
    assert!(rendered.contains("self.__sifr_python_callbacks"));
}

#[test]
fn typed_current_callback_emits_checked_adapter_failure_reconciliation_and_cleanup() {
    let python_error = python_error_type();
    let handler_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "HandlerError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let error = Type::Union(vec![python_error, handler_error.clone()]);
    let handler_type = Type::Callable(
        vec![Type::List(Box::new(Type::Int))],
        vec![ParamConvention::own()],
        Box::new(Type::Result(
            Box::new(Type::Int),
            Box::new(handler_error.clone()),
        )),
    );
    let mut declaration = declaration();
    declaration.parameters = vec![shape("handler", PythonParameterKind::Positional, false)];
    let mut callback = callback_declaration(
        PythonCallbackLifetime::Call,
        PythonCallbackDispatch::Current,
        None,
        Some(handler_error),
        None,
    );
    callback.argument_types = vec![Type::List(Box::new(Type::Int))];
    declaration.callbacks = vec![callback];
    let function = HirFunction {
        name: "map".to_string(),
        params: vec![param("handler", handler_type, ParamConvention::own())],
        return_type: Type::Result(Box::new(Type::Int), Box::new(error)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let rendered = render_stmts(
        &python_interop_function_body(&function, &Default::default()).expect("callback wrapper"),
    );
    assert!(rendered.contains("CallbackFailureSlot::new"), "{rendered}");
    assert!(
        rendered.contains("current_callback_with_owner"),
        "{rendered}"
    );
    assert!(rendered.contains("list_items"), "{rendered}");
    assert!(rendered.contains("Sifr callback handler returned an error"));
    assert!(rendered.contains("__sifr_callback_0.close()"), "{rendered}");
    assert!(
        rendered.contains("__sifr_callback_failure_0.take_if_owner_first"),
        "{rendered}"
    );
    syn::parse_file(&format!("fn generated() {{ {rendered} }}"))
        .expect("generated callback statements should be valid Rust syntax");
}

#[test]
fn retained_foreign_callback_is_aggregated_into_the_opaque_result_owner() {
    let subscription_type = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Subscription".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };
    let opaque = PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Opaque,
        target: Some(PythonTargetPath {
            segments: vec!["pkg".to_string(), "Subscription".to_string()],
            span: TextRange::default(),
        }),
        span: TextRange::default(),
        effect: PythonInteropEffect::BlockingIo,
        cleanup: Some(PythonCleanupPolicy::Close),
        consumes_receiver: false,
        parameters: Vec::new(),
        required_import_root: Some("pkg".to_string()),
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
    };
    let mut opaque_classes = std::collections::HashMap::new();
    opaque_classes.insert("Subscription".to_string(), opaque);
    let mut declaration = declaration();
    declaration.parameters = vec![shape("handler", PythonParameterKind::Positional, false)];
    declaration.callbacks = vec![callback_declaration(
        PythonCallbackLifetime::Result,
        PythonCallbackDispatch::Foreign,
        Some(PythonCallbackConcurrency::Serial),
        None,
        Some(PythonCleanupPolicy::Close),
    )];
    let handler_type = Type::Callable(
        vec![Type::Int],
        vec![ParamConvention::own()],
        Box::new(Type::Int),
    );
    let function = HirFunction {
        name: "subscribe".to_string(),
        params: vec![param("handler", handler_type, ParamConvention::own())],
        return_type: Type::Result(Box::new(subscription_type), Box::new(python_error_type())),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let rendered = render_stmts(
        &python_interop_function_body(&function, &opaque_classes).expect("retained wrapper"),
    );
    assert!(
        rendered.contains("RetainedCallbackGroup::new"),
        "{rendered}"
    );
    assert!(rendered.contains("foreign_callback_with_owner"));
    assert!(rendered.contains("retain_in_owner"));
    assert!(rendered.contains("commit_for_object"));
    assert!(rendered.contains("CallbackOwnerSlot::from_owner"));
    assert!(!rendered.contains("close_call_scope"));
    syn::parse_file(&format!("fn generated() {{ {rendered} }}"))
        .expect("generated retained callback statements should be valid Rust syntax");
}

#[test]
fn receiver_retained_callback_reuses_the_opaque_owner_slot() {
    let mut declaration = declaration();
    declaration.target = Some(PythonTargetPath {
        segments: vec!["Self".to_string(), "register".to_string()],
        span: TextRange::default(),
    });
    declaration.parameters = vec![shape("handler", PythonParameterKind::Positional, false)];
    declaration.callbacks = vec![callback_declaration(
        PythonCallbackLifetime::Receiver,
        PythonCallbackDispatch::Foreign,
        Some(PythonCallbackConcurrency::Parallel),
        None,
        Some(PythonCleanupPolicy::Close),
    )];
    let method = HirFunction {
        name: "register".to_string(),
        params: vec![param(
            "handler",
            Type::Callable(
                vec![Type::Int],
                vec![ParamConvention::own()],
                Box::new(Type::Int),
            ),
            ParamConvention::own(),
        )],
        return_type: Type::Result(Box::new(Type::None), Box::new(python_error_type())),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let rendered = render_stmts(
        &python_interop_method_body(&method, &Default::default(), None)
            .expect("receiver callback wrapper"),
    );
    assert!(rendered.contains("__sifr_python_callbacks.owner_or_insert"));
    assert!(rendered.contains("foreign_callback_with_owner"));
    assert!(rendered.contains("retain_in_owner"));
    assert!(!rendered.contains("close_call_scope"));
    let class = HirClass {
        name: "Subscription".to_string(),
        identity: None,
        fields: Vec::new(),
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
    let bounded = emitter.lower_class_method_param_type(
        &class,
        &method,
        "handler",
        &method.params[0].ty,
        method.params[0].convention,
    );
    let bounded = crate::render_type(&bounded);
    assert!(bounded.contains("+ Send + Sync + 'static"), "{bounded}");
    syn::parse_file(&format!("fn generated() {{ {rendered} }}"))
        .expect("generated receiver callback statements should be valid Rust syntax");
}

#[test]
fn retained_handler_failure_moves_into_typed_owner_sidecar_and_close_observes_it() {
    let handler_error = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "HandlerError".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    };
    let error_channel = Type::Union(vec![python_error_type(), handler_error.clone()]);
    let subscription_type = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Subscription".to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("NonSend".to_string()),
    };
    let mut opaque = declaration();
    opaque.kind = PythonInteropDecoratorKind::Opaque;
    opaque.cleanup = Some(PythonCleanupPolicy::Close);
    opaque.target = Some(PythonTargetPath {
        segments: vec!["pkg".to_string(), "Subscription".to_string()],
        span: TextRange::default(),
    });
    let mut opaque_classes = std::collections::HashMap::new();
    opaque_classes.insert("Subscription".to_string(), opaque.clone());
    let mut retained_errors = std::collections::HashMap::new();
    retained_errors.insert("Subscription".to_string(), vec![handler_error.clone()]);

    let mut subscribe_declaration = declaration();
    subscribe_declaration.parameters =
        vec![shape("handler", PythonParameterKind::Positional, false)];
    subscribe_declaration.callbacks = vec![callback_declaration(
        PythonCallbackLifetime::Result,
        PythonCallbackDispatch::Foreign,
        Some(PythonCallbackConcurrency::Serial),
        Some(handler_error.clone()),
        Some(PythonCleanupPolicy::Close),
    )];
    let subscribe = HirFunction {
        name: "subscribe".to_string(),
        params: vec![param(
            "handler",
            Type::Callable(
                vec![Type::Int],
                vec![ParamConvention::own()],
                Box::new(Type::Result(
                    Box::new(Type::Int),
                    Box::new(handler_error.clone()),
                )),
            ),
            ParamConvention::own(),
        )],
        return_type: Type::Result(Box::new(subscription_type), Box::new(error_channel.clone())),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![subscribe_declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let subscribe_rendered = render_stmts(
        &python_interop_function_body_with_retained_errors(
            &subscribe,
            &opaque_classes,
            &retained_errors,
        )
        .expect("retained typed wrapper"),
    );
    assert!(
        subscribe_rendered.contains("__sifr_python_callback_failure_0 ="),
        "{subscribe_rendered}"
    );
    assert!(
        subscribe_rendered.contains("__sifr_retained__sifr_python_callback_failure_0.clone()"),
        "{subscribe_rendered}"
    );

    let mut inspect_declaration = declaration();
    inspect_declaration.target = Some(PythonTargetPath {
        segments: vec!["Self".to_string(), "inspect".to_string()],
        span: TextRange::default(),
    });
    inspect_declaration.parameters = vec![shape("value", PythonParameterKind::Positional, false)];
    let inspect = HirFunction {
        name: "inspect".to_string(),
        params: vec![param("value", Type::Int, ParamConvention::own())],
        return_type: Type::Result(Box::new(Type::Int), Box::new(error_channel.clone())),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![inspect_declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let inspect_rendered = render_stmts(
        &python_interop_method_body_with_retained_errors(
            &inspect,
            &opaque_classes,
            Some(&opaque),
            &retained_errors,
            &[handler_error.clone()],
        )
        .expect("typed later owner method"),
    );
    let observer = inspect_rendered
        .find("__sifr_python_callbacks.owner()")
        .expect("owner observer setup");
    let conversion = inspect_rendered
        .find("async_from_int")
        .or_else(|| inspect_rendered.find("from_int"))
        .expect("argument conversion");
    let lookup = inspect_rendered.find("get_attr").expect("method lookup");
    assert!(
        observer < conversion && observer < lookup,
        "{inspect_rendered}"
    );
    assert!(
        inspect_rendered[..lookup].contains("attach_callback_failure_evidence"),
        "{inspect_rendered}"
    );

    let mut close_declaration = declaration();
    close_declaration.target = Some(PythonTargetPath {
        segments: vec!["Self".to_string(), "close".to_string()],
        span: TextRange::default(),
    });
    close_declaration.consumes_receiver = true;
    close_declaration.parameters.clear();
    let close = HirFunction {
        name: "close".to_string(),
        params: Vec::new(),
        return_type: Type::Result(Box::new(Type::None), Box::new(error_channel)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: vec![close_declaration],
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };
    let close_rendered = render_stmts(
        &python_interop_method_body_with_retained_errors(
            &close,
            &opaque_classes,
            Some(&opaque),
            &retained_errors,
            &[handler_error],
        )
        .expect("typed close wrapper"),
    );
    assert!(
        close_rendered.contains("__sifr_python_callbacks.owner()"),
        "{close_rendered}"
    );
    assert!(
        close_rendered.contains("take_if_owner_first"),
        "{close_rendered}"
    );
    syn::parse_file(&format!(
        "fn subscribe_generated() {{ {subscribe_rendered} }} fn close_generated() {{ {close_rendered} }}"
    ))
    .expect("generated typed owner statements should be valid Rust syntax");

    let module = HirModule {
        functions: vec![subscribe],
        classes: vec![HirClass {
            name: "Subscription".to_string(),
            identity: None,
            fields: Vec::new(),
            methods: vec![inspect, close],
            is_hashable: false,
            is_error_type: false,
            kind: HirClassKind::PythonOpaque(opaque),
            operator_impls: Vec::new(),
            newtype_inner: None,
            implements_protocols: Vec::new(),
            parent_class: Some("NonSend".to_string()),
            parent_type: None,
            type_params: Vec::new(),
            enum_variants: Vec::new(),
            rust_interop: Vec::new(),
        }],
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: Default::default(),
        type_param_bounds: Default::default(),
    };
    let generated_module = generate_rust(&module);
    assert!(
        generated_module.contains(
            "__sifr_python_callback_failure_0: ::sifr_runtime::python::CallbackFailureSlot<HandlerError>"
        ),
        "{generated_module}"
    );
    assert!(
        generated_module
            .contains("__sifr_python_not_send_sync: ::std::marker::PhantomData<::std::rc::Rc<()>>"),
        "{generated_module}"
    );
    assert!(
        generated_module.contains("fn __sifr_from_python_object("),
        "{generated_module}"
    );
    assert!(
        generated_module.contains("+ Send + Sync + 'static"),
        "{generated_module}"
    );
    syn::parse_file(&generated_module).expect("complete typed owner module should parse as Rust");
}

fn callback_declaration(
    lifetime: PythonCallbackLifetime,
    dispatch: PythonCallbackDispatch,
    concurrency: Option<PythonCallbackConcurrency>,
    handler_error_type: Option<Type>,
    owner_cleanup: Option<PythonCleanupPolicy>,
) -> PythonCallbackDeclaration {
    PythonCallbackDeclaration {
        parameter_name: "handler".to_string(),
        span: TextRange::default(),
        lifetime,
        dispatch,
        concurrency,
        argument_types: vec![Type::Int],
        argument_conventions: vec![ParamConvention::own()],
        success_type: Type::Int,
        handler_error_type,
        is_async: false,
        owner_class: (lifetime != PythonCallbackLifetime::Call).then(|| "Subscription".to_string()),
        owner_cleanup,
    }
}

fn python_error_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "PythonError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
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

fn declaration() -> PythonInteropDeclaration {
    PythonInteropDeclaration {
        kind: PythonInteropDecoratorKind::Function,
        target: Some(PythonTargetPath {
            segments: vec!["pkg".to_string(), "collect".to_string()],
            span: TextRange::default(),
        }),
        span: TextRange::default(),
        effect: PythonInteropEffect::BlockingIo,
        cleanup: None,
        consumes_receiver: false,
        parameters: vec![
            shape("value", PythonParameterKind::Positional, false),
            shape("rest", PythonParameterKind::PositionalVariadic, false),
            shape("label", PythonParameterKind::KeywordOnly, true),
            shape("extra", PythonParameterKind::KeywordVariadic, false),
        ],
        required_import_root: Some("pkg".to_string()),
        callbacks: Vec::new(),
        buffer: None,
        arrow: None,
        dlpack: None,
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
