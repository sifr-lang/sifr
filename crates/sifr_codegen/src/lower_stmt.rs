//! Statement lowering scaffolds for the IR lowering.

use crate::helpers::{
    body_calls_function, codegen_body_always_exits, collect_locally_defined_vars,
    collect_mutated_vars, collect_referenced_vars_with_types, detect_and_not_none_vars,
    detect_is_none_var, detect_is_not_none_var,
};
use crate::{
    try_lower_leaf_expr, try_lower_leaf_expr_result, CodegenError, RustExpr, RustLiteral,
    RustMatchArm, RustParam, RustStmt, RustType, ScopeContext,
};
use sifr_hir::{
    HirExceptHandler, HirExpr, HirFStringPart, HirFunction, HirPattern, HirStmt, MethodKind,
};
use sifr_type_system::Type;
use std::collections::HashSet;

#[cfg(test)]
pub fn lower_stmt_raw(raw: &str) -> Result<Vec<RustStmt>, CodegenError> {
    Ok(vec![RustStmt::RawCode(raw.to_string())])
}

pub(crate) fn is_simple_stmt_candidate(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Expr { expr } => crate::is_leaf_expr_candidate(expr),
        HirStmt::Let { .. }
        | HirStmt::Assign { .. }
        | HirStmt::AugAssign { .. }
        | HirStmt::AttributeAugAssign { .. }
        | HirStmt::FieldAssign { .. }
        | HirStmt::Return { .. }
        | HirStmt::Assert { .. }
        | HirStmt::Raise { .. }
        | HirStmt::If { .. }
        | HirStmt::While { .. }
        | HirStmt::For { .. }
        | HirStmt::Pass
        | HirStmt::Continue
        | HirStmt::Break
        | HirStmt::TupleUnpack { .. }
        | HirStmt::StarUnpack { .. }
        | HirStmt::SubscriptAssign { .. }
        | HirStmt::NestedSubscriptAssign { .. }
        | HirStmt::SubscriptAugAssign { .. }
        | HirStmt::AttributeSubscriptAssign { .. }
        | HirStmt::Delete { .. }
        | HirStmt::Yield { .. }
        | HirStmt::With { .. }
        | HirStmt::Match { .. }
        | HirStmt::NestedFunction { .. }
        | HirStmt::TryExcept { .. } => true,
    }
}

/// Lowers an expression statement when the expression is a leaf
/// supported by `try_lower_leaf_expr`.
pub fn try_lower_expr_stmt(expr: &HirExpr) -> Option<Vec<RustStmt>> {
    if let Some(lowered_print) = try_lower_simple_print_expr_stmt(expr) {
        return Some(vec![lowered_print]);
    }
    try_lower_leaf_expr(expr).map(|lowered_expr| vec![RustStmt::Expr(lowered_expr)])
}

fn try_lower_simple_print_expr_stmt(expr: &HirExpr) -> Option<RustStmt> {
    let HirExpr::Call { func, args, .. } = expr else {
        return None;
    };
    if func != "print" {
        return None;
    }
    match args.as_slice() {
        [] => Some(RustStmt::Expr(RustExpr::MacroCall {
            name: "println".to_string(),
            args: vec![],
        })),
        [HirExpr::StringLiteral(value)] => Some(RustStmt::Expr(RustExpr::MacroCall {
            name: "println".to_string(),
            args: vec![RustExpr::Ident(format!("{value:?}"))],
        })),
        [HirExpr::FString { .. }] => None,
        [_arg] => None,
        _ => None,
    }
}

#[derive(Clone, Copy, Default)]
pub struct SimpleStmtLoweringCtx<'a> {
    pub return_type: Option<&'a Type>,
    pub in_display_impl: bool,
    pub in_class_scope: bool,
    pub in_generator_closure: bool,
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
    let scope_ctx = ScopeContext {
        in_loop_with_else,
        ..ScopeContext::default()
    };
    try_lower_simple_stmt_with_scope(stmt, mutated_vars, borrowed_params, &scope_ctx)
}

pub(crate) fn try_lower_simple_stmt_with_scope(
    stmt: &HirStmt,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    scope_ctx: &ScopeContext,
) -> Option<Vec<RustStmt>> {
    try_lower_simple_stmt_with_ctx(
        stmt,
        scope_ctx.in_loop_with_else,
        mutated_vars,
        borrowed_params,
        SimpleStmtLoweringCtx {
            return_type: scope_ctx.function_return_type.as_ref(),
            in_display_impl: scope_ctx.in_display_impl,
            in_class_scope: matches!(scope_ctx.class_scope, crate::ClassScope::Inside),
            in_generator_closure: scope_ctx.in_generator_closure,
        },
    )
}

pub(crate) fn try_lower_simple_stmt_with_scope_result(
    stmt: &HirStmt,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    scope_ctx: &ScopeContext,
) -> Result<Option<Vec<RustStmt>>, CodegenError> {
    validate_scope_context(scope_ctx)?;
    validate_stmt_lowering_shape(stmt)?;
    Ok(try_lower_simple_stmt_with_scope(
        stmt,
        mutated_vars,
        borrowed_params,
        scope_ctx,
    ))
}

fn validate_scope_context(scope_ctx: &ScopeContext) -> Result<(), CodegenError> {
    if scope_ctx.in_display_impl && scope_ctx.in_generator_closure {
        return Err(CodegenError::new(
            "invalid lowering scope: display impl and generator closure cannot both be active",
        ));
    }
    Ok(())
}

fn validate_stmt_lowering_shape(stmt: &HirStmt) -> Result<(), CodegenError> {
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value }
        | HirStmt::TupleUnpack { value, .. }
        | HirStmt::StarUnpack { value, .. } => validate_expr_lowering_shape(value),
        HirStmt::Return { value: Some(value) } => validate_expr_lowering_shape(value),
        HirStmt::Expr { expr } => validate_expr_lowering_shape(expr),
        HirStmt::Assert { test, msg } => {
            validate_expr_lowering_shape(test)?;
            if let Some(msg) = msg {
                validate_expr_lowering_shape(msg)?;
            }
            Ok(())
        }
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            validate_expr_lowering_shape(condition)?;
            validate_stmt_block_lowering_shape(then_body)?;
            for (elif_cond, elif_body) in elif_clauses {
                validate_expr_lowering_shape(elif_cond)?;
                validate_stmt_block_lowering_shape(elif_body)?;
            }
            if let Some(else_body) = else_body {
                validate_stmt_block_lowering_shape(else_body)?;
            }
            Ok(())
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            validate_expr_lowering_shape(condition)?;
            validate_stmt_block_lowering_shape(body)?;
            if let Some(else_body) = else_body {
                validate_stmt_block_lowering_shape(else_body)?;
            }
            Ok(())
        }
        HirStmt::For {
            iter,
            body,
            else_body,
            ..
        } => {
            validate_expr_lowering_shape(iter)?;
            validate_stmt_block_lowering_shape(body)?;
            if let Some(else_body) = else_body {
                validate_stmt_block_lowering_shape(else_body)?;
            }
            Ok(())
        }
        HirStmt::SubscriptAssign { index, value, .. }
        | HirStmt::SubscriptAugAssign { index, value, .. }
        | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
            validate_expr_lowering_shape(index)?;
            validate_expr_lowering_shape(value)
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        } => {
            validate_expr_lowering_shape(outer_index)?;
            validate_expr_lowering_shape(inner_index)?;
            validate_expr_lowering_shape(value)
        }
        HirStmt::Delete { object, index } => {
            validate_expr_lowering_shape(object)?;
            validate_expr_lowering_shape(index)
        }
        HirStmt::With { items, body } => {
            for (_, context_expr, _) in items {
                validate_expr_lowering_shape(context_expr)?;
            }
            validate_stmt_block_lowering_shape(body)
        }
        HirStmt::NestedFunction { func } => {
            for param in &func.params {
                if let Some(default) = &param.default {
                    validate_expr_lowering_shape(default)?;
                }
            }
            validate_stmt_block_lowering_shape(&func.body)
        }
        HirStmt::Match { subject, arms, .. } => {
            validate_expr_lowering_shape(subject)?;
            for arm in arms {
                validate_pattern_lowering_shape(&arm.pattern)?;
                if let Some(guard) = &arm.guard {
                    validate_expr_lowering_shape(guard)?;
                }
                validate_stmt_block_lowering_shape(&arm.body)?;
            }
            Ok(())
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            validate_stmt_block_lowering_shape(body)?;
            for handler in handlers {
                validate_stmt_block_lowering_shape(&handler.body)?;
            }
            Ok(())
        }
        HirStmt::Pass | HirStmt::Continue | HirStmt::Break | HirStmt::Return { value: None } => {
            Ok(())
        }
    }
}

fn validate_stmt_block_lowering_shape(stmts: &[HirStmt]) -> Result<(), CodegenError> {
    for stmt in stmts {
        validate_stmt_lowering_shape(stmt)?;
    }
    Ok(())
}

