//! Statement lowering scaffolds for the IR migration.

use crate::{try_lower_leaf_expr, CodegenError, RustExpr, RustLiteral, RustParam, RustStmt, RustType};
use sifr_hir::{HirExpr, HirStmt};
use sifr_type_system::Type;
use std::collections::HashSet;

pub fn lower_stmt_raw(raw: &str) -> Result<Vec<RustStmt>, CodegenError> {
    Ok(vec![RustStmt::RawCode(raw.to_string())])
}

/// Lowers an expression statement when the expression is a leaf
/// supported by `try_lower_leaf_expr`.
pub fn try_lower_expr_stmt(expr: &HirExpr) -> Option<Vec<RustStmt>> {
    try_lower_leaf_expr(expr).map(|lowered_expr| vec![RustStmt::Expr(lowered_expr)])
}

#[derive(Clone, Copy, Default)]
pub struct SimpleStmtLoweringCtx<'a> {
    pub return_type: Option<&'a Type>,
    pub in_display_impl: bool,
    pub in_class_scope: bool,
}

#[derive(Clone, Copy)]
struct SimpleStmtBindings<'a> {
    mutated_vars: &'a HashSet<String>,
    borrowed_params: &'a HashSet<String>,
}

/// Lowers statement variants that are context-light and safe to convert
/// without touching complex emitter state.
pub fn try_lower_simple_stmt(
    stmt: &HirStmt,
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
) -> Option<Vec<RustStmt>> {
    try_lower_simple_stmt_with_ctx(
        stmt,
        in_loop_with_else,
        mutated_vars,
        borrowed_params,
        SimpleStmtLoweringCtx::default(),
    )
}

pub(crate) fn try_lower_simple_stmt_with_ctx(
    stmt: &HirStmt,
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let bindings = SimpleStmtBindings {
        mutated_vars,
        borrowed_params,
    };
    match stmt {
        HirStmt::Expr { expr } => try_lower_expr_stmt(expr),
        HirStmt::Let { name, ty, value, .. } if try_lower_simple_let_value(ty, value).is_some() => {
            Some(vec![RustStmt::Let {
                mutable: bindings.mutated_vars.contains(name),
                name: name.clone(),
                ty: Some(crate::sifr_type_to_rust_type(ty)),
                value: try_lower_simple_let_value(ty, value)?,
            }])
        }
        HirStmt::Assign { name, value }
            if try_lower_simple_assign_value(value, bindings.borrowed_params).is_some() =>
        {
            Some(vec![RustStmt::Assign {
                target: crate::RustExpr::Ident(name.clone()),
                value: try_lower_simple_assign_value(value, bindings.borrowed_params)?,
            }])
        }
        HirStmt::AugAssign { name, op, value } => {
            try_lower_simple_augassign_stmt(crate::RustExpr::Ident(name.clone()), op, value)
        }
        HirStmt::AttributeAugAssign {
            object,
            field,
            op,
            value,
        } => try_lower_simple_augassign_stmt(
            RustExpr::Field {
                expr: Box::new(RustExpr::Ident(object.clone())),
                field: field.clone(),
            },
            op,
            value,
        ),
        HirStmt::Return { value: None } => {
            if ctx.in_display_impl {
                return None;
            }
            if ctx.return_type.is_some_and(is_option_like_type) {
                Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                    RustLiteral::None,
                )))])
            } else {
                Some(vec![RustStmt::Return(None)])
            }
        }
        HirStmt::Return { value: Some(value) } => try_lower_simple_return_stmt(value, ctx),
        HirStmt::Assert { test, msg } => {
            let lowered_msg = if let Some(msg_expr) = msg.as_ref() {
                Some(if is_option_like_type(msg_expr.ty()) {
                    RustExpr::MethodCall {
                        receiver: Box::new(try_lower_name_ident_expr(msg_expr)?),
                        method: "map_or".to_string(),
                        args: vec![
                            RustExpr::Literal(RustLiteral::Str("None".to_string())),
                            RustExpr::Closure {
                                params: vec![RustParam::Named {
                                    name: "_v".to_string(),
                                    ty: RustType::Named("_".to_string()),
                                }],
                                body: Box::new(RustExpr::FormatMacro {
                                    name: "format".to_string(),
                                    format_str: "{}".to_string(),
                                    args: vec![RustExpr::Ident("_v".to_string())],
                                }),
                                is_move: false,
                            },
                        ],
                    }
                } else {
                    try_lower_leaf_or_name_expr(msg_expr)?
                })
            } else {
                None
            };
            Some(vec![RustStmt::Assert {
                cond: try_lower_simple_condition_test_expr(test)?,
                msg: lowered_msg,
            }])
        }
        HirStmt::Raise { value } => Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Err".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        }))]),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body: maybe_else_body,
        } => Some(vec![try_lower_simple_if_stmt(
            condition,
            then_body,
            elif_clauses,
            maybe_else_body.as_deref(),
            in_loop_with_else,
            bindings,
            ctx,
        )?]),
        HirStmt::While {
            condition,
            body,
            else_body: None,
        } => Some(vec![RustStmt::While {
            cond: try_lower_simple_condition_test_expr(condition)?,
            // Entering a nested while without else resets loop-else break marker context.
            body: try_lower_simple_stmt_block(
                body,
                false,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?,
        }]),
        HirStmt::While {
            condition,
            body,
            else_body: Some(else_body),
        } => Some(vec![
            RustStmt::Let {
                mutable: true,
                name: "_broke".to_string(),
                ty: None,
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            },
            RustStmt::While {
                cond: try_lower_simple_condition_test_expr(condition)?,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(
                    body,
                    true,
                    bindings.mutated_vars,
                    bindings.borrowed_params,
                    ctx,
                )?,
            },
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::Ident("_broke".to_string())),
                },
                // Else body executes outside this loop scope. Preserve enclosing
                // loop-else context for any break/continue lowering there.
                then_body: try_lower_simple_stmt_block(
                    else_body,
                    in_loop_with_else,
                    bindings.mutated_vars,
                    bindings.borrowed_params,
                    ctx,
                )?,
                else_body: None,
            },
        ]),
        HirStmt::For {
            target,
            iter,
            body,
            else_body: None,
            ..
        } if !target.contains(',') => Some(vec![RustStmt::For {
            var: target.clone(),
            iter: try_lower_leaf_or_name_expr(iter)?,
            // Entering a nested for without else resets loop-else break marker context.
            body: try_lower_simple_stmt_block(
                body,
                false,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?,
        }]),
        HirStmt::For {
            target,
            iter,
            body,
            else_body: Some(else_body),
            ..
        } if !target.contains(',') => Some(vec![
            RustStmt::Let {
                mutable: true,
                name: "_broke".to_string(),
                ty: None,
                value: RustExpr::Literal(RustLiteral::Bool(false)),
            },
            RustStmt::For {
                var: target.clone(),
                iter: try_lower_leaf_or_name_expr(iter)?,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(
                    body,
                    true,
                    bindings.mutated_vars,
                    bindings.borrowed_params,
                    ctx,
                )?,
            },
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::Ident("_broke".to_string())),
                },
                // Else body executes outside this loop scope. Preserve enclosing
                // loop-else context for any break/continue lowering there.
                then_body: try_lower_simple_stmt_block(
                    else_body,
                    in_loop_with_else,
                    bindings.mutated_vars,
                    bindings.borrowed_params,
                    ctx,
                )?,
                else_body: None,
            },
        ]),
        HirStmt::TupleUnpack { targets, value }
            if !targets.is_empty() && try_lower_leaf_or_name_expr(value).is_some() =>
        {
            Some(vec![RustStmt::LetPattern {
                pattern: tuple_unpack_pattern(targets),
                value: try_lower_leaf_or_name_expr(value)?,
            }])
        }
        HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            field_ty,
        } => try_lower_simple_attribute_subscript_assign_stmt(object, field, index, value, field_ty),
        HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
        } if try_lower_leaf_or_name_expr(index).is_some() && try_lower_leaf_or_name_expr(value).is_some() =>
        {
            let lowered_index = try_lower_leaf_or_name_expr(index)?;
            let lowered_value = try_lower_leaf_or_name_expr(value)?;
            match resolve_alias_type(object_ty) {
                Type::List(_) => Some(vec![build_list_subscript_assign_stmt(
                    RustExpr::Ident(object.clone()),
                    lowered_index,
                    lowered_value,
                )]),
                Type::Dict(_, _) => Some(vec![build_dict_subscript_assign_stmt(
                    RustExpr::Ident(object.clone()),
                    lowered_index,
                    lowered_value,
                )]),
                _ => None,
            }
        }
        HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            object_ty: _,
        } if try_lower_leaf_or_name_expr(outer_index).is_some()
            && try_lower_leaf_or_name_expr(inner_index).is_some()
            && try_lower_leaf_or_name_expr(value).is_some() =>
        {
            Some(vec![RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__oi".to_string(),
                    ty: None,
                    value: RustExpr::Cast {
                        expr: Box::new(try_lower_leaf_or_name_expr(outer_index)?),
                        ty: RustType::Named("usize".to_string()),
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__ii".to_string(),
                    ty: None,
                    value: RustExpr::Cast {
                        expr: Box::new(try_lower_leaf_or_name_expr(inner_index)?),
                        ty: RustType::Named("usize".to_string()),
                    },
                },
                RustStmt::IfLet {
                    pattern: "Some(__row)".to_string(),
                    expr: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(object.clone())),
                        method: "get_mut".to_string(),
                        args: vec![RustExpr::Ident("__oi".to_string())],
                    },
                    then_body: vec![RustStmt::IfLet {
                        pattern: "Some(__elem)".to_string(),
                        expr: RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("__row".to_string())),
                            method: "get_mut".to_string(),
                            args: vec![RustExpr::Ident("__ii".to_string())],
                        },
                        then_body: vec![RustStmt::Assign {
                            target: RustExpr::Deref(Box::new(RustExpr::Ident(
                                "__elem".to_string(),
                            ))),
                            value: try_lower_leaf_or_name_expr(value)?,
                        }],
                        else_body: None,
                    }],
                    else_body: None,
                },
            ])])
        }
        HirStmt::SubscriptAugAssign {
            object,
            index,
            op,
            value,
            object_ty,
        } => try_lower_simple_subscript_augassign_stmt(object, index, op, value, object_ty),
        HirStmt::Pass => Some(vec![]),
        HirStmt::Continue => Some(vec![RustStmt::Continue]),
        HirStmt::Break => {
            if in_loop_with_else {
                Some(vec![
                    RustStmt::Assign {
                        target: crate::RustExpr::Ident("_broke".to_string()),
                        value: crate::RustExpr::Literal(crate::RustLiteral::Bool(true)),
                    },
                    RustStmt::Break,
                ])
            } else {
                Some(vec![RustStmt::Break])
            }
        }
        _ => None,
    }
}

