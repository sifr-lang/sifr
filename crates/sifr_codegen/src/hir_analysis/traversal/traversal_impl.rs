use sifr_ir::{HirExpr, HirFStringPart, HirPattern, HirStmt};

/// Traversal configuration for HIR analysis walkers.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TraversalConfig {
    /// When false, nested function bodies are treated as scope boundaries and
    /// not traversed. When true, nested function bodies are included.
    pub descend_nested_functions: bool,
}

impl TraversalConfig {
    pub(crate) const LOCAL_SCOPE_ONLY: Self = Self {
        descend_nested_functions: false,
    };

    pub(crate) const INCLUDE_NESTED_FUNCTIONS: Self = Self {
        descend_nested_functions: true,
    };
}

/// Flow control for canonical traversal callbacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TraversalControl {
    Continue,
    Stop,
}

/// Canonical traversal rules for HIR analysis.
///
/// Invariants:
/// - Expression/statement recursion for analysis must flow through this module.
/// - `on_stmt` is invoked once for each visited statement before children.
/// - `on_expr` is invoked once for each visited expression before children.
/// - Nested function recursion is controlled only by `TraversalConfig`.
/// - `_until` walkers stop recursively as soon as callbacks return `Stop`.
///
/// Extension rules for future HIR variants:
/// - Any added `HirExpr`/`HirStmt`/`HirPattern` variant must be handled here.
/// - If a new variant contains sub-statements or sub-expressions, recurse into
///   every child to preserve traversal completeness.
/// - Callers must not introduce emitter-local recursive descent for analysis;
///   add/extend query helpers on top of this traversal layer instead.
pub(crate) fn walk_expr_until<F>(expr: &HirExpr, on_expr: &mut F) -> TraversalControl
where
    F: FnMut(&HirExpr) -> TraversalControl,
{
    if matches!(on_expr(expr), TraversalControl::Stop) {
        return TraversalControl::Stop;
    }
    match expr {
        HirExpr::BinOp { left, right, .. } => {
            if matches!(walk_expr_until(left, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(right, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::Await { value: operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::WalrusExpr { value: operand, .. }
        | HirExpr::FieldAccess {
            object: operand, ..
        } => {
            if matches!(walk_expr_until(operand, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            if matches!(walk_expr_until(left, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            for comparator in comparators {
                if matches!(walk_expr_until(comparator, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::IntrinsicCall { args: values, .. }
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
                if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::Call { args, .. }
        | HirExpr::PythonCall { args, .. }
        | HirExpr::IteratorCall { args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::SuperCall { args, .. } => {
            for arg in args {
                if matches!(walk_expr_until(arg, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::MethodCall { object, args, .. } => {
            if matches!(walk_expr_until(object, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            for arg in args {
                if matches!(walk_expr_until(arg, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            if matches!(walk_expr_until(condition, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(then_expr, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(else_expr, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            if matches!(walk_expr_until(start, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(end, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if let Some(step) = step {
                if matches!(walk_expr_until(step, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for key in keys {
                if matches!(walk_expr_until(key, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
            for value in values {
                if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::Index { object, index, .. } => {
            if matches!(walk_expr_until(object, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(index, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            if matches!(walk_expr_until(element, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(collection, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    if matches!(walk_expr_until(expr, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            if matches!(walk_expr_until(object, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if let Some(start) = start {
                if matches!(walk_expr_until(start, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
            if let Some(stop) = stop {
                if matches!(walk_expr_until(stop, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
            if let Some(step) = step {
                if matches!(walk_expr_until(step, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::Lambda { body, .. } => {
            if matches!(walk_expr_until(body, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            if matches!(walk_expr_until(expr, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            for (_, iter, filter) in generators {
                if matches!(walk_expr_until(iter, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
                if let Some(filter) = filter {
                    if matches!(walk_expr_until(filter, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            if matches!(walk_expr_until(key_expr, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(val_expr, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            for (_, iter, filter) in generators {
                if matches!(walk_expr_until(iter, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
                if let Some(filter) = filter {
                    if matches!(walk_expr_until(filter, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            if matches!(walk_expr_until(expr, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(iter, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if let Some(filter) = filter {
                if matches!(walk_expr_until(filter, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::LargeIntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => {}
    }
    TraversalControl::Continue
}

pub(crate) fn walk_expr<F>(expr: &HirExpr, on_expr: &mut F)
where
    F: FnMut(&HirExpr),
{
    let mut on_expr_continue = |node: &HirExpr| {
        on_expr(node);
        TraversalControl::Continue
    };
    let _ = walk_expr_until(expr, &mut on_expr_continue);
}

pub(crate) fn walk_pattern_until<F>(pattern: &HirPattern, on_expr: &mut F) -> TraversalControl
where
    F: FnMut(&HirExpr) -> TraversalControl,
{
    match pattern {
        HirPattern::Literal { value } => {
            if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                if matches!(walk_pattern_until(pattern, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirPattern::Class { fields, .. } => {
            for (_, pattern) in fields {
                if matches!(walk_pattern_until(pattern, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirPattern::Wildcard
        | HirPattern::Capture { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => {}
    }
    TraversalControl::Continue
}

pub(crate) fn walk_pattern<F>(pattern: &HirPattern, on_expr: &mut F)
where
    F: FnMut(&HirExpr),
{
    let mut on_expr_continue = |node: &HirExpr| {
        on_expr(node);
        TraversalControl::Continue
    };
    let _ = walk_pattern_until(pattern, &mut on_expr_continue);
}

pub(crate) fn walk_stmt_until<FStmt, FExpr>(
    stmt: &HirStmt,
    config: TraversalConfig,
    on_stmt: &mut FStmt,
    on_expr: &mut FExpr,
) -> TraversalControl
where
    FStmt: FnMut(&HirStmt) -> TraversalControl,
    FExpr: FnMut(&HirExpr) -> TraversalControl,
{
    if matches!(on_stmt(stmt), TraversalControl::Stop) {
        return TraversalControl::Stop;
    }
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::NestedFieldAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value } => {
            if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::Return { value } => {
            if let Some(value) = value {
                if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirStmt::Expr { expr } => {
            if matches!(walk_expr_until(expr, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            if matches!(walk_expr_until(condition, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(
                walk_stmts_until(then_body, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            for (cond, body) in elif_clauses {
                if matches!(walk_expr_until(cond, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
                if matches!(
                    walk_stmts_until(body, config, on_stmt, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
            if let Some(else_body) = else_body {
                if matches!(
                    walk_stmts_until(else_body, config, on_stmt, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            if matches!(walk_expr_until(condition, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(
                walk_stmts_until(body, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            if let Some(else_body) = else_body {
                if matches!(
                    walk_stmts_until(else_body, config, on_stmt, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
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
            if matches!(walk_expr_until(iter, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(
                walk_stmts_until(body, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            if let Some(else_body) = else_body {
                if matches!(
                    walk_stmts_until(else_body, config, on_stmt, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => {
            if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::Assert { test, msg } => {
            if matches!(walk_expr_until(test, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if let Some(msg) = msg {
                if matches!(walk_expr_until(msg, on_expr), TraversalControl::Stop) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            if matches!(
                walk_stmts_until(body, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            for handler in handlers {
                if matches!(
                    walk_stmts_until(&handler.body, config, on_stmt, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirStmt::TryFinally { body, finalbody } => {
            if matches!(
                walk_stmts_until(body, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            if matches!(
                walk_stmts_until(finalbody, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::SubscriptAssign { index, value, .. }
        | HirStmt::SubscriptAugAssign { index, value, .. }
        | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
            if matches!(walk_expr_until(index, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        } => {
            if matches!(
                walk_expr_until(outer_index, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            if matches!(
                walk_expr_until(inner_index, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::AttributeNestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        } => {
            if matches!(
                walk_expr_until(outer_index, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            if matches!(
                walk_expr_until(inner_index, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(value, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::Delete { object, index } => {
            if matches!(walk_expr_until(object, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            if matches!(walk_expr_until(index, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::With { items, body } => {
            for item in items {
                if matches!(
                    walk_expr_until(&item.context, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
            if matches!(
                walk_stmts_until(body, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::AsyncWith { kind, body, .. } => {
            match kind {
                sifr_ir::HirAsyncWithKind::TaskTimeout { duration } => {
                    if matches!(walk_expr_until(duration, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
                sifr_ir::HirAsyncWithKind::UserDefined { context, .. }
                | sifr_ir::HirAsyncWithKind::Python { context, .. } => {
                    if matches!(walk_expr_until(context, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
                sifr_ir::HirAsyncWithKind::TaskGroup {
                    context: Some(context),
                } => {
                    if matches!(walk_expr_until(context, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
                sifr_ir::HirAsyncWithKind::TaskScope
                | sifr_ir::HirAsyncWithKind::TaskGroup { context: None } => {}
            }
            if matches!(
                walk_stmts_until(body, config, on_stmt, on_expr),
                TraversalControl::Stop
            ) {
                return TraversalControl::Stop;
            }
        }
        HirStmt::NestedFunction { func } => {
            if config.descend_nested_functions {
                if matches!(
                    walk_stmts_until(&func.body, config, on_stmt, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirStmt::Match { subject, arms, .. } => {
            if matches!(walk_expr_until(subject, on_expr), TraversalControl::Stop) {
                return TraversalControl::Stop;
            }
            for arm in arms {
                if matches!(
                    walk_pattern_until(&arm.pattern, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
                if let Some(guard) = &arm.guard {
                    if matches!(walk_expr_until(guard, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
                if matches!(
                    walk_stmts_until(&arm.body, config, on_stmt, on_expr),
                    TraversalControl::Stop
                ) {
                    return TraversalControl::Stop;
                }
            }
        }
        HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
    }
    TraversalControl::Continue
}

pub(crate) fn walk_stmt<FStmt, FExpr>(
    stmt: &HirStmt,
    config: TraversalConfig,
    on_stmt: &mut FStmt,
    on_expr: &mut FExpr,
) where
    FStmt: FnMut(&HirStmt),
    FExpr: FnMut(&HirExpr),
{
    let mut on_stmt_continue = |node: &HirStmt| {
        on_stmt(node);
        TraversalControl::Continue
    };
    let mut on_expr_continue = |node: &HirExpr| {
        on_expr(node);
        TraversalControl::Continue
    };
    let _ = walk_stmt_until(stmt, config, &mut on_stmt_continue, &mut on_expr_continue);
}

pub(crate) fn walk_stmts_until<FStmt, FExpr>(
    stmts: &[HirStmt],
    config: TraversalConfig,
    on_stmt: &mut FStmt,
    on_expr: &mut FExpr,
) -> TraversalControl
where
    FStmt: FnMut(&HirStmt) -> TraversalControl,
    FExpr: FnMut(&HirExpr) -> TraversalControl,
{
    for stmt in stmts {
        if matches!(
            walk_stmt_until(stmt, config, on_stmt, on_expr),
            TraversalControl::Stop
        ) {
            return TraversalControl::Stop;
        }
    }
    TraversalControl::Continue
}

pub(crate) fn walk_stmts<FStmt, FExpr>(
    stmts: &[HirStmt],
    config: TraversalConfig,
    on_stmt: &mut FStmt,
    on_expr: &mut FExpr,
) where
    FStmt: FnMut(&HirStmt),
    FExpr: FnMut(&HirExpr),
{
    let mut on_stmt_continue = |node: &HirStmt| {
        on_stmt(node);
        TraversalControl::Continue
    };
    let mut on_expr_continue = |node: &HirExpr| {
        on_expr(node);
        TraversalControl::Continue
    };
    let _ = walk_stmts_until(stmts, config, &mut on_stmt_continue, &mut on_expr_continue);
}
