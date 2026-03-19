//! Expression lowering scaffolds for the IR lowering.

use crate::{CodegenError, RustExpr, RustLiteral, RustParam, RustStmt, RustType};
use sifr_hir::{HirExpr, HirFStringPart, HirParam};
use sifr_type_system::Type;
use std::cell::RefCell;

thread_local! {
    static ALLOWED_PLAIN_CALLS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn with_allowed_plain_calls<T>(allowed_calls: &[String], f: impl FnOnce() -> T) -> T {
    ALLOWED_PLAIN_CALLS.with(|calls| {
        {
            let mut calls_mut = calls.borrow_mut();
            calls_mut.extend(allowed_calls.iter().cloned());
        }
        let result = f();
        {
            let mut calls_mut = calls.borrow_mut();
            let trunc_len = calls_mut.len().saturating_sub(allowed_calls.len());
            calls_mut.truncate(trunc_len);
        }
        result
    })
}

fn is_allowed_plain_call(func: &str) -> bool {
    ALLOWED_PLAIN_CALLS.with(|calls| calls.borrow().iter().any(|name| name == func))
}

fn is_compat_stdlib_alias(func: &str) -> bool {
    func.starts_with("__compat_sifr_")
}

pub fn try_lower_leaf_expr_result(expr: &HirExpr) -> Result<Option<RustExpr>, CodegenError> {
    validate_leaf_expr_shape(expr)?;
    Ok(try_lower_leaf_expr(expr))
}

fn validate_leaf_expr_shape(expr: &HirExpr) -> Result<(), CodegenError> {
    if let HirExpr::Compare {
        ops, comparators, ..
    } = expr
    {
        if !ops.is_empty() && ops.len() != comparators.len() {
            return Err(CodegenError::new(
                "invalid compare expression shape: ops/comparators length mismatch",
            ));
        }
    }
    Ok(())
}

pub(crate) fn is_leaf_expr_candidate(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. }
        | HirExpr::UnaryOp { .. }
        | HirExpr::BinOp { .. }
        | HirExpr::IfExpr { .. }
        | HirExpr::TupleLiteral { .. }
        | HirExpr::ListLiteral { .. }
        | HirExpr::RangeLiteral { .. }
        | HirExpr::FieldAccess { .. }
        | HirExpr::ContainsOp { .. }
        | HirExpr::QuestionMark { .. }
        | HirExpr::OkWrap { .. }
        | HirExpr::ErrWrap { .. }
        | HirExpr::WalrusExpr { .. }
        | HirExpr::SuperCall { .. }
        | HirExpr::FString { .. }
        | HirExpr::Lambda { .. } => true,
        HirExpr::Compare {
            ops, comparators, ..
        } => !ops.is_empty() && ops.len() == comparators.len(),
        HirExpr::BoolOp { values, .. } => values.len() >= 2,
        _ => false,
    }
}

/// Lowers leaf expressions that don't require emitter state.
/// This is the first incremental IR rollout from `emit_expr` string writes
/// to IR + renderer output.
pub fn try_lower_leaf_expr(expr: &HirExpr) -> Option<RustExpr> {
    match expr {
        HirExpr::IntLiteral(v) => Some(RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(RustLiteral::Int(*v))),
            ty: RustType::I64,
        }),
        HirExpr::FloatLiteral(v) => Some(RustExpr::Cast {
            expr: Box::new(RustExpr::Literal(RustLiteral::Float(*v))),
            ty: RustType::F64,
        }),
        HirExpr::StringLiteral(s) => Some(RustExpr::Literal(RustLiteral::Str(s.clone()))),
        HirExpr::BoolLiteral(v) => Some(RustExpr::Literal(RustLiteral::Bool(*v))),
        HirExpr::NoneLiteral => Some(RustExpr::Literal(RustLiteral::None)),
        HirExpr::Name { name, ty }
            if is_bool_like_simple(ty)
                || is_numeric_simple(ty)
                || is_string_like_simple(ty)
                || is_enum_like_simple(ty) =>
        {
            Some(RustExpr::Ident(name.clone()))
        }
        HirExpr::EnumVariant {
            enum_name, variant, ..
        } => Some(RustExpr::Path(vec![enum_name.clone(), variant.clone()])),
        HirExpr::UnaryOp { op, operand, .. } => match op.as_str() {
            "-" => Some(RustExpr::UnaryOp {
                op: "-".to_string(),
                operand: Box::new(try_lower_leaf_expr(operand)?),
            }),
            "+" => Some(try_lower_leaf_expr(operand)?),
            "~" if is_int_like_simple(operand.ty()) => {
                let lowered_operand = try_lower_leaf_expr(operand).or_else(|| {
                    if let HirExpr::Name { name, .. } = operand.as_ref() {
                        return Some(RustExpr::Ident(name.clone()));
                    }
                    None
                })?;
                Some(RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(lowered_operand),
                })
            }
            "not" if is_bool_like_simple(operand.ty()) => {
                let lowered_operand = try_lower_leaf_expr(operand).or_else(|| {
                    if let HirExpr::Name { name, .. } = operand.as_ref() {
                        return Some(RustExpr::Ident(name.clone()));
                    }
                    None
                })?;
                Some(RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(lowered_operand),
                })
            }
            "not" if is_option_like_simple(operand.ty()) => {
                if let HirExpr::Name { name, .. } = operand.as_ref() {
                    return Some(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(name.clone())),
                        method: "is_none".to_string(),
                        args: vec![],
                    });
                }
                None
            }
            _ => None,
        },
        HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } => {
            if (is_option_like_simple(left.ty()) || is_option_like_simple(right.ty()))
                && !is_option_like_simple(ty)
            {
                return None;
            }
            if !is_safe_simple_binop(op, left.ty(), right.ty(), ty) {
                return None;
            }
            if is_mixed_simple_float_binop(op, left.ty(), right.ty(), ty)
                || is_mixed_simple_float_floor_division_binop(op, left.ty(), right.ty(), ty)
                || is_simple_int_true_division_binop(op, left.ty(), right.ty(), ty)
            {
                return Some(RustExpr::BinOp {
                    left: Box::new(try_lower_mixed_float_operand_expr(left)?),
                    op: normalize_binop_op(op).to_string(),
                    right: Box::new(try_lower_mixed_float_operand_expr(right)?),
                });
            }
            Some(RustExpr::BinOp {
                left: Box::new(try_lower_simple_binop_operand_expr(left)?),
                op: normalize_binop_op(op).to_string(),
                right: Box::new(try_lower_simple_binop_operand_expr(right)?),
            })
        }
        HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } if !ops.is_empty() && ops.len() == comparators.len() => {
            if ops.len() == 1 {
                let right = comparators.first()?;
                if let Some(lowered) = try_lower_option_none_compare_expr(left, &ops[0], right) {
                    return Some(lowered);
                }
                if let Some(lowered) = try_lower_none_identity_compare_expr(left, &ops[0], right) {
                    return Some(lowered);
                }
            }

            let mut lhs_expr = left.as_ref();
            let mut lowered_chain: Option<RustExpr> = None;

            for (idx, op) in ops.iter().enumerate() {
                let rhs_expr = comparators.get(idx)?;
                let normalized_op = normalize_compare_op(op);
                if !is_safe_simple_compare(normalized_op, lhs_expr.ty(), rhs_expr.ty()) {
                    return None;
                }

                let cmp = RustExpr::BinOp {
                    left: Box::new(try_lower_simple_compare_operand_expr(lhs_expr)?),
                    op: normalized_op.to_string(),
                    right: Box::new(try_lower_simple_compare_operand_expr(rhs_expr)?),
                };

                lowered_chain = Some(if let Some(existing) = lowered_chain {
                    RustExpr::BinOp {
                        left: Box::new(existing),
                        op: "&&".to_string(),
                        right: Box::new(cmp),
                    }
                } else {
                    cmp
                });

                lhs_expr = rhs_expr;
            }

            lowered_chain
        }
        HirExpr::BoolOp { op, values, .. } if values.len() >= 2 => {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return None,
            };

            let mut iter = values.iter();
            let mut lowered = try_lower_leaf_expr(iter.next()?)?;
            for value in iter {
                lowered = RustExpr::BinOp {
                    left: Box::new(lowered),
                    op: lowered_op.to_string(),
                    right: Box::new(try_lower_leaf_expr(value)?),
                };
            }
            Some(lowered)
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => Some(RustExpr::If {
            cond: Box::new(try_lower_leaf_expr(condition)?),
            then_expr: Box::new(try_lower_leaf_expr(then_expr)?),
            else_expr: Some(Box::new(try_lower_leaf_expr(else_expr)?)),
        }),
        HirExpr::TupleLiteral { elements, .. } => Some(RustExpr::Tuple(
            elements
                .iter()
                .map(try_lower_leaf_expr)
                .collect::<Option<Vec<_>>>()?,
        )),
        HirExpr::ListLiteral { elements, ty } => {
            let list_ty = resolve_alias_type(ty);
            let mut lowered = elements
                .iter()
                .map(try_lower_leaf_expr)
                .collect::<Option<Vec<_>>>()?;
            if matches!(list_ty, Type::Bytes) {
                lowered = lowered
                    .into_iter()
                    .map(|element| RustExpr::Cast {
                        expr: Box::new(element),
                        ty: RustType::Named("u8".to_string()),
                    })
                    .collect();
            }
            Some(RustExpr::Vec(lowered))
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            let lowered_range = RustExpr::Range {
                start: Box::new(try_lower_simple_range_operand_expr(start)?),
                end: Box::new(try_lower_simple_range_operand_expr(end)?),
            };

            if let Some(step_expr) = step.as_ref() {
                Some(RustExpr::MethodCall {
                    receiver: Box::new(lowered_range),
                    method: "step_by".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(try_lower_simple_range_operand_expr(step_expr)?),
                        ty: RustType::Named("usize".to_string()),
                    }],
                })
            } else {
                Some(lowered_range)
            }
        }
        HirExpr::FieldAccess { .. } => None,
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            let collection_ty = resolve_alias_type(collection.ty());
            let method = match collection_ty {
                Type::Dict(_, _) => "contains_key",
                Type::List(_) | Type::Set(_) | Type::Str => "contains",
                _ => return None,
            };
            let arg = RustExpr::Ref {
                mutable: false,
                expr: Box::new(try_lower_leaf_or_name_expr(element)?),
            };
            Some(RustExpr::MethodCall {
                receiver: Box::new(try_lower_leaf_or_name_expr(collection)?),
                method: method.to_string(),
                args: vec![arg],
            })
        }
        HirExpr::QuestionMark { expr, .. } => {
            Some(RustExpr::Try(Box::new(try_lower_leaf_or_name_expr(expr)?)))
        }
        HirExpr::OkWrap { value, .. } => Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        }),
        HirExpr::ErrWrap { value, .. } => Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        }),
        HirExpr::WalrusExpr { name, value, .. } => Some(RustExpr::Block {
            stmts: vec![RustStmt::Let {
                mutable: false,
                name: name.clone(),
                ty: None,
                value: try_lower_leaf_or_name_expr(value)?,
            }],
            expr: Some(Box::new(RustExpr::Ident(name.clone()))),
        }),
        HirExpr::SuperCall {
            parent_class,
            method,
            args,
            ..
        } => {
            let lowered_args = args
                .iter()
                .map(try_lower_leaf_or_name_expr)
                .collect::<Option<Vec<_>>>()?;
            Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![parent_class.clone(), method.clone()])),
                args: lowered_args,
            })
        }
        HirExpr::FString { parts, .. } => try_lower_simple_fstring_expr(parts),
        HirExpr::Lambda { params, body, .. } => try_lower_simple_lambda_expr(params, body),
        HirExpr::Call { func, args, .. } => try_lower_simple_call_expr(func, args),
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } => try_lower_simple_method_call_expr(object, method, args),
        HirExpr::ConstructorCall {
            class_name, args, ..
        } => try_lower_simple_constructor_call_expr(class_name, args),
        HirExpr::Index { object, index, .. } => try_lower_simple_index_expr(object, index),
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            try_lower_simple_slice_expr(object, start.as_deref(), stop.as_deref(), step.as_deref())
        }
        HirExpr::DictLiteral { keys, values, ty } => {
            try_lower_simple_dict_literal_expr(keys, values, ty)
        }
        HirExpr::SetLiteral { elements, ty } => try_lower_simple_set_literal_expr(elements, ty),
        HirExpr::ListComp { .. } | HirExpr::DictComp { .. } | HirExpr::SetComp { .. } => {
            try_lower_simple_comprehension_expr(expr)
        }
        HirExpr::GeneratorExpr {
            expr,
            var,
            iter,
            filter,
            ty,
        } => try_lower_simple_generator_expr(expr, var, iter, filter.as_deref(), ty),
        _ => None,
    }
}

