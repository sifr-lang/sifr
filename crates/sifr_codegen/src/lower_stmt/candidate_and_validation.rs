use super::{
    CodegenError, HashMap, HashSet, HirExpr, HirFStringPart, HirPattern, HirStmt, RustExpr,
    RustStmt, ScopeContext, Type, resolve_alias_type, try_lower_leaf_expr,
    try_lower_leaf_expr_result, try_lower_leaf_or_name_expr,
    try_lower_simple_stmt_with_ctx_and_bindings,
};
pub(crate) fn is_simple_stmt_candidate(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Expr { expr } => crate::is_leaf_expr_candidate(expr),
        HirStmt::Let { .. }
        | HirStmt::Assign { .. }
        | HirStmt::AugAssign { .. }
        | HirStmt::AttributeAugAssign { .. }
        | HirStmt::FieldAssign { .. }
        | HirStmt::NestedFieldAssign { .. }
        | HirStmt::Return { .. }
        | HirStmt::Assert { .. }
        | HirStmt::Raise { .. }
        | HirStmt::If { .. }
        | HirStmt::While { .. }
        | HirStmt::For { .. }
        | HirStmt::AsyncFor { .. }
        | HirStmt::Pass
        | HirStmt::Continue
        | HirStmt::Break
        | HirStmt::TupleUnpack { .. }
        | HirStmt::StarUnpack { .. }
        | HirStmt::SubscriptAssign { .. }
        | HirStmt::NestedSubscriptAssign { .. }
        | HirStmt::AttributeNestedSubscriptAssign { .. }
        | HirStmt::SubscriptAugAssign { .. }
        | HirStmt::AttributeSubscriptAssign { .. }
        | HirStmt::Delete { .. }
        | HirStmt::Yield { .. }
        | HirStmt::With { .. }
        | HirStmt::AsyncWith { .. }
        | HirStmt::Match { .. }
        | HirStmt::NestedFunction { .. }
        | HirStmt::TryExcept { .. }
        | HirStmt::TryFinally { .. } => true,
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

pub(super) fn try_lower_expr_stmt_with_bindings(
    expr: &HirExpr,
    local_binding_types: &HashMap<String, Type>,
) -> Option<Vec<RustStmt>> {
    if let HirExpr::MethodCall {
        object,
        method,
        args,
        ..
    } = expr
    {
        if let HirExpr::Name { name, ty, .. } = object.as_ref() {
            if matches!(resolve_alias_type(ty), Type::Any | Type::Unknown) {
                if let Some(bound_ty) = local_binding_types.get(name) {
                    let lowered_object = try_lower_leaf_or_name_expr(object)?;
                    let lowered_args = args
                        .iter()
                        .map(try_lower_leaf_or_name_expr)
                        .collect::<Option<Vec<_>>>()?;
                    if let Some(lowered) = crate::methods::lower_method(
                        bound_ty,
                        method,
                        &lowered_object,
                        &lowered_args,
                    ) {
                        return Some(vec![RustStmt::Expr(lowered.expr)]);
                    }
                }
            }
        }
    }
    try_lower_expr_stmt(expr)
}

pub(super) fn try_lower_simple_print_expr_stmt(expr: &HirExpr) -> Option<RustStmt> {
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
        [HirExpr::StringLiteral(value)] if value.is_empty() => {
            Some(RustStmt::Expr(RustExpr::MacroCall {
                name: "println".to_string(),
                args: vec![],
            }))
        }
        [HirExpr::StringLiteral(value)] => Some(RustStmt::Expr(RustExpr::MacroCall {
            name: "println".to_string(),
            args: vec![RustExpr::Verbatim(format!("{value:?}"))],
        })),
        [HirExpr::FString { .. } | HirExpr::TemplateString(_)] => None,
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
pub(super) struct SimpleStmtBindings<'a> {
    pub(super) mutated_vars: &'a HashSet<String>,
    pub(super) borrowed_params: &'a HashSet<String>,
    pub(super) mut_borrowed_params: &'a HashSet<String>,
    pub(super) local_binding_types: &'a HashMap<String, Type>,
    pub(super) recursive_fields: &'a HashSet<(String, String)>,
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
    try_lower_simple_stmt_with_scope_and_bindings(
        stmt,
        mutated_vars,
        borrowed_params,
        &HashSet::new(),
        &HashMap::new(),
        &HashSet::new(),
        &scope_ctx,
    )
}

pub(crate) fn try_lower_simple_stmt_with_scope(
    stmt: &HirStmt,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    scope_ctx: &ScopeContext,
) -> Option<Vec<RustStmt>> {
    try_lower_simple_stmt_with_scope_and_bindings(
        stmt,
        mutated_vars,
        borrowed_params,
        &HashSet::new(),
        &HashMap::new(),
        &HashSet::new(),
        scope_ctx,
    )
}

pub(crate) fn try_lower_simple_stmt_with_scope_and_bindings(
    stmt: &HirStmt,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    mut_borrowed_params: &HashSet<String>,
    local_binding_types: &HashMap<String, Type>,
    recursive_fields: &HashSet<(String, String)>,
    scope_ctx: &ScopeContext,
) -> Option<Vec<RustStmt>> {
    try_lower_simple_stmt_with_ctx_and_bindings(
        stmt,
        scope_ctx.in_loop_with_else,
        SimpleStmtBindings {
            mutated_vars,
            borrowed_params,
            mut_borrowed_params,
            local_binding_types,
            recursive_fields,
        },
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
    try_lower_simple_stmt_with_scope_result_and_bindings(
        stmt,
        mutated_vars,
        borrowed_params,
        &HashSet::new(),
        &HashMap::new(),
        &HashSet::new(),
        scope_ctx,
    )
}

pub(crate) fn try_lower_simple_stmt_with_scope_result_and_bindings(
    stmt: &HirStmt,
    mutated_vars: &HashSet<String>,
    borrowed_params: &HashSet<String>,
    mut_borrowed_params: &HashSet<String>,
    local_binding_types: &HashMap<String, Type>,
    recursive_fields: &HashSet<(String, String)>,
    scope_ctx: &ScopeContext,
) -> Result<Option<Vec<RustStmt>>, CodegenError> {
    validate_scope_context(scope_ctx)?;
    validate_stmt_lowering_shape(stmt)?;
    Ok(try_lower_simple_stmt_with_scope_and_bindings(
        stmt,
        mutated_vars,
        borrowed_params,
        mut_borrowed_params,
        local_binding_types,
        recursive_fields,
        scope_ctx,
    ))
}

pub(super) fn validate_scope_context(scope_ctx: &ScopeContext) -> Result<(), CodegenError> {
    if scope_ctx.in_display_impl && scope_ctx.in_generator_closure {
        return Err(CodegenError::new(
            "invalid lowering scope: display impl and generator closure cannot both be active",
        ));
    }
    Ok(())
}

pub(super) fn validate_stmt_lowering_shape(stmt: &HirStmt) -> Result<(), CodegenError> {
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::NestedFieldAssign { value, .. }
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
        }
        | HirStmt::AsyncFor {
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
        HirStmt::AttributeNestedSubscriptAssign {
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
            for item in items {
                validate_expr_lowering_shape(&item.context)?;
            }
            validate_stmt_block_lowering_shape(body)
        }
        HirStmt::AsyncWith { kind, body, .. } => {
            match kind {
                sifr_ir::HirAsyncWithKind::TaskTimeout { duration } => {
                    validate_expr_lowering_shape(duration)?;
                }
                sifr_ir::HirAsyncWithKind::UserDefined { context, .. }
                | sifr_ir::HirAsyncWithKind::Python { context, .. } => {
                    validate_expr_lowering_shape(context)?;
                }
                sifr_ir::HirAsyncWithKind::TaskGroup {
                    context: Some(context),
                } => {
                    validate_expr_lowering_shape(context)?;
                }
                sifr_ir::HirAsyncWithKind::TaskScope
                | sifr_ir::HirAsyncWithKind::TaskGroup { context: None } => {}
            }
            validate_stmt_block_lowering_shape(body)
        }
        HirStmt::NestedFunction { func, .. } => {
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
        HirStmt::TryFinally { body, finalbody } => {
            validate_stmt_block_lowering_shape(body)?;
            validate_stmt_block_lowering_shape(finalbody)
        }
        HirStmt::Pass | HirStmt::Continue | HirStmt::Break | HirStmt::Return { value: None } => {
            Ok(())
        }
    }
}

pub(super) fn validate_stmt_block_lowering_shape(stmts: &[HirStmt]) -> Result<(), CodegenError> {
    for stmt in stmts {
        validate_stmt_lowering_shape(stmt)?;
    }
    Ok(())
}

pub(super) fn validate_pattern_lowering_shape(pattern: &HirPattern) -> Result<(), CodegenError> {
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

pub(super) fn validate_expr_lowering_shape(expr: &HirExpr) -> Result<(), CodegenError> {
    let _ = try_lower_leaf_expr_result(expr)?;
    match expr {
        HirExpr::BinOp { left, right, .. } => {
            validate_expr_lowering_shape(left)?;
            validate_expr_lowering_shape(right)
        }
        HirExpr::Await { value, .. } => validate_expr_lowering_shape(value),
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
        | HirExpr::GenericCall { args: values, .. }
        | HirExpr::PythonCall { args: values, .. }
        | HirExpr::IntrinsicCall { args: values, .. }
        | HirExpr::IteratorCall { args: values, .. }
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
        HirExpr::TemplateString(template) => {
            let mut result = Ok(());
            template.for_each_value(&mut |value| {
                if result.is_ok() {
                    result = validate_expr_lowering_shape(value);
                }
            });
            result
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
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. } => Ok(()),
    }
}
