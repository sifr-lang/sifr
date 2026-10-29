use crate::render_expr;
use ruff_text_size::TextRange;
use sifr_ir::{
    HirParam, MethodKind, RustInteropAbiRequirements, RustInteropEffect, RustTargetPath,
};
use sifr_type_system::{FixedIntType, ParamConvention, Type};

use super::*;

fn error_type(name: &str, fields: Vec<(&str, Type)>) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: fields
            .into_iter()
            .map(|(name, ty)| (name.to_string(), ty))
            .collect(),
        methods: Vec::new().into(),
        parent_class: Some("Error".to_string()),
    }
}

#[test]
fn direct_rust_function_body_calls_cargo_dependency_path() {
    let func = HirFunction {
        name: "crc32".to_string(),
        params: vec![HirParam {
            name: "data".to_string(),
            ty: Type::Bytes,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::FixedInt(FixedIntType::U32),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["crc32fast", "hash"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    assert_eq!(
        direct_rust_function_body(&func),
        Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "crc32fast".to_string(),
                "hash".to_string()
            ])),
            args: vec![RustExpr::Ident("data".to_string())],
        }))])
    );
}

#[test]
fn direct_rust_function_body_converts_owned_integer_list_arguments() {
    let func = HirFunction {
        name: "roundtrip_ints".to_string(),
        params: vec![HirParam {
            name: "items".to_string(),
            ty: Type::List(Box::new(Type::Int)),
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::List(Box::new(Type::Int)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["fixture_bridge", "roundtrip_ints"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = direct_rust_function_body(&func)
        .expect("direct integer-list interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct integer-list interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "fixture_bridge::roundtrip_ints(items.into_iter().map(::sifr_runtime::interop::SifrIntBridge::from).collect::<Vec<_>>()).into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int()).collect()"
    );
}

#[test]
fn direct_rust_function_body_converts_borrowed_integer_list_arguments() {
    let func = HirFunction {
        name: "count_ints".to_string(),
        params: vec![HirParam {
            name: "items".to_string(),
            ty: Type::List(Box::new(Type::Int)),
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Int,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["fixture_bridge", "count_ints"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = direct_rust_function_body(&func)
        .expect("direct borrowed integer-list interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct borrowed integer-list interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "fixture_bridge::count_ints(&items.iter().cloned().map(::sifr_runtime::interop::SifrIntBridge::from).collect::<Vec<_>>()).into_sifr_int()"
    );
}

#[test]
fn direct_rust_function_body_converts_int_arguments_and_return() {
    let func = HirFunction {
        name: "weekday".to_string(),
        params: vec![HirParam {
            name: "year".to_string(),
            ty: Type::Int,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Int,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "calendar", "calendar_weekday"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = direct_rust_function_body(&func).expect("direct int interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct int interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "::sifr_stdlib::calendar::calendar_weekday(::sifr_runtime::interop::SifrIntBridge::from(year)).into_sifr_int()"
    );
}

#[test]
fn direct_rust_function_body_converts_optional_int_arguments() {
    let func = HirFunction {
        name: "url_build".to_string(),
        params: vec![
            HirParam {
                name: "query".to_string(),
                ty: Type::Union(vec![Type::Str, Type::None]),
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            },
            HirParam {
                name: "port".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            },
        ],
        return_type: Type::Str,
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "url", "url_build_parts"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = direct_rust_function_body(&func)
        .expect("direct optional-int interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct optional-int interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "::sifr_stdlib::url::url_build_parts(query.clone(), port.as_ref().map(::sifr_runtime::interop::SifrIntBridge::from))"
    );
}

#[test]
fn direct_rust_function_body_converts_integer_list_return() {
    let func = HirFunction {
        name: "monthrange".to_string(),
        params: vec![HirParam {
            name: "month".to_string(),
            ty: Type::Int,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::List(Box::new(Type::Int)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "calendar", "calendar_monthrange"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body = direct_rust_function_body(&func)
        .expect("direct integer-list interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct integer-list interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "::sifr_stdlib::calendar::calendar_monthrange(::sifr_runtime::interop::SifrIntBridge::from(month)).into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int()).collect()"
    );
}

#[test]
fn direct_rust_function_body_maps_result_error_return() {
    let parse_error = error_type("ParseError", vec![("message", Type::Str)]);
    let func = HirFunction {
        name: "base64_decode".to_string(),
        params: vec![HirParam {
            name: "s".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Result(Box::new(Type::Str), Box::new(parse_error)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "base64", "base64_decode"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        direct_rust_function_body(&func).expect("direct result interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct result interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "::sifr_stdlib::base64::base64_decode(s).map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(|__sifr_bridge_error| ParseError { message: __sifr_bridge_error.to_string() })"
    );
}

#[test]
fn direct_rust_function_body_maps_io_error_through_kind_helper() {
    let io_error = error_type("IOError", vec![("message", Type::Str), ("kind", Type::Str)]);
    let func = HirFunction {
        name: "read_text".to_string(),
        params: vec![HirParam {
            name: "path".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Result(Box::new(Type::Str), Box::new(io_error)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "fs", "read_text"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        direct_rust_function_body(&func).expect("direct result interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct result interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "::sifr_stdlib::fs::read_text(path).map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))"
    );
}

#[test]
fn direct_rust_function_body_maps_string_error_fields() {
    let regex_error = error_type(
        "RegexError",
        vec![("message", Type::Str), ("detail", Type::Str)],
    );
    let func = HirFunction {
        name: "re_match".to_string(),
        params: vec![HirParam {
            name: "pattern".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Result(Box::new(Type::Bool), Box::new(regex_error)),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "regex", "re_match"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        direct_rust_function_body(&func).expect("direct result interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct result interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "::sifr_stdlib::regex::re_match(pattern).map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(|__sifr_bridge_error| RegexError { message: __sifr_bridge_error.to_string(), detail: __sifr_bridge_error.to_string() })"
    );
}

#[test]
fn direct_rust_function_body_maps_json_decode_error_fields() {
    let json_decode_error = error_type(
        "JSONDecodeError",
        vec![
            ("message", Type::Str),
            ("line", Type::Int),
            ("column", Type::Int),
        ],
    );
    let func = HirFunction {
        name: "json_load_tokens".to_string(),
        params: vec![HirParam {
            name: "text".to_string(),
            ty: Type::Str,
            default: None,
            keyword_only: false,
            convention: ParamConvention::borrow(),
        }],
        return_type: Type::Result(
            Box::new(Type::List(Box::new(Type::Str))),
            Box::new(json_decode_error),
        ),
        body: Vec::new(),
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Function,
            &["sifr_stdlib", "json", "json_load_tokens"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    let body =
        direct_rust_function_body(&func).expect("direct result interop should lower to a body");
    let [RustStmt::Return(Some(expr))] = body.as_slice() else {
        panic!("direct result interop should lower to a return expression");
    };

    assert_eq!(
        render_expr(expr),
        "::sifr_stdlib::json::json_load_tokens(text).map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(|__sifr_bridge_error| JSONDecodeError { message: __sifr_bridge_error.message().to_string(), line: SifrInt::from(__sifr_bridge_error.line()), column: SifrInt::from(__sifr_bridge_error.column()) })"
    );
}

#[test]
fn direct_rust_function_body_skips_reserved_bridge_roots() {
    let mut func = HirFunction {
        name: "digest".to_string(),
        params: Vec::new(),
        return_type: Type::None,
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
    assert_eq!(direct_rust_function_body(&func), None);

    func.rust_interop[0] = declaration(RustInteropDecoratorKind::Function, &["Self", "poll"]);
    assert_eq!(direct_rust_function_body(&func), None);
}

#[test]
fn direct_rust_function_body_awaits_async_targets() {
    let func = HirFunction {
        name: "fetch".to_string(),
        params: Vec::new(),
        return_type: Type::Bool,
        body: Vec::new(),
        is_async: true,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: vec![declaration(
            RustInteropDecoratorKind::Async,
            &["remote", "fetch_ready"],
        )],
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    assert_eq!(
        direct_rust_function_body(&func),
        Some(vec![RustStmt::Return(Some(RustExpr::Await(Box::new(
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "remote".to_string(),
                    "fetch_ready".to_string()
                ])),
                args: Vec::new(),
            }
        ))))])
    );
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
