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
    "anext",
    "aclose",
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
        HirStmt::AsyncWith {
            kind:
                sifr_hir::HirAsyncWithKind::UserDefined {
                    context: HirExpr::Name { name, .. },
                    ..
                },
            ..
        } => {
            mutated.borrow_mut().insert(name.clone());
        }
        HirStmt::AsyncFor {
            iter: HirExpr::Name { name, .. },
            ..
        } => {
            mutated.borrow_mut().insert(name.clone());
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
                .or_else(|| local_func_param_conventions.get(func).map(Vec::as_slice))
                .or_else(|| {
                    local_func_param_conventions
                        .get(canonical_func)
                        .map(Vec::as_slice)
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
            if canonical_func == "anext" {
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
        HirStmt::For { target, .. } | HirStmt::AsyncFor { target, .. } => {
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

pub(super) fn collect_capture_pattern_names(pattern: &HirPattern, defined: &mut HashSet<String>) {
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