fn validate_pattern_lowering_shape(pattern: &HirPattern) -> Result<(), CodegenError> {
    match pattern {
        HirPattern::Literal { value } => validate_expr_lowering_shape(value),
        HirPattern::Or { patterns } => {
            for pattern in patterns {
                validate_pattern_lowering_shape(pattern)?;
            }
            Ok(())
        }
        HirPattern::Class { fields, .. } => {
            for (_, pattern) in fields {
                validate_pattern_lowering_shape(pattern)?;
            }
            Ok(())
        }
        HirPattern::Tuple { elements } => {
            for pattern in elements {
                validate_pattern_lowering_shape(pattern)?;
            }
            Ok(())
        }
        HirPattern::Wildcard
        | HirPattern::Capture { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => Ok(()),
    }
}

fn validate_expr_lowering_shape(expr: &HirExpr) -> Result<(), CodegenError> {
    let _ = try_lower_leaf_expr_result(expr)?;
    match expr {
        HirExpr::BinOp { left, right, .. } => {
            validate_expr_lowering_shape(left)?;
            validate_expr_lowering_shape(right)
        }
        HirExpr::UnaryOp { operand, .. } => validate_expr_lowering_shape(operand),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            validate_expr_lowering_shape(left)?;
            for comparator in comparators {
                validate_expr_lowering_shape(comparator)?;
            }
            Ok(())
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::Call { args: values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        } => {
            for value in values {
                validate_expr_lowering_shape(value)?;
            }
            Ok(())
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            validate_expr_lowering_shape(condition)?;
            validate_expr_lowering_shape(then_expr)?;
            validate_expr_lowering_shape(else_expr)
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            validate_expr_lowering_shape(start)?;
            validate_expr_lowering_shape(end)?;
            if let Some(step) = step {
                validate_expr_lowering_shape(step)?;
            }
            Ok(())
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for key in keys {
                validate_expr_lowering_shape(key)?;
            }
            for value in values {
                validate_expr_lowering_shape(value)?;
            }
            Ok(())
        }
        HirExpr::Index { object, index, .. } => {
            validate_expr_lowering_shape(object)?;
            validate_expr_lowering_shape(index)
        }
        HirExpr::MethodCall { object, args, .. } => {
            validate_expr_lowering_shape(object)?;
            for arg in args {
                validate_expr_lowering_shape(arg)?;
            }
            Ok(())
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            validate_expr_lowering_shape(element)?;
            validate_expr_lowering_shape(collection)
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    validate_expr_lowering_shape(expr)?;
                }
            }
            Ok(())
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            validate_expr_lowering_shape(object)?;
            if let Some(start) = start {
                validate_expr_lowering_shape(start)?;
            }
            if let Some(stop) = stop {
                validate_expr_lowering_shape(stop)?;
            }
            if let Some(step) = step {
                validate_expr_lowering_shape(step)?;
            }
            Ok(())
        }
        HirExpr::WalrusExpr { value, .. }
        | HirExpr::QuestionMark { expr: value, .. }
        | HirExpr::OkWrap { value, .. }
        | HirExpr::ErrWrap { value, .. } => validate_expr_lowering_shape(value),
        HirExpr::FieldAccess { object, .. } => validate_expr_lowering_shape(object),
        HirExpr::ConstructorCall { args, .. } | HirExpr::SuperCall { args, .. } => {
            for arg in args {
                validate_expr_lowering_shape(arg)?;
            }
            Ok(())
        }
        HirExpr::Lambda { params, body, .. } => {
            for param in params {
                if let Some(default) = &param.default {
                    validate_expr_lowering_shape(default)?;
                }
            }
            validate_expr_lowering_shape(body)
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            validate_expr_lowering_shape(expr)?;
            for (_, iter_expr, filter) in generators {
                validate_expr_lowering_shape(iter_expr)?;
                if let Some(filter) = filter {
                    validate_expr_lowering_shape(filter)?;
                }
            }
            Ok(())
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            validate_expr_lowering_shape(key_expr)?;
            validate_expr_lowering_shape(val_expr)?;
            for (_, iter_expr, filter) in generators {
                validate_expr_lowering_shape(iter_expr)?;
                if let Some(filter) = filter {
                    validate_expr_lowering_shape(filter)?;
                }
            }
            Ok(())
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            validate_expr_lowering_shape(expr)?;
            validate_expr_lowering_shape(iter)?;
            if let Some(filter) = filter {
                validate_expr_lowering_shape(filter)?;
            }
            Ok(())
        }
        HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. } => Ok(()),
    }
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
        HirStmt::Let {
            name, ty, value, ..
        } if try_lower_simple_let_value(ty, value).is_some() => Some(vec![RustStmt::Let {
            mutable: bindings.mutated_vars.contains(name),
            name: name.clone(),
            ty: Some(crate::sifr_type_to_rust_type(ty)),
            value: try_lower_simple_let_value(ty, value)?,
        }]),
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
        HirStmt::FieldAssign {
            object,
            field,
            value,
        } => try_lower_simple_field_assign_stmt(object, field, value),
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
                cond: try_lower_simple_condition_test_expr(test, bindings.borrowed_params)?,
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
        } => try_lower_simple_if_stmt(
            condition,
            then_body,
            elif_clauses,
            maybe_else_body.as_deref(),
            in_loop_with_else,
            bindings,
            ctx,
        ),
        HirStmt::While {
            condition,
            body,
            else_body,
        } => try_lower_simple_while_stmt(
            condition,
            body,
            else_body.as_deref(),
            in_loop_with_else,
            bindings,
            ctx,
        ),
        HirStmt::For {
            target,
            iter,
            body,
            else_body,
            ..
        } => try_lower_simple_for_stmt(
            target,
            iter,
            body,
            else_body.as_deref(),
            in_loop_with_else,
            bindings,
            ctx,
        ),
        HirStmt::TupleUnpack { targets, value } => {
            try_lower_simple_tuple_unpack_stmt(targets, value)
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            value,
        } => try_lower_simple_star_unpack_stmt(before, star, after, value),
        HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            field_ty,
        } => {
            try_lower_simple_attribute_subscript_assign_stmt(object, field, index, value, field_ty)
        }
        HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
        } => try_lower_simple_subscript_assign_stmt(object, index, value, object_ty),
        HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            object_ty: _,
        } => try_lower_simple_nested_subscript_assign_stmt(object, outer_index, inner_index, value),
        HirStmt::SubscriptAugAssign {
            object,
            index,
            op,
            value,
            object_ty,
        } => try_lower_simple_subscript_augassign_stmt(object, index, op, value, object_ty),
        HirStmt::Delete { object, index } => try_lower_simple_delete_stmt(object, index),
        HirStmt::Yield { value } => try_lower_simple_yield_stmt(value, ctx),
        HirStmt::With { items, body } => {
            try_lower_simple_with_stmt(items, body, in_loop_with_else, bindings, ctx)
        }
        HirStmt::Match {
            subject,
            subject_ty,
            arms,
        } => {
            try_lower_simple_match_stmt(subject, subject_ty, arms, in_loop_with_else, bindings, ctx)
        }
        HirStmt::NestedFunction { func } => {
            try_lower_simple_nested_function_stmt(func, in_loop_with_else, bindings)
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            try_lower_simple_try_except_stmt(body, handlers, in_loop_with_else, bindings, ctx)
        }
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

fn try_lower_simple_nested_function_stmt(
    func: &HirFunction,
    in_loop_with_else: bool,
    outer_bindings: SimpleStmtBindings<'_>,
) -> Option<Vec<RustStmt>> {
    if func.method_kind != MethodKind::Regular
        || !func.decorators.is_empty()
        || !func.type_params.is_empty()
    {
        return None;
    }
    if func
        .params
        .iter()
        .any(|param| param.default.is_some() || param.keyword_only)
    {
        return None;
    }

    let nested_mutated_vars = collect_mutated_vars(&func.body);
    let nested_borrowed_params: HashSet<String> = HashSet::new();
    let is_recursive = body_calls_function(&func.body, &func.name);
    let allowed_calls = if is_recursive {
        vec![func.name.clone()]
    } else {
        vec![]
    };
    let mut lowered_body = crate::with_allowed_plain_calls(&allowed_calls, || {
        try_lower_simple_stmt_block(
            &func.body,
            in_loop_with_else,
            &nested_mutated_vars,
            &nested_borrowed_params,
            SimpleStmtLoweringCtx {
                return_type: Some(&func.return_type),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
    })?;
    let param_names: HashSet<String> = func.params.iter().map(|param| param.name.clone()).collect();
    let referenced_with_types = collect_referenced_vars_with_types(&func.body);
    let locally_defined = collect_locally_defined_vars(&func.body);
    let captures: Vec<(String, Type)> = referenced_with_types
        .into_iter()
        .filter(|(name, _)| !param_names.contains(name) && !locally_defined.contains(name))
        .collect();

    if is_recursive {
        let capture_names = captures
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        if !capture_names.is_empty() {
            append_recursive_capture_args_to_stmts(&mut lowered_body, &func.name, &capture_names);
        }
        let fn_params = func
            .params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: crate::sifr_type_to_rust_type(&param.ty),
            })
            .chain(captures.iter().map(|(name, ty)| RustParam::Named {
                name: name.clone(),
                ty: crate::sifr_type_to_rust_type(ty),
            }))
            .collect::<Vec<_>>();
        let ret = if matches!(func.return_type, Type::None) {
            None
        } else {
            Some(crate::sifr_type_to_rust_type(&func.return_type))
        };
        return Some(vec![RustStmt::LocalFn {
            name: func.name.clone(),
            params: fn_params,
            ret,
            body: lowered_body,
        }]);
    }

    let lowered_params = func
        .params
        .iter()
        .map(|param| RustParam::Named {
            name: param.name.clone(),
            ty: RustType::Named("_".to_string()),
        })
        .collect::<Vec<_>>();

    Some(vec![RustStmt::Let {
        mutable: outer_bindings.mutated_vars.contains(&func.name),
        name: func.name.clone(),
        ty: None,
        value: RustExpr::ClosureBlock {
            params: lowered_params,
            body: lowered_body,
            is_move: false,
        },
    }])
}

fn append_recursive_capture_args_to_stmts(
    stmts: &mut [RustStmt],
    fn_name: &str,
    capture_names: &[String],
) {
    for stmt in stmts {
        match stmt {
            RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => {
                append_recursive_capture_args_to_expr(value, fn_name, capture_names);
            }
            RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
                append_recursive_capture_args_to_expr(target, fn_name, capture_names);
                append_recursive_capture_args_to_expr(value, fn_name, capture_names);
            }
            RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            }
            RustStmt::Assert { cond, msg } => {
                append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
                if let Some(msg) = msg {
                    append_recursive_capture_args_to_expr(msg, fn_name, capture_names);
                }
            }
            RustStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(then_body, fn_name, capture_names);
                if let Some(else_body) = else_body {
                    append_recursive_capture_args_to_stmts(else_body, fn_name, capture_names);
                }
            }
            RustStmt::IfLet {
                expr,
                then_body,
                else_body,
                ..
            } => {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(then_body, fn_name, capture_names);
                if let Some(else_body) = else_body {
                    append_recursive_capture_args_to_stmts(else_body, fn_name, capture_names);
                }
            }
            RustStmt::Match { expr, arms } => {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        append_recursive_capture_args_to_expr(guard, fn_name, capture_names);
                    }
                    append_recursive_capture_args_to_stmts(&mut arm.body, fn_name, capture_names);
                }
            }
            RustStmt::For { iter, body, .. } => {
                append_recursive_capture_args_to_expr(iter, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::With { items, body } => {
                for item in items {
                    append_recursive_capture_args_to_expr(&mut item.value, fn_name, capture_names);
                }
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::While { cond, body } => {
                append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::Loop { body } | RustStmt::Block(body) => {
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::LocalFn { body, .. } => {
                append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
            }
            RustStmt::Return(None)
            | RustStmt::Break
            | RustStmt::Continue
            | RustStmt::RawCode(_) => {}
        }
    }
}

fn append_recursive_capture_args_to_expr(
    expr: &mut RustExpr,
    fn_name: &str,
    capture_names: &[String],
) {
    match expr {
        RustExpr::FnCall { func, args } => {
            append_recursive_capture_args_to_expr(func, fn_name, capture_names);
            for arg in args.iter_mut() {
                append_recursive_capture_args_to_expr(arg, fn_name, capture_names);
            }
            if matches!(func.as_ref(), RustExpr::Ident(name) if name == fn_name) {
                for capture_name in capture_names {
                    args.push(RustExpr::Ident(capture_name.clone()));
                }
            }
        }
        RustExpr::MethodCall { receiver, args, .. } => {
            append_recursive_capture_args_to_expr(receiver, fn_name, capture_names);
            for arg in args {
                append_recursive_capture_args_to_expr(arg, fn_name, capture_names);
            }
        }
        RustExpr::Field { expr, .. } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
        }
        RustExpr::Index { expr, index } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            append_recursive_capture_args_to_expr(index, fn_name, capture_names);
        }
        RustExpr::Slice { expr, start, stop } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            if let Some(start) = start {
                append_recursive_capture_args_to_expr(start, fn_name, capture_names);
            }
            if let Some(stop) = stop {
                append_recursive_capture_args_to_expr(stop, fn_name, capture_names);
            }
        }
        RustExpr::BinOp { left, right, .. } => {
            append_recursive_capture_args_to_expr(left, fn_name, capture_names);
            append_recursive_capture_args_to_expr(right, fn_name, capture_names);
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Paren(operand) => {
            append_recursive_capture_args_to_expr(operand, fn_name, capture_names);
        }
        RustExpr::Cast { expr, .. } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
        }
        RustExpr::Ref { expr, .. } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
        }
        RustExpr::Block { stmts, expr } => {
            append_recursive_capture_args_to_stmts(stmts, fn_name, capture_names);
            if let Some(expr) = expr {
                append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            }
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            append_recursive_capture_args_to_expr(cond, fn_name, capture_names);
            append_recursive_capture_args_to_expr(then_expr, fn_name, capture_names);
            if let Some(else_expr) = else_expr {
                append_recursive_capture_args_to_expr(else_expr, fn_name, capture_names);
            }
        }
        RustExpr::Match { expr, arms } => {
            append_recursive_capture_args_to_expr(expr, fn_name, capture_names);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    append_recursive_capture_args_to_expr(guard, fn_name, capture_names);
                }
                append_recursive_capture_args_to_stmts(&mut arm.body, fn_name, capture_names);
            }
        }
        RustExpr::Closure { body, .. } => {
            append_recursive_capture_args_to_expr(body, fn_name, capture_names);
        }
        RustExpr::ClosureBlock { body, .. } => {
            append_recursive_capture_args_to_stmts(body, fn_name, capture_names);
        }
        RustExpr::StructInit { fields, .. } => {
            for (_, field_value) in fields {
                append_recursive_capture_args_to_expr(field_value, fn_name, capture_names);
            }
        }
        RustExpr::Tuple(items) | RustExpr::Vec(items) | RustExpr::MacroCall { args: items, .. } => {
            for item in items {
                append_recursive_capture_args_to_expr(item, fn_name, capture_names);
            }
        }
        RustExpr::FormatMacro { args, .. } => {
            for arg in args {
                append_recursive_capture_args_to_expr(arg, fn_name, capture_names);
            }
        }
        RustExpr::Range { start, end } => {
            append_recursive_capture_args_to_expr(start, fn_name, capture_names);
            append_recursive_capture_args_to_expr(end, fn_name, capture_names);
        }
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) | RustExpr::RawCode(_) => {}
    }
}

