use crate::python_interop_direct::{
    input_conversion, output_value_expr, python_interop_function_body, python_interop_method_body,
};
use crate::render_stmts;
use ruff_text_size::TextRange;
use sifr_ir::{
    HirFunction, HirParam, MethodKind, PythonInteropDeclaration, PythonInteropDecoratorKind,
    PythonInteropEffect, PythonInteropParameter, PythonParameterKind, PythonTargetPath,
};
use sifr_type_system::{ParamConvention, Type};

#[test]
fn sync_wrapper_emits_complete_owned_argument_frame() {
    let error_type = Type::Class {
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
    assert!(rendered.contains("sifr_runtime::python::resolve_target"));
    assert!(rendered.contains("for __sifr_python_value_1 in rest.iter()"));
    assert!(rendered.contains("if let Some(__sifr_python_value_2) = label"));
    assert!(rendered.contains("for (__sifr_python_key_3, __sifr_python_value_3) in extra.iter()"));
    assert!(rendered.contains("sifr_runtime::python::call_object_owned"));
    assert!(rendered.contains("sifr_runtime::python::to_int"));
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
    assert!(rendered.contains("semantic_close(self.__sifr_python_object"));
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
