use sifr_ir::{HirFunction, RustInteropDeclaration, RustInteropDecoratorKind, RustTargetPath};
use sifr_type_system::Type;

use crate::rust_interop_callback::call_scoped_callbacks;
use crate::rust_interop_direct_args::direct_rust_arg_expr;
use crate::rust_interop_direct_collections::{
    bridge_composite_to_sifr_expr, composite_conversion_required,
};
use crate::rust_interop_panic::{
    bridge_declared_error_expr, recoverable_sync_panic_result_expr, stored_rust_panic_error_value,
};
use crate::{RustExpr, RustLiteral, RustMatchArm, RustParam, RustStmt, RustType};

const BRIDGE_ROOT: &str = "bridge";
const SELF_ROOT: &str = "Self";
const OPAQUE_INNER: &str = "__sifr_opaque_inner";

pub(crate) fn rust_interop_function_body(func: &HirFunction) -> Option<Vec<RustStmt>> {
    let declaration = rust_interop_function_declaration(func)?;
    let target = declaration.target.as_ref()?;
    if target
        .segments
        .first()
        .is_some_and(|root| root == SELF_ROOT)
    {
        return None;
    }
    let call = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(rust_function_path(target))),
        args: func
            .params
            .iter()
            .map(|param| direct_rust_arg_expr(param, target, call_scoped_callbacks(func)))
            .collect(),
    };
    let is_async = direct_rust_function_is_async(func, declaration);
    let value = if is_async {
        RustExpr::Await(Box::new(call))
    } else {
        call
    };
    Some(return_stmt_for_type(
        value,
        &func.return_type,
        declaration,
        is_async,
    ))
}

#[cfg(test)]
fn direct_rust_function_body(func: &HirFunction) -> Option<Vec<RustStmt>> {
    let declaration = rust_interop_function_declaration(func)?;
    let target = declaration.target.as_ref()?;
    if target
        .segments
        .first()
        .is_some_and(|root| root == BRIDGE_ROOT || root == SELF_ROOT)
    {
        return None;
    }
    rust_interop_function_body(func)
}

fn direct_rust_return_expr(value: RustExpr, return_type: &Type) -> RustExpr {
    match return_type.resolve_alias() {
        Type::Int => bridge_int_to_i64_expr(value),
        Type::List(inner) if inner.resolve_alias() == &Type::Int => {
            bridge_int_vec_to_i64_vec_expr(value)
        }
        _ if composite_conversion_required(return_type) => {
            bridge_composite_to_sifr_expr(&value, return_type)
        }
        Type::Result(ok, err) => bridge_result_expr(value, ok, err),
        _ => value,
    }
}

pub(crate) fn rust_interop_method_body(
    func: &HirFunction,
    include_receiver: bool,
) -> Option<Vec<RustStmt>> {
    let declaration = rust_interop_function_declaration(func)?;
    let target = declaration.target.as_ref()?;
    let root = target.segments.first()?;
    let (mut body, value) = if root == SELF_ROOT {
        let (binding, call) = self_method_call(func, declaration, target)?;
        (vec![binding], call)
    } else {
        let mut args = if include_receiver && func.method_kind == sifr_ir::MethodKind::Regular {
            vec![RustExpr::Ident("self".to_string())]
        } else {
            Vec::new()
        };
        args.extend(
            func.params
                .iter()
                .map(|param| direct_rust_arg_expr(param, target, call_scoped_callbacks(func))),
        );
        (
            Vec::new(),
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(rust_function_path(target))),
                args,
            },
        )
    };
    let is_async = direct_rust_function_is_async(func, declaration);
    let value = if is_async {
        RustExpr::Await(Box::new(value))
    } else {
        value
    };
    body.extend(return_stmt_for_type(
        value,
        &func.return_type,
        declaration,
        is_async,
    ));
    Some(body)
}