pub(crate) fn try_lower_simple_comprehension_expr(expr: &HirExpr) -> Option<RustExpr> {
    match expr {
        HirExpr::ListComp {
            expr,
            generators,
            ty,
        } => try_lower_simple_list_comp_expr(expr, generators, ty),
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ty,
        } => try_lower_simple_dict_comp_expr(key_expr, val_expr, generators, ty),
        HirExpr::SetComp {
            expr,
            generators,
            ty,
        } => try_lower_simple_set_comp_expr(expr, generators, ty),
        _ => None,
    }
}

fn try_lower_leaf_or_name_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_expr(expr) {
        return Some(lowered);
    }
    if let HirExpr::Name { name, .. } = expr {
        return Some(RustExpr::Ident(name.clone()));
    }
    None
}

fn try_lower_simple_call_expr(func: &str, args: &[HirExpr]) -> Option<RustExpr> {
    if func == "hash" {
        return try_lower_simple_hash_call_expr(args);
    }
    if func == "divmod" {
        return try_lower_simple_divmod_call_expr(args);
    }
    if func == "map" {
        return try_lower_simple_map_call_expr(args);
    }
    if func == "filter" {
        return try_lower_simple_filter_call_expr(args);
    }

    if is_reserved_builtin_call_func(func) {
        return None;
    }
    if is_compat_stdlib_alias(func) {
        return None;
    }
    // Keep namespaced calls on the structured emitter path so ownership/convention
    // handling can use full signature metadata.
    if func.contains("::") {
        return None;
    }
    if !is_allowed_plain_call(func) {
        return None;
    }
    // Result-typed arguments frequently need target-parameter error coercion.
    if args
        .iter()
        .any(|arg| matches!(resolve_alias_type(arg.ty()), Type::Result(_, _)))
    {
        return None;
    }

    let lowered_args = args
        .iter()
        .map(try_lower_leaf_or_name_expr)
        .collect::<Option<Vec<_>>>()?;

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Ident(func.to_string())),
        args: lowered_args,
    })
}

fn try_lower_simple_hash_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [arg] = args else {
        return None;
    };
    let lowered_arg = try_lower_leaf_or_name_expr(arg)?;
    let hasher_ident = "__sifr_hash".to_string();

    Some(RustExpr::Block {
        stmts: vec![
            RustStmt::Let {
                mutable: true,
                name: hasher_ident.clone(),
                ty: None,
                value: RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "std".to_string(),
                        "collections".to_string(),
                        "hash_map".to_string(),
                        "DefaultHasher".to_string(),
                        "new".to_string(),
                    ])),
                    args: vec![],
                },
            },
            RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "hash".to_string(),
                    "Hash".to_string(),
                    "hash".to_string(),
                ])),
                args: vec![
                    RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(lowered_arg),
                    },
                    RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(RustExpr::Ident(hasher_ident.clone())),
                    },
                ],
            }),
        ],
        expr: Some(Box::new(RustExpr::Cast {
            expr: Box::new(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "hash".to_string(),
                    "Hasher".to_string(),
                    "finish".to_string(),
                ])),
                args: vec![RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(hasher_ident)),
                }],
            }),
            ty: RustType::I64,
        })),
    })
}

fn try_lower_simple_divmod_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [left, right] = args else {
        return None;
    };
    let lowered_left = try_lower_leaf_or_name_expr(left)?;
    let lowered_right = try_lower_leaf_or_name_expr(right)?;

    Some(RustExpr::Tuple(vec![
        RustExpr::BinOp {
            left: Box::new(lowered_left.clone()),
            op: "/".to_string(),
            right: Box::new(lowered_right.clone()),
        },
        RustExpr::BinOp {
            left: Box::new(lowered_left),
            op: "%".to_string(),
            right: Box::new(lowered_right),
        },
    ]))
}

fn try_lower_simple_callable_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Lambda { params, body, .. } = expr {
        let lowered_params = params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: RustType::Named("_".to_string()),
            })
            .collect::<Vec<_>>();
        return Some(RustExpr::Closure {
            params: lowered_params,
            body: Box::new(try_lower_leaf_or_name_expr(body)?),
            is_move: false,
        });
    }
    try_lower_leaf_or_name_expr(expr)
}

fn try_lower_simple_map_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [callable, iter] = args else {
        return None;
    };
    let lowered_callable = try_lower_simple_callable_expr(callable)?;
    let lowered_iter = try_lower_leaf_or_name_expr(iter)?;
    let iter_source = match resolve_alias_type(iter.ty()) {
        Type::Iterator(_) | Type::Range => RustExpr::MethodCall {
            receiver: Box::new(lowered_iter),
            method: "into_iter".to_string(),
            args: vec![],
        },
        Type::Str => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_char".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__sifr_char".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        },
        Type::Dict(_, _) => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "keys".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        },
        _ => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "clone".to_string(),
                args: vec![],
            }),
            method: "into_iter".to_string(),
            args: vec![],
        },
    };
    let mapped_iter = RustExpr::MethodCall {
        receiver: Box::new(iter_source),
        method: "map".to_string(),
        args: vec![lowered_callable],
    };
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
        args: vec![mapped_iter],
    })
}

fn try_lower_simple_filter_call_expr(args: &[HirExpr]) -> Option<RustExpr> {
    let [callable, iter] = args else {
        return None;
    };
    let lowered_callable = if let HirExpr::Lambda { params, body, .. } = callable {
        if params.len() != 1 {
            return None;
        }
        let param_name = params[0].name.clone();
        RustExpr::ClosureBlock {
            params: vec![RustParam::Named {
                name: param_name.clone(),
                ty: RustType::Named("_".to_string()),
            }],
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: param_name.clone(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(param_name.clone())),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                },
                RustStmt::Return(Some(try_lower_leaf_or_name_expr(body)?)),
            ],
            is_move: false,
        }
    } else {
        try_lower_simple_callable_expr(callable)?
    };
    let lowered_iter = try_lower_leaf_or_name_expr(iter)?;
    let filtered_iter = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "clone".to_string(),
                args: vec![],
            }),
            method: "into_iter".to_string(),
            args: vec![],
        }),
        method: "filter".to_string(),
        args: vec![lowered_callable],
    };
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "Vec".to_string(),
            "from_iter".to_string(),
        ])),
        args: vec![filtered_iter],
    })
}

fn try_lower_simple_method_call_expr(
    object: &HirExpr,
    method: &str,
    args: &[HirExpr],
) -> Option<RustExpr> {
    // Typed method calls are handled by the method emission path because they
    // frequently need type-specific rewrites.
    if object.ty() != &Type::Any || method == "len" {
        return None;
    }

    let lowered_object = try_lower_leaf_or_name_expr(object)?;
    let lowered_args = args
        .iter()
        .map(try_lower_leaf_or_name_expr)
        .collect::<Option<Vec<_>>>()?;

    Some(RustExpr::MethodCall {
        receiver: Box::new(lowered_object),
        method: method.to_string(),
        args: lowered_args,
    })
}