fn try_lower_simple_try_except_stmt(
    body: &[HirStmt],
    handlers: &[HirExceptHandler],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if handlers.len() != 1 {
        return None;
    }
    let handler = handlers.first()?;
    if handler
        .error_type
        .as_deref()
        .is_some_and(|error_type| error_type != "Error")
    {
        return None;
    }
    if !body.iter().all(is_simple_try_except_body_stmt)
        || !handler.body.iter().all(is_simple_try_except_body_stmt)
    {
        return None;
    }
    if !body.iter().any(stmt_has_result_flow) {
        return None;
    }

    let lowered_try_body = try_lower_simple_stmt_block(
        body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?;
    let lowered_handler_body = try_lower_simple_stmt_block(
        &handler.body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?;

    let mut closure_body = lowered_try_body;
    closure_body.push(RustStmt::Return(Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
        args: vec![RustExpr::Literal(RustLiteral::Unit)],
    })));
    let handler_name = handler.name.clone().unwrap_or_else(|| "_e".to_string());

    Some(vec![
        RustStmt::Let {
            mutable: false,
            name: "__sifr_try_res".to_string(),
            ty: Some(RustType::Result(
                Box::new(RustType::Unit),
                Box::new(RustType::Named("Error".to_string())),
            )),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::ClosureBlock {
                    params: vec![],
                    body: closure_body,
                    is_move: false,
                }),
                args: vec![],
            },
        },
        RustStmt::IfLet {
            pattern: format!("Err({handler_name})"),
            expr: RustExpr::Ident("__sifr_try_res".to_string()),
            then_body: lowered_handler_body,
            else_body: None,
        },
    ])
}

fn is_simple_try_except_body_stmt(stmt: &HirStmt) -> bool {
    matches!(
        stmt,
        HirStmt::Expr { .. }
            | HirStmt::Let { .. }
            | HirStmt::Assign { .. }
            | HirStmt::AugAssign { .. }
            | HirStmt::AttributeAugAssign { .. }
            | HirStmt::FieldAssign { .. }
            | HirStmt::Assert { .. }
            | HirStmt::Raise { .. }
            | HirStmt::TupleUnpack { .. }
            | HirStmt::StarUnpack { .. }
            | HirStmt::SubscriptAssign { .. }
            | HirStmt::NestedSubscriptAssign { .. }
            | HirStmt::SubscriptAugAssign { .. }
            | HirStmt::AttributeSubscriptAssign { .. }
            | HirStmt::Delete { .. }
            | HirStmt::Pass
    )
}

fn stmt_has_result_flow(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Raise { .. } => true,
        HirStmt::Expr { expr } => expr_has_result_flow(expr),
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. } => expr_has_result_flow(value),
        HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::SubscriptAssign { value, .. }
        | HirStmt::NestedSubscriptAssign { value, .. }
        | HirStmt::SubscriptAugAssign { value, .. }
        | HirStmt::AttributeSubscriptAssign { value, .. } => expr_has_result_flow(value),
        HirStmt::Assert { test, msg } => {
            expr_has_result_flow(test) || msg.as_ref().is_some_and(expr_has_result_flow)
        }
        HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => {
            expr_has_result_flow(value)
        }
        HirStmt::Delete { object, index } => {
            expr_has_result_flow(object) || expr_has_result_flow(index)
        }
        HirStmt::Pass => false,
        HirStmt::Return { .. }
        | HirStmt::If { .. }
        | HirStmt::While { .. }
        | HirStmt::For { .. }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::TryExcept { .. }
        | HirStmt::With { .. }
        | HirStmt::Match { .. }
        | HirStmt::Yield { .. }
        | HirStmt::NestedFunction { .. } => false,
    }
}

fn expr_has_result_flow(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::QuestionMark { .. } | HirExpr::OkWrap { .. } | HirExpr::ErrWrap { .. } => true,
        HirExpr::UnaryOp { operand, .. } => expr_has_result_flow(operand),
        HirExpr::BinOp { left, right, .. } => {
            expr_has_result_flow(left) || expr_has_result_flow(right)
        }
        HirExpr::Compare {
            left, comparators, ..
        } => expr_has_result_flow(left) || comparators.iter().any(expr_has_result_flow),
        HirExpr::BoolOp { values, .. } => values.iter().any(expr_has_result_flow),
        HirExpr::Call { args, .. }
        | HirExpr::MethodCall { args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::SuperCall { args, .. } => args.iter().any(expr_has_result_flow),
        HirExpr::Index { object, index, .. } => {
            expr_has_result_flow(object) || expr_has_result_flow(index)
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            expr_has_result_flow(object)
                || start.as_ref().is_some_and(|e| expr_has_result_flow(e))
                || stop.as_ref().is_some_and(|e| expr_has_result_flow(e))
                || step.as_ref().is_some_and(|e| expr_has_result_flow(e))
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_has_result_flow(condition)
                || expr_has_result_flow(then_expr)
                || expr_has_result_flow(else_expr)
        }
        HirExpr::TupleLiteral { elements, .. }
        | HirExpr::ListLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. } => elements.iter().any(expr_has_result_flow),
        HirExpr::DictLiteral { keys, values, .. } => keys
            .iter()
            .zip(values.iter())
            .any(|(k, v)| expr_has_result_flow(k) || expr_has_result_flow(v)),
        HirExpr::FString { parts, .. } => parts.iter().any(|part| match part {
            sifr_hir::HirFStringPart::Literal(_) => false,
            sifr_hir::HirFStringPart::Expr(e) => expr_has_result_flow(e),
        }),
        HirExpr::Lambda { body, .. } => expr_has_result_flow(body),
        HirExpr::WalrusExpr { value, .. } => expr_has_result_flow(value),
        HirExpr::FieldAccess { object, .. } => expr_has_result_flow(object),
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => expr_has_result_flow(element) || expr_has_result_flow(collection),
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            expr_has_result_flow(start)
                || expr_has_result_flow(end)
                || step.as_ref().is_some_and(|e| expr_has_result_flow(e))
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            expr_has_result_flow(expr)
                || generators.iter().any(|(_, iter, cond)| {
                    expr_has_result_flow(iter) || cond.as_ref().is_some_and(expr_has_result_flow)
                })
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            expr_has_result_flow(key_expr)
                || expr_has_result_flow(val_expr)
                || generators.iter().any(|(_, iter, cond)| {
                    expr_has_result_flow(iter) || cond.as_ref().is_some_and(expr_has_result_flow)
                })
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            expr_has_result_flow(expr)
                || expr_has_result_flow(iter)
                || filter.as_ref().is_some_and(|c| expr_has_result_flow(c))
        }
        HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => false,
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

fn try_lower_simple_tuple_unpack_stmt(
    targets: &[(String, Type)],
    value: &HirExpr,
) -> Option<Vec<RustStmt>> {
    if targets.is_empty() {
        return None;
    }
    Some(vec![RustStmt::LetPattern {
        pattern: tuple_unpack_pattern(targets),
        value: try_lower_leaf_or_name_expr(value)?,
    }])
}

fn try_lower_simple_star_unpack_stmt(
    before: &[(String, Type)],
    star: &(String, Type),
    after: &[(String, Type)],
    value: &HirExpr,
) -> Option<Vec<RustStmt>> {
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    let mut lowered = vec![RustStmt::Let {
        mutable: false,
        name: "_star_tmp".to_string(),
        ty: None,
        value: RustExpr::MethodCall {
            receiver: Box::new(lowered_value),
            method: "clone".to_string(),
            args: vec![],
        },
    }];

    let tmp_ident = || RustExpr::Ident("_star_tmp".to_string());
    let tmp_len = || RustExpr::MethodCall {
        receiver: Box::new(tmp_ident()),
        method: "len".to_string(),
        args: vec![],
    };

    for (idx, (name, _)) in before.iter().enumerate() {
        lowered.push(RustStmt::Let {
            mutable: false,
            name: name.clone(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Index {
                    expr: Box::new(tmp_ident()),
                    index: Box::new(RustExpr::Literal(RustLiteral::Int(
                        i64::try_from(idx).ok()?,
                    ))),
                }),
                method: "clone".to_string(),
                args: vec![],
            },
        });
    }

    let (star_name, _) = star;
    let slice_end = if after.is_empty() {
        tmp_len()
    } else {
        RustExpr::BinOp {
            left: Box::new(tmp_len()),
            op: "-".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(
                i64::try_from(after.len()).ok()?,
            ))),
        }
    };
    lowered.push(RustStmt::Let {
        mutable: false,
        name: star_name.clone(),
        ty: None,
        value: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Index {
                expr: Box::new(tmp_ident()),
                index: Box::new(RustExpr::Range {
                    start: Box::new(RustExpr::Literal(RustLiteral::Int(
                        i64::try_from(before.len()).ok()?,
                    ))),
                    end: Box::new(slice_end),
                }),
            }),
            method: "to_vec".to_string(),
            args: vec![],
        },
    });

    for (idx, (name, _)) in after.iter().enumerate() {
        lowered.push(RustStmt::Let {
            mutable: false,
            name: name.clone(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Index {
                    expr: Box::new(tmp_ident()),
                    index: Box::new(RustExpr::BinOp {
                        left: Box::new(tmp_len()),
                        op: "-".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(
                            i64::try_from(after.len() - idx).ok()?,
                        ))),
                    }),
                }),
                method: "clone".to_string(),
                args: vec![],
            },
        });
    }

    Some(lowered)
}

