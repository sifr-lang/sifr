use crate::python_interop_direct::{python_interop_function_body, python_interop_method_body};
use crate::{generate_rust, render_stmts};
use ruff_text_size::TextRange;
use sifr_ir::{
    HirFunction, HirModule, HirParam, MethodKind, PythonInteropDeclaration,
    PythonInteropDecoratorKind, PythonInteropEffect, PythonInteropParameter, PythonParameterKind,
    PythonTargetPath,
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
fn recursive_factory_emits_loop_thread_schema_and_owned_opaque_result() {
    let payload = Type::Class {
        name: "Payload".to_string(),
        fields: vec![
            ("name".to_string(), Type::Str),
            ("scores".to_string(), Type::List(Box::new(Type::Int))),
        ],
        methods: Vec::new(),
        parent_class: None,
    };
    let client = Type::Class {
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
    assert!(rendered.contains("Client { __sifr_python_object:"));
}

#[test]
fn borrowed_and_consuming_async_methods_select_distinct_identity_transfers() {
    let borrowed = method_function(false);
    let consuming = method_function(true);

    let borrowed = render_stmts(
        &python_interop_method_body(&borrowed, &Default::default())
            .expect("borrowed method should lower"),
    );
    assert!(borrowed.contains("PythonAsyncRequest::borrowed_method"));
    assert!(borrowed.contains("&self.__sifr_python_object"));

    let consuming = render_stmts(
        &python_interop_method_body(&consuming, &Default::default())
            .expect("consuming method should lower"),
    );
    assert!(consuming.contains("PythonAsyncRequest::owned_method"));
    assert!(consuming.contains("self.__sifr_python_object"));
    assert!(!consuming.contains("&self.__sifr_python_object"));
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
    }
}

fn python_error_type() -> Type {
    Type::Class {
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

fn shape(name: &str, kind: PythonParameterKind, omit: bool) -> PythonInteropParameter {
    PythonInteropParameter {
        name: name.to_string(),
        kind,
        has_default: omit,
        omit_when_absent: omit,
        span: TextRange::default(),
    }
}