fn self_method_call(
    func: &HirFunction,
    declaration: &RustInteropDeclaration,
    target: &RustTargetPath,
) -> Option<(RustStmt, RustExpr)> {
    let method = target.segments.get(1)?.clone();
    let Type::Result(_, error_type) = func.return_type.resolve_alias() else {
        return None;
    };
    let receiver = opaque_self_receiver(declaration, error_type);
    Some((
        RustStmt::Let {
            mutable: false,
            name: OPAQUE_INNER.to_string(),
            ty: None,
            value: receiver,
        },
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(OPAQUE_INNER.to_string())),
            method,
            args: func
                .params
                .iter()
                .map(|param| direct_rust_arg_expr(param, target, call_scoped_callbacks(func)))
                .collect(),
        },
    ))
}

fn opaque_self_receiver(declaration: &RustInteropDeclaration, error_type: &Type) -> RustExpr {
    let accessor = if declaration.consumes_receiver {
        "into_inner"
    } else {
        "inner_ref"
    };
    let access = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("self".to_string())),
        method: accessor.to_string(),
        args: Vec::new(),
    };
    let closed = bridge_declared_error_expr(
        RustExpr::Literal(RustLiteral::Str("Rust handle is closed".to_string())),
        error_type,
    );
    let panic_name = "__sifr_stored_panic";
    let poisoned =
        stored_rust_panic_error_value(RustExpr::Ident(panic_name.to_string()), error_type)
            .unwrap_or_else(|| {
                bridge_declared_error_expr(RustExpr::Ident(panic_name.to_string()), error_type)
            });
    let mapped = RustExpr::MethodCall {
        receiver: Box::new(access),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: "__sifr_handle_error".to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(RustExpr::Match {
                expr: Box::new(RustExpr::Ident("__sifr_handle_error".to_string())),
                arms: vec![
                    RustMatchArm {
                        pattern: "sifr_runtime::interop::HandleStateError::Closed".to_string(),
                        bindings: Vec::new(),
                        guard: None,
                        body: vec![RustStmt::TailExpr(closed)],
                    },
                    RustMatchArm {
                        pattern: format!(
                            "sifr_runtime::interop::HandleStateError::Poisoned({panic_name})"
                        ),
                        bindings: vec![panic_name.to_string()],
                        guard: None,
                        body: vec![RustStmt::TailExpr(poisoned)],
                    },
                ],
            }),
            is_move: false,
        }],
    };
    RustExpr::Try(Box::new(mapped))
}

fn return_stmt_for_type(
    value: RustExpr,
    return_type: &Type,
    declaration: &RustInteropDeclaration,
    is_async: bool,
) -> Vec<RustStmt> {
    if return_type == &sifr_type_system::Type::None {
        vec![RustStmt::Expr(value)]
    } else {
        let converted = if is_async {
            direct_rust_return_expr(value, return_type)
        } else if let Type::Result(ok, err) = return_type.resolve_alias() {
            let successful_call =
                bridge_result_expr(RustExpr::Ident("__sifr_bridge_result".to_string()), ok, err);
            recoverable_sync_panic_result_expr(
                value.clone(),
                return_type,
                declaration,
                successful_call,
            )
            .unwrap_or_else(|| direct_rust_return_expr(value, return_type))
        } else {
            direct_rust_return_expr(value, return_type)
        };
        vec![RustStmt::Return(Some(converted))]
    }
}

fn bridge_result_expr(value: RustExpr, ok_type: &Type, err_type: &Type) -> RustExpr {
    let ok = "__sifr_bridge_ok";
    let err = "__sifr_bridge_error";
    let mapped_ok = RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: ok.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(direct_rust_return_expr(
                RustExpr::Ident(ok.to_string()),
                ok_type,
            )),
            is_move: false,
        }],
    };
    RustExpr::MethodCall {
        receiver: Box::new(mapped_ok),
        method: "map_err".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: err.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(bridge_declared_error_expr(
                RustExpr::Ident(err.to_string()),
                err_type,
            )),
            is_move: false,
        }],
    }
}

fn bridge_int_to_i64_expr(value: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "to_i64_saturating".to_string(),
        args: Vec::new(),
    }
}

fn bridge_int_vec_to_i64_vec_expr(value: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(value),
                method: "into_iter".to_string(),
                args: Vec::new(),
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_bridge_value".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(bridge_int_to_i64_expr(RustExpr::Ident(
                    "__sifr_bridge_value".to_string(),
                ))),
                is_move: false,
            }],
        }),
        method: "collect".to_string(),
        args: Vec::new(),
    }
}