fn try_lower_loop_else_stmts(
    loop_stmt: RustStmt,
    else_body: &[HirStmt],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    Some(vec![
        RustStmt::Let {
            mutable: true,
            name: "_broke".to_string(),
            ty: None,
            value: RustExpr::Literal(RustLiteral::Bool(false)),
        },
        loop_stmt,
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
    ])
}

fn try_lower_simple_with_stmt(
    items: &[(String, HirExpr, bool)],
    body: &[HirStmt],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if items.iter().any(|(_, _, has_cm)| *has_cm) {
        return None;
    }

    let mut block = Vec::new();
    for (name, value, _) in items {
        block.push(RustStmt::Let {
            mutable: false,
            name: name.clone(),
            ty: None,
            value: try_lower_leaf_or_name_expr(value)?,
        });
    }

    block.extend(try_lower_simple_stmt_block(
        body,
        in_loop_with_else,
        bindings.mutated_vars,
        bindings.borrowed_params,
        ctx,
    )?);

    Some(vec![RustStmt::Block(block)])
}

fn try_lower_simple_yield_stmt(
    value: &HirExpr,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    if ctx.in_generator_closure {
        return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_value],
        }))]);
    }

    Some(vec![RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("_yields".to_string())),
        method: "push".to_string(),
        args: vec![lowered_value],
    })])
}

fn try_lower_simple_match_stmt(
    subject: &HirExpr,
    subject_ty: &Type,
    arms: &[sifr_hir::HirMatchArm],
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    let lowered_subject = try_lower_leaf_or_name_expr(subject)?;
    let subject_is_borrowed_name =
        matches!(subject, HirExpr::Name { name, .. } if bindings.borrowed_params.contains(name));
    let lowered_arms = arms
        .iter()
        .map(|arm| {
            let (pattern, arm_bindings, auto_guard) =
                if matches!(resolve_alias_type(subject_ty), Type::Str) {
                    try_lower_match_pattern_for_string_subject(&arm.pattern)?
                } else if let Some((pattern, bindings)) =
                    try_lower_union_class_match_pattern(&arm.pattern, subject_ty)
                {
                    (pattern, bindings, None)
                } else {
                    let (pattern, arm_bindings) = try_lower_match_pattern(&arm.pattern)?;
                    (pattern, arm_bindings, None)
                };
            let mut lowered_guard = arm.guard.as_ref().and_then(try_lower_leaf_or_name_expr);
            if subject_is_borrowed_name {
                let copy_captures = collect_copy_capture_names(&arm.pattern);
                if !copy_captures.is_empty() {
                    lowered_guard =
                        lowered_guard.map(|guard| deref_guard_copy_captures(guard, &copy_captures));
                }
            }
            let guard = match (auto_guard, lowered_guard) {
                (Some(left), Some(right)) => Some(RustExpr::BinOp {
                    left: Box::new(left),
                    op: "&&".to_string(),
                    right: Box::new(right),
                }),
                (Some(left), None) => Some(left),
                (None, Some(right)) => Some(right),
                (None, None) => None,
            };
            let body = try_lower_simple_stmt_block(
                &arm.body,
                in_loop_with_else,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?;
            Some(RustMatchArm {
                pattern,
                bindings: arm_bindings,
                guard,
                body,
            })
        })
        .collect::<Option<Vec<_>>>()?;

    Some(vec![RustStmt::Match {
        expr: lowered_subject,
        arms: lowered_arms,
    }])
}

fn try_lower_match_pattern_for_string_subject(
    pattern: &HirPattern,
) -> Option<(String, Vec<String>, Option<RustExpr>)> {
    match pattern {
        HirPattern::Literal {
            value: HirExpr::StringLiteral(_),
        } => Some((
            "__s".to_string(),
            vec![],
            Some(try_lower_string_literal_match_guard(pattern)?),
        )),
        HirPattern::Or { patterns } => {
            if patterns.iter().any(|p| matches!(p, HirPattern::Wildcard)) {
                return Some(("_".to_string(), vec![], None));
            }
            Some((
                "__s".to_string(),
                vec![],
                Some(try_lower_string_literal_match_guard(pattern)?),
            ))
        }
        _ => {
            let (pattern, bindings) = try_lower_match_pattern(pattern)?;
            Some((pattern, bindings, None))
        }
    }
}

fn try_lower_string_literal_match_guard(pattern: &HirPattern) -> Option<RustExpr> {
    match pattern {
        HirPattern::Literal {
            value: HirExpr::StringLiteral(expected),
        } => Some(RustExpr::BinOp {
            left: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__s".to_string())),
                method: "as_str".to_string(),
                args: vec![],
            }),
            op: "==".to_string(),
            right: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Literal(RustLiteral::Str(expected.clone()))),
                method: "as_str".to_string(),
                args: vec![],
            }),
        }),
        HirPattern::Or { patterns } => {
            let mut guards = Vec::with_capacity(patterns.len());
            for pattern in patterns {
                guards.push(try_lower_string_literal_match_guard(pattern)?);
            }
            guards.into_iter().reduce(|left, right| RustExpr::BinOp {
                left: Box::new(left),
                op: "||".to_string(),
                right: Box::new(right),
            })
        }
        _ => None,
    }
}

fn try_lower_match_pattern(pattern: &HirPattern) -> Option<(String, Vec<String>)> {
    match pattern {
        HirPattern::Wildcard => Some(("_".to_string(), vec![])),
        HirPattern::Capture { name, .. } => Some((name.clone(), vec![name.clone()])),
        HirPattern::Literal { value } => Some((try_lower_match_literal_pattern(value)?, vec![])),
        HirPattern::None => Some(("None".to_string(), vec![])),
        HirPattern::Value { path } => Some((path.join("::"), vec![])),
        HirPattern::Or { patterns } => {
            let mut rendered = Vec::new();
            for p in patterns {
                let (pat, binds) = try_lower_match_pattern(p)?;
                // Keep conservative support only for OR-patterns without bindings.
                if !binds.is_empty() {
                    return None;
                }
                rendered.push(pat);
            }
            Some((rendered.join(" | "), vec![]))
        }
        HirPattern::Tuple { elements } => {
            let mut rendered = Vec::new();
            let mut bindings = Vec::new();
            for element in elements {
                let (pat, binds) = try_lower_match_pattern(element)?;
                rendered.push(pat);
                bindings.extend(binds);
            }
            Some((format!("({})", rendered.join(", ")), bindings))
        }
        HirPattern::Class { class_name, fields } => {
            let mut rendered_fields = Vec::new();
            let mut bindings = Vec::new();
            for (field_name, field_pattern) in fields {
                let (field_pat, field_binds) = try_lower_match_pattern(field_pattern)?;
                rendered_fields.push(format!("{field_name}: {field_pat}"));
                bindings.extend(field_binds);
            }
            if rendered_fields.is_empty() {
                Some((format!("{class_name} {{ .. }}"), bindings))
            } else {
                Some((
                    format!("{class_name} {{ {}, .. }}", rendered_fields.join(", ")),
                    bindings,
                ))
            }
        }
    }
}

fn try_lower_union_class_match_pattern(
    pattern: &HirPattern,
    subject_ty: &Type,
) -> Option<(String, Vec<String>)> {
    let Type::Union(members) = resolve_alias_type(subject_ty) else {
        return None;
    };
    let HirPattern::Class { class_name, fields } = pattern else {
        return None;
    };

    let target_ty = match class_name.as_str() {
        "int" => Some(Type::Int),
        "str" => Some(Type::Str),
        "float" => Some(Type::Float),
        "bool" => Some(Type::Bool),
        other => members
            .iter()
            .find(|m| matches!(m, Type::Class { name, .. } if name == other))
            .cloned(),
    }?;
    if !members.contains(&target_ty) {
        return None;
    }

    let enum_name = resolve_alias_type(subject_ty).union_enum_name();
    let variant_name = target_ty.union_variant_name();
    if fields.is_empty() {
        return Some((format!("{enum_name}::{variant_name}(..)"), vec![]));
    }
    if !matches!(target_ty, Type::Class { .. }) {
        return None;
    }
    let mut rendered_fields = Vec::new();
    let mut bindings = Vec::new();
    for (field_name, field_pattern) in fields {
        let (field_pat, field_binds) = try_lower_match_pattern(field_pattern)?;
        rendered_fields.push(format!("{field_name}: {field_pat}"));
        bindings.extend(field_binds);
    }
    Some((
        format!(
            "{enum_name}::{variant_name}({class_name} {{ {}, .. }})",
            rendered_fields.join(", ")
        ),
        bindings,
    ))
}

fn is_copy_capture_type(ty: &Type) -> bool {
    matches!(
        resolve_alias_type(ty),
        Type::Int | Type::LiteralInt(_) | Type::Float | Type::Bool | Type::LiteralBool(_)
    )
}

fn collect_copy_capture_names(pattern: &HirPattern) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_copy_capture_names_inner(pattern, &mut names);
    names
}

fn collect_copy_capture_names_inner(pattern: &HirPattern, out: &mut HashSet<String>) {
    match pattern {
        HirPattern::Capture { name, ty } if is_copy_capture_type(ty) => {
            out.insert(name.clone());
        }
        HirPattern::Class { fields, .. } => {
            for (_, field_pattern) in fields {
                collect_copy_capture_names_inner(field_pattern, out);
            }
        }
        HirPattern::Tuple { elements } => {
            for element in elements {
                collect_copy_capture_names_inner(element, out);
            }
        }
        HirPattern::Or { patterns } => {
            for pattern in patterns {
                collect_copy_capture_names_inner(pattern, out);
            }
        }
        _ => {}
    }
}

fn deref_guard_copy_captures(expr: RustExpr, captures: &HashSet<String>) -> RustExpr {
    match expr {
        RustExpr::Ident(name) if captures.contains(&name) => {
            RustExpr::Deref(Box::new(RustExpr::Ident(name)))
        }
        RustExpr::BinOp { left, op, right } => RustExpr::BinOp {
            left: Box::new(deref_guard_copy_captures(*left, captures)),
            op,
            right: Box::new(deref_guard_copy_captures(*right, captures)),
        },
        RustExpr::UnaryOp { op, operand } => RustExpr::UnaryOp {
            op,
            operand: Box::new(deref_guard_copy_captures(*operand, captures)),
        },
        RustExpr::FnCall { func, args } => RustExpr::FnCall {
            func: Box::new(deref_guard_copy_captures(*func, captures)),
            args: args
                .into_iter()
                .map(|arg| deref_guard_copy_captures(arg, captures))
                .collect(),
        },
        RustExpr::MethodCall {
            receiver,
            method,
            args,
        } => RustExpr::MethodCall {
            receiver: Box::new(deref_guard_copy_captures(*receiver, captures)),
            method,
            args: args
                .into_iter()
                .map(|arg| deref_guard_copy_captures(arg, captures))
                .collect(),
        },
        RustExpr::Ref { mutable, expr } => RustExpr::Ref {
            mutable,
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
        },
        RustExpr::Deref(expr) => {
            RustExpr::Deref(Box::new(deref_guard_copy_captures(*expr, captures)))
        }
        RustExpr::Cast { expr, ty } => RustExpr::Cast {
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
            ty,
        },
        RustExpr::Field { expr, field } => RustExpr::Field {
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
            field,
        },
        RustExpr::Index { expr, index } => RustExpr::Index {
            expr: Box::new(deref_guard_copy_captures(*expr, captures)),
            index: Box::new(deref_guard_copy_captures(*index, captures)),
        },
        other => other,
    }
}

