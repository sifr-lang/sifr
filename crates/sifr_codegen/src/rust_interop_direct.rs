use sifr_ir::{HirFunction, RustInteropDeclaration, RustInteropDecoratorKind, RustTargetPath};
use sifr_type_system::Type;

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
        func: Box::new(RustExpr::Path(rust_function_path(target, func))),
        args: func
            .params
            .iter()
            .map(|param| direct_rust_arg_expr(param, target, func))
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
        Type::Int => bridge_int_to_sifr_expr(value),
        Type::List(inner) if inner.resolve_alias() == &Type::Int => {
            bridge_int_vec_to_sifr_vec_expr(value)
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
                .map(|param| direct_rust_arg_expr(param, target, func)),
        );
        (
            Vec::new(),
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(rust_function_path(target, func))),
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
                .map(|param| direct_rust_arg_expr(param, target, func))
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

fn bridge_int_to_sifr_expr(value: RustExpr) -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(value),
        method: "into_sifr_int".to_string(),
        args: Vec::new(),
    }
}

fn bridge_int_vec_to_sifr_vec_expr(value: RustExpr) -> RustExpr {
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
                body: Box::new(bridge_int_to_sifr_expr(RustExpr::Ident(
                    "__sifr_bridge_value".to_string(),
                ))),
                is_move: false,
            }],
        }),
        method: "collect".to_string(),
        args: Vec::new(),
    }
}

pub(crate) fn sifr_int_vec_to_bridge_int_vec_expr(value: RustExpr, borrowed: bool) -> RustExpr {
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
            method: "cloned".to_string(),
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

fn rust_function_path(target: &RustTargetPath, func: &HirFunction) -> Vec<String> {
    structural_target_path(target, func)
}

fn structural_target_path(target: &RustTargetPath, func: &HirFunction) -> Vec<String> {
    let mut path = target.segments.clone();
    if func
        .rust_interop
        .iter()
        .any(|declaration| declaration.kind == RustInteropDecoratorKind::Structural)
        && !func.type_params.is_empty()
    {
        if let Some(last) = path.last_mut() {
            last.push_str("::<");
            last.push_str(&func.type_params.join(", "));
            last.push('>');
        }
    }
    path
}

fn direct_rust_function_is_async(func: &HirFunction, declaration: &RustInteropDeclaration) -> bool {
    func.is_async
        || declaration.kind == RustInteropDecoratorKind::Async
        || declaration.abi_requirements.async_boundary
}

#[cfg(test)]
#[path = "../tests/unit/rust_interop_direct/tests.rs"]
mod tests;
