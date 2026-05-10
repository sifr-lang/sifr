use sifr_hir::{HirExpr, HirFStringPart, HirPattern, HirStmt};

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

/// Canonical traversal contract for HIR analysis.
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
            for (_, expr, _) in items {
                if matches!(walk_expr_until(expr, on_expr), TraversalControl::Stop) {
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
                sifr_hir::HirAsyncWithKind::TaskTimeout { duration } => {
                    if matches!(walk_expr_until(duration, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
                sifr_hir::HirAsyncWithKind::UserDefined { context, .. } => {
                    if matches!(walk_expr_until(context, on_expr), TraversalControl::Stop) {
                        return TraversalControl::Stop;
                    }
                }
                sifr_hir::HirAsyncWithKind::TaskScope | sifr_hir::HirAsyncWithKind::TaskGroup => {}
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

#[cfg(test)]
mod tests {
    use super::{walk_expr_until, walk_stmts, walk_stmts_until, TraversalConfig, TraversalControl};
    use sifr_hir::{
        HirExceptHandler, HirExpr, HirFunction, HirMatchArm, HirParam, HirPattern, HirStmt,
        MethodKind,
    };
    use sifr_type_system::{ParamConvention, Type};

    #[test]
    fn walk_stmts_covers_try_handlers_loop_else_and_match_patterns() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![HirParam {
                name: "p".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::own(),
            }],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "nested_only".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };

        let stmts = vec![
            HirStmt::TryExcept {
                body: vec![HirStmt::For {
                    target: "item".to_string(),
                    target_ty: Type::Int,
                    iter: HirExpr::ListLiteral {
                        elements: vec![HirExpr::IntLiteral(1)],
                        ty: Type::List(Box::new(Type::Int)),
                    },
                    body: vec![HirStmt::Pass],
                    else_body: Some(vec![HirStmt::Expr {
                        expr: HirExpr::Call {
                            func: "loop_else_call".to_string(),
                            args: vec![],
                            ty: Type::None,
                        },
                    }]),
                }],
                handlers: vec![HirExceptHandler {
                    error_type: Some("Error".to_string()),
                    error_resolved_type: None,
                    name: Some("e".to_string()),
                    body: vec![HirStmt::Match {
                        subject: HirExpr::Name {
                            name: "value".to_string(),
                            ty: Type::Int,
                        },
                        subject_ty: Type::Int,
                        arms: vec![HirMatchArm {
                            pattern: HirPattern::Literal {
                                value: HirExpr::Call {
                                    func: "pattern_expr".to_string(),
                                    args: vec![],
                                    ty: Type::Int,
                                },
                            },
                            guard: Some(HirExpr::Call {
                                func: "guard_expr".to_string(),
                                args: vec![],
                                ty: Type::Bool,
                            }),
                            body: vec![HirStmt::Expr {
                                expr: HirExpr::Call {
                                    func: "arm_body_call".to_string(),
                                    args: vec![],
                                    ty: Type::None,
                                },
                            }],
                        }],
                    }],
                }],
                body_error_types: vec!["Error".to_string()],
            },
            HirStmt::NestedFunction { func: nested },
        ];

        let mut calls = Vec::<String>::new();
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if let HirExpr::Call { func, .. } = expr {
                calls.push(func.clone());
            }
        };
        walk_stmts(
            &stmts,
            TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
            &mut on_stmt,
            &mut on_expr,
        );

        assert!(calls.contains(&"loop_else_call".to_string()));
        assert!(calls.contains(&"pattern_expr".to_string()));
        assert!(calls.contains(&"guard_expr".to_string()));
        assert!(calls.contains(&"arm_body_call".to_string()));
        assert!(calls.contains(&"nested_only".to_string()));
    }

    #[test]
    fn walk_stmts_respects_nested_function_scope_boundary() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "nested_only".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };
        let stmts = vec![HirStmt::NestedFunction { func: nested }];

        let mut saw_nested_call = false;
        let mut on_stmt = |_stmt: &HirStmt| {};
        let mut on_expr = |expr: &HirExpr| {
            if let HirExpr::Call { func, .. } = expr {
                if func == "nested_only" {
                    saw_nested_call = true;
                }
            }
        };

        walk_stmts(
            &stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );

        assert!(!saw_nested_call);
    }

    #[test]
    fn walk_expr_until_stops_at_first_match() {
        let expr = HirExpr::TupleLiteral {
            elements: vec![
                HirExpr::Call {
                    func: "first".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
                HirExpr::Call {
                    func: "second".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
            ],
            ty: Type::Tuple(vec![Type::None, Type::None]),
        };

        let mut seen_calls = Vec::new();
        let result = walk_expr_until(&expr, &mut |node| {
            if let HirExpr::Call { func, .. } = node {
                seen_calls.push(func.clone());
                if func == "first" {
                    return TraversalControl::Stop;
                }
            }
            TraversalControl::Continue
        });

        assert_eq!(result, TraversalControl::Stop);
        assert_eq!(seen_calls, vec!["first".to_string()]);
    }

    #[test]
    fn walk_stmts_until_stops_before_later_statements() {
        let stmts = vec![
            HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            },
            HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "later".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
            },
        ];

        let mut seen_stmt_kinds = Vec::new();
        let mut seen_later_call = false;
        let result = walk_stmts_until(
            &stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut |stmt| {
                if matches!(stmt, HirStmt::Return { .. }) {
                    seen_stmt_kinds.push("return");
                    return TraversalControl::Stop;
                }
                seen_stmt_kinds.push("other");
                TraversalControl::Continue
            },
            &mut |expr| {
                if let HirExpr::Call { func, .. } = expr {
                    if func == "later" {
                        seen_later_call = true;
                    }
                }
                TraversalControl::Continue
            },
        );

        assert_eq!(result, TraversalControl::Stop);
        assert_eq!(seen_stmt_kinds, vec!["return"]);
        assert!(!seen_later_call);
    }
}