fn try_lower_match_literal_pattern(expr: &HirExpr) -> Option<String> {
    match expr {
        HirExpr::IntLiteral(v) => Some(v.to_string()),
        HirExpr::FloatLiteral(v) => {
            let mut s = v.to_string();
            if !s.contains('.') {
                s.push_str(".0");
            }
            Some(s)
        }
        HirExpr::StringLiteral(s) => Some(format!("{s:?}")),
        HirExpr::BoolLiteral(v) => Some(v.to_string()),
        HirExpr::NoneLiteral => Some("None".to_string()),
        HirExpr::EnumVariant {
            enum_name, variant, ..
        } => Some(format!("{enum_name}::{variant}")),
        _ => None,
    }
}

fn try_lower_simple_while_stmt(
    condition: &HirExpr,
    body: &[HirStmt],
    else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if let Some(else_body) = else_body {
        return try_lower_loop_else_stmts(
            RustStmt::While {
                cond: try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(
                    body,
                    true,
                    bindings.mutated_vars,
                    bindings.borrowed_params,
                    ctx,
                )?,
            },
            else_body,
            in_loop_with_else,
            bindings,
            ctx,
        );
    }

    Some(vec![RustStmt::While {
        cond: try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?,
        // Entering a nested while without else resets loop-else break marker context.
        body: try_lower_simple_stmt_block(
            body,
            false,
            bindings.mutated_vars,
            bindings.borrowed_params,
            ctx,
        )?,
    }])
}

fn try_lower_simple_for_stmt(
    target: &str,
    iter: &HirExpr,
    body: &[HirStmt],
    else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if target.contains(',') {
        return None;
    }

    if let Some(else_body) = else_body {
        return try_lower_loop_else_stmts(
            RustStmt::For {
                var: target.to_string(),
                iter: try_lower_simple_for_iter_expr(iter)?,
                // Breaks in the loop body should mark this loop's `_broke`.
                body: try_lower_simple_stmt_block(
                    body,
                    true,
                    bindings.mutated_vars,
                    bindings.borrowed_params,
                    ctx,
                )?,
            },
            else_body,
            in_loop_with_else,
            bindings,
            ctx,
        );
    }

    Some(vec![RustStmt::For {
        var: target.to_string(),
        iter: try_lower_simple_for_iter_expr(iter)?,
        // Entering a nested for without else resets loop-else break marker context.
        body: try_lower_simple_stmt_block(
            body,
            false,
            bindings.mutated_vars,
            bindings.borrowed_params,
            ctx,
        )?,
    }])
}

fn try_lower_simple_for_iter_expr(iter: &HirExpr) -> Option<RustExpr> {
    fn is_collect_call_expr(expr: &RustExpr) -> bool {
        match expr {
            RustExpr::MethodCall { method, .. } => {
                method == "collect" || method.starts_with("collect::<")
            }
            RustExpr::Paren(inner) => is_collect_call_expr(inner),
            _ => false,
        }
    }

    fn normalize_for_iter_expr(expr: RustExpr) -> RustExpr {
        match expr {
            RustExpr::MethodCall {
                receiver,
                method,
                args,
            } => {
                let normalized_receiver = Box::new(normalize_for_iter_expr(*receiver));
                let normalized_args = args
                    .into_iter()
                    .map(normalize_for_iter_expr)
                    .collect::<Vec<_>>();

                if method == "cloned"
                    && normalized_args.is_empty()
                    && is_collect_call_expr(&normalized_receiver)
                {
                    return *normalized_receiver;
                }

                RustExpr::MethodCall {
                    receiver: normalized_receiver,
                    method,
                    args: normalized_args,
                }
            }
            RustExpr::Paren(inner) => RustExpr::Paren(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Try(inner) => RustExpr::Try(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Await(inner) => RustExpr::Await(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Deref(inner) => RustExpr::Deref(Box::new(normalize_for_iter_expr(*inner))),
            RustExpr::Clone(inner) => RustExpr::Clone(Box::new(normalize_for_iter_expr(*inner))),
            other => other,
        }
    }

    let lowered_iter = try_lower_leaf_or_name_expr(iter)?;
    let lowered_iter = normalize_for_iter_expr(lowered_iter);
    Some(match resolve_alias_type(iter.ty()) {
        Type::List(_) => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
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
        Type::Str => RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(lowered_iter),
                method: "chars".to_string(),
                args: vec![],
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "c".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("c".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                }),
                is_move: false,
            }],
        },
        _ => lowered_iter,
    })
}

fn try_lower_simple_if_stmt(
    condition: &HirExpr,
    then_body: &[HirStmt],
    elif_clauses: &[(HirExpr, Vec<HirStmt>)],
    maybe_else_body: Option<&[HirStmt]>,
    in_loop_with_else: bool,
    bindings: SimpleStmtBindings<'_>,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if elif_clauses.is_empty() && maybe_else_body.is_none() && codegen_body_always_exits(then_body)
    {
        if let Some(option_var) = detect_is_none_var(condition) {
            let lowered_cond =
                try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?;
            let lowered_then_body = try_lower_simple_stmt_block(
                then_body,
                in_loop_with_else,
                bindings.mutated_vars,
                bindings.borrowed_params,
                ctx,
            )?;
            return Some(vec![
                RustStmt::If {
                    cond: lowered_cond,
                    then_body: lowered_then_body,
                    else_body: None,
                },
                RustStmt::Let {
                    mutable: false,
                    name: option_var.clone(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(option_var)),
                        method: "unwrap".to_string(),
                        args: vec![],
                    },
                },
            ]);
        }
    }

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

    Some(vec![try_lower_simple_if_clause(
        condition,
        then_body,
        nested_else,
        in_loop_with_else,
        bindings,
        ctx,
    )?])
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

    if let Some(option_var) = detect_is_not_none_var(condition) {
        return Some(RustStmt::IfLet {
            pattern: format!("Some({option_var})"),
            expr: RustExpr::Ident(option_var),
            then_body: lowered_then_body,
            else_body: nested_else,
        });
    }

    if let Some(option_vars) = detect_and_not_none_vars(condition) {
        return lower_if_not_none_chain(&option_vars, lowered_then_body, nested_else);
    }

    if let Some(option_var) = detect_option_truthiness_alias(condition) {
        return Some(RustStmt::IfLet {
            pattern: format!("Some({option_var})"),
            expr: RustExpr::Ident(option_var),
            then_body: lowered_then_body,
            else_body: nested_else,
        });
    }

    if let Some(option_var) = detect_is_none_var(condition) {
        let lowered_cond =
            try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?;
        let lowered_else = nested_else.map(|else_body| {
            vec![RustStmt::IfLet {
                pattern: format!("Some({option_var})"),
                expr: RustExpr::Ident(option_var.clone()),
                then_body: else_body,
                else_body: None,
            }]
        });
        return Some(RustStmt::If {
            cond: lowered_cond,
            then_body: lowered_then_body,
            else_body: lowered_else,
        });
    }

    Some(RustStmt::If {
        cond: try_lower_simple_condition_test_expr(condition, bindings.borrowed_params)?,
        then_body: lowered_then_body,
        else_body: nested_else,
    })
}

fn try_lower_simple_condition_test_expr(
    expr: &HirExpr,
    borrowed_params: &HashSet<String>,
) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_borrowed_typevar_compare_condition(expr, borrowed_params) {
        return Some(lowered);
    }
    // Borrowed-name comparisons require context-sensitive ownership rewrites.
    // Defer them to the structured stmt emitter path.
    if expr_uses_borrowed_name(expr, borrowed_params) {
        return None;
    }
    if let Some(lowered) = try_lower_structured_compare_condition_expr(expr) {
        return Some(lowered);
    }
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

fn try_lower_structured_compare_condition_expr(expr: &HirExpr) -> Option<RustExpr> {
    if try_lower_leaf_expr(expr).is_some() {
        return None;
    }
    let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    else {
        return None;
    };
    if ops.len() != 1 || comparators.len() != 1 {
        return None;
    }
    let rhs_expr = comparators.first()?;
    let lowered_op = match ops[0].as_str() {
        "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].as_str(),
        "is" => "==",
        "is not" => "!=",
        _ => return None,
    };
    let mut lowered_left = try_lower_condition_operand_expr(left)?;
    let mut lowered_right = try_lower_condition_operand_expr(rhs_expr)?;
    if is_option_like_type(left.ty())
        && !is_option_like_type(rhs_expr.ty())
        && !matches!(rhs_expr, HirExpr::NoneLiteral)
    {
        lowered_right = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_right],
        };
    } else if !is_option_like_type(left.ty())
        && is_option_like_type(rhs_expr.ty())
        && !matches!(left.as_ref(), HirExpr::NoneLiteral)
    {
        lowered_left = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
            args: vec![lowered_left],
        };
    } else if matches!(
        resolve_alias_type(left.ty()),
        Type::Str | Type::LiteralStr(_)
    ) && matches!(
        resolve_alias_type(rhs_expr.ty()),
        Type::Str | Type::LiteralStr(_)
    ) {
        lowered_left = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered_left))),
            method: "as_str".to_string(),
            args: vec![],
        };
        lowered_right = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(lowered_right))),
            method: "as_str".to_string(),
            args: vec![],
        };
    }
    Some(RustExpr::BinOp {
        left: Box::new(lowered_left),
        op: lowered_op.to_string(),
        right: Box::new(lowered_right),
    })
}

fn try_lower_condition_operand_expr(expr: &HirExpr) -> Option<RustExpr> {
    if let Some(lowered) = try_lower_leaf_or_name_expr(expr) {
        return Some(lowered);
    }
    match expr {
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } if method == "len" && args.is_empty() => Some(RustExpr::Cast {
            expr: Box::new(RustExpr::MethodCall {
                receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                method: "len".to_string(),
                args: vec![],
            }),
            ty: RustType::I64,
        }),
        HirExpr::Index { object, index, .. } => {
            try_lower_condition_index_operand_expr(object, index)
        }
        _ => None,
    }
}

fn try_lower_condition_index_operand_expr(object: &HirExpr, index: &HirExpr) -> Option<RustExpr> {
    match resolve_alias_type(object.ty()) {
        Type::Dict(_, _) => {
            let lowered_key = if let HirExpr::StringLiteral(value) = index {
                RustExpr::Ident(format!("{value:?}"))
            } else {
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                }
            };
            Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                    method: "get".to_string(),
                    args: vec![lowered_key],
                }),
                method: "cloned".to_string(),
                args: vec![],
            })
        }
        Type::List(_) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                method: "get".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                    ty: RustType::Named("usize".to_string()),
                }],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        _ => None,
    }
}