fn try_lower_simple_stmt_block(
    stmts: &[HirStmt],
    in_loop_with_else: bool,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let mut lowered = Vec::new();
    for stmt in stmts {
        lowered.extend(try_lower_simple_stmt_with_ctx(
            stmt,
            in_loop_with_else,
            mutated_vars,
            borrowed_params,
            ctx,
        )?);
    }
    Some(lowered)
}

fn tuple_unpack_pattern(targets: &[(String, Type)]) -> String {
    let names = targets
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!("({names})")
}

fn try_lower_simple_if_stmt(
    condition: &HirExpr,
    then_body: &[HirStmt],
    elif_clauses: &[(HirExpr, Vec<HirStmt>)],
    maybe_else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<RustStmt> {
    let mut nested_else = if let Some(else_body) = maybe_else_body {
        Some(try_lower_simple_stmt_block(
            else_body,
            in_loop_with_else,
            bindings.mutated_vars,
            bindings.borrowed_params,
            ctx,
        )?)
    } else {
        None
    };

    for (elif_cond, elif_body) in elif_clauses.iter().rev() {
        nested_else = Some(vec![try_lower_simple_if_clause(
            elif_cond,
            elif_body,
            nested_else,
            in_loop_with_else,
            bindings,
            ctx,
        )?]);
    }

    try_lower_simple_if_clause(
        condition,
        then_body,
        nested_else,
        in_loop_with_else,
        bindings,
        ctx,
    )
}

fn try_lower_simple_if_clause(
    condition: &HirExpr,
    then_body: &[HirStmt],
    nested_else: Option<Vec<RustStmt>>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<RustStmt> {
    let lowered_then_body = try_lower_simple_stmt_block(
        then_body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?;

    if let Some(option_var) = detect_option_truthiness_alias(condition) {
        return Some(RustStmt::IfLet {
            pattern: format!("Some({option_var})"),
            expr: RustExpr::Ident(option_var),
            then_body: lowered_then_body,
            else_body: nested_else,
        });
    }

    Some(RustStmt::If {
        cond: try_lower_leaf_expr(condition)?,
        then_body: lowered_then_body,
        else_body: nested_else,
    })
}

fn try_lower_simple_condition_test_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_expr(expr) {
        return Some(lowered);
    }
    let option_var = detect_option_truthiness_alias(expr)?;
    Some(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(option_var)),
        method: "is_some".to_string(),
        args: vec![],
    })
}

fn resolve_alias_type(ty: &Type) -> &Type {
    match ty {
        Type::Alias(_, inner) => resolve_alias_type(inner),
        _ => ty,
    }
}

fn is_option_like_type(ty: &Type) -> bool {
    if let Type::Union(members) = resolve_alias_type(ty) {
        let non_none = members.iter().filter(|m| !matches!(m, Type::None)).count();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none == 1
    } else {
        false
    }
}

fn detect_option_truthiness_alias(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_like_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

fn is_alias_equivalent_type(left: &Type, right: &Type) -> bool {
    left == right || resolve_alias_type(left) == resolve_alias_type(right)
}

fn is_none_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::None)
}

fn try_lower_name_ident_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let HirExpr::Name { name, .. } = expr {
        return Some(RustExpr::Ident(name.clone()));
    }
    None
}

fn try_lower_leaf_or_name_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_expr(expr) {
        return Some(lowered);
    }
    try_lower_name_ident_expr(expr)
}

fn try_lower_attribute_dict_insert_key_expr(index: &HirExpr, field_ty: &Type) -> Option<RustExpr> {
    let Type::Dict(key_ty, _) = resolve_alias_type(field_ty) else {
        return None;
    };

    if matches!(resolve_alias_type(key_ty), Type::Str | Type::TypeVar(_))
        && matches!(index, HirExpr::Name { .. })
    {
        // Preserve fallback path for potential borrowed-name key cloning semantics.
        return None;
    }

    try_lower_leaf_or_name_expr(index)
}

fn build_list_subscript_assign_stmt(
    receiver: RustExpr,
    lowered_index: RustExpr,
    lowered_value: RustExpr,
) -> RustStmt {
    build_list_get_mut_block_stmt(
        receiver,
        lowered_index,
        RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: lowered_value,
        },
    )
}

fn build_list_get_mut_block_stmt(
    receiver: RustExpr,
    lowered_index: RustExpr,
    then_body_stmt: RustStmt,
) -> RustStmt {
    RustStmt::Block(vec![
        RustStmt::Let {
            mutable: false,
            name: "__idx".to_string(),
            ty: None,
            value: RustExpr::Cast {
                expr: Box::new(lowered_index),
                ty: RustType::Named("usize".to_string()),
            },
        },
        RustStmt::IfLet {
            pattern: "Some(__elem)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(receiver),
                method: "get_mut".to_string(),
                args: vec![RustExpr::Ident("__idx".to_string())],
            },
            then_body: vec![then_body_stmt],
            else_body: None,
        },
    ])
}

fn build_dict_subscript_assign_stmt(
    receiver: RustExpr,
    lowered_index: RustExpr,
    lowered_value: RustExpr,
) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "insert".to_string(),
        args: vec![lowered_index, lowered_value],
    })
}

fn try_lower_simple_attribute_subscript_assign_stmt(
    object: &str,
    field: &str,
    index: &HirExpr,
    value: &HirExpr,
    field_ty: &Type,
) -> Option<Vec<RustStmt>> {
    let lowered_value = try_lower_leaf_or_name_expr(value)?;

    match resolve_alias_type(field_ty) {
        Type::List(_) => Some(vec![build_list_subscript_assign_stmt(
            RustExpr::Field {
                expr: Box::new(RustExpr::Ident(object.to_string())),
                field: field.to_string(),
            },
            try_lower_leaf_or_name_expr(index)?,
            lowered_value,
        )]),
        Type::Dict(_, _) => Some(vec![build_dict_subscript_assign_stmt(
            RustExpr::Field {
                expr: Box::new(RustExpr::Ident(object.to_string())),
                field: field.to_string(),
            },
            try_lower_attribute_dict_insert_key_expr(index, field_ty)?,
            lowered_value,
        )]),
        _ => None,
    }
}

fn try_lower_simple_subscript_augassign_stmt(
    object: &str,
    index: &HirExpr,
    op: &str,
    value: &HirExpr,
    object_ty: &Type,
) -> Option<Vec<RustStmt>> {
    if !is_supported_subscript_augassign_op(op) {
        return None;
    }
    let lowered_index = try_lower_leaf_or_name_expr(index)?;
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    let lowered_body_stmt = build_subscript_augassign_elem_stmt(op, lowered_value);

    match resolve_alias_type(object_ty) {
        Type::List(_) => Some(vec![build_list_get_mut_block_stmt(
            RustExpr::Ident(object.to_string()),
            lowered_index,
            lowered_body_stmt,
        )]),
        Type::Dict(_, _) => Some(vec![RustStmt::IfLet {
            pattern: "Some(__elem)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "get_mut".to_string(),
                args: vec![build_dict_get_mut_key_arg(lowered_index)],
            },
            then_body: vec![lowered_body_stmt],
            else_body: None,
        }]),
        _ => None,
    }
}

fn build_dict_get_mut_key_arg(lowered_index: RustExpr) -> RustExpr {
    if matches!(&lowered_index, RustExpr::Literal(RustLiteral::Str(_))) {
        lowered_index
    } else {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered_index),
        }
    }
}

fn is_supported_subscript_augassign_op(op: &str) -> bool {
    matches!(
        op,
        "+=" | "-=" | "*=" | "/=" | "%=" | "//=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>="
    )
}

fn build_subscript_augassign_elem_stmt(op: &str, lowered_value: RustExpr) -> RustStmt {
    if op == "**=" {
        return RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__elem".to_string())),
                method: "pow".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(lowered_value),
                    ty: RustType::Named("u32".to_string()),
                }],
            },
        };
    }
    if op == "//=" {
        return RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: RustExpr::BinOp {
                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                    "__elem".to_string(),
                )))),
                op: "/".to_string(),
                right: Box::new(lowered_value),
            },
        };
    }
    RustStmt::AugAssign {
        target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
        op: op.strip_suffix('=').unwrap_or(op).to_string(),
        value: lowered_value,
    }
}

fn try_lower_simple_return_stmt(value: &HirExpr, ctx: SimpleStmtLoweringCtx<'_>) -> Option<Vec<RustStmt>> {
    if ctx.in_display_impl || ctx.in_class_scope {
        return None;
    }
    let option_return = ctx.return_type.is_some_and(is_option_like_type);
    if matches!(value.ty(), Type::TypeVar(_)) {
        return None;
    }

    if option_return {
        if is_option_like_type(value.ty()) && !is_none_type(value.ty()) {
            return Some(vec![RustStmt::Return(Some(try_lower_name_ident_expr(value)?))]);
        }
        if matches!(value, HirExpr::NoneLiteral) {
            return Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                RustLiteral::None,
            )))]);
        }
        if is_none_type(value.ty()) {
            if matches!(value, HirExpr::Name { .. }) {
                return Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                    RustLiteral::None,
                )))]);
            }
            return None;
        }
        let lowered = try_lower_leaf_or_name_expr(value)?;
        return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered],
        }))]);
    }
    if let Some(return_ty) = ctx.return_type {
        if let Type::Union(members) = resolve_alias_type(return_ty) {
            if is_option_like_type(value.ty()) && !matches!(value.ty(), Type::None) {
                return None;
            }
            let lowered = try_lower_leaf_or_name_expr(value)?;
            let variant = crate::helpers::find_union_variant(members, value.ty())?;
            let enum_name = return_ty.union_enum_name();
            return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![enum_name, variant])),
                args: vec![lowered],
            }))]);
        }
    }
    if is_option_like_type(value.ty()) && !matches!(value.ty(), Type::None) {
        return Some(vec![RustStmt::Return(Some(RustExpr::MethodCall {
            receiver: Box::new(try_lower_name_ident_expr(value)?),
            method: "unwrap".to_string(),
            args: vec![],
        }))]);
    }
    Some(vec![RustStmt::Return(Some(try_lower_leaf_or_name_expr(value)?))])
}

