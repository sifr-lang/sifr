use crate::hir_analysis::traversal::{self, TraversalConfig};
use crate::ModuleFuncSignatures;
use sifr_hir::{HirExpr, HirPattern, HirStmt};
use sifr_type_system::{ParamConvention, Type};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

pub(crate) const MUTATING_METHODS: &[&str] = &[
    "append",
    "appendleft",
    "extend",
    "insert",
    "clear",
    "reverse",
    "sort",
    "pop",
    "popleft",
    "remove",
    "push_str",
    "update",
    "add",
    "discard",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlFlowEffect {
    FallsThrough,
    AlwaysReturns,
    AlwaysRaises,
    AlwaysExits,
}

impl ControlFlowEffect {
    pub(crate) fn always_exits(self) -> bool {
        !matches!(self, Self::FallsThrough)
    }
}

fn merge_branch_effects(effects: &[ControlFlowEffect]) -> ControlFlowEffect {
    if effects.iter().any(|effect| !effect.always_exits()) {
        return ControlFlowEffect::FallsThrough;
    }
    if effects
        .iter()
        .all(|effect| matches!(effect, ControlFlowEffect::AlwaysReturns))
    {
        return ControlFlowEffect::AlwaysReturns;
    }
    if effects
        .iter()
        .all(|effect| matches!(effect, ControlFlowEffect::AlwaysRaises))
    {
        return ControlFlowEffect::AlwaysRaises;
    }
    ControlFlowEffect::AlwaysExits
}

pub(crate) fn stmt_control_flow_effect(stmt: &HirStmt) -> ControlFlowEffect {
    match stmt {
        HirStmt::Return { .. } => ControlFlowEffect::AlwaysReturns,
        HirStmt::Raise { .. } => ControlFlowEffect::AlwaysRaises,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            let Some(else_body) = else_body else {
                return ControlFlowEffect::FallsThrough;
            };
            let mut branch_effects = Vec::with_capacity(2 + elif_clauses.len());
            branch_effects.push(block_control_flow_effect(then_body));
            for (_, body) in elif_clauses {
                branch_effects.push(block_control_flow_effect(body));
            }
            branch_effects.push(block_control_flow_effect(else_body));
            merge_branch_effects(&branch_effects)
        }
        HirStmt::Match { arms, .. } => {
            if arms.is_empty() {
                return ControlFlowEffect::FallsThrough;
            }
            let mut arm_effects = Vec::with_capacity(arms.len());
            for arm in arms {
                arm_effects.push(block_control_flow_effect(&arm.body));
            }
            merge_branch_effects(&arm_effects)
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            if handlers.is_empty() {
                return ControlFlowEffect::FallsThrough;
            }
            let mut branch_effects = Vec::with_capacity(1 + handlers.len());
            branch_effects.push(block_control_flow_effect(body));
            for handler in handlers {
                branch_effects.push(block_control_flow_effect(&handler.body));
            }
            merge_branch_effects(&branch_effects)
        }
        _ => ControlFlowEffect::FallsThrough,
    }
}

pub(crate) fn block_control_flow_effect(stmts: &[HirStmt]) -> ControlFlowEffect {
    for stmt in stmts {
        let effect = stmt_control_flow_effect(stmt);
        if effect.always_exits() {
            return effect;
        }
    }
    ControlFlowEffect::FallsThrough
}

pub(crate) fn body_contains_return(stmts: &[HirStmt]) -> bool {
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if matches!(stmt, HirStmt::Return { .. }) {
            found.set(true);
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    found.get()
}

pub(crate) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::Return { value: Some(val) } = stmt {
            if !matches!(val, HirExpr::NoneLiteral) {
                found.set(true);
            }
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    found.get()
}

pub(crate) fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if matches!(stmt, HirStmt::Yield { .. }) {
            found.set(true);
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    found.get()
}

pub(crate) fn body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    let mut found = false;
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if found {
            return;
        }
        if let HirExpr::Call { func, .. } = expr {
            if func == func_name {
                found = true;
            }
        }
    };
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    found
}

pub(crate) fn expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    let mut found = false;
    traversal::walk_expr(expr, &mut |node| {
        if found {
            return;
        }
        if let HirExpr::Call { func, .. } = node {
            if func == func_name {
                found = true;
            }
        }
    });
    found
}

pub(crate) fn expr_references_var(expr: &HirExpr, var_name: &str) -> bool {
    let mut found = false;
    traversal::walk_expr(expr, &mut |node| {
        if found {
            return;
        }
        if let HirExpr::Name { name, .. } = node {
            if name == var_name {
                found = true;
            }
        }
    });
    found
}

pub(crate) fn stmts_reference_var(stmts: &[HirStmt], var_name: &str) -> bool {
    let mut found = false;
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if found {
            return;
        }
        if let HirExpr::Name { name, .. } = expr {
            if name == var_name {
                found = true;
            }
        }
    };
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    found
}