fn try_lower_borrowed_typevar_compare_condition(
    expr: &HirExpr,
    borrowed_params: &HashSet<String>,
) -> Option<RustExpr> {
    let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    else {
        return None;
    };
    if ops.len() != 1 || comparators.len() != 1 {
        return None;
    }

    let rhs_expr = comparators.first()?;
    if !matches!(resolve_alias_type(left.ty()), Type::TypeVar(_))
        || !matches!(resolve_alias_type(rhs_expr.ty()), Type::TypeVar(_))
    {
        return None;
    }

    let lowered_op = match ops[0].as_str() {
        "==" | "!=" | "<" | "<=" | ">" | ">=" => ops[0].as_str(),
        "is" => "==",
        "is not" => "!=",
        _ => return None,
    };

    let lower_operand = |operand: &HirExpr| -> Option<RustExpr> {
        let HirExpr::Name { name, .. } = operand else {
            return None;
        };
        let ident = RustExpr::Ident(name.clone());
        if borrowed_params.contains(name) {
            return Some(RustExpr::Deref(Box::new(ident)));
        }
        Some(ident)
    };

    Some(RustExpr::BinOp {
        left: Box::new(lower_operand(left)?),
        op: lowered_op.to_string(),
        right: Box::new(lower_operand(rhs_expr)?),
    })
}

fn expr_uses_borrowed_name(expr: &HirExpr, borrowed_params: &HashSet<String>) -> bool {
    match expr {
        HirExpr::Name { name, .. } => borrowed_params.contains(name),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            expr_uses_borrowed_name(left, borrowed_params)
                || comparators
                    .iter()
                    .any(|c| expr_uses_borrowed_name(c, borrowed_params))
        }
        HirExpr::BoolOp { values, .. } => values
            .iter()
            .any(|v| expr_uses_borrowed_name(v, borrowed_params)),
        HirExpr::UnaryOp { operand, .. } => expr_uses_borrowed_name(operand, borrowed_params),
        HirExpr::BinOp { left, right, .. } => {
            expr_uses_borrowed_name(left, borrowed_params)
                || expr_uses_borrowed_name(right, borrowed_params)
        }
        _ => false,
    }
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

fn lower_if_not_none_chain(
    option_vars: &[String],
    lowered_then_body: Vec<RustStmt>,
    nested_else: Option<Vec<RustStmt>>,
) -> Option<RustStmt> {
    let mut chain_then = lowered_then_body;
    for option_var in option_vars.iter().rev() {
        chain_then = vec![RustStmt::IfLet {
            pattern: format!("Some({option_var})"),
            expr: RustExpr::Ident(option_var.clone()),
            then_body: chain_then,
            else_body: None,
        }];
    }

    let mut chain_root = chain_then.into_iter().next()?;
    if let RustStmt::IfLet { else_body, .. } = &mut chain_root {
        *else_body = nested_else;
    }
    Some(chain_root)
}

fn is_alias_equivalent_type(left: &Type, right: &Type) -> bool {
    left == right || resolve_alias_type(left) == resolve_alias_type(right)
}

fn is_none_type(ty: &Type) -> bool {
    matches!(resolve_alias_type(ty), Type::None)
}

fn is_okwrap_none_expr(expr: &HirExpr) -> bool {
    matches!(
        expr,
        HirExpr::OkWrap { value, .. }
            if matches!(value.as_ref(), HirExpr::NoneLiteral) || is_none_type(value.ty())
    )
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
    if let Some(lowered) = try_lower_stmt_index_expr(expr) {
        return Some(lowered);
    }
    if let Some(lowered) = try_lower_stmt_string_concat_expr(expr) {
        return Some(lowered);
    }
    try_lower_name_ident_expr(expr)
}

fn try_lower_stmt_index_expr(expr: &HirExpr) -> Option<RustExpr> {
    let HirExpr::Index { object, index, .. } = expr else {
        return None;
    };
    match resolve_alias_type(object.ty()) {
        Type::Dict(_, _) => {
            let lowered_key = if let HirExpr::StringLiteral(value) = index.as_ref() {
                RustExpr::Ident(format!("{value:?}"))
            } else {
                RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                }
            };
            Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                    method: "get".to_string(),
                    args: vec![lowered_key],
                }),
                method: "cloned".to_string(),
                args: vec![],
            })
        }
        Type::List(_) => Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(try_lower_leaf_or_name_expr(object)?),
                method: "get".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(try_lower_leaf_or_name_expr(index)?),
                    ty: RustType::Named("usize".to_string()),
                }],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        _ => None,
    }
}

fn try_lower_stmt_string_concat_expr(expr: &HirExpr) -> Option<RustExpr> {
    let HirExpr::BinOp {
        left,
        op,
        right,
        ty,
    } = expr
    else {
        return None;
    };
    if op != "+" || !matches!(resolve_alias_type(ty), Type::Str) {
        return None;
    }

    let mut parts = Vec::new();
    collect_stmt_string_concat_parts(left, &mut parts);
    collect_stmt_string_concat_parts(right, &mut parts);

    if parts
        .iter()
        .all(|part| matches!(part, HirExpr::StringLiteral(_)))
    {
        let mut combined = String::new();
        for part in parts {
            if let HirExpr::StringLiteral(value) = part {
                combined.push_str(value);
            }
        }
        return Some(RustExpr::Literal(RustLiteral::Str(combined)));
    }

    Some(RustExpr::FormatMacro {
        name: "format".to_string(),
        format_str: "{}".repeat(parts.len()),
        args: parts
            .iter()
            .map(|part| try_lower_leaf_or_name_expr(part))
            .collect::<Option<Vec<_>>>()?,
    })
}

fn collect_stmt_string_concat_parts<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
    if let HirExpr::BinOp {
        left,
        op,
        right,
        ty,
    } = expr
    {
        if op == "+" && matches!(resolve_alias_type(ty), Type::Str) {
            collect_stmt_string_concat_parts(left, parts);
            collect_stmt_string_concat_parts(right, parts);
            return;
        }
    }
    parts.push(expr);
}

fn try_lower_attribute_dict_insert_key_expr(index: &HirExpr, field_ty: &Type) -> Option<RustExpr> {
    let Type::Dict(key_ty, _) = resolve_alias_type(field_ty) else {
        return None;
    };

    if matches!(resolve_alias_type(key_ty), Type::Str | Type::TypeVar(_))
        && matches!(index, HirExpr::Name { .. })
    {
        // Preserve borrowed-name key cloning semantics.
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

fn try_lower_simple_subscript_assign_stmt(
    object: &str,
    index: &HirExpr,
    value: &HirExpr,
    object_ty: &Type,
) -> Option<Vec<RustStmt>> {
    let lowered_index = try_lower_leaf_or_name_expr(index)?;
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    match resolve_alias_type(object_ty) {
        Type::List(_) => Some(vec![build_list_subscript_assign_stmt(
            RustExpr::Ident(object.to_string()),
            lowered_index,
            lowered_value,
        )]),
        Type::Dict(_, _) => Some(vec![build_dict_subscript_assign_stmt(
            RustExpr::Ident(object.to_string()),
            lowered_index,
            lowered_value,
        )]),
        _ => None,
    }
}

fn try_lower_simple_delete_stmt(object: &HirExpr, index: &HirExpr) -> Option<Vec<RustStmt>> {
    let receiver = try_lower_name_ident_expr(object)?;
    let lowered_index = try_lower_leaf_or_name_expr(index)?;
    match resolve_alias_type(object.ty()) {
        Type::List(_) => Some(vec![RustStmt::Let {
            mutable: false,
            name: "_".to_string(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(receiver),
                method: "remove".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(lowered_index),
                    ty: RustType::Named("usize".to_string()),
                }],
            },
        }]),
        Type::Dict(_, _) => Some(vec![RustStmt::Let {
            mutable: false,
            name: "_".to_string(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(receiver),
                method: "remove".to_string(),
                args: vec![build_dict_delete_key_arg(index)?],
            },
        }]),
        _ => None,
    }
}

fn build_dict_delete_key_arg(index: &HirExpr) -> Option<RustExpr> {
    if matches!(index, HirExpr::Name { .. }) {
        // Preserve name-key borrowing behavior.
        return None;
    }
    let lowered_index = try_lower_leaf_expr(index)?;
    Some(RustExpr::Ref {
        mutable: false,
        expr: Box::new(lowered_index),
    })
}

fn try_lower_simple_nested_subscript_assign_stmt(
    object: &str,
    outer_index: &HirExpr,
    inner_index: &HirExpr,
    value: &HirExpr,
) -> Option<Vec<RustStmt>> {
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
                receiver: Box::new(RustExpr::Ident(object.to_string())),
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
                    target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                    value: try_lower_leaf_or_name_expr(value)?,
                }],
                else_body: None,
            }],
            else_body: None,
        },
    ])])
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

fn try_lower_simple_return_stmt(
    value: &HirExpr,
    ctx: SimpleStmtLoweringCtx<'_>,
) -> Option<Vec<RustStmt>> {
    if ctx.in_display_impl {
        return None;
    }
    if ctx.in_class_scope && matches!(value, HirExpr::Name { name, .. } if name == "self") {
        return Some(vec![RustStmt::Return(Some(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("self".to_string())),
            method: "clone".to_string(),
            args: vec![],
        }))]);
    }
    let option_return = ctx.return_type.is_some_and(is_option_like_type);
    if matches!(value.ty(), Type::TypeVar(_)) {
        return None;
    }

    if option_return {
        if is_option_like_type(value.ty()) && !is_none_type(value.ty()) {
            return Some(vec![RustStmt::Return(Some(try_lower_name_ident_expr(
                value,
            )?))]);
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

    if matches!(value, HirExpr::NoneLiteral)
        || is_none_type(value.ty())
        || is_okwrap_none_expr(value)
    {
        if let Some(return_ty) = ctx.return_type {
            match resolve_alias_type(return_ty) {
                Type::Result(ok_ty, _) if is_none_type(ok_ty.as_ref()) => {
                    return Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![RustExpr::Literal(RustLiteral::Unit)],
                    }))]);
                }
                Type::None => return Some(vec![RustStmt::Return(None)]),
                _ => {}
            }
        }
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
    Some(vec![RustStmt::Return(Some(try_lower_leaf_or_name_expr(
        value,
    )?))])
}