fn try_lower_simple_let_value(ty: &Type, value: &HirExpr) -> Option<RustExpr> {
    if is_option_like_type(ty) && matches!(value, HirExpr::NoneLiteral) {
        return Some(RustExpr::Literal(RustLiteral::None));
    }
    if is_option_like_type(ty)
        && is_option_like_type(value.ty())
        && !is_none_type(value.ty())
    {
        return try_lower_name_ident_expr(value);
    }
    if is_option_like_type(ty)
        && !is_option_like_type(value.ty())
        && !is_none_type(value.ty())
    {
        return Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![try_lower_leaf_or_name_expr(value)?],
        });
    }
    if is_option_like_type(ty) && is_none_type(value.ty()) {
        if matches!(value, HirExpr::Name { .. }) {
            return Some(RustExpr::Literal(RustLiteral::None));
        }
        return None;
    }
    if is_none_type(ty) && matches!(value, HirExpr::NoneLiteral) {
        return Some(RustExpr::Literal(RustLiteral::Unit));
    }
    if !is_alias_equivalent_type(ty, value.ty()) {
        return None;
    }
    if !matches!(
        resolve_alias_type(ty),
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::Enum { .. } | Type::None
    ) {
        return None;
    }
    try_lower_leaf_or_name_expr(value)
}

fn try_lower_simple_assign_value(value: &HirExpr, borrowed_params: &HashSet<String>) -> Option<RustExpr> {
    // Preserve legacy behavior where TypeVar assignment from borrowed params appends `.clone()`.
    if matches!(value.ty(), Type::TypeVar(_))
        && matches!(value, HirExpr::Name { name, .. } if borrowed_params.contains(name))
    {
        return None;
    }
    try_lower_leaf_or_name_expr(value)
}

fn try_lower_simple_aug_assign_value(op: &str, value: &HirExpr) -> Option<RustExpr> {
    let is_numeric_op = matches!(op, "+=" | "-=" | "*=" | "/=" | "//=" | "%=");
    let is_int_only_op = matches!(op, "&=" | "|=" | "^=" | "<<=" | ">>=");
    let supports_op = match resolve_alias_type(value.ty()) {
        Type::Int | Type::LiteralInt(_) => is_numeric_op || is_int_only_op,
        Type::Float => is_numeric_op,
        _ => false,
    };
    if !supports_op {
        return None;
    }
    try_lower_leaf_or_name_expr(value)
}

fn try_lower_simple_augassign_stmt(target: RustExpr, op: &str, value: &HirExpr) -> Option<Vec<RustStmt>> {
    Some(vec![RustStmt::AugAssign {
        target,
        op: normalize_augassign_op(op),
        value: try_lower_simple_aug_assign_value(op, value)?,
    }])
}