pub(crate) fn collect_mutated_vars(
    stmts: &[HirStmt],
    func_signatures: Option<&ModuleFuncSignatures>,
) -> HashSet<String> {
    let mutated = RefCell::new(HashSet::new());

    let mut on_stmt = |stmt: &HirStmt| match stmt {
        HirStmt::Assign { name, .. } | HirStmt::AugAssign { name, .. } => {
            mutated.borrow_mut().insert(name.clone());
        }
        HirStmt::SubscriptAssign { object, .. }
        | HirStmt::NestedSubscriptAssign { object, .. }
        | HirStmt::SubscriptAugAssign { object, .. }
        | HirStmt::AttributeAugAssign { object, .. } => {
            mutated.borrow_mut().insert(object.clone());
        }
        HirStmt::Delete {
            object: HirExpr::Name { name, .. },
            ..
        } => {
            mutated.borrow_mut().insert(name.clone());
        }
        _ => {}
    };

    let mut on_expr = |expr: &HirExpr| match expr {
        HirExpr::MethodCall {
            object,
            method,
            args: _,
            ..
        } => {
            if MUTATING_METHODS.contains(&method.as_str()) {
                if let HirExpr::Name { name, .. } = object.as_ref() {
                    mutated.borrow_mut().insert(name.clone());
                }
            }
            if matches!(object.ty(), Type::Class { .. }) {
                if let HirExpr::Name { name, .. } = object.as_ref() {
                    mutated.borrow_mut().insert(name.clone());
                }
            }
        }
        HirExpr::Call { func, args, .. } => {
            if let Some(sigs) = func_signatures {
                if let Some((param_convs, _)) = sigs.get(func) {
                    for (idx, arg) in args.iter().enumerate() {
                        if let Some((_, ParamConvention::MutBorrow)) = param_convs.get(idx) {
                            if let HirExpr::Name { name, .. } = arg {
                                mutated.borrow_mut().insert(name.clone());
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    };

    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );

    mutated.into_inner()
}

pub(crate) fn collect_referenced_vars_with_types(stmts: &[HirStmt]) -> Vec<(String, Type)> {
    let mut refs: HashMap<String, Type> = HashMap::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::Name { name, ty } = expr {
            refs.entry(name.clone()).or_insert_with(|| ty.clone());
        }
    };
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    refs.into_iter().collect()
}

pub(crate) fn collect_typed_refs_in_expr(expr: &HirExpr, refs: &mut HashMap<String, Type>) {
    traversal::walk_expr(expr, &mut |node| {
        if let HirExpr::Name { name, ty } = node {
            refs.entry(name.clone()).or_insert_with(|| ty.clone());
        }
    });
}

pub(crate) fn collect_locally_defined_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let mut defined = HashSet::new();
    let mut on_stmt = |stmt: &HirStmt| match stmt {
        HirStmt::Let { name, .. } => {
            defined.insert(name.clone());
        }
        HirStmt::For { target, .. } => {
            defined.insert(target.clone());
        }
        HirStmt::TupleUnpack { targets, .. } => {
            for (name, _) in targets {
                defined.insert(name.clone());
            }
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            ..
        } => {
            for (name, _) in before {
                defined.insert(name.clone());
            }
            defined.insert(star.0.clone());
            for (name, _) in after {
                defined.insert(name.clone());
            }
        }
        HirStmt::NestedFunction { func } => {
            defined.insert(func.name.clone());
        }
        HirStmt::Match { arms, .. } => {
            for arm in arms {
                collect_capture_pattern_names(&arm.pattern, &mut defined);
            }
        }
        _ => {}
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    defined
}

fn collect_capture_pattern_names(pattern: &HirPattern, defined: &mut HashSet<String>) {
    match pattern {
        HirPattern::Capture { name, .. } => {
            defined.insert(name.clone());
        }
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                collect_capture_pattern_names(pattern, defined);
            }
        }
        HirPattern::Class { fields, .. } => {
            for (_, pattern) in fields {
                collect_capture_pattern_names(pattern, defined);
            }
        }
        HirPattern::Wildcard
        | HirPattern::Literal { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_hir::{HirFunction, HirParam, MethodKind};

    #[test]
    fn collect_mutated_vars_marks_mutborrow_call_argument() {
        let stmts = vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "touch".to_string(),
                args: vec![HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                }],
                ty: Type::None,
            },
        }];

        let mut sigs: ModuleFuncSignatures = HashMap::new();
        sigs.insert(
            "touch".to_string(),
            (
                vec![(Type::List(Box::new(Type::Int)), ParamConvention::MutBorrow)],
                Type::None,
            ),
        );

        let mutated = collect_mutated_vars(&stmts, Some(&sigs));
        assert!(mutated.contains("items"));
    }

    #[test]
    fn body_calls_function_ignores_nested_function_scope() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![HirParam {
                name: "n".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::Own,
            }],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "target".to_string(),
                    args: vec![HirExpr::Name {
                        name: "n".to_string(),
                        ty: Type::Int,
                    }],
                    ty: Type::Int,
                }),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };
        let stmts = vec![HirStmt::NestedFunction { func: nested }];

        assert!(!body_calls_function(&stmts, "target"));
    }

    #[test]
    fn body_contains_yield_detects_try_except_and_loop_else_paths() {
        let stmts = vec![HirStmt::TryExcept {
            body: vec![HirStmt::While {
                condition: HirExpr::BoolLiteral(false),
                body: vec![HirStmt::Pass],
                else_body: Some(vec![HirStmt::Yield {
                    value: HirExpr::IntLiteral(1),
                }]),
            }],
            handlers: vec![sifr_hir::HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Yield {
                    value: HirExpr::IntLiteral(2),
                }],
            }],
            body_error_types: vec!["Error".to_string()],
        }];

        assert!(body_contains_yield(&stmts));
    }

    #[test]
    fn collect_locally_defined_vars_includes_match_captures() {
        let stmts = vec![HirStmt::Match {
            subject: HirExpr::IntLiteral(3),
            subject_ty: Type::Int,
            arms: vec![sifr_hir::HirMatchArm {
                pattern: HirPattern::Capture {
                    name: "x".to_string(),
                    ty: Type::Int,
                },
                guard: None,
                body: vec![HirStmt::Pass],
            }],
        }];

        let defined = collect_locally_defined_vars(&stmts);
        assert!(defined.contains("x"));
    }

    #[test]
    fn collect_locally_defined_vars_ignores_nested_function_body_bindings() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Let {
                name: "nested_local".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(1),
                is_mutable: true,
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };

        let stmts = vec![HirStmt::NestedFunction { func: nested }];
        let defined = collect_locally_defined_vars(&stmts);

        assert!(defined.contains("inner"));
        assert!(!defined.contains("nested_local"));
    }

    #[test]
    fn collect_mutated_vars_handles_nested_exprs() {
        let stmts = vec![HirStmt::Let {
            name: "x".to_string(),
            ty: Type::List(Box::new(Type::Int)),
            value: HirExpr::Call {
                func: "id".to_string(),
                args: vec![HirExpr::MethodCall {
                    object: Box::new(HirExpr::Name {
                        name: "x".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    }),
                    method: "append".to_string(),
                    args: vec![HirExpr::IntLiteral(1)],
                    ty: Type::None,
                }],
                ty: Type::None,
            },
            is_mutable: true,
        }];

        let mutated = collect_mutated_vars(&stmts, None);
        assert!(mutated.contains("x"));
    }

    #[test]
    fn collect_mutated_vars_ignores_nested_function_scope() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Assign {
                name: "inside".to_string(),
                value: HirExpr::IntLiteral(1),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };

        let mutated = collect_mutated_vars(&[HirStmt::NestedFunction { func: nested }], None);
        assert!(!mutated.contains("inside"));
    }

    #[test]
    fn collect_typed_refs_in_expr_includes_fstring_interpolations() {
        let expr = HirExpr::FString {
            parts: vec![
                sifr_hir::HirFStringPart::Literal("value=".to_string()),
                sifr_hir::HirFStringPart::Expr(HirExpr::Name {
                    name: "n".to_string(),
                    ty: Type::Int,
                }),
            ],
            ty: Type::Str,
        };
        let mut refs = HashMap::new();
        collect_typed_refs_in_expr(&expr, &mut refs);

        assert_eq!(refs.get("n"), Some(&Type::Int));
    }

    #[test]
    fn block_control_flow_effect_reports_always_returns_for_exhaustive_if() {
        let effect = block_control_flow_effect(&[HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
            elif_clauses: vec![],
            else_body: Some(vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(2)),
            }]),
        }]);

        assert_eq!(effect, ControlFlowEffect::AlwaysReturns);
        assert!(effect.always_exits());
    }

    #[test]
    fn block_control_flow_effect_reports_fallthrough_for_non_exhaustive_if() {
        let effect = block_control_flow_effect(&[HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
            elif_clauses: vec![],
            else_body: None,
        }]);

        assert_eq!(effect, ControlFlowEffect::FallsThrough);
        assert!(!effect.always_exits());
    }

    #[test]
    fn block_control_flow_effect_reports_always_exits_for_mixed_return_raise() {
        let effect = block_control_flow_effect(&[HirStmt::TryExcept {
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
            handlers: vec![sifr_hir::HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Raise {
                    value: HirExpr::Call {
                        func: "ValueError".to_string(),
                        args: vec![HirExpr::StringLiteral("bad".to_string())],
                        ty: Type::Unknown,
                    },
                }],
            }],
            body_error_types: vec!["Error".to_string()],
        }]);

        assert_eq!(effect, ControlFlowEffect::AlwaysExits);
        assert!(effect.always_exits());
    }
}
