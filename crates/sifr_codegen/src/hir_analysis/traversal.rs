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

/// Canonical traversal contract for HIR analysis.
///
/// Invariants:
/// - Expression/statement recursion for analysis must flow through this module.
/// - `on_stmt` is invoked once for each visited statement before children.
/// - `on_expr` is invoked once for each visited expression before children.
/// - Nested function recursion is controlled only by `TraversalConfig`.
///
/// Extension rules for future HIR variants:
/// - Any added `HirExpr`/`HirStmt`/`HirPattern` variant must be handled here.
/// - If a new variant contains sub-statements or sub-expressions, recurse into
///   every child to preserve traversal completeness.
/// - Callers must not introduce emitter-local recursive descent for analysis;
///   add/extend query helpers on top of this traversal layer instead.
pub(crate) fn walk_expr<F>(expr: &HirExpr, on_expr: &mut F)
where
    F: FnMut(&HirExpr),
{
    on_expr(expr);
    match expr {
        HirExpr::BinOp { left, right, .. } => {
            walk_expr(left, on_expr);
            walk_expr(right, on_expr);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::WalrusExpr { value: operand, .. }
        | HirExpr::FieldAccess {
            object: operand, ..
        } => {
            walk_expr(operand, on_expr);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            walk_expr(left, on_expr);
            for comparator in comparators {
                walk_expr(comparator, on_expr);
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
                walk_expr(value, on_expr);
            }
        }
        HirExpr::Call { args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::SuperCall { args, .. } => {
            for arg in args {
                walk_expr(arg, on_expr);
            }
        }
        HirExpr::MethodCall { object, args, .. } => {
            walk_expr(object, on_expr);
            for arg in args {
                walk_expr(arg, on_expr);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            walk_expr(condition, on_expr);
            walk_expr(then_expr, on_expr);
            walk_expr(else_expr, on_expr);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            walk_expr(start, on_expr);
            walk_expr(end, on_expr);
            if let Some(step) = step {
                walk_expr(step, on_expr);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for key in keys {
                walk_expr(key, on_expr);
            }
            for value in values {
                walk_expr(value, on_expr);
            }
        }
        HirExpr::Index { object, index, .. } => {
            walk_expr(object, on_expr);
            walk_expr(index, on_expr);
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            walk_expr(element, on_expr);
            walk_expr(collection, on_expr);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    walk_expr(expr, on_expr);
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
            walk_expr(object, on_expr);
            if let Some(start) = start {
                walk_expr(start, on_expr);
            }
            if let Some(stop) = stop {
                walk_expr(stop, on_expr);
            }
            if let Some(step) = step {
                walk_expr(step, on_expr);
            }
        }
        HirExpr::Lambda { body, .. } => walk_expr(body, on_expr),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            walk_expr(expr, on_expr);
            for (_, iter, filter) in generators {
                walk_expr(iter, on_expr);
                if let Some(filter) = filter {
                    walk_expr(filter, on_expr);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            walk_expr(key_expr, on_expr);
            walk_expr(val_expr, on_expr);
            for (_, iter, filter) in generators {
                walk_expr(iter, on_expr);
                if let Some(filter) = filter {
                    walk_expr(filter, on_expr);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            walk_expr(expr, on_expr);
            walk_expr(iter, on_expr);
            if let Some(filter) = filter {
                walk_expr(filter, on_expr);
            }
        }
        HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => {}
    }
}

pub(crate) fn walk_pattern<F>(pattern: &HirPattern, on_expr: &mut F)
where
    F: FnMut(&HirExpr),
{
    match pattern {
        HirPattern::Literal { value } => walk_expr(value, on_expr),
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                walk_pattern(pattern, on_expr);
            }
        }
        HirPattern::Class { fields, .. } => {
            for (_, pattern) in fields {
                walk_pattern(pattern, on_expr);
            }
        }
        HirPattern::Wildcard
        | HirPattern::Capture { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => {}
    }
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
    on_stmt(stmt);
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value } => walk_expr(value, on_expr),
        HirStmt::Return { value } => {
            if let Some(value) = value {
                walk_expr(value, on_expr);
            }
        }
        HirStmt::Expr { expr } => walk_expr(expr, on_expr),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            walk_expr(condition, on_expr);
            walk_stmts(then_body, config, on_stmt, on_expr);
            for (cond, body) in elif_clauses {
                walk_expr(cond, on_expr);
                walk_stmts(body, config, on_stmt, on_expr);
            }
            if let Some(else_body) = else_body {
                walk_stmts(else_body, config, on_stmt, on_expr);
            }
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            walk_expr(condition, on_expr);
            walk_stmts(body, config, on_stmt, on_expr);
            if let Some(else_body) = else_body {
                walk_stmts(else_body, config, on_stmt, on_expr);
            }
        }
        HirStmt::For {
            iter,
            body,
            else_body,
            ..
        } => {
            walk_expr(iter, on_expr);
            walk_stmts(body, config, on_stmt, on_expr);
            if let Some(else_body) = else_body {
                walk_stmts(else_body, config, on_stmt, on_expr);
            }
        }
        HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => {
            walk_expr(value, on_expr);
        }
        HirStmt::Assert { test, msg } => {
            walk_expr(test, on_expr);
            if let Some(msg) = msg {
                walk_expr(msg, on_expr);
            }
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            walk_stmts(body, config, on_stmt, on_expr);
            for handler in handlers {
                walk_stmts(&handler.body, config, on_stmt, on_expr);
            }
        }
        HirStmt::SubscriptAssign { index, value, .. }
        | HirStmt::SubscriptAugAssign { index, value, .. }
        | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
            walk_expr(index, on_expr);
            walk_expr(value, on_expr);
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        } => {
            walk_expr(outer_index, on_expr);
            walk_expr(inner_index, on_expr);
            walk_expr(value, on_expr);
        }
        HirStmt::Delete { object, index } => {
            walk_expr(object, on_expr);
            walk_expr(index, on_expr);
        }
        HirStmt::With { items, body } => {
            for (_, expr, _) in items {
                walk_expr(expr, on_expr);
            }
            walk_stmts(body, config, on_stmt, on_expr);
        }
        HirStmt::NestedFunction { func } => {
            if config.descend_nested_functions {
                walk_stmts(&func.body, config, on_stmt, on_expr);
            }
        }
        HirStmt::Match { subject, arms, .. } => {
            walk_expr(subject, on_expr);
            for arm in arms {
                walk_pattern(&arm.pattern, on_expr);
                if let Some(guard) = &arm.guard {
                    walk_expr(guard, on_expr);
                }
                walk_stmts(&arm.body, config, on_stmt, on_expr);
            }
        }
        HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
    }
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
    for stmt in stmts {
        walk_stmt(stmt, config, on_stmt, on_expr);
    }
}

#[cfg(test)]
mod tests {
    use super::{walk_stmts, TraversalConfig};
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
                convention: ParamConvention::Own,
            }],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "nested_only".to_string(),
                    args: vec![],
                    ty: Type::None,
                },
            }],
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
}
