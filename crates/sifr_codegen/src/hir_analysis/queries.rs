use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use crate::ModuleFuncSignatures;
use sifr_hir::{cfg, HirExpr, HirIteratorOp, HirPattern, HirStmt};
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
    "setdefault",
    "add",
    "intersection_update",
    "difference_update",
    "symmetric_difference_update",
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

impl From<cfg::FlowExitEffect> for ControlFlowEffect {
    fn from(effect: cfg::FlowExitEffect) -> Self {
        match effect {
            cfg::FlowExitEffect::FallsThrough => Self::FallsThrough,
            cfg::FlowExitEffect::AlwaysReturns => Self::AlwaysReturns,
            cfg::FlowExitEffect::AlwaysRaises => Self::AlwaysRaises,
            cfg::FlowExitEffect::AlwaysExits => Self::AlwaysExits,
        }
    }
}

pub(crate) fn block_control_flow_effect(stmts: &[HirStmt]) -> ControlFlowEffect {
    ControlFlowEffect::from(cfg::flow_facts(stmts).exit_effect())
}

pub(crate) fn reachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    cfg::flow_facts(stmts)
        .reachable_top_level_stmt_indices()
        .to_vec()
}

pub(crate) fn unreachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    cfg::flow_facts(stmts)
        .unreachable_top_level_stmt_indices()
        .to_vec()
}

pub(crate) fn body_contains_return(stmts: &[HirStmt]) -> bool {
    cfg::flow_facts(stmts).has_reachable_return()
}

pub(crate) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    cfg::flow_facts(stmts).has_reachable_value_return()
}

pub(crate) fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    let mut on_stmt = |stmt: &HirStmt| {
        if matches!(stmt, HirStmt::Yield { .. }) {
            return TraversalControl::Stop;
        }
        TraversalControl::Continue
    };
    let mut on_expr = |_expr: &HirExpr| TraversalControl::Continue;
    matches!(
        traversal::walk_stmts_until(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        ),
        TraversalControl::Stop
    )
}

pub(crate) fn body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::Call { func, .. } = expr {
            if func == func_name {
                return TraversalControl::Stop;
            }
        }
        TraversalControl::Continue
    };
    matches!(
        traversal::walk_stmts_until(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        ),
        TraversalControl::Stop
    )
}

pub(crate) fn expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    matches!(
        traversal::walk_expr_until(expr, &mut |node| {
            if let HirExpr::Call { func, .. } = node {
                if func == func_name {
                    return TraversalControl::Stop;
                }
            }
            TraversalControl::Continue
        }),
        TraversalControl::Stop
    )
}

pub(crate) fn expr_references_var(expr: &HirExpr, var_name: &str) -> bool {
    matches!(
        traversal::walk_expr_until(expr, &mut |node| {
            if let HirExpr::Name { name, .. } = node {
                if name == var_name {
                    return TraversalControl::Stop;
                }
            }
            TraversalControl::Continue
        }),
        TraversalControl::Stop
    )
}

pub(crate) fn stmts_reference_var(stmts: &[HirStmt], var_name: &str) -> bool {
    let mut on_stmt = |_stmt: &HirStmt| TraversalControl::Continue;
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::Name { name, .. } = expr {
            if name == var_name {
                return TraversalControl::Stop;
            }
        }
        TraversalControl::Continue
    };
    matches!(
        traversal::walk_stmts_until(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        ),
        TraversalControl::Stop
    )
}