fn try_lower_dict_get_key_expr(index: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::StringLiteral(value) = index {
        return Some(RustExpr::Ident(format!("{value:?}")));
    }
    Some(RustExpr::Ref {
        mutable: false,
        expr: Box::new(try_lower_leaf_or_name_expr(index)?),
    })
}

fn try_lower_simple_constructor_call_expr(class_name: &str, args: &[HirExpr]) -> Option<RustExpr> {
    if !class_name.contains("::") {
        return None;
    }

    let lowered_args = args
        .iter()
        .map(try_lower_leaf_or_name_expr)
        .collect::<Option<Vec<_>>>()?;

    let mut path = class_name
        .split("::")
        .map(str::to_string)
        .collect::<Vec<_>>();
    path.push("new".to_string());

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(path)),
        args: lowered_args,
    })
}

fn try_lower_simple_defaultdict_index_expr(object: &HirExpr, index: &HirExpr) -> Option<RustExpr> {
    let Type::Alias {
        name: alias_name,
        body,
        ..
    } = object.ty()
    else {
        return None;
    };
    if !alias_name.starts_with("__compat_defaultdict_") {
        return None;
    }
    let Type::Dict(key_ty, value_ty) = body.resolve_alias() else {
        return None;
    };
    let lowered_object = try_lower_leaf_or_name_expr(object)?;
    let lowered_index = try_lower_leaf_or_name_expr(index)?;
    let key_arg = if let HirExpr::StringLiteral(value) = index {
        RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
    } else {
        let _ = key_ty;
        RustExpr::Clone(Box::new(lowered_index))
    };
    let default_expr = match alias_name.as_str() {
        "__compat_defaultdict_int" => RustExpr::Literal(crate::RustLiteral::Int(0)),
        "__compat_defaultdict_list" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
            args: vec![],
        },
        "__compat_defaultdict_set" => RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "HashSet".to_string(),
                "new".to_string(),
            ])),
            args: vec![],
        },
        _ => return None,
    };
    let entry_expr = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(lowered_object),
            method: "entry".to_string(),
            args: vec![key_arg],
        }),
        method: "or_insert".to_string(),
        args: vec![default_expr],
    };
    Some(match resolve_alias_type(value_ty.as_ref()) {
        Type::Int => RustExpr::Deref(Box::new(entry_expr)),
        _ => RustExpr::MethodCall {
            receiver: Box::new(entry_expr),
            method: "clone".to_string(),
            args: vec![],
        },
    })
}

fn try_lower_simple_index_expr(object: &HirExpr, index: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_simple_defaultdict_index_expr(object, index) {
        return Some(lowered);
    }
    match resolve_alias_type(object.ty()) {
        Type::Dict(_, _) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                method: "get".to_string(),
                args: vec![try_lower_dict_get_key_expr(index)?],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        Type::Any => Some(RustExpr::Index {
            expr: Box::new(try_lower_leaf_or_name_expr(object)?),
            index: Box::new(try_lower_leaf_or_name_expr(index)?),
        }),
        _ => None,
    }
}

fn try_lower_simple_slice_expr(
    object: &HirExpr,
    start: Option<&HirExpr>,
    stop: Option<&HirExpr>,
    step: Option<&HirExpr>,
) -> Option<RustExpr> {
    if object.ty() != &Type::Any || step.is_some() {
        return None;
    }

    let lowered_start = start.and_then(try_lower_leaf_or_name_expr).map(Box::new);
    let lowered_stop = stop.and_then(try_lower_leaf_or_name_expr).map(Box::new);

    Some(RustExpr::Slice {
        expr: Box::new(try_lower_leaf_or_name_expr(object)?),
        start: lowered_start,
        stop: lowered_stop,
    })
}

fn try_lower_simple_dict_literal_expr(
    keys: &[HirExpr],
    values: &[HirExpr],
    _ty: &Type,
) -> Option<RustExpr> {
    if keys.len() != values.len() {
        return None;
    }
    let mut entries = Vec::with_capacity(keys.len());
    for (key, value) in keys.iter().zip(values.iter()) {
        let lowered_key = try_lower_leaf_or_name_expr(key)?;
        let lowered_value = try_lower_leaf_or_name_expr(value)?;
        entries.push(RustExpr::Tuple(vec![lowered_key, lowered_value]));
    }

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "HashMap".to_string(),
            "from".to_string(),
        ])),
        args: vec![RustExpr::Array(entries)],
    })
}

fn try_lower_simple_set_literal_expr(elements: &[HirExpr], _ty: &Type) -> Option<RustExpr> {
    let mut lowered_elements = Vec::with_capacity(elements.len());
    for element in elements {
        lowered_elements.push(try_lower_leaf_or_name_expr(element)?);
    }

    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![
            "HashSet".to_string(),
            "from".to_string(),
        ])),
        args: vec![RustExpr::Array(lowered_elements)],
    })
}

fn try_lower_simple_list_comp_expr(
    expr: &HirExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
    ty: &Type,
) -> Option<RustExpr> {
    if generators.is_empty() || !matches!(resolve_alias_type(ty), Type::Any | Type::List(_)) {
        return None;
    }

    let result_ident = "__sifr_list_comp".to_string();
    let mut nested_body = vec![RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(result_ident.clone())),
        method: "push".to_string(),
        args: vec![try_lower_leaf_or_name_expr(expr)?],
    })];

    for (var, iter_expr, maybe_filter) in generators.iter().rev() {
        if var.contains(',') {
            return None;
        }
        let lowered_iter = try_lower_leaf_or_name_expr(iter_expr)?;
        let iter = if matches!(iter_expr.ty(), Type::Range) {
            lowered_iter
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(lowered_iter),
                    method: "clone".to_string(),
                    args: vec![],
                }),
                method: "into_iter".to_string(),
                args: vec![],
            }
        };
        let loop_body = if let Some(filter) = maybe_filter {
            vec![RustStmt::If {
                cond: try_lower_leaf_or_name_expr(filter)?,
                then_body: nested_body,
                else_body: None,
            }]
        } else {
            nested_body
        };
        nested_body = vec![RustStmt::For {
            var: var.clone(),
            iter,
            body: loop_body,
        }];
    }

    let mut stmts = vec![RustStmt::Let {
        mutable: true,
        name: result_ident.clone(),
        ty: None,
        value: RustExpr::Vec(vec![]),
    }];
    stmts.extend(nested_body);

    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident(result_ident))),
    })
}

fn try_lower_simple_dict_comp_expr(
    key_expr: &HirExpr,
    val_expr: &HirExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
    ty: &Type,
) -> Option<RustExpr> {
    if generators.len() != 1 || !matches!(resolve_alias_type(ty), Type::Any | Type::Dict(_, _)) {
        return None;
    }

    let (var, iter_expr, maybe_filter) = generators.first()?;
    if var.contains(',') {
        return None;
    }

    let lowered_iter = try_lower_leaf_or_name_expr(iter_expr)?;
    let iter = if matches!(iter_expr.ty(), Type::Range) {
        lowered_iter
    } else {
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "clone".to_string(),
                args: vec![],
            }),
            method: "into_iter".to_string(),
            args: vec![],
        }
    };

    let result_ident = "__sifr_dict_comp".to_string();
    let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(result_ident.clone())),
        method: "insert".to_string(),
        args: vec![
            try_lower_leaf_or_name_expr(key_expr)?,
            try_lower_leaf_or_name_expr(val_expr)?,
        ],
    });

    let loop_body = if let Some(filter) = maybe_filter {
        vec![RustStmt::If {
            cond: try_lower_leaf_or_name_expr(filter)?,
            then_body: vec![insert_stmt],
            else_body: None,
        }]
    } else {
        vec![insert_stmt]
    };

    let stmts = vec![
        RustStmt::Let {
            mutable: true,
            name: result_ident.clone(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "HashMap".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        },
        RustStmt::For {
            var: var.clone(),
            iter,
            body: loop_body,
        },
    ];

    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident(result_ident))),
    })
}

fn try_lower_simple_set_comp_expr(
    expr: &HirExpr,
    generators: &[(String, HirExpr, Option<HirExpr>)],
    ty: &Type,
) -> Option<RustExpr> {
    if generators.len() != 1 || !matches!(resolve_alias_type(ty), Type::Any | Type::Set(_)) {
        return None;
    }

    let (var, iter_expr, maybe_filter) = generators.first()?;
    if var.contains(',') {
        return None;
    }

    let lowered_iter = try_lower_leaf_or_name_expr(iter_expr)?;
    let iter = if matches!(iter_expr.ty(), Type::Range) {
        lowered_iter
    } else {
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "clone".to_string(),
                args: vec![],
            }),
            method: "into_iter".to_string(),
            args: vec![],
        }
    };

    let result_ident = "__sifr_set_comp".to_string();
    let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(result_ident.clone())),
        method: "insert".to_string(),
        args: vec![try_lower_leaf_or_name_expr(expr)?],
    });

    let loop_body = if let Some(filter) = maybe_filter {
        vec![RustStmt::If {
            cond: try_lower_leaf_or_name_expr(filter)?,
            then_body: vec![insert_stmt],
            else_body: None,
        }]
    } else {
        vec![insert_stmt]
    };

    let stmts = vec![
        RustStmt::Let {
            mutable: true,
            name: result_ident.clone(),
            ty: None,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "HashSet".to_string(),
                    "new".to_string(),
                ])),
                args: vec![],
            },
        },
        RustStmt::For {
            var: var.clone(),
            iter,
            body: loop_body,
        },
    ];

    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident(result_ident))),
    })
}

