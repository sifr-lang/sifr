use super::{
    collect_module_intrinsic_function_names, collect_referenced_builtin_error_classes,
};
use sifr_ir::{
    CompilerIntrinsicId, HirAsyncWithKind, HirClass, HirClassKind, HirExpr, HirFunction,
    HirModule, HirParam, HirStmt, MethodKind,
};
use sifr_type_system::{ParamConvention, Type};
use std::collections::{HashMap, HashSet};

fn error_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Vec::new().into(),
        methods: Vec::new().into(),
        parent_class: None,
    }
}

fn empty_module() -> HirModule {
    HirModule {
        functions: Vec::new(),
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}

#[test]
fn collects_builtin_error_refs_from_function_type_positions() {
    let mut module = empty_module();
    module.functions.push(HirFunction {
        name: "work".to_string(),
        params: vec![HirParam {
            name: "err".to_string(),
            ty: error_type("ValueError"),
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::Result(Box::new(Type::Int), Box::new(error_type("IOError"))),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    });

    let referenced = collect_referenced_builtin_error_classes(
        &module,
        "",
        &HashSet::new(),
        false,
        &["ValueError", "IOError"],
    );

    assert!(referenced.contains("ValueError"));
    assert!(referenced.contains("IOError"));
}

#[test]
fn collects_builtin_error_refs_from_class_fields_and_constant_types() {
    let mut module = empty_module();
    module.classes.push(HirClass {
        name: "Holder".to_string(),
        identity: None,
        fields: vec![("err".to_string(), error_type("ParseError"))],
        field_defaults: Vec::new(),
        field_default_identities: Vec::new(),
        declaration_metadata: Vec::new(),
        methods: vec![HirFunction {
            name: "check".to_string(),
            params: vec![HirParam {
                name: "e".to_string(),
                ty: Type::alias("AliasErr", error_type("RegexError")),
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
            rust_interop: Vec::new(),
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
        rust_interop: Vec::new(),
    });
    module.constants.push((
        "LAST_ERR".to_string(),
        Type::Result(Box::new(Type::Int), Box::new(error_type("JSONDecodeError"))),
        sifr_ir::HirExpr::NoneLiteral,
    ));

    let referenced = collect_referenced_builtin_error_classes(
        &module,
        "",
        &HashSet::new(),
        false,
        &["ParseError", "RegexError", "JSONDecodeError"],
    );

    assert!(referenced.contains("ParseError"));
    assert!(referenced.contains("RegexError"));
    assert!(referenced.contains("JSONDecodeError"));
}

#[test]
fn intrinsic_helpers_reference_scope_failure() {
    let intrinsic_functions = HashSet::from(["__sifr_spawn_infallible".to_string()]);

    let referenced = collect_referenced_builtin_error_classes(
        &empty_module(),
        "",
        &intrinsic_functions,
        false,
        &["ScopeFailure", "TimeoutError"],
    );

    assert!(referenced.contains("ScopeFailure"));
    assert!(referenced.contains("TimeoutError"));
}

#[test]
fn project_intrinsic_scan_includes_module_constant_expressions() {
    let mut module = empty_module();
    module.constants.push((
        "DATA".to_string(),
        Type::Bytes,
        HirExpr::IntrinsicCall {
            intrinsic: CompilerIntrinsicId::BytesWithSize,
            args: vec![HirExpr::IntLiteral(4)],
            ty: Type::Result(Box::new(Type::Bytes), Box::new(Type::Any)),
            call_range: Default::default(),
            arg_ranges: vec![Default::default()],
        },
    ));

    let names = collect_module_intrinsic_function_names(&module);

    assert_eq!(names, HashSet::from(["bytes_with_size".to_string()]));
}

#[test]
fn async_task_runtime_helpers_reference_generated_error_types() {
    let mut module = empty_module();
    module.functions.push(HirFunction {
        name: "main".to_string(),
        params: Vec::new(),
        return_type: Type::Result(Box::new(Type::None), Box::new(error_type("Error"))),
        body: vec![
            HirStmt::AsyncWith {
                kind: HirAsyncWithKind::TaskScope,
                target: Some("scope".to_string()),
                body: Vec::new(),
            },
            HirStmt::AsyncWith {
                kind: HirAsyncWithKind::TaskTimeout {
                    duration: HirExpr::FloatLiteral(1.0),
                },
                target: None,
                body: Vec::new(),
            },
        ],
        is_async: true,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    });

    let referenced = collect_referenced_builtin_error_classes(
        &module,
        "",
        &HashSet::new(),
        false,
        &["Error", "ScopeFailure", "TimeoutError"],
    );

    assert!(referenced.contains("Error"));
    assert!(referenced.contains("ScopeFailure"));
    assert!(referenced.contains("TimeoutError"));
}