fn try_lower_simple_let_value(ty: &Type, value: &HirExpr) -> Option<RustExpr> {
    if is_option_like_type(ty) && matches!(value, HirExpr::NoneLiteral) {
        return Some(RustExpr::Literal(RustLiteral::None));
    }
    if is_option_like_type(ty) && is_option_like_type(value.ty()) && !is_none_type(value.ty()) {
        return try_lower_name_ident_expr(value);
    }
    if is_option_like_type(ty) && !is_option_like_type(value.ty()) && !is_none_type(value.ty()) {
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
    try_lower_leaf_or_name_expr(value)
}

fn try_lower_simple_assign_value(
    value: &HirExpr,
    borrowed_params: &HashSet<String>,
) -> Option<RustExpr> {
    // Preserve TypeVar assignment behavior for borrowed params by appending `.clone()`.
    if matches!(value.ty(), Type::TypeVar(_))
        && matches!(value, HirExpr::Name { name, .. } if borrowed_params.contains(name))
    {
        return None;
    }
    try_lower_leaf_or_name_expr(value)
}

fn try_lower_simple_field_assign_stmt(
    object: &str,
    field: &str,
    value: &HirExpr,
) -> Option<Vec<RustStmt>> {
    if object == "self" {
        return None;
    }
    Some(vec![RustStmt::Assign {
        target: RustExpr::Field {
            expr: Box::new(RustExpr::Ident(object.to_string())),
            field: field.to_string(),
        },
        value: try_lower_leaf_or_name_expr(value)?,
    }])
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

fn try_lower_simple_augassign_stmt(
    target: RustExpr,
    op: &str,
    value: &HirExpr,
) -> Option<Vec<RustStmt>> {
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
    fn scope_result_reports_invalid_scope_context() {
        let stmt = HirStmt::Pass;
        let scope_ctx = ScopeContext {
            in_display_impl: true,
            in_generator_closure: true,
            ..ScopeContext::default()
        };

        let err = try_lower_simple_stmt_with_scope_result(
            &stmt,
            &HashSet::new(),
            &HashSet::new(),
            &scope_ctx,
        )
        .expect_err("expected invalid scope context to return lowering error");

        assert!(err
            .message
            .contains("display impl and generator closure cannot both be active"));
    }

    #[test]
    fn scope_result_propagates_stmt_expr_shape_errors() {
        let stmt = HirStmt::Let {
            name: "ok".to_string(),
            ty: Type::Bool,
            value: HirExpr::Compare {
                left: Box::new(HirExpr::IntLiteral(1)),
                ops: vec!["==".to_string()],
                comparators: vec![],
                ty: Type::Bool,
            },
            is_mutable: false,
        };

        let err = try_lower_simple_stmt_with_scope_result(
            &stmt,
            &HashSet::new(),
            &HashSet::new(),
            &ScopeContext::default(),
        )
        .expect_err("invalid compare shape should return lowering error");

        assert!(err.message.contains("ops/comparators length mismatch"));
    }

    #[test]
    fn lowers_pass_and_continue_and_break() {
        let pass = try_lower_simple_stmt(&HirStmt::Pass, false, &HashSet::new(), &HashSet::new())
            .expect("pass lowered");
        assert!(pass.is_empty());

        let cont =
            try_lower_simple_stmt(&HirStmt::Continue, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assign lowered");
        assert!(matches!(lowered[0], RustStmt::Assign { .. }));
    }

    #[test]
    fn lowers_simple_field_assign_for_non_self_target() {
        let stmt = HirStmt::FieldAssign {
            object: "node".to_string(),
            field: "value".to_string(),
            value: HirExpr::Name {
                name: "next_value".to_string(),
                ty: Type::Int,
            },
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("field assign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assign {
                target: RustExpr::Field { ref expr, ref field },
                value: RustExpr::Ident(ref rhs),
            } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "node")
                && field == "value"
                && rhs == "next_value"
        ));
    }

    #[test]
    fn does_not_lower_field_assign_on_self_target() {
        let stmt = HirStmt::FieldAssign {
            object: "self".to_string(),
            field: "value".to_string(),
            value: HirExpr::IntLiteral(1),
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn does_not_lower_field_assign_with_non_leaf_value() {
        let stmt = HirStmt::FieldAssign {
            object: "node".to_string(),
            field: "value".to_string(),
            value: HirExpr::Call {
                func: "compute".to_string(),
                args: vec![],
                ty: Type::Int,
            },
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn lowers_simple_tuple_unpack_stmt() {
        let tuple_unpack = HirStmt::TupleUnpack {
            targets: vec![("a".to_string(), Type::Int), ("b".to_string(), Type::Bool)],
            value: HirExpr::TupleLiteral {
                elements: vec![HirExpr::IntLiteral(1), HirExpr::BoolLiteral(true)],
                ty: Type::Tuple(vec![Type::Int, Type::Bool]),
            },
        };
        let lowered = try_lower_simple_stmt(&tuple_unpack, false, &HashSet::new(), &HashSet::new())
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
            targets: vec![("a".to_string(), Type::Int), ("b".to_string(), Type::Bool)],
            value: HirExpr::Call {
                func: "pair".to_string(),
                args: vec![],
                ty: Type::Tuple(vec![Type::Int, Type::Bool]),
            },
        };

        assert!(
            try_lower_simple_stmt(&tuple_unpack, false, &HashSet::new(), &HashSet::new(),)
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn lowers_simple_list_delete_stmt() {
        let stmt = HirStmt::Delete {
            object: HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            },
            index: HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            },
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("list delete lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                ref name,
                value: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if name == "_"
                && method == "remove"
                && matches!(recv.as_ref(), RustExpr::Ident(obj) if obj == "items")
                && matches!(
                    args.first(),
                    Some(RustExpr::Cast {
                        expr: inner,
                        ty: RustType::Named(usize_ty),
                    }) if matches!(inner.as_ref(), RustExpr::Ident(idx) if idx == "i")
                        && usize_ty == "usize"
                )
        ));
    }

    #[test]
    fn lowers_simple_dict_delete_with_string_literal_key_stmt() {
        let stmt = HirStmt::Delete {
            object: HirExpr::Name {
                name: "mapping".to_string(),
                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
            },
            index: HirExpr::StringLiteral("key".to_string()),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("dict delete lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                mutable: false,
                ref name,
                value: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if name == "_"
                && method == "remove"
                && matches!(recv.as_ref(), RustExpr::Ident(obj) if obj == "mapping")
                && matches!(
                    args.first(),
                    Some(RustExpr::Ref {
                        mutable: false,
                        expr: inner,
                    }) if matches!(inner.as_ref(), RustExpr::Literal(RustLiteral::Str(key)) if key == "key")
                )
        ));
    }

    #[test]
    fn does_not_lower_dict_delete_with_name_key() {
        let stmt = HirStmt::Delete {
            object: HirExpr::Name {
                name: "mapping".to_string(),
                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
            },
            index: HirExpr::Name {
                name: "k".to_string(),
                ty: Type::Str,
            },
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
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

        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
            try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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
            try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
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
            try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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
        let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
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

        assert!(try_lower_simple_stmt(
            &assign_stmt,
            false,
            &HashSet::new(),
            &HashSet::from(["param".to_string()]),
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_augassign_for_supported_ops() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "-=".to_string(),
            value: HirExpr::IntLiteral(2),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
    }

    #[test]
    fn lowers_simple_augassign_plus_equal_numeric() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "+=".to_string(),
            value: HirExpr::IntLiteral(1),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
    }

    #[test]
    fn lowers_simple_augassign_floor_div_equal() {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: "//=".to_string(),
            value: HirExpr::IntLiteral(2),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: None,
                in_display_impl: true,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_return_with_leaf_expr() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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
                in_generator_closure: false,
            },
        )
        .expect("option return leaf lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(
                    matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
                );
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
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
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
                in_generator_closure: false,
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
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_return_in_class_scope() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Int),
                in_display_impl: false,
                in_class_scope: true,
                in_generator_closure: false,
            },
        )
        .is_some());
    }

    #[test]
    fn lowers_self_return_in_class_scope_with_clone() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "self".to_string(),
                ty: Type::Class {
                    name: "Point".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                },
            }),
        };

        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Class {
                    name: "Point".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                }),
                in_display_impl: false,
                in_class_scope: true,
                in_generator_closure: false,
            },
        )
        .expect("self return in class scope lowered");

        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::MethodCall { ref receiver, ref method, ref args }))
                if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "self")
                    && method == "clone"
                    && args.is_empty()
        ));
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
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_raise_with_leaf_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::IntLiteral(7),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("raise lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(
                    matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()])
                );
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
    }

    #[test]
    fn lowers_simple_raise_with_name_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::Name {
                name: "e".to_string(),
                ty: Type::Int,
            },
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("raise name lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, args })) => {
                assert!(
                    matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()])
                );
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::Assert { msg: None, .. }));
    }

    #[test]
    fn lowers_simple_assert_with_leaf_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::StringLiteral("boom".to_string())),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert not-option truthiness name test lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Assert {
                cond:
                    RustExpr::MethodCall {
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert option truthiness name test lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Assert {
                cond:
                    RustExpr::MethodCall {
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
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

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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
            try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("if with not-option truthiness condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::If {
                cond:
                    RustExpr::MethodCall {
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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
    fn lowers_option_is_none_if_with_exiting_body_and_post_unwrap_without_rawcode() {
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
            then_body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(0)),
            }],
            elif_clauses: vec![],
            else_body: None,
        };

        let ret_ty = Type::Int;
        let lowered = try_lower_simple_stmt_with_ctx(
            &if_stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&ret_ty),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("if with exiting body lowered");

        assert_eq!(lowered.len(), 2);
        assert!(matches!(
            lowered[0],
            RustStmt::If {
                cond: RustExpr::MethodCall { .. },
                ..
            }
        ));
        assert!(matches!(
            lowered[1],
            RustStmt::Let {
                ref name,
                value: RustExpr::MethodCall { ref method, .. },
                ..
            } if name == "maybe_x" && method == "unwrap"
        ));
        assert!(lowered
            .iter()
            .all(|stmt| !matches!(stmt, RustStmt::RawCode(_))));
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("if with option is-not-none compare condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::IfLet {
                ref pattern,
                expr: RustExpr::Ident(ref name),
                ..
            } if pattern == "Some(maybe_x)" && name == "maybe_x"
        ));
    }

    #[test]
    fn lowers_simple_if_with_option_and_not_none_chain_condition() {
        let if_stmt = HirStmt::If {
            condition: HirExpr::BoolOp {
                op: "and".to_string(),
                values: vec![
                    HirExpr::Compare {
                        left: Box::new(HirExpr::Name {
                            name: "a".to_string(),
                            ty: Type::Union(vec![Type::Int, Type::None]),
                        }),
                        ops: vec!["is not".to_string()],
                        comparators: vec![HirExpr::NoneLiteral],
                        ty: Type::Bool,
                    },
                    HirExpr::Compare {
                        left: Box::new(HirExpr::Name {
                            name: "b".to_string(),
                            ty: Type::Union(vec![Type::Int, Type::None]),
                        }),
                        ops: vec!["is not".to_string()],
                        comparators: vec![HirExpr::NoneLiteral],
                        ty: Type::Bool,
                    },
                ],
                ty: Type::Bool,
            },
            then_body: vec![HirStmt::Pass],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Pass]),
        };

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("if with option and-not-none chain condition lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::IfLet {
                ref pattern,
                expr: RustExpr::Ident(ref name),
                ..
            } if pattern == "Some(a)" && name == "a"
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
            try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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
            try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
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
            try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("while with not-option truthiness name condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While {
                cond:
                    RustExpr::MethodCall {
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("while with option truthiness name condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While {
                cond:
                    RustExpr::MethodCall {
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("while with alias option truthiness name condition lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::While {
                cond:
                    RustExpr::MethodCall {
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
            try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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
            try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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
            try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
        );
    }

    #[test]
    fn lowers_simple_while_with_else() {
        let while_stmt = HirStmt::While {
            condition: HirExpr::BoolLiteral(true),
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Pass]),
        };

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("for with name iter lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::For {
                var: ref var_name,
                iter: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if var_name == "i"
                && matches!(
                    recv.as_ref(),
                    RustExpr::MethodCall {
                        receiver: ref inner_recv,
                        ref method,
                        ref args,
                    } if matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "items")
                        && method == "iter"
                        && args.is_empty()
                )
                && method == "cloned"
                && args.is_empty()
        ));
    }

    #[test]
    fn lowers_simple_for_with_dict_iter_to_keys_cloned() {
        let for_stmt = HirStmt::For {
            target: "k".to_string(),
            target_ty: Type::Str,
            iter: HirExpr::Name {
                name: "m".to_string(),
                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: None,
        };

        let lowered = try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new())
            .expect("for with dict iter lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::For {
                iter: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if matches!(
                recv.as_ref(),
                RustExpr::MethodCall {
                    receiver: ref inner_recv,
                    ref method,
                    ref args,
                } if matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "m")
                    && method == "keys"
                    && args.is_empty()
            )
                && method == "cloned"
                && args.is_empty()
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
            try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new(),).is_none()
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
        let lowered =
            try_lower_simple_stmt(&for_with_else, false, &HashSet::new(), &HashSet::new())
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
        let lowered =
            try_lower_simple_stmt(&for_with_else, false, &HashSet::new(), &HashSet::new())
                .expect("for with else and name iter lowered");
        assert_eq!(lowered.len(), 3);
        assert!(matches!(lowered[0], RustStmt::Let { .. }));
        assert!(matches!(
            lowered[1],
            RustStmt::For {
                iter: RustExpr::MethodCall {
                    receiver: ref recv,
                    ref method,
                    ref args,
                },
                ..
            } if matches!(
                recv.as_ref(),
                RustExpr::MethodCall {
                    receiver: ref inner_recv,
                    ref method,
                    ref args,
                } if matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "items")
                    && method == "iter"
                    && args.is_empty()
            )
                && method == "cloned"
                && args.is_empty()
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
            try_lower_simple_stmt(&for_with_else, false, &HashSet::new(), &HashSet::new(),)
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
            try_lower_simple_stmt(&for_tuple_target, false, &HashSet::new(), &HashSet::new(),)
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

        let lowered = try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&for_stmt, true, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
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

        let lowered = try_lower_simple_stmt(&while_stmt, true, &HashSet::new(), &HashSet::new())
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

    #[test]
    fn lowers_simple_yield_inside_generator_closure() {
        let stmt = HirStmt::Yield {
            value: HirExpr::IntLiteral(7),
        };
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: None,
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: true,
            },
        )
        .expect("yield lowered");

        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { .. }))
        ));
    }

    #[test]
    fn lowers_simple_yield_outside_generator_closure() {
        let stmt = HirStmt::Yield {
            value: HirExpr::IntLiteral(7),
        };
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx::default(),
        )
        .expect("yield lowered");

        assert!(matches!(
            lowered[0],
            RustStmt::Expr(RustExpr::MethodCall { .. })
        ));
    }

    #[test]
    fn lowers_simple_star_unpack_from_name() {
        let stmt = HirStmt::StarUnpack {
            before: vec![("head".to_string(), Type::Int)],
            star: ("mid".to_string(), Type::List(Box::new(Type::Int))),
            after: vec![("tail".to_string(), Type::Int)],
            value: HirExpr::Name {
                name: "xs".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            },
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("star unpack lowered");
        assert_eq!(lowered.len(), 4);
        assert!(matches!(
            lowered[0],
            RustStmt::Let { ref name, .. } if name == "_star_tmp"
        ));
        assert!(lowered
            .iter()
            .all(|stmt| !matches!(stmt, RustStmt::RawCode(_))));
    }

    #[test]
    fn lowers_simple_with_without_context_manager_protocol() {
        let stmt = HirStmt::With {
            items: vec![("x".to_string(), HirExpr::IntLiteral(1), false)],
            body: vec![HirStmt::Expr {
                expr: HirExpr::Name {
                    name: "x".to_string(),
                    ty: Type::Int,
                },
            }],
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("with lowered");
        assert!(matches!(lowered[0], RustStmt::Block(_)));
    }

    #[test]
    fn lowers_simple_match_with_literal_and_wildcard_patterns() {
        let stmt = HirStmt::Match {
            subject: HirExpr::Name {
                name: "n".to_string(),
                ty: Type::Int,
            },
            subject_ty: Type::Int,
            arms: vec![
                sifr_hir::HirMatchArm {
                    pattern: HirPattern::Literal {
                        value: HirExpr::IntLiteral(1),
                    },
                    guard: None,
                    body: vec![HirStmt::Expr {
                        expr: HirExpr::IntLiteral(10),
                    }],
                },
                sifr_hir::HirMatchArm {
                    pattern: HirPattern::Wildcard,
                    guard: None,
                    body: vec![HirStmt::Expr {
                        expr: HirExpr::IntLiteral(0),
                    }],
                },
            ],
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("match lowered");
        assert!(matches!(lowered[0], RustStmt::Match { .. }));
    }

    #[test]
    fn lowers_match_with_class_patterns_and_captures() {
        let point_ty = Type::Class {
            name: "Point".to_string(),
            fields: vec![("x".to_string(), Type::Int), ("y".to_string(), Type::Int)],
            methods: vec![],
            parent_class: None,
        };
        let stmt = HirStmt::Match {
            subject: HirExpr::Name {
                name: "p".to_string(),
                ty: point_ty.clone(),
            },
            subject_ty: point_ty,
            arms: vec![
                sifr_hir::HirMatchArm {
                    pattern: HirPattern::Class {
                        class_name: "Point".to_string(),
                        fields: vec![
                            (
                                "x".to_string(),
                                HirPattern::Literal {
                                    value: HirExpr::IntLiteral(0),
                                },
                            ),
                            (
                                "y".to_string(),
                                HirPattern::Capture {
                                    name: "py".to_string(),
                                    ty: Type::Int,
                                },
                            ),
                        ],
                    },
                    guard: None,
                    body: vec![HirStmt::Return {
                        value: Some(HirExpr::StringLiteral("axis".to_string())),
                    }],
                },
                sifr_hir::HirMatchArm {
                    pattern: HirPattern::Wildcard,
                    guard: None,
                    body: vec![HirStmt::Return {
                        value: Some(HirExpr::StringLiteral("other".to_string())),
                    }],
                },
            ],
        };

        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Str),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("class match lowered");

        assert!(matches!(
            lowered[0],
            RustStmt::Match { ref arms, .. }
                if arms.len() == 2
                    && arms[0].pattern.contains("Point { x: 0, y: py")
                    && arms[0].bindings.iter().any(|name| name == "py")
        ));
    }

    #[test]
    fn lowers_match_with_string_literal_patterns() {
        let stmt = HirStmt::Match {
            subject: HirExpr::Name {
                name: "method".to_string(),
                ty: Type::Str,
            },
            subject_ty: Type::Str,
            arms: vec![
                sifr_hir::HirMatchArm {
                    pattern: HirPattern::Literal {
                        value: HirExpr::StringLiteral("GET".to_string()),
                    },
                    guard: None,
                    body: vec![HirStmt::Return {
                        value: Some(HirExpr::StringLiteral("read".to_string())),
                    }],
                },
                sifr_hir::HirMatchArm {
                    pattern: HirPattern::Wildcard,
                    guard: None,
                    body: vec![HirStmt::Return {
                        value: Some(HirExpr::StringLiteral("other".to_string())),
                    }],
                },
            ],
        };

        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Str),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("string match lowered");

        assert!(matches!(
            lowered[0],
            RustStmt::Match { ref arms, .. }
                if arms.len() == 2
                    && arms[0].pattern == "__s"
                    && arms[0].guard.is_some()
        ));
    }

    #[test]
    fn lowers_simple_nested_function_to_closure_block() {
        let stmt = HirStmt::NestedFunction {
            func: HirFunction {
                name: "inner".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(1)),
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("nested function lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                ref name,
                value: RustExpr::ClosureBlock { .. },
                ..
            } if name == "inner"
        ));
    }

    #[test]
    fn lowers_recursive_nested_function_without_captures_to_local_fn() {
        let stmt = HirStmt::NestedFunction {
            func: HirFunction {
                name: "inner".to_string(),
                params: vec![],
                return_type: Type::Int,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "inner".to_string(),
                        args: vec![],
                        ty: Type::Int,
                    },
                }],
                method_kind: MethodKind::Regular,
                decorators: vec![],
                type_params: vec![],
            },
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("recursive nested function lowered");
        assert!(matches!(
            lowered[0],
            RustStmt::LocalFn { ref name, .. } if name == "inner"
        ));
    }

    #[test]
    fn lowers_simple_try_except_catch_all_with_result_flow() {
        let stmt = HirStmt::TryExcept {
            body: vec![HirStmt::Expr {
                expr: HirExpr::QuestionMark {
                    expr: Box::new(HirExpr::Name {
                        name: "res".to_string(),
                        ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
                    }),
                    ty: Type::Int,
                },
            }],
            handlers: vec![HirExceptHandler {
                error_type: None,
                error_resolved_type: None,
                name: None,
                body: vec![HirStmt::Pass],
            }],
            body_error_types: vec!["Error".to_string()],
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("try/except lowered");
        assert_eq!(lowered.len(), 2);
        assert!(matches!(
            lowered[0],
            RustStmt::Let {
                ref name,
                value: RustExpr::FnCall { .. },
                ..
            } if name == "__sifr_try_res"
        ));
        assert!(matches!(
            lowered[1],
            RustStmt::IfLet {
                ref pattern,
                expr: RustExpr::Ident(ref expr_name),
                ..
            } if pattern == "Err(_e)" && expr_name == "__sifr_try_res"
        ));
        assert!(lowered
            .iter()
            .all(|stmt| !matches!(stmt, RustStmt::RawCode(_))));
    }

    #[test]
    fn does_not_lower_try_except_with_typed_handler() {
        let stmt = HirStmt::TryExcept {
            body: vec![HirStmt::Expr {
                expr: HirExpr::QuestionMark {
                    expr: Box::new(HirExpr::Name {
                        name: "res".to_string(),
                        ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
                    }),
                    ty: Type::Int,
                },
            }],
            handlers: vec![HirExceptHandler {
                error_type: Some("IOError".to_string()),
                error_resolved_type: Some(Type::Class {
                    name: "IOError".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                }),
                name: None,
                body: vec![HirStmt::Pass],
            }],
            body_error_types: vec!["IOError".to_string()],
        };
        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn does_not_lower_try_except_without_result_flow() {
        let stmt = HirStmt::TryExcept {
            body: vec![HirStmt::Pass],
            handlers: vec![HirExceptHandler {
                error_type: None,
                error_resolved_type: None,
                name: None,
                body: vec![HirStmt::Pass],
            }],
            body_error_types: vec!["Error".to_string()],
        };
        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }
}