fn try_lower_simple_generator_expr(
    expr: &HirExpr,
    var: &str,
    iter: &HirExpr,
    filter: Option<&HirExpr>,
    ty: &Type,
) -> Option<RustExpr> {
    if !matches!(resolve_alias_type(ty), Type::Any | Type::Iterator(_))
        || filter.is_some()
        || var.contains(',')
    {
        return None;
    }

    let iter_chain = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(try_lower_leaf_or_name_expr(iter)?),
            method: "clone".to_string(),
            args: vec![],
        }),
        method: "into_iter".to_string(),
        args: vec![],
    };

    Some(RustExpr::MethodCall {
        receiver: Box::new(iter_chain),
        method: "map".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![RustParam::Named {
                name: var.to_string(),
                ty: RustType::Named("_".to_string()),
            }],
            body: Box::new(try_lower_leaf_or_name_expr(expr)?),
            is_move: false,
        }],
    })
}

fn is_reserved_builtin_call_func(func: &str) -> bool {
    matches!(
        func,
        "print"
            | "isinstance"
            | "list"
            | "str"
            | "tuple"
            | "pow"
            | "abs"
            | "hash"
            | "round"
            | "repr"
            | "dict"
            | "int"
            | "bigint"
            | "Decimal"
            | "BigDecimal"
            | "float"
            | "bool"
            | "ord"
            | "chr"
            | "min"
            | "max"
            | "sum"
            | "sorted"
            | "reversed"
            | "enumerate"
            | "zip"
            | "any"
            | "all"
            | "map"
            | "filter"
            | "builtin_open"
    )
}

fn try_lower_simple_fstring_expr(parts: &[HirFStringPart]) -> Option<RustExpr> {
    let mut format_str = String::new();
    let mut lowered_args = Vec::new();

    for part in parts {
        match part {
            HirFStringPart::Literal(s) => {
                for ch in s.chars() {
                    match ch {
                        '{' => format_str.push_str("{{"),
                        '}' => format_str.push_str("}}"),
                        _ => format_str.push(ch),
                    }
                }
            }
            HirFStringPart::Expr(expr) => {
                if is_option_like_simple(expr.ty()) {
                    return None;
                }
                format_str.push_str("{}");
                lowered_args.push(try_lower_leaf_or_name_expr(expr)?);
            }
        }
    }

    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str,
        args: lowered_args,
    })
}

fn try_lower_simple_lambda_expr(params: &[HirParam], body: &HirExpr) -> Option<RustExpr> {
    if params.iter().any(|param| param.ty != Type::Any) {
        return None;
    }

    let lowered_params = params
        .iter()
        .map(|param| RustParam::Named {
            name: param.name.clone(),
            ty: RustType::Named("_".to_string()),
        })
        .collect::<Vec<_>>();

    Some(RustExpr::Closure {
        params: lowered_params,
        body: Box::new(try_lower_leaf_or_name_expr(body)?),
        is_move: false,
    })
}

fn is_numeric_simple(ty: &Type) -> bool {
    normalize_simple_numeric_scalar_type(ty).is_some()
}

fn is_int_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_numeric_scalar_type(ty), Some("int"))
}

fn is_float_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_numeric_scalar_type(ty), Some("float"))
}

fn is_bool_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_compare_scalar_type(ty), Some("bool"))
}

fn is_string_like_simple(ty: &Type) -> bool {
    matches!(normalize_simple_compare_scalar_type(ty), Some("str"))
}

fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias { body, .. } => resolve_alias_type(body),
        _ => ty,
    }
}

fn is_enum_like_simple(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::Enum { .. })
}

fn is_option_like_simple(ty: &Type) -> bool {
    if let Type::Union(members) = resolve_alias_type(ty) {
        let non_none = members.iter().filter(|m| !matches!(m, Type::None)).count();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none == 1
    } else {
        false
    }
}

fn normalize_compare_op(op: &str) -> &str {
    match op {
        "is" => "==",
        "is not" => "!=",
        _ => op,
    }
}

fn normalize_binop_op(op: &str) -> &str {
    match op {
        "//" => "/",
        _ => op,
    }
}

fn is_mixed_simple_float_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    if !matches!(op, "/" | "+" | "-" | "*" | "%") {
        return false;
    }
    if !is_float_like_simple(result_ty) {
        return false;
    }
    (is_int_like_simple(left_ty) && is_float_like_simple(right_ty))
        || (is_float_like_simple(left_ty) && is_int_like_simple(right_ty))
}

fn is_mixed_simple_float_floor_division_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    op == "//"
        && is_float_like_simple(result_ty)
        && ((is_int_like_simple(left_ty) && is_float_like_simple(right_ty))
            || (is_float_like_simple(left_ty) && is_int_like_simple(right_ty)))
}

fn is_simple_int_true_division_binop(
    op: &str,
    left_ty: &Type,
    right_ty: &Type,
    result_ty: &Type,
) -> bool {
    op == "/"
        && is_float_like_simple(result_ty)
        && is_int_like_simple(left_ty)
        && is_int_like_simple(right_ty)
}

fn is_safe_simple_compare(op: &str, left_ty: &Type, right_ty: &Type) -> bool {
    if !matches!(op, "==" | "!=" | "<" | "<=" | ">" | ">=") {
        return false;
    }
    let left_unaliased = resolve_alias_type(left_ty);
    let right_unaliased = resolve_alias_type(right_ty);
    if left_unaliased == right_unaliased && matches!(left_unaliased, Type::TypeVar(_)) {
        return true;
    }
    if left_unaliased == right_unaliased && matches!(left_unaliased, Type::Enum { .. }) {
        return matches!(op, "==" | "!=");
    }
    let left_norm = normalize_simple_compare_scalar_type(left_ty);
    let right_norm = normalize_simple_compare_scalar_type(right_ty);
    left_norm.is_some() && left_norm == right_norm
}

fn is_safe_simple_binop(op: &str, left_ty: &Type, right_ty: &Type, result_ty: &Type) -> bool {
    if op == "//" {
        if is_mixed_simple_float_floor_division_binop(op, left_ty, right_ty, result_ty) {
            return true;
        }
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && (is_int_like_simple(left_ty) || is_float_like_simple(left_ty));
    }
    if op == "/" {
        if is_mixed_simple_float_binop(op, left_ty, right_ty, result_ty)
            || is_simple_int_true_division_binop(op, left_ty, right_ty, result_ty)
        {
            return true;
        }
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && is_float_like_simple(left_ty);
    }
    if matches!(op, "+" | "-" | "*" | "%")
        && is_mixed_simple_float_binop(op, left_ty, right_ty, result_ty)
    {
        return true;
    }
    if matches!(op, "&" | "|" | "^" | "<<" | ">>") {
        return is_same_simple_numeric_kind(left_ty, right_ty)
            && is_same_simple_numeric_kind(left_ty, result_ty)
            && is_int_like_simple(left_ty);
    }
    if !matches!(op, "+" | "-" | "*" | "%") {
        return false;
    }
    is_same_simple_numeric_kind(left_ty, right_ty)
        && is_same_simple_numeric_kind(left_ty, result_ty)
        && is_numeric_simple(left_ty)
}

fn is_same_simple_numeric_kind(left: &Type, right: &Type) -> bool {
    let Some(left_kind) = normalize_simple_numeric_scalar_type(left) else {
        return false;
    };
    normalize_simple_numeric_scalar_type(right).is_some_and(|right_kind| right_kind == left_kind)
}

fn try_lower_option_none_compare_expr(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
) -> Option<RustExpr> {
    let name_expr = if matches!(right, HirExpr::NoneLiteral) {
        left
    } else if matches!(left, HirExpr::NoneLiteral) {
        right
    } else {
        return None;
    };
    let HirExpr::Name { name, ty } = name_expr else {
        return None;
    };
    if !is_option_like_simple(ty) {
        return None;
    }
    let method = match op {
        "is" => "is_none",
        "is not" => "is_some",
        _ => return None,
    };
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(name.clone())),
        method: method.to_string(),
        args: vec![],
    })
}

fn try_lower_none_identity_compare_expr(
    left: &HirExpr,
    op: &str,
    right: &HirExpr,
) -> Option<RustExpr> {
    if !matches!(op, "is" | "is not") {
        return None;
    }
    let other = if matches!(right, HirExpr::NoneLiteral) {
        left
    } else if matches!(left, HirExpr::NoneLiteral) {
        right
    } else {
        return None;
    };
    if !(matches!(other, HirExpr::NoneLiteral)
        || matches!(resolve_alias_type(other.ty()), Type::None))
    {
        return None;
    }
    Some(RustExpr::Literal(RustLiteral::Bool(op == "is")))
}

fn try_lower_simple_range_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty } = expr {
        if is_int_like_simple(ty) {
            return Some(RustExpr::Ident(name.clone()));
        }
        return None;
    }
    if matches!(expr, HirExpr::RangeLiteral { .. }) {
        return None;
    }
    try_lower_leaf_expr(expr)
}

fn try_lower_mixed_float_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    let lowered = try_lower_simple_binop_operand_expr(expr)?;
    if is_int_like_simple(expr.ty()) {
        return Some(RustExpr::Cast {
            expr: Box::new(lowered),
            ty: RustType::F64,
        });
    }
    Some(lowered)
}

fn try_lower_simple_binop_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty } = expr {
        if is_numeric_simple(ty) {
            return Some(RustExpr::Ident(name.clone()));
        }
    }
    try_lower_leaf_expr(expr)
}

fn try_lower_simple_compare_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, ty } = expr {
        if normalize_simple_compare_scalar_type(ty).is_some()
            || is_enum_like_simple(ty)
            || matches!(resolve_alias_type(ty), Type::TypeVar(_))
        {
            return Some(RustExpr::Ident(name.clone()));
        }
    }
    try_lower_leaf_expr(expr)
}

fn normalize_simple_compare_scalar_type(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Alias { body, .. } => normalize_simple_compare_scalar_type(body),
        Type::Int | Type::LiteralInt(_) => Some("int"),
        Type::Float => Some("float"),
        Type::Bool | Type::LiteralBool(_) => Some("bool"),
        Type::Str | Type::LiteralStr(_) => Some("str"),
        _ => None,
    }
}