pub(crate) fn i64_vec_to_bridge_int_vec_expr(value: RustExpr, borrowed: bool) -> RustExpr {
    let iter = RustExpr::MethodCall {
        receiver: Box::new(value),
        method: if borrowed {
            "iter".to_string()
        } else {
            "into_iter".to_string()
        },
        args: Vec::new(),
    };
    let values = if borrowed {
        RustExpr::MethodCall {
            receiver: Box::new(iter),
            method: "copied".to_string(),
            args: Vec::new(),
        }
    } else {
        iter
    };
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(values),
            method: "map".to_string(),
            args: vec![sifr_int_bridge_path("from")],
        }),
        method: "collect::<Vec<_>>".to_string(),
        args: Vec::new(),
    }
}

pub(crate) fn sifr_int_bridge_path(method: &str) -> RustExpr {
    RustExpr::Path(vec![
        "sifr_runtime".to_string(),
        "interop".to_string(),
        "SifrIntBridge".to_string(),
        method.to_string(),
    ])
}

fn rust_interop_function_declaration(func: &HirFunction) -> Option<&RustInteropDeclaration> {
    func.rust_interop
        .iter()
        .find(|declaration| is_function_declaration(declaration))
}

fn is_function_declaration(declaration: &RustInteropDeclaration) -> bool {
    matches!(
        declaration.kind,
        RustInteropDecoratorKind::Function | RustInteropDecoratorKind::Async
    ) && declaration.target.is_some()
}

fn rust_function_path(target: &RustTargetPath) -> Vec<String> {
    let Some(root) = target.segments.first() else {
        return Vec::new();
    };
    if root == BRIDGE_ROOT {
        return target.segments.clone();
    }
    target.segments.clone()
}

fn direct_rust_function_is_async(func: &HirFunction, declaration: &RustInteropDeclaration) -> bool {
    func.is_async
        || declaration.kind == RustInteropDecoratorKind::Async
        || declaration.abi_requirements.async_boundary
}

#[cfg(test)]
mod tests {
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
            methods: Vec::new(),
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
            name: "set_from_list".to_string(),
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
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                &["sifr_stdlib", "collections", "set_from_list"],
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
            render_expr(&expr),
            "::sifr_stdlib::collections::set_from_list(items.into_iter().map(::sifr_runtime::interop::SifrIntBridge::from).collect::<Vec<_>>()).into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating()).collect()"
        );
    }

    #[test]
    fn direct_rust_function_body_converts_borrowed_integer_list_arguments() {
        let func = HirFunction {
            name: "set_len".to_string(),
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
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                &["sifr_stdlib", "collections", "set_len"],
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
            render_expr(&expr),
            "::sifr_stdlib::collections::set_len(&items.iter().copied().map(::sifr_runtime::interop::SifrIntBridge::from).collect::<Vec<_>>()).to_i64_saturating()"
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
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                &["sifr_stdlib", "calendar", "calendar_weekday"],
            )],
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        };

        let body =
            direct_rust_function_body(&func).expect("direct int interop should lower to a body");
        let [RustStmt::Return(Some(expr))] = body.as_slice() else {
            panic!("direct int interop should lower to a return expression");
        };

        assert_eq!(
            render_expr(&expr),
            "::sifr_stdlib::calendar::calendar_weekday(::sifr_runtime::interop::SifrIntBridge::from(year)).to_i64_saturating()"
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
            render_expr(&expr),
            "::sifr_stdlib::url::url_build_parts(query.clone(), port.map(::sifr_runtime::interop::SifrIntBridge::from))"
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
            render_expr(&expr),
            "::sifr_stdlib::calendar::calendar_monthrange(::sifr_runtime::interop::SifrIntBridge::from(month)).into_iter().map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating()).collect()"
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
            render_expr(&expr),
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
            render_expr(&expr),
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
            render_expr(&expr),
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
            render_expr(&expr),
            "::sifr_stdlib::json::json_load_tokens(text).map(|__sifr_bridge_ok| __sifr_bridge_ok).map_err(|__sifr_bridge_error| JSONDecodeError { message: __sifr_bridge_error.message().to_string(), line: __sifr_bridge_error.line() as i64, column: __sifr_bridge_error.column() as i64 })"
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
}