pub(crate) fn collect_mutated_vars(
    stmts: &[HirStmt],
    func_signatures: Option<&ModuleFuncSignatures>,
) -> HashSet<String> {
    fn collect_nested_assign_targets(stmts: &[HirStmt]) -> HashSet<String> {
        let mut assigned = HashSet::new();
        let mut on_stmt = |stmt: &HirStmt| {
            if let HirStmt::Assign { name, .. } = stmt {
                assigned.insert(name.clone());
            }
        };
        let mut on_expr = |_expr: &HirExpr| {};
        traversal::walk_stmts(
            stmts,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        assigned
    }

    fn canonical_mutating_call_name(func: &str) -> &str {
        let canonical = func.strip_prefix("__compat_sifr_heapq_").unwrap_or(func);
        canonical.rsplit('.').next().unwrap_or(canonical)
    }

    fn effective_nested_param_convention(
        param_convention: ParamConvention,
        param_ty: &Type,
        nested_mutated_vars: &HashSet<String>,
        param_name: &str,
    ) -> ParamConvention {
        if !nested_mutated_vars.contains(param_name) {
            return param_convention;
        }
        if param_ty.ownership() == sifr_type_system::OwnershipKind::Copy {
            return if param_convention.is_owned() {
                ParamConvention::own_mut()
            } else {
                param_convention
            };
        }
        if param_convention.is_borrowed() {
            ParamConvention::mut_borrow()
        } else {
            ParamConvention::own_mut()
        }
    }

    fn collect_local_function_param_conventions(
        stmts: &[HirStmt],
        func_signatures: Option<&ModuleFuncSignatures>,
    ) -> HashMap<String, Vec<(Type, ParamConvention)>> {
        let mut local = HashMap::new();
        for stmt in stmts {
            let HirStmt::NestedFunction { func } = stmt else {
                continue;
            };
            let nested_mutated_vars = collect_mutated_vars(&func.body, func_signatures);
            let params = func
                .params
                .iter()
                .map(|param| {
                    (
                        param.ty.clone(),
                        effective_nested_param_convention(
                            param.convention,
                            &param.ty,
                            &nested_mutated_vars,
                            &param.name,
                        ),
                    )
                })
                .collect::<Vec<_>>();
            local.insert(func.name.clone(), params);
        }
        local
    }

    let local_func_param_conventions =
        collect_local_function_param_conventions(stmts, func_signatures);
    let mutated = RefCell::new(HashSet::new());

    let mut on_stmt = |stmt: &HirStmt| match stmt {
        HirStmt::Assign { name, .. } | HirStmt::AugAssign { name, .. } => {
            mutated.borrow_mut().insert(name.clone());
        }
        HirStmt::NestedFunction { func } => {
            let param_names = func
                .params
                .iter()
                .map(|param| param.name.clone())
                .collect::<HashSet<_>>();
            let locally_defined = collect_locally_defined_vars(&func.body);
            let assigned_in_nested = collect_nested_assign_targets(&func.body);
            let captured_mutated = collect_mutated_vars(&func.body, func_signatures)
                .into_iter()
                .filter(|name| {
                    !param_names.contains(name)
                        && !locally_defined.contains(name)
                        && !assigned_in_nested.contains(name)
                })
                .collect::<Vec<_>>();
            mutated.borrow_mut().extend(captured_mutated);
        }
        HirStmt::SubscriptAssign { object, .. }
        | HirStmt::NestedSubscriptAssign { object, .. }
        | HirStmt::AttributeNestedSubscriptAssign { object, .. }
        | HirStmt::SubscriptAugAssign { object, .. }
        | HirStmt::AttributeAugAssign { object, .. }
        | HirStmt::FieldAssign { object, .. }
        | HirStmt::NestedFieldAssign { object, .. }
        | HirStmt::AttributeSubscriptAssign { object, .. } => {
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
            let root_name = match object.as_ref() {
                HirExpr::Name { name, .. } => Some(name.clone()),
                HirExpr::FieldAccess { object: inner, .. } => match inner.as_ref() {
                    HirExpr::Name { name, .. } => Some(name.clone()),
                    _ => None,
                },
                _ => None,
            };
            if MUTATING_METHODS.contains(&method.as_str())
                || matches!(object.ty(), Type::Class { .. })
            {
                if let Some(name) = root_name {
                    mutated.borrow_mut().insert(name);
                }
            }
        }
        HirExpr::Call { func, args, .. } => {
            let canonical_func = canonical_mutating_call_name(func);
            let param_convs = func_signatures
                .and_then(|sigs| {
                    sigs.get(func)
                        .map(|(param_convs, _)| param_convs.as_slice())
                })
                .or_else(|| {
                    func_signatures.and_then(|sigs| {
                        sigs.get(canonical_func)
                            .map(|(param_convs, _)| param_convs.as_slice())
                    })
                })
                .or_else(|| {
                    local_func_param_conventions
                        .get(func)
                        .map(|param_convs| param_convs.as_slice())
                })
                .or_else(|| {
                    local_func_param_conventions
                        .get(canonical_func)
                        .map(|param_convs| param_convs.as_slice())
                });
            if let Some(param_convs) = param_convs {
                for (idx, arg) in args.iter().enumerate() {
                    if param_convs
                        .get(idx)
                        .is_some_and(|(_, convention)| convention.is_mutable())
                    {
                        if let HirExpr::Name { name, .. } = arg {
                            mutated.borrow_mut().insert(name.clone());
                        }
                    }
                }
            }
            if matches!(
                canonical_func,
                "heappush" | "heappop" | "heapify" | "heapreplace" | "heappushpop"
            ) {
                if let Some(HirExpr::Name { name, .. }) = args.first() {
                    mutated.borrow_mut().insert(name.clone());
                }
            }
        }
        HirExpr::IteratorCall { op, args, .. } => {
            if *op == HirIteratorOp::Next {
                if let Some(HirExpr::Name { name, .. }) = args.first() {
                    mutated.borrow_mut().insert(name.clone());
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

pub(crate) fn collect_reassigned_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let reassigned = RefCell::new(HashSet::new());

    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::Assign { name, .. } | HirStmt::AugAssign { name, .. } = stmt {
            reassigned.borrow_mut().insert(name.clone());
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};

    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );

    reassigned.into_inner()
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
            for target in targets {
                if !target.rebind_existing {
                    if let sifr_hir::HirTupleTargetBinding::Name(name) = &target.binding {
                        defined.insert(name.clone());
                    }
                }
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TypeVarOpRequirements {
    pub needs_add: bool,
    pub needs_sub: bool,
}

pub(crate) fn collect_typevar_operator_requirements(
    stmts: &[HirStmt],
    type_param_name: &str,
) -> TypeVarOpRequirements {
    let mut requirements = TypeVarOpRequirements::default();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } = expr
        {
            let left_is_tp =
                matches!(left.ty(), Type::TypeVar(ref name) if name == type_param_name);
            let right_is_tp =
                matches!(right.ty(), Type::TypeVar(ref name) if name == type_param_name);
            let result_is_tp = matches!(ty, Type::TypeVar(ref name) if name == type_param_name);
            if left_is_tp || right_is_tp || result_is_tp {
                match op.as_str() {
                    "+" => requirements.needs_add = true,
                    "-" => requirements.needs_sub = true,
                    _ => {}
                }
            }
        }
    };
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    requirements
}

pub(crate) fn collect_let_declared_types(stmts: &[HirStmt]) -> Vec<Type> {
    let mut declared = Vec::new();
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::Let { ty, .. } = stmt {
            declared.push(ty.clone());
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    declared
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
    use sifr_type_system::ParamConvention;

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
                vec![(
                    Type::List(Box::new(Type::Int)),
                    ParamConvention::mut_borrow(),
                )],
                Type::None,
            ),
        );

        let mutated = collect_mutated_vars(&stmts, Some(&sigs));
        assert!(mutated.contains("items"));
    }

    #[test]
    fn collect_mutated_vars_marks_local_nested_function_mutborrow_call_argument() {
        let nested = HirFunction {
            name: "touch_local".to_string(),
            params: vec![HirParam {
                name: "xs".to_string(),
                ty: Type::List(Box::new(Type::Int)),
                default: None,
                keyword_only: false,
                convention: ParamConvention::own(),
            }],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::MethodCall {
                    object: Box::new(HirExpr::Name {
                        name: "xs".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    }),
                    method: "append".to_string(),
                    args: vec![HirExpr::IntLiteral(1)],
                    ty: Type::None,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };

        let stmts = vec![
            HirStmt::NestedFunction { func: nested },
            HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "touch_local".to_string(),
                    args: vec![HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    }],
                    ty: Type::None,
                },
            },
        ];

        let mutated = collect_mutated_vars(&stmts, None);
        assert!(mutated.contains("items"));
    }

    #[test]
    fn collect_mutated_vars_marks_iterator_next_argument() {
        let iterator_ty = Type::Class {
            name: "CountdownIter".to_string(),
            fields: vec![],
            methods: vec![(
                "__next__".to_string(),
                sifr_type_system::FunctionType {
                    params: vec![],
                    return_type: Box::new(Type::Union(vec![Type::Int, Type::None])),
                },
            )],
            parent_class: None,
        };
        let stmts = vec![HirStmt::Expr {
            expr: HirExpr::IteratorCall {
                op: HirIteratorOp::Next,
                args: vec![HirExpr::Name {
                    name: "it".to_string(),
                    ty: iterator_ty,
                }],
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
        }];

        let mutated = collect_mutated_vars(&stmts, None);
        assert!(mutated.contains("it"));
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
                convention: ParamConvention::own(),
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
    fn collect_mutated_vars_includes_captured_rebinds_from_nested_functions() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::AugAssign {
                name: "total".to_string(),
                op: "+=".to_string(),
                value: HirExpr::IntLiteral(1),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };

        let mutated = collect_mutated_vars(&[HirStmt::NestedFunction { func: nested }], None);
        assert!(mutated.contains("total"));
    }

    #[test]
    fn collect_mutated_vars_marks_captured_outer_mutation_from_nested_function() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::Expr {
                expr: HirExpr::MethodCall {
                    object: Box::new(HirExpr::Name {
                        name: "items".to_string(),
                        ty: Type::List(Box::new(Type::Int)),
                    }),
                    method: "append".to_string(),
                    args: vec![HirExpr::IntLiteral(1)],
                    ty: Type::None,
                },
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };

        let mutated = collect_mutated_vars(&[HirStmt::NestedFunction { func: nested }], None);
        assert!(mutated.contains("items"));
    }

    #[test]
    fn collect_mutated_vars_marks_dict_setdefault_receiver() {
        let stmts = vec![HirStmt::Expr {
            expr: HirExpr::MethodCall {
                object: Box::new(HirExpr::Name {
                    name: "data".to_string(),
                    ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
                }),
                method: "setdefault".to_string(),
                args: vec![
                    HirExpr::StringLiteral("k".to_string()),
                    HirExpr::IntLiteral(1),
                ],
                ty: Type::Int,
            },
        }];

        let mutated = collect_mutated_vars(&stmts, None);
        assert!(mutated.contains("data"));
    }

    #[test]
    fn collect_mutated_vars_marks_set_update_receiver() {
        let stmts = vec![HirStmt::Expr {
            expr: HirExpr::MethodCall {
                object: Box::new(HirExpr::Name {
                    name: "seen".to_string(),
                    ty: Type::Set(Box::new(Type::Int)),
                }),
                method: "intersection_update".to_string(),
                args: vec![HirExpr::ListLiteral {
                    elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
                    ty: Type::List(Box::new(Type::Int)),
                }],
                ty: Type::None,
            },
        }];

        let mutated = collect_mutated_vars(&stmts, None);
        assert!(mutated.contains("seen"));
    }

    #[test]
    fn collect_mutated_vars_marks_self_for_delegated_field_class_method_call() {
        let writer_ty = Type::Class {
            name: "writer".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let holder_ty = Type::Class {
            name: "DictWriter".to_string(),
            fields: vec![("_writer".to_string(), writer_ty.clone())],
            methods: vec![],
            parent_class: None,
        };
        let stmts = vec![HirStmt::Expr {
            expr: HirExpr::MethodCall {
                object: Box::new(HirExpr::FieldAccess {
                    object: Box::new(HirExpr::Name {
                        name: "self".to_string(),
                        ty: holder_ty,
                    }),
                    field: "_writer".to_string(),
                    ty: writer_ty,
                }),
                method: "writerow".to_string(),
                args: vec![],
                ty: Type::None,
            },
        }];

        let mutated = collect_mutated_vars(&stmts, None);
        assert!(mutated.contains("self"));
    }

    #[test]
    fn collect_mutated_vars_marks_field_assign_object() {
        let stmts = vec![HirStmt::FieldAssign {
            object: "root".to_string(),
            field: "left".to_string(),
            field_ty: Type::Int,
            value: HirExpr::IntLiteral(1),
        }];

        let mutated = collect_mutated_vars(&stmts, None);
        assert!(mutated.contains("root"));
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

    #[test]
    fn reachable_stmt_indices_omit_unreachable_tail_after_return() {
        let stmts = vec![
            HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            },
            HirStmt::Expr {
                expr: HirExpr::IntLiteral(2),
            },
        ];
        assert_eq!(reachable_top_level_stmt_indices(&stmts), vec![0]);
        assert_eq!(unreachable_top_level_stmt_indices(&stmts), vec![1]);
    }

    #[test]
    fn body_contains_return_ignores_unreachable_return() {
        let stmts = vec![
            HirStmt::Raise {
                value: HirExpr::Call {
                    func: "ValueError".to_string(),
                    args: vec![HirExpr::StringLiteral("bad".to_string())],
                    ty: Type::Unknown,
                },
            },
            HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            },
        ];
        assert!(!body_contains_return(&stmts));
    }

    #[test]
    fn try_body_has_value_return_ignores_unreachable_value_return() {
        let stmts = vec![
            HirStmt::Raise {
                value: HirExpr::Call {
                    func: "ValueError".to_string(),
                    args: vec![HirExpr::StringLiteral("bad".to_string())],
                    ty: Type::Unknown,
                },
            },
            HirStmt::Return {
                value: Some(HirExpr::IntLiteral(99)),
            },
        ];
        assert!(!try_body_has_value_return(&stmts));
    }

    #[test]
    fn collect_typevar_operator_requirements_detects_add_and_sub() {
        let stmts = vec![
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        name: "a".to_string(),
                        ty: Type::TypeVar("T".to_string()),
                    }),
                    op: "+".to_string(),
                    right: Box::new(HirExpr::Name {
                        name: "b".to_string(),
                        ty: Type::TypeVar("T".to_string()),
                    }),
                    ty: Type::TypeVar("T".to_string()),
                },
            },
            HirStmt::Expr {
                expr: HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        name: "a".to_string(),
                        ty: Type::TypeVar("T".to_string()),
                    }),
                    op: "-".to_string(),
                    right: Box::new(HirExpr::Name {
                        name: "b".to_string(),
                        ty: Type::TypeVar("T".to_string()),
                    }),
                    ty: Type::TypeVar("T".to_string()),
                },
            },
        ];

        let req = collect_typevar_operator_requirements(&stmts, "T");
        assert!(req.needs_add);
        assert!(req.needs_sub);
    }

    #[test]
    fn collect_let_declared_types_covers_nested_blocks() {
        let stmts = vec![HirStmt::If {
            condition: HirExpr::BoolLiteral(true),
            then_body: vec![HirStmt::Let {
                name: "x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::Str]),
                value: HirExpr::IntLiteral(1),
                is_mutable: true,
            }],
            elif_clauses: vec![],
            else_body: None,
        }];

        let declared = collect_let_declared_types(&stmts);
        assert_eq!(declared.len(), 1);
        assert!(matches!(declared[0], Type::Union(_)));
    }
}