fn normalize_simple_numeric_scalar_type(ty: &Type) -> Option<&'static str> {
    match ty {
        Type::Alias { body, .. } => normalize_simple_numeric_scalar_type(body),
        Type::Int | Type::LiteralInt(_) => Some("int"),
        Type::Float => Some("float"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_leaf_expr_variants() {
        let int_expr = try_lower_leaf_expr(&HirExpr::IntLiteral(7)).expect("int lowered");
        let str_expr =
            try_lower_leaf_expr(&HirExpr::StringLiteral("ok".to_string())).expect("str lowered");
        let bool_expr = try_lower_leaf_expr(&HirExpr::BoolLiteral(true)).expect("bool lowered");
        let bool_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "ok".to_string(),
            ty: Type::Bool,
        })
        .expect("bool name lowered");
        let none_expr = try_lower_leaf_expr(&HirExpr::NoneLiteral).expect("none lowered");
        let enum_expr = try_lower_leaf_expr(&HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "RED".to_string(),
            ty: sifr_type_system::Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1))],
            },
        })
        .expect("enum variant lowered");

        assert!(matches!(
            int_expr,
            RustExpr::Cast {
                ty: RustType::I64,
                ..
            }
        ));
        assert!(matches!(str_expr, RustExpr::Literal(RustLiteral::Str(_))));
        assert!(matches!(
            bool_expr,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(bool_name_expr, RustExpr::Ident(ref name) if name == "ok"));
        assert!(matches!(none_expr, RustExpr::Literal(RustLiteral::None)));
        assert!(matches!(enum_expr, RustExpr::Path(_)));
    }

    #[test]
    fn leaf_expr_result_reports_invalid_compare_shape() {
        let expr = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["==".to_string()],
            comparators: vec![],
            ty: Type::Bool,
        };
        let err =
            try_lower_leaf_expr_result(&expr).expect_err("invalid compare shape should error");
        assert!(err.message.contains("ops/comparators length mismatch"));
    }

    #[test]
    fn lowers_numeric_name_leaf_expr_variants() {
        let int_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "count".to_string(),
            ty: Type::Int,
        })
        .expect("int name lowered");
        let float_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "ratio".to_string(),
            ty: Type::Float,
        })
        .expect("float name lowered");
        let alias_int_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "index".to_string(),
            ty: Type::alias("Index", Type::Int),
        })
        .expect("alias-int name lowered");
        let alias_float_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "weight".to_string(),
            ty: Type::alias("Weight", Type::Float),
        })
        .expect("alias-float name lowered");

        assert!(matches!(int_name_expr, RustExpr::Ident(name) if name == "count"));
        assert!(matches!(float_name_expr, RustExpr::Ident(name) if name == "ratio"));
        assert!(matches!(alias_int_name_expr, RustExpr::Ident(name) if name == "index"));
        assert!(matches!(alias_float_name_expr, RustExpr::Ident(name) if name == "weight"));
    }

    #[test]
    fn lowers_bool_and_enum_name_leaf_expr_variants() {
        let alias_bool_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "ready".to_string(),
            ty: Type::alias("ReadyFlag", Type::Bool),
        })
        .expect("alias-bool name lowered");
        let enum_ty = Type::Enum {
            name: "Mode".to_string(),
            variants: vec![("A".to_string(), Some(1)), ("B".to_string(), Some(2))],
        };
        let enum_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "mode".to_string(),
            ty: enum_ty.clone(),
        })
        .expect("enum name lowered");
        let alias_enum_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "mode_alias".to_string(),
            ty: Type::alias("ModeAlias", enum_ty),
        })
        .expect("alias-enum name lowered");

        assert!(matches!(alias_bool_name_expr, RustExpr::Ident(name) if name == "ready"));
        assert!(matches!(enum_name_expr, RustExpr::Ident(name) if name == "mode"));
        assert!(matches!(alias_enum_name_expr, RustExpr::Ident(name) if name == "mode_alias"));
    }

    #[test]
    fn lowers_string_name_leaf_expr_variants() {
        let string_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "label".to_string(),
            ty: Type::Str,
        })
        .expect("string name lowered");
        let alias_string_name_expr = try_lower_leaf_expr(&HirExpr::Name {
            name: "title".to_string(),
            ty: Type::alias("Title", Type::Str),
        })
        .expect("alias-string name lowered");

        assert!(matches!(string_name_expr, RustExpr::Ident(name) if name == "label"));
        assert!(matches!(
            alias_string_name_expr,
            RustExpr::Ident(name) if name == "title"
        ));
    }

    #[test]
    fn compat_stdlib_alias_calls_stay_off_plain_call_fast_path() {
        let lowered = try_lower_simple_call_expr(
            "__compat_sifr_math_fmod",
            &[HirExpr::IntLiteral(7), HirExpr::IntLiteral(2)],
        );
        assert!(lowered.is_none());
    }

    #[test]
    fn lowers_simple_compound_expr_variants() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(1)),
            op: "+".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Int,
        };
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(3)),
            ops: vec![">".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };
        let cond = HirExpr::IfExpr {
            condition: Box::new(HirExpr::BoolLiteral(true)),
            then_expr: Box::new(HirExpr::IntLiteral(1)),
            else_expr: Box::new(HirExpr::IntLiteral(0)),
            ty: Type::Int,
        };

        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { .. })
        ));
        assert!(matches!(
            try_lower_leaf_expr(&cmp),
            Some(RustExpr::BinOp { .. })
        ));
        assert!(matches!(
            try_lower_leaf_expr(&cond),
            Some(RustExpr::If { .. })
        ));
    }

    #[test]
    fn lowers_simple_float_division_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(6.0)),
            op: "/".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, .. }) if op == "/"
        ));
    }

    #[test]
    fn lowers_simple_numeric_binop_with_name_operands() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: Type::Int,
            }),
            op: "+".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("int-name binop lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_simple_mixed_int_float_division_with_name_operands() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: Type::Int,
            }),
            op: "/".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Float,
            }),
            ty: Type::Float,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("mixed int/float-name division lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "/"
                    && matches!(
                        left.as_ref(),
                        RustExpr::Cast {
                            expr,
                            ty: RustType::F64
                        } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    )
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_alias_wrapped_numeric_binop_with_name_operands() {
        let alias_int = Type::alias("Meters", Type::Int);
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_int.clone(),
            }),
            op: "+".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: alias_int.clone(),
            }),
            ty: alias_int,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("alias int-name binop lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_simple_alias_base_int_binop_with_name_operands() {
        let alias_int = Type::alias("Meters", Type::Int);
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_int,
            }),
            op: "+".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("alias/base int-name binop lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_alias_wrapped_mixed_int_float_division_with_name_operands() {
        let alias_int = Type::alias("Count", Type::Int);
        let alias_float = Type::alias("Ratio", Type::Float);
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_int,
            }),
            op: "/".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: alias_float.clone(),
            }),
            ty: alias_float,
        };

        let lowered =
            try_lower_leaf_expr(&bin).expect("alias mixed int/float-name division lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "/"
                    && matches!(
                        left.as_ref(),
                        RustExpr::Cast {
                            expr,
                            ty: RustType::F64
                        } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    )
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn lowers_simple_alias_base_float_division_with_name_operands() {
        let alias_float = Type::alias("Ratio", Type::Float);
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: alias_float,
            }),
            op: "/".to_string(),
            right: Box::new(HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Float,
            }),
            ty: Type::Float,
        };

        let lowered = try_lower_leaf_expr(&bin).expect("alias/base float-name division lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn does_not_lower_simple_int_division_binop_with_non_float_result() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(6)),
            op: "/".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Int,
        };
        assert!(try_lower_leaf_expr(&bin).is_none());
    }

    #[test]
    fn lowers_simple_floor_division_int_binop_as_div() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "//".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Int,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, .. }) if op == "/"
        ));
    }

    #[test]
    fn lowers_simple_floor_division_float_binop_as_div() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "//".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, .. }) if op == "/"
        ));
    }

    #[test]
    fn does_not_lower_simple_floor_division_float_binop_with_non_float_result() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "//".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Int,
        };
        assert!(try_lower_leaf_expr(&bin).is_none());
    }

    #[test]
    fn lowers_simple_mixed_int_float_floor_division_binop_as_div_with_casts() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "//".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_float_int_floor_division_binop_as_div_with_casts() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "//".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_int_float_division_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "/".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_float_int_division_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "/".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_int_float_addition_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "+".to_string(),
            right: Box::new(HirExpr::FloatLiteral(2.0)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "+"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_mixed_float_int_modulo_binop() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::FloatLiteral(7.0)),
            op: "%".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "%"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_simple_int_true_division_binop_with_float_casts() {
        let bin = HirExpr::BinOp {
            left: Box::new(HirExpr::IntLiteral(7)),
            op: "/".to_string(),
            right: Box::new(HirExpr::IntLiteral(2)),
            ty: Type::Float,
        };
        assert!(matches!(
            try_lower_leaf_expr(&bin),
            Some(RustExpr::BinOp { op, left, right })
                if op == "/"
                    && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                    && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
        ));
    }

    #[test]
    fn lowers_multi_operand_boolop_variants() {
        let and_expr = HirExpr::BoolOp {
            op: "and".to_string(),
            values: vec![
                HirExpr::BoolLiteral(true),
                HirExpr::BoolLiteral(false),
                HirExpr::BoolLiteral(true),
            ],
            ty: Type::Bool,
        };
        let or_expr = HirExpr::BoolOp {
            op: "or".to_string(),
            values: vec![
                HirExpr::BoolLiteral(true),
                HirExpr::BoolLiteral(false),
                HirExpr::BoolLiteral(true),
            ],
            ty: Type::Bool,
        };

        assert!(matches!(
            try_lower_leaf_expr(&and_expr),
            Some(RustExpr::BinOp { op, .. }) if op == "&&"
        ));
        assert!(matches!(
            try_lower_leaf_expr(&or_expr),
            Some(RustExpr::BinOp { op, .. }) if op == "||"
        ));
    }

    #[test]
    fn lowers_unary_not_with_bool_name_operand() {
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not bool-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn lowers_unary_not_with_option_name_operand() {
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not option-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_unary_not_with_alias_option_name_operand() {
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not alias-option-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_unary_not_with_alias_bool_name_operand() {
        let alias_bool = Type::alias("Decision", Type::Bool);
        let unary = HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                ty: alias_bool,
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary not alias-bool-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn lowers_unary_bitwise_invert_with_int_operand() {
        let unary = HirExpr::UnaryOp {
            op: "~".to_string(),
            operand: Box::new(HirExpr::IntLiteral(7)),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary invert int lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Cast { ty: RustType::I64, .. })
        ));
    }

    #[test]
    fn lowers_unary_bitwise_invert_with_alias_int_name_operand() {
        let alias_int = Type::alias("Bits", Type::Int);
        let unary = HirExpr::UnaryOp {
            op: "~".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "mask".to_string(),
                ty: alias_int,
            }),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&unary).expect("unary invert alias-int-name lowered");
        assert!(matches!(
            lowered,
            RustExpr::UnaryOp {
                op: ref operator,
                operand: ref inner,
            } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "mask")
        ));
    }

    #[test]
    fn does_not_lower_unary_bitwise_invert_with_non_int_operand() {
        let unary = HirExpr::UnaryOp {
            op: "~".to_string(),
            operand: Box::new(HirExpr::BoolLiteral(true)),
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&unary).is_none());
    }

    #[test]
    fn lowers_option_is_none_compare_with_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("option is-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_none_compare_with_alias_option_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias option is-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_not_none_compare_with_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("option is-not-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_not_none_compare_with_alias_option_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias option is-not-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_none_compare_with_reversed_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("reversed option is-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_option_is_not_none_compare_with_reversed_alias_option_name_operand() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
            }],
            ty: Type::Bool,
        };

        let lowered =
            try_lower_leaf_expr(&cmp).expect("reversed alias option is-not-none compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_is_compare_as_eq() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("is compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn lowers_simple_is_not_compare_as_ne() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::IntLiteral(2)],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("is-not compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "!="
        ));
    }

    #[test]
    fn lowers_bool_compare_with_literal_bool_name_operands() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "lhs".to_string(),
                ty: Type::LiteralBool(true),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::Name {
                name: "rhs".to_string(),
                ty: Type::Bool,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("bool/literal-bool compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, left, right }
                if op == "=="
                    && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
        ));
    }

    #[test]
    fn does_not_lower_mismatched_bool_int_compare() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::BoolLiteral(true)),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_string_literal_compare() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::StringLiteral("alpha".to_string())),
            ops: vec!["<".to_string()],
            comparators: vec![HirExpr::StringLiteral("beta".to_string())],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("string compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "<"
        ));
    }

    #[test]
    fn does_not_lower_mismatched_string_int_compare() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::StringLiteral("x".to_string())),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::IntLiteral(1)],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_enum_variant_equality_compare() {
        let enum_ty = Type::Enum {
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
        };
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: enum_ty.clone(),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: enum_ty,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("enum equality compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn does_not_lower_enum_variant_ordering_compare() {
        let enum_ty = Type::Enum {
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
        };
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: enum_ty.clone(),
            }),
            ops: vec!["<".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: enum_ty,
            }],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_alias_wrapped_enum_variant_equality_compare() {
        let alias_enum_ty = Type::alias(
            "ColorAlias",
            Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
            },
        );
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: alias_enum_ty.clone(),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: alias_enum_ty,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias enum equality compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn does_not_lower_alias_wrapped_enum_variant_ordering_compare() {
        let alias_enum_ty = Type::alias(
            "ColorAlias",
            Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
            },
        );
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "RED".to_string(),
                ty: alias_enum_ty.clone(),
            }),
            ops: vec!["<".to_string()],
            comparators: vec![HirExpr::EnumVariant {
                enum_name: "Color".to_string(),
                variant: "BLUE".to_string(),
                ty: alias_enum_ty,
            }],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_alias_wrapped_scalar_compare() {
        let alias_int = Type::alias("Meters", Type::Int);
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: alias_int.clone(),
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::Name {
                name: "y".to_string(),
                ty: alias_int,
            }],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("alias scalar compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp { op, .. } if op == "=="
        ));
    }

    #[test]
    fn does_not_lower_mismatched_alias_wrapped_scalar_compare() {
        let int_alias = Type::alias("Meters", Type::Int);
        let bool_alias = Type::alias("Flag", Type::Bool);
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: int_alias,
            }),
            ops: vec!["==".to_string()],
            comparators: vec![HirExpr::Name {
                name: "ok".to_string(),
                ty: bool_alias,
            }],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_simple_chained_compare_variants() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["<".to_string(), "<".to_string()],
            comparators: vec![HirExpr::IntLiteral(2), HirExpr::IntLiteral(3)],
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&cmp).expect("chained compare lowered");
        assert!(matches!(
            lowered,
            RustExpr::BinOp {
                op: ref top_op,
                left: ref top_left,
                right: ref top_right,
            } if top_op == "&&"
                && matches!(top_left.as_ref(), RustExpr::BinOp { op, .. } if op == "<")
                && matches!(top_right.as_ref(), RustExpr::BinOp { op, .. } if op == "<")
        ));
    }

    #[test]
    fn does_not_lower_option_is_none_compare_with_non_leaf_left() {
        let cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        assert!(try_lower_leaf_expr(&cmp).is_none());
    }

    #[test]
    fn lowers_range_literal_with_step() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(1)),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: Some(Box::new(HirExpr::IntLiteral(2))),
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with step lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver,
                method,
                args,
            } if method == "step_by"
                && matches!(receiver.as_ref(), RustExpr::Range { .. })
                && matches!(
                    args.first(),
                    Some(RustExpr::Cast { ty: RustType::Named(name), .. }) if name == "usize"
                )
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_none_typed_left() {
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered_is = try_lower_leaf_expr(&is_cmp).expect("none identity is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("none identity is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_alias_none_typed_left() {
        let alias_none = Type::alias("Nothing", Type::None);
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none.clone(),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none,
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        };

        let lowered_is = try_lower_leaf_expr(&is_cmp).expect("alias-none identity is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("alias-none identity is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_none_typed_right() {
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            }],
            ty: Type::Bool,
        };

        let lowered_is = try_lower_leaf_expr(&is_cmp).expect("none identity reversed is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("none identity reversed is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_none_identity_compare_with_alias_none_typed_right() {
        let alias_none = Type::alias("Nothing", Type::None);
        let is_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none.clone(),
            }],
            ty: Type::Bool,
        };
        let is_not_cmp = HirExpr::Compare {
            left: Box::new(HirExpr::NoneLiteral),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none,
            }],
            ty: Type::Bool,
        };

        let lowered_is =
            try_lower_leaf_expr(&is_cmp).expect("alias-none identity reversed is lowered");
        let lowered_is_not =
            try_lower_leaf_expr(&is_not_cmp).expect("alias-none identity reversed is-not lowered");

        assert!(matches!(
            lowered_is,
            RustExpr::Literal(RustLiteral::Bool(true))
        ));
        assert!(matches!(
            lowered_is_not,
            RustExpr::Literal(RustLiteral::Bool(false))
        ));
    }

    #[test]
    fn lowers_range_literal_with_name_bounds() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::Name {
                name: "start".to_string(),
                ty: Type::Int,
            }),
            end: Box::new(HirExpr::Name {
                name: "end".to_string(),
                ty: Type::Int,
            }),
            step: None,
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with name bounds lowered");
        assert!(matches!(
            lowered,
            RustExpr::Range { start, end }
                if matches!(start.as_ref(), RustExpr::Ident(name) if name == "start")
                    && matches!(end.as_ref(), RustExpr::Ident(name) if name == "end")
        ));
    }

    #[test]
    fn lowers_range_literal_with_name_step() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(1)),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: Some(Box::new(HirExpr::Name {
                name: "step".to_string(),
                ty: Type::Int,
            })),
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with name step lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall { method, args, .. }
                if method == "step_by"
                    && matches!(
                        args.first(),
                        Some(RustExpr::Cast { expr, ty: RustType::Named(name) })
                            if matches!(expr.as_ref(), RustExpr::Ident(step_name) if step_name == "step")
                                && name == "usize"
                    )
        ));
    }

    #[test]
    fn lowers_range_literal_with_alias_name_bounds() {
        let alias_int = Type::alias("Index", Type::Int);
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::Name {
                name: "start".to_string(),
                ty: alias_int.clone(),
            }),
            end: Box::new(HirExpr::Name {
                name: "end".to_string(),
                ty: alias_int,
            }),
            step: None,
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with alias-name bounds lowered");
        assert!(matches!(
            lowered,
            RustExpr::Range { start, end }
                if matches!(start.as_ref(), RustExpr::Ident(name) if name == "start")
                    && matches!(end.as_ref(), RustExpr::Ident(name) if name == "end")
        ));
    }

    #[test]
    fn lowers_range_literal_with_alias_name_step() {
        let alias_int = Type::alias("Step", Type::Int);
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(1)),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: Some(Box::new(HirExpr::Name {
                name: "step".to_string(),
                ty: alias_int,
            })),
            ty: Type::Range,
        };

        let lowered = try_lower_leaf_expr(&range).expect("range with alias-name step lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall { method, args, .. }
                if method == "step_by"
                    && matches!(
                        args.first(),
                        Some(RustExpr::Cast { expr, ty: RustType::Named(name) })
                            if matches!(expr.as_ref(), RustExpr::Ident(step_name) if step_name == "step")
                                && name == "usize"
                    )
        ));
    }

    #[test]
    fn does_not_lower_range_literal_with_non_int_name_operand() {
        let range = HirExpr::RangeLiteral {
            start: Box::new(HirExpr::Name {
                name: "start".to_string(),
                ty: Type::Bool,
            }),
            end: Box::new(HirExpr::IntLiteral(10)),
            step: None,
            ty: Type::Range,
        };

        assert!(try_lower_leaf_expr(&range).is_none());
    }

    #[test]
    fn does_not_lower_field_access_for_non_self_name() {
        let expr = HirExpr::FieldAccess {
            object: Box::new(HirExpr::Name {
                name: "point".to_string(),
                ty: Type::Class {
                    name: "Point".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                },
            }),
            field: "x".to_string(),
            ty: Type::Int,
        };

        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn does_not_lower_self_field_access() {
        let expr = HirExpr::FieldAccess {
            object: Box::new(HirExpr::Name {
                name: "self".to_string(),
                ty: Type::Class {
                    name: "Point".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                },
            }),
            field: "x".to_string(),
            ty: Type::Int,
        };

        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn does_not_lower_subclass_field_access() {
        let expr = HirExpr::FieldAccess {
            object: Box::new(HirExpr::Name {
                name: "dog".to_string(),
                ty: Type::Class {
                    name: "Dog".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: Some("Animal".to_string()),
                },
            }),
            field: "name".to_string(),
            ty: Type::Str,
        };

        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_contains_for_list_name_collection() {
        let expr = HirExpr::ContainsOp {
            element: Box::new(HirExpr::Name {
                name: "needle".to_string(),
                ty: Type::Int,
            }),
            collection: Box::new(HirExpr::Name {
                name: "haystack".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("contains lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver,
                method,
                args
            } if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "haystack")
                && method == "contains"
                && matches!(
                    args.first(),
                    Some(RustExpr::Ref { expr, .. })
                        if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "needle")
                )
        ));
    }

    #[test]
    fn lowers_contains_for_string_collection_with_borrowed_arg() {
        let expr = HirExpr::ContainsOp {
            element: Box::new(HirExpr::StringLiteral("T".to_string())),
            collection: Box::new(HirExpr::Name {
                name: "current_iso".to_string(),
                ty: Type::Str,
            }),
            ty: Type::Bool,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("string contains lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver,
                method,
                args
            } if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "current_iso")
                && method == "contains"
                && matches!(args.first(), Some(RustExpr::Ref { .. }))
        ));
    }

    #[test]
    fn lowers_question_mark_ok_err_wrap_variants() {
        let q = HirExpr::QuestionMark {
            expr: Box::new(HirExpr::Name {
                name: "res".to_string(),
                ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
            }),
            ty: Type::Int,
        };
        let ok = HirExpr::OkWrap {
            value: Box::new(HirExpr::IntLiteral(1)),
            ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
        };
        let err = HirExpr::ErrWrap {
            value: Box::new(HirExpr::StringLiteral("boom".to_string())),
            ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
        };

        assert!(matches!(try_lower_leaf_expr(&q), Some(RustExpr::Try(_))));
        assert!(matches!(
            try_lower_leaf_expr(&ok),
            Some(RustExpr::FnCall { func, .. })
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Ok".to_string()])
        ));
        assert!(matches!(
            try_lower_leaf_expr(&err),
            Some(RustExpr::FnCall { func, .. })
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Err".to_string()])
        ));
    }

    #[test]
    fn lowers_walrus_expr_with_leaf_value() {
        let expr = HirExpr::WalrusExpr {
            name: "n".to_string(),
            value: Box::new(HirExpr::IntLiteral(3)),
            ty: Type::Int,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("walrus lowered");
        assert!(matches!(
            lowered,
            RustExpr::Block { stmts, expr: Some(inner) }
                if matches!(stmts.first(), Some(RustStmt::Let { name, .. }) if name == "n")
                    && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "n")
        ));
    }

    #[test]
    fn lowers_super_call_with_leaf_args() {
        let expr = HirExpr::SuperCall {
            parent_class: "Base".to_string(),
            method: "new".to_string(),
            args: vec![HirExpr::IntLiteral(1)],
            ty: Type::Class {
                name: "Base".to_string(),
                fields: vec![],
                methods: vec![],
                parent_class: None,
            },
        };

        let lowered = try_lower_leaf_expr(&expr).expect("super call lowered");
        assert!(matches!(
            lowered,
            RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Base".to_string(), "new".to_string()])
                    && args.len() == 1
        ));
    }

    #[test]
    fn does_not_lower_non_path_call_with_leaf_args() {
        let expr = HirExpr::Call {
            func: "compute".to_string(),
            args: vec![
                HirExpr::IntLiteral(1),
                HirExpr::Name {
                    name: "n".to_string(),
                    ty: Type::Int,
                },
            ],
            ty: Type::Int,
        };

        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn does_not_lower_simple_path_call_with_leaf_args() {
        let expr = HirExpr::Call {
            func: "pkg::helper".to_string(),
            args: vec![HirExpr::BoolLiteral(true)],
            ty: Type::Int,
        };

        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn does_not_lower_special_builtin_call() {
        let expr = HirExpr::Call {
            func: "print".to_string(),
            args: vec![HirExpr::StringLiteral("x".to_string())],
            ty: Type::None,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_hash_builtin_call_with_leaf_arg() {
        let expr = HirExpr::Call {
            func: "hash".to_string(),
            args: vec![HirExpr::Name {
                name: "item".to_string(),
                ty: Type::Class {
                    name: "Color".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                },
            }],
            ty: Type::Int,
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_divmod_builtin_call_with_leaf_args() {
        let expr = HirExpr::Call {
            func: "divmod".to_string(),
            args: vec![HirExpr::IntLiteral(17), HirExpr::IntLiteral(5)],
            ty: Type::Tuple(vec![Type::Int, Type::Int]),
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_map_builtin_call_with_typed_lambda() {
        let expr = HirExpr::Call {
            func: "map".to_string(),
            args: vec![
                HirExpr::Lambda {
                    params: vec![HirParam {
                        name: "x".to_string(),
                        ty: Type::Int,
                        default: None,
                        keyword_only: false,
                        convention: sifr_type_system::ParamConvention::borrow(),
                    }],
                    body: Box::new(HirExpr::BinOp {
                        left: Box::new(HirExpr::Name {
                            name: "x".to_string(),
                            ty: Type::Int,
                        }),
                        op: "*".to_string(),
                        right: Box::new(HirExpr::IntLiteral(2)),
                        ty: Type::Int,
                    }),
                    ty: Type::Callable(
                        vec![Type::Int],
                        vec![sifr_type_system::ParamConvention::own()],
                        Box::new(Type::Int),
                    ),
                },
                HirExpr::Name {
                    name: "nums".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                },
            ],
            ty: Type::Iterator(Box::new(Type::Int)),
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn does_not_lower_call_with_non_leaf_arg() {
        let expr = HirExpr::Call {
            func: "compute".to_string(),
            args: vec![HirExpr::ListComp {
                expr: Box::new(HirExpr::Name {
                    name: "x".to_string(),
                    ty: Type::Int,
                }),
                generators: vec![(
                    "x".to_string(),
                    HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                )],
                ty: Type::List(Box::new(Type::Int)),
            }],
            ty: Type::Int,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_simple_method_call_on_any_with_leaf_args() {
        let expr = HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "obj".to_string(),
                ty: Type::Any,
            }),
            method: "work".to_string(),
            args: vec![HirExpr::IntLiteral(2)],
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("method call lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall {
                receiver,
                method,
                args
            } if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "obj")
                && method == "work"
                && args.len() == 1
        ));
    }

    #[test]
    fn does_not_lower_method_call_on_typed_object() {
        let expr = HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            method: "append".to_string(),
            args: vec![HirExpr::IntLiteral(1)],
            ty: Type::None,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn does_not_lower_len_method_call_on_any_object() {
        let expr = HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "obj".to_string(),
                ty: Type::Any,
            }),
            method: "len".to_string(),
            args: vec![],
            ty: Type::Int,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_path_constructor_call_with_leaf_args() {
        let expr = HirExpr::ConstructorCall {
            class_name: "pkg::Widget".to_string(),
            args: vec![HirExpr::IntLiteral(1)],
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("constructor lowered");
        assert!(matches!(
            lowered,
            RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["pkg".to_string(), "Widget".to_string(), "new".to_string()])
                    && args.len() == 1
        ));
    }

    #[test]
    fn does_not_lower_non_path_constructor_call() {
        let expr = HirExpr::ConstructorCall {
            class_name: "Widget".to_string(),
            args: vec![HirExpr::IntLiteral(1)],
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_simple_index_on_any_with_leaf_index() {
        let expr = HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "data".to_string(),
                ty: Type::Any,
            }),
            index: Box::new(HirExpr::IntLiteral(0)),
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("index lowered");
        assert!(matches!(
            lowered,
            RustExpr::Index { expr, index }
                if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "data")
                    && matches!(index.as_ref(), RustExpr::Cast { .. })
        ));
    }

    #[test]
    fn does_not_lower_index_on_typed_object() {
        let expr = HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            index: Box::new(HirExpr::IntLiteral(0)),
            ty: Type::Union(vec![Type::Int, Type::None]),
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_simple_slice_on_any_without_step() {
        let expr = HirExpr::Slice {
            object: Box::new(HirExpr::Name {
                name: "values".to_string(),
                ty: Type::Any,
            }),
            start: Some(Box::new(HirExpr::IntLiteral(1))),
            stop: Some(Box::new(HirExpr::IntLiteral(3))),
            step: None,
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("slice lowered");
        assert!(matches!(
            lowered,
            RustExpr::Slice { expr, start, stop }
                if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "values")
                    && matches!(start.as_ref(), Some(s) if matches!(s.as_ref(), RustExpr::Cast { .. }))
                    && matches!(stop.as_ref(), Some(s) if matches!(s.as_ref(), RustExpr::Cast { .. }))
        ));
    }

    #[test]
    fn does_not_lower_slice_with_step_on_any() {
        let expr = HirExpr::Slice {
            object: Box::new(HirExpr::Name {
                name: "values".to_string(),
                ty: Type::Any,
            }),
            start: None,
            stop: None,
            step: Some(Box::new(HirExpr::IntLiteral(2))),
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn does_not_lower_slice_on_typed_object() {
        let expr = HirExpr::Slice {
            object: Box::new(HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            start: Some(Box::new(HirExpr::IntLiteral(1))),
            stop: Some(Box::new(HirExpr::IntLiteral(3))),
            step: None,
            ty: Type::List(Box::new(Type::Int)),
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_simple_dict_literal_with_leaf_entries() {
        let expr = HirExpr::DictLiteral {
            keys: vec![HirExpr::StringLiteral("k".to_string())],
            values: vec![HirExpr::IntLiteral(1)],
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("dict literal lowered");
        assert!(matches!(
            lowered,
            RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashMap".to_string(), "from".to_string()])
                    && matches!(args.first(), Some(RustExpr::Array(entries)) if !entries.is_empty())
        ));
    }

    #[test]
    fn lowers_dict_literal_with_nested_lowerable_entry() {
        let expr = HirExpr::DictLiteral {
            keys: vec![HirExpr::StringLiteral("k".to_string())],
            values: vec![HirExpr::ListComp {
                expr: Box::new(HirExpr::Name {
                    name: "x".to_string(),
                    ty: Type::Int,
                }),
                generators: vec![(
                    "x".to_string(),
                    HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                )],
                ty: Type::List(Box::new(Type::Int)),
            }],
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_dict_literal_on_typed_dict() {
        let expr = HirExpr::DictLiteral {
            keys: vec![HirExpr::StringLiteral("k".to_string())],
            values: vec![HirExpr::IntLiteral(1)],
            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        };
        let lowered = try_lower_leaf_expr(&expr).expect("typed dict literal lowered");
        assert!(matches!(
            lowered,
            RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashMap".to_string(), "from".to_string()])
        ));
    }

    #[test]
    fn lowers_simple_set_literal_with_leaf_entries() {
        let expr = HirExpr::SetLiteral {
            elements: vec![HirExpr::IntLiteral(1)],
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("set literal lowered");
        assert!(matches!(
            lowered,
            RustExpr::FnCall { func, args }
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashSet".to_string(), "from".to_string()])
                    && matches!(args.first(), Some(RustExpr::Array(entries)) if !entries.is_empty())
        ));
    }

    #[test]
    fn lowers_set_literal_with_nested_lowerable_entry() {
        let expr = HirExpr::SetLiteral {
            elements: vec![HirExpr::ListComp {
                expr: Box::new(HirExpr::Name {
                    name: "x".to_string(),
                    ty: Type::Int,
                }),
                generators: vec![(
                    "x".to_string(),
                    HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                )],
                ty: Type::List(Box::new(Type::Int)),
            }],
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_set_literal_on_typed_set() {
        let expr = HirExpr::SetLiteral {
            elements: vec![HirExpr::IntLiteral(1)],
            ty: Type::Set(Box::new(Type::Int)),
        };
        let lowered = try_lower_leaf_expr(&expr).expect("typed set literal lowered");
        assert!(matches!(
            lowered,
            RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashSet".to_string(), "from".to_string()])
        ));
    }

    #[test]
    fn lowers_simple_list_comp_with_single_generator() {
        let expr = HirExpr::ListComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("list comp lowered");
        assert!(matches!(
            lowered,
            RustExpr::Block { stmts, expr: Some(result) }
                if matches!(stmts.first(), Some(RustStmt::Let { name, mutable, .. }) if name == "__sifr_list_comp" && *mutable)
                    && matches!(stmts.get(1), Some(RustStmt::For { var, .. }) if var == "x")
                    && matches!(result.as_ref(), RustExpr::Ident(name) if name == "__sifr_list_comp")
        ));
    }

    #[test]
    fn lowers_list_comp_with_multiple_generators() {
        let expr = HirExpr::ListComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![
                (
                    "x".to_string(),
                    HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                ),
                (
                    "y".to_string(),
                    HirExpr::Name {
                        name: "other".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                ),
            ],
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_list_comp_on_typed_list() {
        let expr = HirExpr::ListComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::List(Box::new(Type::Int)),
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_simple_dict_comp_with_single_generator() {
        let expr = HirExpr::DictComp {
            key_expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            val_expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("dict comp lowered");
        assert!(matches!(
            lowered,
            RustExpr::Block { stmts, expr: Some(result) }
                if matches!(stmts.first(), Some(RustStmt::Let { name, mutable, .. }) if name == "__sifr_dict_comp" && *mutable)
                    && matches!(stmts.get(1), Some(RustStmt::For { var, .. }) if var == "x")
                    && matches!(result.as_ref(), RustExpr::Ident(name) if name == "__sifr_dict_comp")
        ));
    }

    #[test]
    fn does_not_lower_dict_comp_with_multiple_generators() {
        let expr = HirExpr::DictComp {
            key_expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            val_expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![
                (
                    "x".to_string(),
                    HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                ),
                (
                    "y".to_string(),
                    HirExpr::Name {
                        name: "other".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                ),
            ],
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_dict_comp_on_typed_dict() {
        let expr = HirExpr::DictComp {
            key_expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            val_expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_simple_set_comp_with_single_generator() {
        let expr = HirExpr::SetComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("set comp lowered");
        assert!(matches!(
            lowered,
            RustExpr::Block { stmts, expr: Some(result) }
                if matches!(stmts.first(), Some(RustStmt::Let { name, mutable, .. }) if name == "__sifr_set_comp" && *mutable)
                    && matches!(stmts.get(1), Some(RustStmt::For { var, .. }) if var == "x")
                    && matches!(result.as_ref(), RustExpr::Ident(name) if name == "__sifr_set_comp")
        ));
    }

    #[test]
    fn does_not_lower_set_comp_with_multiple_generators() {
        let expr = HirExpr::SetComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![
                (
                    "x".to_string(),
                    HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                ),
                (
                    "y".to_string(),
                    HirExpr::Name {
                        name: "other".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    None,
                ),
            ],
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_set_comp_on_typed_set() {
        let expr = HirExpr::SetComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::Set(Box::new(Type::Int)),
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_simple_generator_expr_without_filter() {
        let expr = HirExpr::GeneratorExpr {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            var: "x".to_string(),
            iter: Box::new(HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            filter: None,
            ty: Type::Any,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("generator expr lowered");
        assert!(matches!(
            lowered,
            RustExpr::MethodCall { method, args, .. }
                if method == "map"
                    && args.len() == 1
                    && matches!(args.first(), Some(RustExpr::Closure { params, .. }) if params.len() == 1)
        ));
    }

    #[test]
    fn does_not_lower_generator_expr_with_filter() {
        let expr = HirExpr::GeneratorExpr {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            var: "x".to_string(),
            iter: Box::new(HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            filter: Some(Box::new(HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "x".to_string(),
                    ty: Type::Int,
                }),
                ops: vec![">".to_string()],
                comparators: vec![HirExpr::IntLiteral(0)],
                ty: Type::Bool,
            })),
            ty: Type::Any,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_generator_expr_on_typed_iterator_result() {
        let expr = HirExpr::GeneratorExpr {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            var: "x".to_string(),
            iter: Box::new(HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            filter: None,
            ty: Type::Iterator(Box::new(Type::Int)),
        };
        assert!(try_lower_leaf_expr(&expr).is_some());
    }

    #[test]
    fn lowers_fstring_with_leaf_parts() {
        let expr = HirExpr::FString {
            parts: vec![
                HirFStringPart::Literal("value=".to_string()),
                HirFStringPart::Expr(HirExpr::IntLiteral(7)),
            ],
            ty: Type::Str,
        };

        let lowered = try_lower_leaf_expr(&expr).expect("fstring lowered");
        assert!(matches!(
            lowered,
            RustExpr::FormatMacro {
                ref name,
                ref format_str,
                ref args
            } if name == "format" && format_str == "value={}" && args.len() == 1
        ));
    }

    #[test]
    fn does_not_lower_fstring_with_option_expr_part() {
        let expr = HirExpr::FString {
            parts: vec![HirFStringPart::Expr(HirExpr::Name {
                name: "maybe_v".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            })],
            ty: Type::Str,
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }

    #[test]
    fn lowers_lambda_with_any_params_and_leaf_body() {
        let expr = HirExpr::Lambda {
            params: vec![HirParam {
                name: "x".to_string(),
                ty: Type::Any,
                default: None,
                keyword_only: false,
                convention: sifr_type_system::ParamConvention::own(),
            }],
            body: Box::new(HirExpr::IntLiteral(1)),
            ty: Type::Callable(
                vec![Type::Any],
                vec![sifr_type_system::ParamConvention::own()],
                Box::new(Type::Int),
            ),
        };

        let lowered = try_lower_leaf_expr(&expr).expect("lambda lowered");
        assert!(matches!(
            lowered,
            RustExpr::Closure {
                ref params,
                ref body,
                is_move: false
            } if params.len() == 1 && matches!(body.as_ref(), RustExpr::Cast { ty: RustType::I64, .. })
        ));
    }

    #[test]
    fn does_not_lower_lambda_with_typed_param() {
        let expr = HirExpr::Lambda {
            params: vec![HirParam {
                name: "x".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: sifr_type_system::ParamConvention::own(),
            }],
            body: Box::new(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Callable(
                vec![Type::Int],
                vec![sifr_type_system::ParamConvention::own()],
                Box::new(Type::Int),
            ),
        };
        assert!(try_lower_leaf_expr(&expr).is_none());
    }
}