fn normalize_augassign_op(op: &str) -> String {
    if op == "//=" {
        "/".to_string()
    } else {
        op.strip_suffix('=').unwrap_or(op).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_raw_stmt_placeholder() {
        let stmts = lower_stmt_raw("let x = 1;").expect("placeholder lower should succeed");
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], RustStmt::RawCode(_)));
    }

    #[test]
    fn lowers_leaf_expression_statement() {
        let stmts = try_lower_expr_stmt(&HirExpr::IntLiteral(1)).expect("leaf stmt lowered");
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], RustStmt::Expr(_)));
    }

    #[test]
    fn lowers_pass_and_continue_and_break() {
        let pass = try_lower_simple_stmt(&HirStmt::Pass, false, &HashSet::new(), &HashSet::new())
            .expect("pass lowered");
        assert!(pass.is_empty());

        let cont = try_lower_simple_stmt(
            &HirStmt::Continue,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("continue lowered");
        assert!(matches!(cont[0], RustStmt::Continue));

        let brk = try_lower_simple_stmt(&HirStmt::Break, true, &HashSet::new(), &HashSet::new())
            .expect("break lowered");
        assert_eq!(brk.len(), 2);
        assert!(matches!(brk[0], RustStmt::Assign { .. }));
        assert!(matches!(brk[1], RustStmt::Break));
    }

    #[test]
    fn lowers_simple_let_and_assign() {
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Int,
            value: HirExpr::IntLiteral(1),
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::from(["x".to_string()]),
            &HashSet::new(),
        )
        .expect("let lowered");
        assert!(matches!(lowered[0], RustStmt::Let { mutable: true, .. }));

        let assign_stmt = HirStmt::Assign {
            name: "x".to_string(),
            value: HirExpr::IntLiteral(2),
        };
        let lowered = try_lower_simple_stmt(
            &assign_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assign lowered");
        assert!(matches!(lowered[0], RustStmt::Assign { .. }));
    }

    #[test]
    fn lowers_simple_tuple_unpack_stmt() {
        let tuple_unpack = HirStmt::TupleUnpack {
            targets: vec![
                ("a".to_string(), Type::Int),
                ("b".to_string(), Type::Bool),
            ],
            value: HirExpr::TupleLiteral {
                elements: vec![HirExpr::IntLiteral(1), HirExpr::BoolLiteral(true)],
                ty: Type::Tuple(vec![Type::Int, Type::Bool]),
            },
        };
        let lowered = try_lower_simple_stmt(
            &tuple_unpack,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("tuple unpack lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::LetPattern {
                ref pattern,
                value: RustExpr::Tuple(ref elements),
            } if pattern == "(a, b)" && elements.len() == 2
        ));
    }

    #[test]
    fn does_not_lower_tuple_unpack_with_non_leaf_value() {
        let tuple_unpack = HirStmt::TupleUnpack {
            targets: vec![
                ("a".to_string(), Type::Int),
                ("b".to_string(), Type::Bool),
            ],
            value: HirExpr::Call {
                func: "pair".to_string(),
                args: vec![],
                ty: Type::Tuple(vec![Type::Int, Type::Bool]),
            },
        };

        assert!(
            try_lower_simple_stmt(
                &tuple_unpack,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_attribute_list_subscript_assign_stmt() {
        let stmt = HirStmt::AttributeSubscriptAssign {
            object: "self".to_string(),
            field: "items".to_string(),
            index: HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            },
            value: HirExpr::Name {
                name: "v".to_string(),
                ty: Type::Int,
            },
            field_ty: Type::List(Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("attribute list subscript assign lowered");
        assert_eq!(lowered.len(), 1);
        let RustStmt::Block(stmts) = &lowered[0] else {
            panic!("expected block-lowered attribute list subscript assignment");
        };
        assert_eq!(stmts.len(), 2);
        assert!(matches!(
            &stmts[1],
            RustStmt::IfLet {
                pattern,
                expr: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                then_body,
                else_body: None,
            } if pattern == "Some(__elem)"
                && method == "get_mut"
                && matches!(
                    receiver.as_ref(),
                    RustExpr::Field { expr, field }
                        if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                        && field == "items"
                )
                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "__idx")
                && matches!(
                    then_body.first(),
                    Some(RustStmt::Assign {
                        target: RustExpr::Deref(target),
                        value: RustExpr::Ident(rhs),
                    }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && rhs == "v"
                )
        ));
    }

    #[test]
    fn lowers_simple_alias_attribute_list_subscript_assign_stmt() {
        let stmt = HirStmt::AttributeSubscriptAssign {
            object: "self".to_string(),
            field: "items".to_string(),
            index: HirExpr::IntLiteral(0),
            value: HirExpr::IntLiteral(1),
            field_ty: Type::Alias(
                "IntList".to_string(),
                Box::new(Type::List(Box::new(Type::Int))),
            ),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("alias attribute-list subscript assign lowered");
        assert!(matches!(lowered[0], RustStmt::Block(_)));
    }

    #[test]
    fn lowers_simple_attribute_dict_subscript_assign_stmt() {
        let stmt = HirStmt::AttributeSubscriptAssign {
            object: "self".to_string(),
            field: "mapping".to_string(),
            index: HirExpr::Name {
                name: "key".to_string(),
                ty: Type::Int,
            },
            value: HirExpr::Name {
                name: "val".to_string(),
                ty: Type::Int,
            },
            field_ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("attribute dict subscript assign lowered");
        assert!(matches!(
            lowered[0],
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            }) if method == "insert"
                && matches!(
                    recv.as_ref(),
                    RustExpr::Field { expr, field }
                        if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                        && field == "mapping"
                )
                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "key")
                && matches!(args.get(1), Some(RustExpr::Ident(name)) if name == "val")
        ));
    }

    #[test]
    fn lowers_simple_alias_attribute_dict_subscript_assign_stmt() {
        let stmt = HirStmt::AttributeSubscriptAssign {
            object: "self".to_string(),
            field: "mapping".to_string(),
            index: HirExpr::IntLiteral(1),
            value: HirExpr::IntLiteral(2),
            field_ty: Type::Alias(
                "IntMap".to_string(),
                Box::new(Type::Dict(Box::new(Type::Int), Box::new(Type::Int))),
            ),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("alias attribute-dict subscript assign lowered");
        assert!(matches!(lowered[0], RustStmt::Expr(_)));
    }

    #[test]
    fn does_not_lower_attribute_dict_subscript_assign_with_string_name_key() {
        let stmt = HirStmt::AttributeSubscriptAssign {
            object: "self".to_string(),
            field: "mapping".to_string(),
            index: HirExpr::Name {
                name: "k".to_string(),
                ty: Type::Str,
            },
            value: HirExpr::IntLiteral(1),
            field_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        };

        assert!(
            try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none()
        );
    }

    #[test]
    fn does_not_lower_attribute_subscript_assign_with_non_leaf_index() {
        let stmt = HirStmt::AttributeSubscriptAssign {
            object: "self".to_string(),
            field: "items".to_string(),
            index: HirExpr::Call {
                func: "next_idx".to_string(),
                args: vec![],
                ty: Type::Int,
            },
            value: HirExpr::IntLiteral(1),
            field_ty: Type::List(Box::new(Type::Int)),
        };

        assert!(
            try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none()
        );
    }

    #[test]
    fn lowers_simple_list_subscript_assign_stmt() {
        let stmt = HirStmt::SubscriptAssign {
            object: "items".to_string(),
            index: HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            },
            value: HirExpr::Name {
                name: "v".to_string(),
                ty: Type::Int,
            },
            object_ty: Type::List(Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("list subscript assign lowered");
        assert_eq!(lowered.len(), 1);
        let RustStmt::Block(stmts) = &lowered[0] else {
            panic!("expected block-lowered list subscript assignment");
        };
        assert_eq!(stmts.len(), 2);
        assert!(matches!(
            stmts[0],
            RustStmt::Let {
                mutable: false,
                ref name,
                value: RustExpr::Cast {
                    expr: ref inner,
                    ty: RustType::Named(ref usize_ty),
                },
                ..
            } if name == "__idx"
                && usize_ty == "usize"
                && matches!(inner.as_ref(), RustExpr::Ident(idx) if idx == "i")
        ));
        assert!(matches!(
            stmts[1],
            RustStmt::IfLet {
                ref pattern,
                expr: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                then_body: ref body,
                else_body: None,
            } if pattern == "Some(__elem)"
                && method == "get_mut"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "items")
                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "__idx")
                && matches!(
                    body.first(),
                    Some(RustStmt::Assign {
                        target: RustExpr::Deref(target),
                        value: RustExpr::Ident(rhs),
                    }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && rhs == "v"
                )
        ));
    }

    #[test]
    fn lowers_simple_alias_list_subscript_assign_stmt() {
        let stmt = HirStmt::SubscriptAssign {
            object: "items".to_string(),
            index: HirExpr::IntLiteral(0),
            value: HirExpr::IntLiteral(1),
            object_ty: Type::Alias(
                "IntList".to_string(),
                Box::new(Type::List(Box::new(Type::Int))),
            ),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("alias-list subscript assign lowered");
        assert!(matches!(lowered[0], RustStmt::Block(_)));
    }

    #[test]
    fn lowers_simple_dict_subscript_assign_stmt() {
        let stmt = HirStmt::SubscriptAssign {
            object: "mapping".to_string(),
            index: HirExpr::Name {
                name: "key".to_string(),
                ty: Type::Str,
            },
            value: HirExpr::Name {
                name: "val".to_string(),
                ty: Type::Int,
            },
            object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("dict subscript assign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            }) if method == "insert"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "mapping")
                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "key")
                && matches!(args.get(1), Some(RustExpr::Ident(name)) if name == "val")
        ));
    }

    #[test]
    fn does_not_lower_subscript_assign_with_non_leaf_index() {
        let stmt = HirStmt::SubscriptAssign {
            object: "items".to_string(),
            index: HirExpr::Call {
                func: "next_idx".to_string(),
                args: vec![],
                ty: Type::Int,
            },
            value: HirExpr::IntLiteral(1),
            object_ty: Type::List(Box::new(Type::Int)),
        };

        assert!(
            try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none()
        );
    }

    #[test]
    fn lowers_simple_nested_subscript_assign_stmt() {
        let stmt = HirStmt::NestedSubscriptAssign {
            object: "matrix".to_string(),
            outer_index: HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            },
            inner_index: HirExpr::Name {
                name: "j".to_string(),
                ty: Type::Int,
            },
            value: HirExpr::Name {
                name: "v".to_string(),
                ty: Type::Int,
            },
            object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("nested subscript assign lowered");
        assert_eq!(lowered.len(), 1);
        let RustStmt::Block(stmts) = &lowered[0] else {
            panic!("expected block-lowered nested subscript assignment");
        };
        assert_eq!(stmts.len(), 3);
        assert!(matches!(
            &stmts[0],
            RustStmt::Let {
                ref name,
                value: RustExpr::Cast { expr, ty: RustType::Named(ref usize_ty), .. },
                ..
            } if name == "__oi"
                && usize_ty == "usize"
                && matches!(expr.as_ref(), RustExpr::Ident(idx) if idx == "i")
        ));
        assert!(matches!(
            &stmts[1],
            RustStmt::Let {
                ref name,
                value: RustExpr::Cast { expr, ty: RustType::Named(ref usize_ty), .. },
                ..
            } if name == "__ii"
                && usize_ty == "usize"
                && matches!(expr.as_ref(), RustExpr::Ident(idx) if idx == "j")
        ));
        assert!(matches!(
            &stmts[2],
            RustStmt::IfLet {
                ref pattern,
                expr: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                then_body: ref outer_body,
                else_body: None,
            } if pattern == "Some(__row)"
                && method == "get_mut"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "matrix")
                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "__oi")
                && matches!(
                    outer_body.first(),
                    Some(RustStmt::IfLet {
                        pattern: inner_pattern,
                        expr: RustExpr::MethodCall {
                            receiver: inner_recv,
                            method: inner_method,
                            args: inner_args,
                        },
                        then_body: inner_then,
                        else_body: None,
                    }) if inner_pattern == "Some(__elem)"
                        && inner_method == "get_mut"
                        && matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "__row")
                        && matches!(inner_args.first(), Some(RustExpr::Ident(name)) if name == "__ii")
                        && matches!(
                            inner_then.first(),
                            Some(RustStmt::Assign {
                                target: RustExpr::Deref(target),
                                value: RustExpr::Ident(rhs),
                            }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                                && rhs == "v"
                        )
                )
        ));
    }

    #[test]
    fn does_not_lower_nested_subscript_assign_with_non_leaf_inner_index() {
        let stmt = HirStmt::NestedSubscriptAssign {
            object: "matrix".to_string(),
            outer_index: HirExpr::IntLiteral(0),
            inner_index: HirExpr::Call {
                func: "inner_idx".to_string(),
                args: vec![],
                ty: Type::Int,
            },
            value: HirExpr::IntLiteral(1),
            object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
        };

        assert!(
            try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none()
        );
    }

    #[test]
    fn lowers_simple_list_subscript_augassign_plus_equal_stmt() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "items".to_string(),
            index: HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            },
            op: "+=".to_string(),
            value: HirExpr::Name {
                name: "delta".to_string(),
                ty: Type::Int,
            },
            object_ty: Type::List(Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("list subscript augassign lowered");
        let RustStmt::Block(stmts) = &lowered[0] else {
            panic!("expected block-lowered list subscript augassign");
        };
        assert_eq!(stmts.len(), 2);
        assert!(matches!(
            &stmts[1],
            RustStmt::IfLet {
                then_body,
                ..
            } if matches!(
                then_body.first(),
                Some(RustStmt::AugAssign {
                    target: RustExpr::Deref(target),
                    op,
                    value: RustExpr::Ident(rhs),
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && op == "+"
                    && rhs == "delta"
            )
        ));
    }

    #[test]
    fn lowers_simple_list_subscript_augassign_bitwise_and_shift_ops() {
        for (op, expected) in [
            ("&=", "&"),
            ("|=", "|"),
            ("^=", "^"),
            ("<<=", "<<"),
            (">>=", ">>"),
        ] {
            let stmt = HirStmt::SubscriptAugAssign {
                object: "items".to_string(),
                index: HirExpr::IntLiteral(0),
                op: op.to_string(),
                value: HirExpr::Name {
                    name: "rhs".to_string(),
                    ty: Type::Int,
                },
                object_ty: Type::List(Box::new(Type::Int)),
            };
            let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
                .expect("list subscript bitwise/shift augassign lowered");
            let RustStmt::Block(stmts) = &lowered[0] else {
                panic!("expected block-lowered list subscript bitwise/shift augassign");
            };
            assert!(matches!(
                &stmts[1],
                RustStmt::IfLet {
                    then_body,
                    ..
                } if matches!(
                    then_body.first(),
                    Some(RustStmt::AugAssign {
                        target: RustExpr::Deref(target),
                        op,
                        value: RustExpr::Ident(rhs),
                    }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && op == expected
                        && rhs == "rhs"
                )
            ));
        }
    }

    #[test]
    fn lowers_simple_dict_subscript_augassign_with_name_key() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "mapping".to_string(),
            index: HirExpr::Name {
                name: "key".to_string(),
                ty: Type::Str,
            },
            op: "+=".to_string(),
            value: HirExpr::Name {
                name: "delta".to_string(),
                ty: Type::Int,
            },
            object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("dict subscript augassign lowered");
        assert!(matches!(
            lowered[0],
            RustStmt::IfLet {
                ref pattern,
                expr: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                then_body: ref body,
                else_body: None,
            } if pattern == "Some(__elem)"
                && method == "get_mut"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "mapping")
                && matches!(
                    args.first(),
                    Some(RustExpr::Ref {
                        mutable: false,
                        expr
                    }) if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "key")
                )
                && matches!(
                    body.first(),
                    Some(RustStmt::AugAssign {
                        target: RustExpr::Deref(target),
                        op,
                        value: RustExpr::Ident(rhs),
                    }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && op == "+"
                        && rhs == "delta"
                )
        ));
    }

    #[test]
    fn lowers_simple_dict_subscript_augassign_with_string_literal_key() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "mapping".to_string(),
            index: HirExpr::StringLiteral("k".to_string()),
            op: "-=".to_string(),
            value: HirExpr::IntLiteral(1),
            object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("dict subscript augassign lowered");
        assert!(matches!(
            lowered[0],
            RustStmt::IfLet {
                expr: RustExpr::MethodCall { ref args, .. },
                ..
            } if matches!(
                args.first(),
                Some(RustExpr::Literal(RustLiteral::Str(key))) if key == "k"
            )
        ));
    }

    #[test]
    fn lowers_simple_alias_dict_subscript_augassign_stmt() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "mapping".to_string(),
            index: HirExpr::Name {
                name: "key".to_string(),
                ty: Type::Str,
            },
            op: "|=".to_string(),
            value: HirExpr::IntLiteral(2),
            object_ty: Type::Alias(
                "IntMap".to_string(),
                Box::new(Type::Dict(Box::new(Type::Str), Box::new(Type::Int))),
            ),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("alias-dict subscript augassign lowered");
        assert!(matches!(lowered[0], RustStmt::IfLet { .. }));
    }

    #[test]
    fn lowers_simple_list_subscript_augassign_floor_div_equal_stmt() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "items".to_string(),
            index: HirExpr::IntLiteral(0),
            op: "//=".to_string(),
            value: HirExpr::Name {
                name: "d".to_string(),
                ty: Type::Int,
            },
            object_ty: Type::List(Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("list subscript floor-div augassign lowered");
        let RustStmt::Block(stmts) = &lowered[0] else {
            panic!("expected block-lowered list subscript floor-div augassign");
        };
        assert!(matches!(
            &stmts[1],
            RustStmt::IfLet {
                then_body,
                ..
            } if matches!(
                then_body.first(),
                Some(RustStmt::Assign {
                    target: RustExpr::Deref(target),
                    value: RustExpr::BinOp { left, op, right },
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && matches!(left.as_ref(), RustExpr::Deref(inner) if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "__elem"))
                    && op == "/"
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "d")
            )
        ));
    }

    #[test]
    fn lowers_simple_list_subscript_augassign_power_equal_stmt() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "items".to_string(),
            index: HirExpr::IntLiteral(0),
            op: "**=".to_string(),
            value: HirExpr::Name {
                name: "p".to_string(),
                ty: Type::Int,
            },
            object_ty: Type::List(Box::new(Type::Int)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("list subscript power augassign lowered");
        let RustStmt::Block(stmts) = &lowered[0] else {
            panic!("expected block-lowered list subscript power augassign");
        };
        assert!(matches!(
            &stmts[1],
            RustStmt::IfLet {
                then_body,
                ..
            } if matches!(
                then_body.first(),
                Some(RustStmt::Assign {
                    target: RustExpr::Deref(target),
                    value: RustExpr::MethodCall { receiver, method, args },
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && method == "pow"
                    && matches!(
                        args.first(),
                        Some(RustExpr::Cast {
                            expr,
                            ty: RustType::Named(name),
                        }) if matches!(expr.as_ref(), RustExpr::Ident(v) if v == "p") && name == "u32"
                    )
            )
        ));
    }

    #[test]
    fn lowers_simple_alias_list_subscript_augassign_stmt() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "items".to_string(),
            index: HirExpr::IntLiteral(0),
            op: "+=".to_string(),
            value: HirExpr::IntLiteral(1),
            object_ty: Type::Alias(
                "IntList".to_string(),
                Box::new(Type::List(Box::new(Type::Int))),
            ),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("alias-list subscript augassign lowered");
        assert!(matches!(lowered[0], RustStmt::Block(_)));
    }

    #[test]
    fn does_not_lower_subscript_augassign_with_non_leaf_value() {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "items".to_string(),
            index: HirExpr::IntLiteral(0),
            op: "+=".to_string(),
            value: HirExpr::Call {
                func: "next_value".to_string(),
                args: vec![],
                ty: Type::Int,
            },
            object_ty: Type::List(Box::new(Type::Int)),
        };

        assert!(
            try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none()
        );
    }

    #[test]
    fn lowers_simple_let_with_not_bool_name_rhs() {
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Bool,
            value: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "ok".to_string(),
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
            is_mutable: false,
        };

        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let not-bool name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                name: ref let_name,
                value: RustExpr::UnaryOp {
                    ref op,
                    ref operand,
                },
                ..
            } if let_name == "x"
                && op == "!"
                && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn lowers_simple_assign_with_not_bool_name_rhs() {
        let assign_stmt = HirStmt::Assign {
            name: "x".to_string(),
            value: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "ok".to_string(),
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
        };

        let lowered = try_lower_simple_stmt(
            &assign_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assign not-bool name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assign {
                target: RustExpr::Ident(ref target_name),
                value: RustExpr::UnaryOp {
                    ref op,
                    ref operand,
                },
            } if target_name == "x"
                && op == "!"
                && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn lowers_simple_let_with_not_option_name_rhs() {
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Bool,
            value: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ty: Type::Bool,
            },
            is_mutable: false,
        };

        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let not-option name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                name: ref let_name,
                value: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if let_name == "x"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_assign_with_not_option_name_rhs() {
        let assign_stmt = HirStmt::Assign {
            name: "x".to_string(),
            value: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ty: Type::Bool,
            },
        };

        let lowered = try_lower_simple_stmt(
            &assign_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assign not-option name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assign {
                target: RustExpr::Ident(ref target_name),
                value: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
            } if target_name == "x"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_let_name_rhs() {
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Int,
            value: HirExpr::Name {
                name: "y".to_string(),
                ty: Type::Int,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                value: RustExpr::Ident(ref rhs),
                ..
            } if let_name == "x" && rhs == "y"
        ));
    }

    #[test]
    fn lowers_simple_let_alias_int_literal_rhs() {
        let alias_int = Type::Alias("Meters".to_string(), Box::new(Type::Int));
        let let_stmt = HirStmt::Let {
            name: "distance".to_string(),
            ty: alias_int,
            value: HirExpr::IntLiteral(7),
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let alias-int literal rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                value: RustExpr::Cast { ty: RustType::I64, .. },
                ..
            } if let_name == "distance"
        ));
    }

    #[test]
    fn lowers_simple_let_alias_enum_name_rhs() {
        let alias_enum = Type::Alias(
            "ColorAlias".to_string(),
            Box::new(Type::Enum {
                name: "Color".to_string(),
                variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
            }),
        );
        let let_stmt = HirStmt::Let {
            name: "shade".to_string(),
            ty: alias_enum.clone(),
            value: HirExpr::Name {
                name: "selected".to_string(),
                ty: alias_enum,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let alias-enum name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                value: RustExpr::Ident(ref rhs),
                ..
            } if let_name == "shade" && rhs == "selected"
        ));
    }

    #[test]
    fn lowers_simple_let_none_literal_to_unit() {
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: Type::None,
            value: HirExpr::NoneLiteral,
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let none lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                ty: Some(RustType::Unit),
                value: RustExpr::Literal(RustLiteral::Unit),
            } if let_name == "x"
        ));
    }

    #[test]
    fn lowers_simple_let_alias_none_literal_to_unit() {
        let alias_none = Type::Alias("Nothing".to_string(), Box::new(Type::None));
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: alias_none,
            value: HirExpr::NoneLiteral,
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let alias-none lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                value: RustExpr::Literal(RustLiteral::Unit),
                ..
            } if let_name == "x"
        ));
    }

    #[test]
    fn lowers_simple_let_none_name_rhs() {
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: Type::None,
            value: HirExpr::Name {
                name: "n".to_string(),
                ty: Type::None,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let none-name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                value: RustExpr::Ident(ref rhs),
                ..
            } if let_name == "x" && rhs == "n"
        ));
    }

    #[test]
    fn lowers_simple_let_alias_none_name_rhs() {
        let alias_none = Type::Alias("Nothing".to_string(), Box::new(Type::None));
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: alias_none.clone(),
            value: HirExpr::Name {
                name: "n".to_string(),
                ty: alias_none,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("let alias-none-name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                value: RustExpr::Ident(ref rhs),
                ..
            } if let_name == "x" && rhs == "n"
        ));
    }

    #[test]
    fn lowers_simple_option_let_none_literal_to_none() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty.clone(),
            value: HirExpr::NoneLiteral,
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("option let none lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                ty: Some(RustType::Option(_)),
                value: RustExpr::Literal(RustLiteral::None),
            } if let_name == "x"
        ));
    }

    #[test]
    fn lowers_simple_option_let_none_literal_to_none_with_alias_option_ty() {
        let option_ty = Type::Alias(
            "MaybeInt".to_string(),
            Box::new(Type::Union(vec![Type::Int, Type::None])),
        );
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty,
            value: HirExpr::NoneLiteral,
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("alias-option let none lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                value: RustExpr::Literal(RustLiteral::None),
                ..
            } if let_name == "x"
        ));
    }

    #[test]
    fn lowers_simple_option_let_name_rhs_to_some() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty,
            value: HirExpr::Name {
                name: "y".to_string(),
                ty: Type::Int,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("option let name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                ty: Some(RustType::Option(_)),
                value: RustExpr::FnCall { ref func, ref args },
            } if let_name == "x"
                && matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "y")
        ));
    }

    #[test]
    fn lowers_simple_option_let_leaf_rhs_to_some() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty,
            value: HirExpr::IntLiteral(7),
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("option let leaf rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                ty: Some(RustType::Option(_)),
                value: RustExpr::FnCall { ref func, ref args },
            } if let_name == "x"
                && matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
                && matches!(args.first(), Some(RustExpr::Cast { ty: RustType::I64, .. }))
        ));
    }

    #[test]
    fn lowers_simple_option_let_option_name_rhs_passthrough() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty.clone(),
            value: HirExpr::Name {
                name: "maybe_y".to_string(),
                ty: option_ty,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("option let option name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                ty: Some(RustType::Option(_)),
                value: RustExpr::Ident(ref rhs),
            } if let_name == "x" && rhs == "maybe_y"
        ));
    }

    #[test]
    fn does_not_lower_option_let_option_non_leaf_rhs_passthrough() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty.clone(),
            value: HirExpr::Call {
                func: "maybe_value".to_string(),
                args: vec![],
                ty: option_ty,
            },
            is_mutable: false,
        };

        assert!(
            try_lower_simple_stmt(
                &let_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_option_let_non_leaf_rhs_to_some() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty,
            value: HirExpr::Call {
                func: "value".to_string(),
                args: vec![],
                ty: Type::Int,
            },
            is_mutable: false,
        };

        assert!(
            try_lower_simple_stmt(
                &let_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_option_let_none_name_rhs_to_none() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty,
            value: HirExpr::Name {
                name: "none_value".to_string(),
                ty: Type::None,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("option let none-name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                ty: Some(RustType::Option(_)),
                value: RustExpr::Literal(RustLiteral::None),
            } if let_name == "x"
        ));
    }

    #[test]
    fn lowers_simple_option_let_alias_none_name_rhs_to_none() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let alias_none = Type::Alias("Nothing".to_string(), Box::new(Type::None));
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty,
            value: HirExpr::Name {
                name: "none_value".to_string(),
                ty: alias_none,
            },
            is_mutable: false,
        };
        let lowered = try_lower_simple_stmt(
            &let_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("option let alias-none-name rhs lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                name: ref let_name,
                ty: Some(RustType::Option(_)),
                value: RustExpr::Literal(RustLiteral::None),
            } if let_name == "x"
        ));
    }

    #[test]
    fn does_not_lower_option_let_non_leaf_none_typed_rhs_to_none() {
        let option_ty = Type::Union(vec![Type::Int, Type::None]);
        let let_stmt = HirStmt::Let {
            name: "x".to_string(),
            ty: option_ty,
            value: HirExpr::Call {
                func: "none_value".to_string(),
                args: vec![],
                ty: Type::None,
            },
            is_mutable: false,
        };

        assert!(
            try_lower_simple_stmt(
                &let_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_assign_name_rhs() {
        let assign_stmt = HirStmt::Assign {
            name: "x".to_string(),
            value: HirExpr::Name {
                name: "y".to_string(),
                ty: Type::Int,
            },
        };
        let lowered = try_lower_simple_stmt(
            &assign_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("name assign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assign {
                target: RustExpr::Ident(ref lhs),
                value: RustExpr::Ident(ref rhs),
            } if lhs == "x" && rhs == "y"
        ));
    }

    #[test]
    fn does_not_lower_assign_borrowed_typevar_name() {
        let assign_stmt = HirStmt::Assign {
            name: "dst".to_string(),
            value: HirExpr::Name {
                name: "param".to_string(),
                ty: Type::TypeVar("T".to_string()),
            },
        };

        assert!(
            try_lower_simple_stmt(
                &assign_stmt,
                false,
                &HashSet::new(),
                &HashSet::from(["param".to_string()]),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_augassign_for_supported_ops() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "-=".to_string(),
            value: HirExpr::IntLiteral(2),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                ..
            } if name == "x" && lowered_op == "-"
        ));
    }

    #[test]
    fn does_not_lower_augassign_plus_equal() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "+=".to_string(),
            value: HirExpr::StringLiteral("a".to_string()),
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_augassign_plus_equal_numeric() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "+=".to_string(),
            value: HirExpr::IntLiteral(1),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("numeric += lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                ..
            } if name == "x" && lowered_op == "+"
        ));
    }

    #[test]
    fn lowers_simple_augassign_plus_equal_numeric_name() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "+=".to_string(),
            value: HirExpr::Name {
                name: "delta".to_string(),
                ty: Type::Int,
            },
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("numeric name += lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                value: RustExpr::Ident(ref rhs),
            } if name == "x" && lowered_op == "+" && rhs == "delta"
        ));
    }

    #[test]
    fn lowers_simple_augassign_plus_equal_alias_numeric_name() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "+=".to_string(),
            value: HirExpr::Name {
                name: "delta".to_string(),
                ty: Type::Alias("Meters".to_string(), Box::new(Type::Int)),
            },
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("alias numeric name += lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                value: RustExpr::Ident(ref rhs),
            } if name == "x" && lowered_op == "+" && rhs == "delta"
        ));
    }

    #[test]
    fn does_not_lower_augassign_plus_equal_string_name() {
        let stmt = HirStmt::AugAssign {
            name: "s".to_string(),
            op: "+=".to_string(),
            value: HirExpr::Name {
                name: "suffix".to_string(),
                ty: Type::Str,
            },
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_augassign_floor_div_equal() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "//=".to_string(),
            value: HirExpr::IntLiteral(2),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("floor-div augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                ..
            } if name == "x" && lowered_op == "/"
        ));
    }

    #[test]
    fn lowers_simple_augassign_floor_div_equal_alias_numeric_name() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "//=".to_string(),
            value: HirExpr::Name {
                name: "step".to_string(),
                ty: Type::Alias("Step".to_string(), Box::new(Type::Int)),
            },
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("alias numeric //= lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                value: RustExpr::Ident(ref rhs),
            } if name == "x" && lowered_op == "/" && rhs == "step"
        ));
    }

    #[test]
    fn does_not_lower_augassign_power_equal() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "**=".to_string(),
            value: HirExpr::IntLiteral(3),
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_augassign_bitwise_and_shift_ops() {
        for (op, expected) in [
            ("&=", "&"),
            ("|=", "|"),
            ("^=", "^"),
            ("<<=", "<<"),
            (">>=", ">>"),
        ] {
            let stmt = HirStmt::AugAssign {
                name: "x".to_string(),
                op: op.to_string(),
                value: HirExpr::Name {
                    name: "delta".to_string(),
                    ty: Type::Alias("Bits".to_string(), Box::new(Type::Int)),
                },
            };

            let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
                .expect("bitwise/shift augassign lowered");
            assert_eq!(lowered.len(), 1);
            assert!(matches!(
                lowered[0],
                RustStmt::AugAssign {
                    target: RustExpr::Ident(ref name),
                    op: ref lowered_op,
                    value: RustExpr::Ident(ref rhs),
                } if name == "x" && lowered_op == expected && rhs == "delta"
            ));
        }
    }

    #[test]
    fn does_not_lower_augassign_bitwise_for_float() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "&=".to_string(),
            value: HirExpr::Name {
                name: "mask".to_string(),
                ty: Type::Float,
            },
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_attribute_augassign_for_supported_ops() {
        let stmt = HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "count".to_string(),
            op: "-=".to_string(),
            value: HirExpr::IntLiteral(2),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("attribute augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Field { ref expr, ref field },
                op: ref lowered_op,
                ..
            } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                && field == "count"
                && lowered_op == "-"
        ));
    }

    #[test]
    fn lowers_simple_attribute_augassign_floor_div_equal_alias_numeric_name() {
        let stmt = HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "count".to_string(),
            op: "//=".to_string(),
            value: HirExpr::Name {
                name: "step".to_string(),
                ty: Type::Alias("Step".to_string(), Box::new(Type::Int)),
            },
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("attribute floor-div augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Field { ref expr, ref field },
                op: ref lowered_op,
                value: RustExpr::Ident(ref rhs),
            } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                && field == "count"
                && lowered_op == "/"
                && rhs == "step"
        ));
    }

    #[test]
    fn does_not_lower_attribute_augassign_plus_equal_string_name() {
        let stmt = HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "label".to_string(),
            op: "+=".to_string(),
            value: HirExpr::Name {
                name: "suffix".to_string(),
                ty: Type::Str,
            },
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn does_not_lower_attribute_augassign_power_equal() {
        let stmt = HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "count".to_string(),
            op: "**=".to_string(),
            value: HirExpr::IntLiteral(3),
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn lowers_simple_attribute_augassign_bitwise_and_shift_ops() {
        for (op, expected) in [
            ("&=", "&"),
            ("|=", "|"),
            ("^=", "^"),
            ("<<=", "<<"),
            (">>=", ">>"),
        ] {
            let stmt = HirStmt::AttributeAugAssign {
                object: "self".to_string(),
                field: "flags".to_string(),
                op: op.to_string(),
                value: HirExpr::Name {
                    name: "delta".to_string(),
                    ty: Type::Int,
                },
            };

            let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
                .expect("attribute bitwise/shift augassign lowered");
            assert_eq!(lowered.len(), 1);
            assert!(matches!(
                lowered[0],
                RustStmt::AugAssign {
                    target: RustExpr::Field { ref expr, ref field },
                    op: ref lowered_op,
                    value: RustExpr::Ident(ref rhs),
                } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                    && field == "flags"
                    && lowered_op == expected
                    && rhs == "delta"
            ));
        }
    }

    #[test]
    fn lowers_simple_bare_return_without_option_context() {
        let stmt = HirStmt::Return { value: None };
        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("bare return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::Return(None)));
    }

    #[test]
    fn lowers_simple_bare_return_to_none_in_option_context() {
        let stmt = HirStmt::Return { value: None };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("bare return lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn lowers_simple_bare_return_to_none_in_alias_option_context() {
        let stmt = HirStmt::Return { value: None };
        let option_ret = Type::Alias(
            "MaybeInt".to_string(),
            Box::new(Type::Union(vec![Type::Int, Type::None])),
        );
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("bare return lowered for alias option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn does_not_lower_bare_return_in_display_impl_context() {
        let stmt = HirStmt::Return { value: None };
        assert!(
            try_lower_simple_stmt_with_ctx(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
                SimpleStmtLoweringCtx {
                    return_type: None,
                    in_display_impl: true,
                    in_class_scope: false,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_return_with_leaf_expr() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("return with value lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::Return(Some(_))));
    }

    #[test]
    fn lowers_simple_return_name_in_plain_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("plain return name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Ident(ref name))) if name == "x"
        ));
    }

    #[test]
    fn lowers_return_leaf_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("option return leaf lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()]));
            }
            _ => panic!("expected return Some(...)"),
        }
    }

    #[test]
    fn lowers_return_name_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("option return name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, ref args }))
                if matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
                    && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "x")
        ));
    }

    #[test]
    fn lowers_return_option_name_with_unwrap_in_plain_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
        };
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Int),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("plain return option name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            &lowered[0],
            RustStmt::Return(Some(RustExpr::MethodCall {
                receiver,
                ref method,
                ref args,
            })) if method == "unwrap"
                && args.is_empty()
                && matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
        ));
    }

    #[test]
    fn lowers_option_name_return_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("option passthrough name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Ident(ref name))) if name == "maybe_x"
        ));
    }

    #[test]
    fn lowers_option_name_return_with_alias_option_return_context() {
        let alias_option = Type::Alias(
            "MaybeInt".to_string(),
            Box::new(Type::Union(vec![Type::Int, Type::None])),
        );
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: alias_option.clone(),
            }),
        };
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&alias_option),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("alias option passthrough name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Ident(ref name))) if name == "maybe_x"
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_option_return_passthrough_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(
            try_lower_simple_stmt_with_ctx(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
                SimpleStmtLoweringCtx {
                    return_type: Some(&option_ret),
                    in_display_impl: false,
                    in_class_scope: false,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_return_none_literal_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::NoneLiteral),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("return None lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn lowers_return_none_name_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "none_value".to_string(),
                ty: Type::None,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("return none-typed name lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn lowers_return_alias_none_name_with_option_return_context() {
        let alias_none = Type::Alias("Nothing".to_string(), Box::new(Type::None));
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "none_value".to_string(),
                ty: alias_none,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("return alias-none name lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_none_typed_return_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "produce_none".to_string(),
                args: vec![],
                ty: Type::None,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(
            try_lower_simple_stmt_with_ctx(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
                SimpleStmtLoweringCtx {
                    return_type: Some(&option_ret),
                    in_display_impl: false,
                    in_class_scope: false,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_non_leaf_alias_none_typed_return_with_option_return_context() {
        let alias_none = Type::Alias("Nothing".to_string(), Box::new(Type::None));
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "produce_none".to_string(),
                args: vec![],
                ty: alias_none,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(
            try_lower_simple_stmt_with_ctx(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
                SimpleStmtLoweringCtx {
                    return_type: Some(&option_ret),
                    in_display_impl: false,
                    in_class_scope: false,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_return_leaf_with_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let union_ret = Type::Union(vec![Type::Int, Type::Str]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("non-option union leaf return lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(matches!(func.as_ref(), RustExpr::Path(parts) if parts.len() == 2));
            }
            _ => panic!("expected union-variant wrapped return"),
        }
    }

    #[test]
    fn lowers_return_name_with_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let union_ret = Type::Union(vec![Type::Int, Type::Str]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("non-option union name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, ref args }))
                if matches!(func.as_ref(), RustExpr::Path(parts) if parts.len() == 2)
                    && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "x")
        ));
    }

    #[test]
    fn lowers_return_leaf_with_alias_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let union_ret = Type::Alias(
            "ValueUnion".to_string(),
            Box::new(Type::Union(vec![Type::Int, Type::Str])),
        );
        let expected_enum = union_ret.union_enum_name();
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("alias non-option union leaf return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, .. }))
                if matches!(
                    func.as_ref(),
                    RustExpr::Path(parts) if parts.first().is_some_and(|n| n == &expected_enum)
                )
        ));
    }

    #[test]
    fn lowers_return_name_with_alias_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let union_ret = Type::Alias(
            "ValueUnion".to_string(),
            Box::new(Type::Union(vec![Type::Int, Type::Str])),
        );
        let expected_enum = union_ret.union_enum_name();
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
            },
        )
        .expect("alias non-option union name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, ref args }))
                if matches!(
                    func.as_ref(),
                    RustExpr::Path(parts) if parts.first().is_some_and(|n| n == &expected_enum)
                ) && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "x")
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_return_with_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "value".to_string(),
                args: vec![],
                ty: Type::Int,
            }),
        };
        let union_ret = Type::Union(vec![Type::Int, Type::Str]);
        assert!(
            try_lower_simple_stmt_with_ctx(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
                SimpleStmtLoweringCtx {
                    return_type: Some(&union_ret),
                    in_display_impl: false,
                    in_class_scope: false,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_return_in_class_scope() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        assert!(
            try_lower_simple_stmt_with_ctx(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
                SimpleStmtLoweringCtx {
                    return_type: Some(&Type::Int),
                    in_display_impl: false,
                    in_class_scope: true,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_non_leaf_return_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "value".to_string(),
                args: vec![],
                ty: Type::Int,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(
            try_lower_simple_stmt_with_ctx(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
                SimpleStmtLoweringCtx {
                    return_type: Some(&option_ret),
                    in_display_impl: false,
                    in_class_scope: false,
                },
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_raise_with_leaf_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::IntLiteral(7),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("raise lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()]));
            }
            _ => panic!("expected return Err(...)"),
        }
    }

    #[test]
    fn does_not_lower_raise_with_non_leaf_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::Call {
                func: "err".to_string(),
                args: vec![],
                ty: Type::Int,
            },
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_raise_with_name_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::Name {
                name: "e".to_string(),
                ty: Type::Int,
            },
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("raise name lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, args })) => {
                assert!(matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()]));
                assert!(matches!(args.first(), Some(RustExpr::Ident(name)) if name == "e"));
            }
            _ => panic!("expected return Err(e)"),
        }
    }

    #[test]
    fn lowers_simple_assert_without_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert { msg: None, .. }
        ));
    }

    #[test]
    fn lowers_simple_assert_with_leaf_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::StringLiteral("boom".to_string())),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert with msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::Literal(RustLiteral::Str(_))),
                ..
            }
        ));
    }

    #[test]
    fn lowers_simple_assert_with_name_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::Name {
                name: "msg".to_string(),
                ty: Type::Str,
            }),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert with name msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::Ident(ref name)),
                ..
            } if name == "msg"
        ));
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Call {
                func: "is_ok".to_string(),
                args: vec![],
                ty: Type::Bool,
            },
            msg: None,
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_assert_with_bool_name_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert bool name test lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                cond: RustExpr::Ident(ref name),
                msg: None,
            } if name == "ok"
        ));
    }

    #[test]
    fn lowers_simple_assert_with_not_bool_name_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "ok".to_string(),
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert not-bool name test lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                cond: RustExpr::UnaryOp {
                    ref op,
                    ref operand,
                },
                msg: None,
            } if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn lowers_simple_assert_with_not_option_truthiness_name_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ty: Type::Bool,
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert not-option truthiness name test lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Assert {
                cond: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                msg: None,
            } => {
                assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
                assert_eq!(method, "is_none");
                assert!(args.is_empty());
            }
            _ => panic!("expected assert with method-call condition"),
        }
    }

    #[test]
    fn lowers_simple_assert_with_option_truthiness_name_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert option truthiness name test lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Assert {
                cond: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                msg: None,
            } => {
                assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
                assert_eq!(method, "is_some");
                assert!(args.is_empty());
            }
            _ => panic!("expected assert with method-call condition"),
        }
    }

    #[test]
    fn lowers_simple_assert_with_option_is_none_compare_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert option is-none compare lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                cond: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                msg: None,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_assert_with_option_is_not_none_compare_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is not".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert option is-not-none compare lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                cond: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                msg: None,
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_not_bool_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Call {
                    func: "is_ok".to_string(),
                    args: vec![],
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
            msg: None,
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_not_option_truthiness_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Call {
                    func: "maybe_x".to_string(),
                    args: vec![],
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ty: Type::Bool,
            },
            msg: None,
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_option_is_none_compare_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Compare {
                left: Box::new(HirExpr::Call {
                    func: "maybe_x".to_string(),
                    args: vec![],
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            msg: None,
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_option_truthiness_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
            msg: None,
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_assert_with_option_name_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::Name {
                name: "msg".to_string(),
                ty: Type::Union(vec![Type::Str, Type::None]),
            }),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert with option msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::MethodCall { ref method, .. }),
                ..
            } if method == "map_or"
        ));
    }

    #[test]
    fn lowers_simple_assert_with_alias_option_name_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::Name {
                name: "msg".to_string(),
                ty: Type::Alias(
                    "MaybeStr".to_string(),
                    Box::new(Type::Union(vec![Type::Str, Type::None])),
                ),
            }),
        };

        let lowered = try_lower_simple_stmt(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("assert with alias option msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::MethodCall { ref method, .. }),
                ..
            } if method == "map_or"
        ));
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_option_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::Call {
                func: "maybe_msg".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Str, Type::None]),
            }),
        };

        assert!(
            try_lower_simple_stmt(
                &stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_if_without_elif() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(0),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::If { .. }));
    }

    #[test]
    fn lowers_simple_if_with_name_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with name condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::If {
                cond: RustExpr::Ident(ref name),
                ..
            } if name == "ok"
        ));
    }

    #[test]
    fn lowers_simple_if_with_not_bool_name_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "ok".to_string(),
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with not-bool name condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    ref op,
                    ref operand,
                },
                ..
            } if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

    #[test]
    fn does_not_lower_if_with_non_leaf_not_bool_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Call {
                    func: "ok".to_string(),
                    args: vec![],
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &if_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_if_with_not_option_truthiness_name_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with not-option truthiness condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::If {
                cond: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                ..
            } => {
                assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
                assert_eq!(method, "is_none");
                assert!(args.is_empty());
            }
            _ => panic!("expected if with method-call condition"),
        }
    }

    #[test]
    fn lowers_simple_if_with_option_is_none_compare_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with option is-none compare condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::If {
                cond: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_if_with_option_is_not_none_compare_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is not".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with option is-not-none compare condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::If {
                cond: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn does_not_lower_if_with_non_leaf_option_is_none_compare_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Call {
                    func: "maybe_x".to_string(),
                    args: vec![],
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &if_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_if_with_non_leaf_not_option_truthiness_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Call {
                    func: "maybe_x".to_string(),
                    args: vec![],
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &if_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_if_with_option_truthiness_name_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Pass]),
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if option truthiness lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::IfLet {
                pattern: ref p,
                expr: RustExpr::Ident(ref n),
                else_body: Some(_),
                ..
            } if p == "Some(maybe_x)" && n == "maybe_x"
        ));
    }

    #[test]
    fn lowers_simple_if_with_alias_option_truthiness_name_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Alias(
                    "MaybeInt".to_string(),
                    Box::new(Type::Union(vec![Type::Int, Type::None])),
                ),
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Pass]),
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if alias option truthiness lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::IfLet {
                pattern: ref p,
                expr: RustExpr::Ident(ref n),
                else_body: Some(_),
                ..
            } if p == "Some(maybe_x)" && n == "maybe_x"
        ));
    }

    #[test]
    fn lowers_if_option_truthiness_with_elif() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![(HirExpr::BoolLiteral(true), vec![HirStmt::Pass])],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if option truthiness with elif lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::IfLet { else_body, .. } => {
                assert!(else_body.is_some());
                if let Some(else_body) = else_body {
                    assert_eq!(else_body.len(), 1);
                    assert!(matches!(else_body[0], RustStmt::If { .. }));
                }
            }
            _ => panic!("expected if let stmt"),
        }
    }

    #[test]
    fn lowers_if_with_option_truthiness_elif_clause() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(false),
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![(
                HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                },
                vec![HirStmt::Pass],
            )],
            else_body: Some(vec![HirStmt::Pass]),
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with option truthiness elif lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::If { else_body, .. } => {
                assert!(else_body.is_some());
                if let Some(else_body) = else_body {
                    assert_eq!(else_body.len(), 1);
                    assert!(matches!(
                        else_body[0],
                        RustStmt::IfLet {
                            pattern: ref p,
                            expr: RustExpr::Ident(ref n),
                            else_body: Some(_),
                            ..
                        } if p == "Some(maybe_x)" && n == "maybe_x"
                    ));
                }
            }
            _ => panic!("expected if stmt"),
        }
    }

    #[test]
    fn lowers_simple_if_with_elif() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }],
            elif_clauses: vec![(
                HirExpr::BoolLiteral(false),
                vec![HirStmt::Expr {
                    expr: HirExpr::IntLiteral(2),
                }],
            )],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(3),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("if with elif lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::If { else_body, .. } => {
                assert!(else_body.is_some());
                if let Some(else_body) = else_body {
                    assert_eq!(else_body.len(), 1);
                    assert!(matches!(else_body[0], RustStmt::If { .. }));
                }
            }
            _ => panic!("expected if stmt"),
        }
    }

    #[test]
    fn does_not_lower_if_with_non_leaf_elif_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![(
                HirExpr::Call {
                    func: "flag".to_string(),
                    args: vec![],
                    ty: Type::Bool,
                },
                vec![HirStmt::Pass],
            )],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &if_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_while_without_else() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::Break],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            true, // outer context has else, inner while should not inherit it
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While { body, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], RustStmt::Break));
            }
            _ => panic!("expected RustStmt::While"),
        }
    }

    #[test]
    fn lowers_simple_while_with_name_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Name {
                name: "ready".to_string(),
                ty: Type::Bool,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with name condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::While {
                cond: RustExpr::Ident(ref name),
                ..
            } if name == "ready"
        ));
    }

    #[test]
    fn lowers_simple_while_with_not_bool_name_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "ready".to_string(),
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with not-bool name condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::While {
                cond: RustExpr::UnaryOp {
                    ref op,
                    ref operand,
                },
                ..
            } if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ready")
        ));
    }

    #[test]
    fn lowers_simple_while_with_not_option_truthiness_name_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ty: Type::Bool,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with not-option truthiness name condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While {
                cond: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                ..
            } => {
                assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
                assert_eq!(method, "is_none");
                assert!(args.is_empty());
            }
            _ => panic!("expected while with method-call condition"),
        }
    }

    #[test]
    fn lowers_simple_while_with_option_is_none_compare_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with option is-none compare condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::While {
                cond: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_none"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_while_with_option_is_not_none_compare_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "maybe_x".to_string(),
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is not".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with option is-not-none compare condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::While {
                cond: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
                && method == "is_some"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_while_with_option_truthiness_name_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with option truthiness name condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While {
                cond: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                ..
            } => {
                assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
                assert_eq!(method, "is_some");
                assert!(args.is_empty());
            }
            _ => panic!("expected while with method-call condition"),
        }
    }

    #[test]
    fn lowers_simple_while_with_alias_option_truthiness_name_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Alias(
                    "MaybeInt".to_string(),
                    Box::new(Type::Union(vec![Type::Int, Type::None])),
                ),
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with alias option truthiness name condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While {
                cond: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                ..
            } => {
                assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
                assert_eq!(method, "is_some");
                assert!(args.is_empty());
            }
            _ => panic!("expected while with method-call condition"),
        }
    }

    #[test]
    fn does_not_lower_while_with_non_leaf_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Call {
                func: "ready".to_string(),
                args: vec![],
                ty: Type::Bool,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &while_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_while_with_non_leaf_option_truthiness_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &while_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_while_with_non_leaf_option_is_none_compare_condition() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::Compare {
                left: Box::new(HirExpr::Call {
                    func: "maybe_x".to_string(),
                    args: vec![],
                    ty: Type::Union(vec![Type::Int, Type::None]),
                }),
                ops: vec!["is".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &while_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_while_with_else() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Pass]),
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while with else lowered");
        assert_eq!(lowered.len(), 3);
        assert!(matches!(lowered[0], RustStmt::Let { .. }));
        assert!(matches!(lowered[1], RustStmt::While { .. }));
        assert!(matches!(lowered[2], RustStmt::If { .. }));
    }

    #[test]
    fn lowers_simple_for_without_else() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Break],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &for_stmt,
            true, // outer loop-else context should not leak into inner loop body
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::For { body, .. } => {
                assert_eq!(body.len(), 1);
                assert!(matches!(body[0], RustStmt::Break));
            }
            _ => panic!("expected RustStmt::For"),
        }
    }

    #[test]
    fn lowers_simple_for_with_name_iter() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(
            &for_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for with name iter lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::For {
                var: ref var_name,
                iter: RustExpr::Ident(ref iter_name),
                ..
            } if var_name == "i" && iter_name == "items"
        ));
    }

    #[test]
    fn does_not_lower_for_with_non_leaf_iter() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::Call {
                func: "items".to_string(),
                args: vec![],
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        assert!(
            try_lower_simple_stmt(
                &for_stmt,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_simple_for_with_else() {
        let for_with_else = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Pass]),
        };
        let lowered = try_lower_simple_stmt(
            &for_with_else,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for with else lowered");
        assert_eq!(lowered.len(), 3);
        assert!(matches!(lowered[0], RustStmt::Let { .. }));
        assert!(matches!(lowered[1], RustStmt::For { .. }));
        assert!(matches!(lowered[2], RustStmt::If { .. }));
    }

    #[test]
    fn lowers_simple_for_with_else_and_name_iter() {
        let for_with_else = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Pass]),
        };
        let lowered = try_lower_simple_stmt(
            &for_with_else,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for with else and name iter lowered");
        assert_eq!(lowered.len(), 3);
        assert!(matches!(lowered[0], RustStmt::Let { .. }));
        assert!(matches!(
            lowered[1],
            RustStmt::For {
                iter: RustExpr::Ident(ref iter_name),
                ..
            } if iter_name == "items"
        ));
        assert!(matches!(lowered[2], RustStmt::If { .. }));
    }

    #[test]
    fn does_not_lower_for_with_else_and_non_leaf_iter() {
        let for_with_else = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::Call {
                func: "items".to_string(),
                args: vec![],
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Pass]),
        };
        assert!(
            try_lower_simple_stmt(
                &for_with_else,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn does_not_lower_for_with_tuple_target() {
        let for_tuple_target = HirStmt::For {
            target: "i,v".to_string(),
            target_ty: Type::Tuple(vec![Type::Int, Type::Int]),
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };
        assert!(
            try_lower_simple_stmt(
                &for_tuple_target,
                false,
                &HashSet::new(),
                &HashSet::new(),
            )
            .is_none()
        );
    }

    #[test]
    fn lowers_for_else_with_broke_marker_in_loop_body() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Break],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &for_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for else lowered");

        match &lowered[1] {
            RustStmt::For { body, .. } => {
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], RustStmt::Assign { .. }));
                assert!(matches!(body[1], RustStmt::Break));
            }
            _ => panic!("expected for stmt"),
        }
    }

    #[test]
    fn for_else_body_break_uses_outer_loop_else_context() {
        let for_stmt = HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::RangeLiteral {
                start: Box::new(HirExpr::IntLiteral(0)),
                end: Box::new(HirExpr::IntLiteral(3)),
                step: None,
                ty: Type::Range,
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Break]),
        };

        let lowered = try_lower_simple_stmt(
            &for_stmt,
            true,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("for else lowered");

        match &lowered[2] {
            RustStmt::If { then_body, .. } => {
                assert_eq!(then_body.len(), 2);
                assert!(matches!(then_body[0], RustStmt::Assign { .. }));
                assert!(matches!(then_body[1], RustStmt::Break));
            }
            _ => panic!("expected if stmt"),
        }
    }

    #[test]
    fn lowers_while_else_with_broke_marker_in_loop_body() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::Break],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(1),
            }]),
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while else lowered");

        match &lowered[1] {
            RustStmt::While { body, .. } => {
                assert_eq!(body.len(), 2);
                assert!(matches!(body[0], RustStmt::Assign { .. }));
                assert!(matches!(body[1], RustStmt::Break));
            }
            _ => panic!("expected while stmt"),
        }
    }

    #[test]
    fn while_else_body_break_uses_outer_loop_else_context() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(false),
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Break]),
        };

        let lowered = try_lower_simple_stmt(
            &while_stmt,
            true,
            &HashSet::new(),
            &HashSet::new(),
        )
        .expect("while else lowered");

        match &lowered[2] {
            RustStmt::If { then_body, .. } => {
                assert_eq!(then_body.len(), 2);
                assert!(matches!(then_body[0], RustStmt::Assign { .. }));
                assert!(matches!(then_body[1], RustStmt::Break));
            }
            _ => panic!("expected if stmt"),
        }
    }
}
