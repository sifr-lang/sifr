use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
use crate::ModuleFuncSignatures;
use sifr_ir::{HirExpr, HirIteratorOp, HirPattern, HirStmt};
use sifr_type_system::{ParamConvention, ReceiverConvention, Type};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

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

pub(crate) fn block_control_flow_effect(stmts: &[HirStmt]) -> ControlFlowEffect {
    flow_summary(stmts).control_flow_effect()
}

pub(crate) fn reachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    let mut reachable = Vec::new();
    let mut falls_through = true;
    for (idx, stmt) in stmts.iter().enumerate() {
        if !falls_through {
            break;
        }
        reachable.push(idx);
        falls_through = stmt_summary(stmt).falls_through;
    }
    reachable
}

pub(crate) fn unreachable_top_level_stmt_indices(stmts: &[HirStmt]) -> Vec<usize> {
    let reachable_len = reachable_top_level_stmt_indices(stmts).len();
    (reachable_len..stmts.len()).collect()
}

pub(crate) fn body_contains_return(stmts: &[HirStmt]) -> bool {
    flow_summary(stmts).has_return
}

pub(crate) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    flow_summary(stmts).has_value_return
}

#[derive(Debug, Clone, Copy)]
struct FlowSummary {
    falls_through: bool,
    has_return: bool,
    has_value_return: bool,
    has_raise: bool,
}

impl FlowSummary {
    const fn fallthrough() -> Self {
        Self {
            falls_through: true,
            has_return: false,
            has_value_return: false,
            has_raise: false,
        }
    }

    const fn return_stmt(has_value: bool) -> Self {
        Self {
            falls_through: false,
            has_return: true,
            has_value_return: has_value,
            has_raise: false,
        }
    }

    const fn raise_stmt() -> Self {
        Self {
            falls_through: false,
            has_return: false,
            has_value_return: false,
            has_raise: true,
        }
    }

    fn union(self, other: Self) -> Self {
        Self {
            falls_through: self.falls_through || other.falls_through,
            has_return: self.has_return || other.has_return,
            has_value_return: self.has_value_return || other.has_value_return,
            has_raise: self.has_raise || other.has_raise,
        }
    }

    fn sequence(self, next: Self) -> Self {
        if self.falls_through {
            Self {
                falls_through: next.falls_through,
                has_return: self.has_return || next.has_return,
                has_value_return: self.has_value_return || next.has_value_return,
                has_raise: self.has_raise || next.has_raise,
            }
        } else {
            self
        }
    }

    fn control_flow_effect(self) -> ControlFlowEffect {
        if self.falls_through {
            ControlFlowEffect::FallsThrough
        } else if self.has_return && !self.has_raise {
            ControlFlowEffect::AlwaysReturns
        } else if self.has_raise && !self.has_return {
            ControlFlowEffect::AlwaysRaises
        } else {
            ControlFlowEffect::AlwaysExits
        }
    }
}

fn flow_summary(stmts: &[HirStmt]) -> FlowSummary {
    stmts
        .iter()
        .fold(FlowSummary::fallthrough(), |summary, stmt| {
            summary.sequence(stmt_summary(stmt))
        })
}

fn branch_summary<'a>(branches: impl IntoIterator<Item = &'a [HirStmt]>) -> FlowSummary {
    branches
        .into_iter()
        .map(flow_summary)
        .reduce(FlowSummary::union)
        .unwrap_or_else(FlowSummary::fallthrough)
}

fn stmt_summary(stmt: &HirStmt) -> FlowSummary {
    match stmt {
        HirStmt::Return { value } => FlowSummary::return_stmt(
            value
                .as_ref()
                .is_some_and(|expr| !matches!(expr, HirExpr::NoneLiteral)),
        ),
        HirStmt::Raise { .. } => FlowSummary::raise_stmt(),
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            let elif_bodies = elif_clauses.iter().map(|(_, body)| body.as_slice());
            let else_branch = else_body
                .as_deref()
                .map_or_else(FlowSummary::fallthrough, flow_summary);
            branch_summary(std::iter::once(then_body.as_slice()).chain(elif_bodies))
                .union(else_branch)
        }
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        }
        | HirStmt::AsyncFor {
            body, else_body, ..
        } => {
            let body_summary = flow_summary(body);
            let else_summary = else_body
                .as_deref()
                .map_or_else(FlowSummary::fallthrough, flow_summary);
            body_summary.union(else_summary)
        }
        HirStmt::Match { arms, .. } => branch_summary(arms.iter().map(|arm| arm.body.as_slice())),
        HirStmt::TryExcept { body, handlers, .. } => branch_summary(
            std::iter::once(body.as_slice())
                .chain(handlers.iter().map(|handler| handler.body.as_slice())),
        ),
        HirStmt::TryFinally { body, finalbody } => {
            flow_summary(body).sequence(flow_summary(finalbody))
        }
        HirStmt::With { items, body }
            if items
                .iter()
                .any(|item| matches!(item.kind, sifr_ir::HirWithItemKind::Python { .. })) =>
        {
            python_context_flow_summary(body)
        }
        HirStmt::AsyncWith {
            kind: sifr_ir::HirAsyncWithKind::Python { .. },
            body,
            ..
        } => python_context_flow_summary(body),
        HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => flow_summary(body),
        _ => FlowSummary::fallthrough(),
    }
}

fn python_context_flow_summary(body: &[HirStmt]) -> FlowSummary {
    let mut summary = flow_summary(body);
    if summary.has_raise {
        summary.falls_through = true;
    }
    summary
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
        let func = crate::stmt_support_emitter::canonical_plain_call_name_for_ir(func);
        func.rsplit('.').next().unwrap_or(func)
    }

    fn expression_root_name(expr: &HirExpr) -> Option<&str> {
        match expr {
            HirExpr::Name { name, .. } => Some(name),
            HirExpr::FieldAccess { object, .. } | HirExpr::Index { object, .. } => {
                expression_root_name(object)
            }
            _ => None,
        }
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
            let HirStmt::NestedFunction { func, .. } = stmt else {
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
        HirStmt::NestedFunction { func, .. } => {
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
                sifr_ir::HirAsyncWithKind::UserDefined {
                    context: HirExpr::Name { name, .. },
                    ..
                }
                | sifr_ir::HirAsyncWithKind::Python {
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
            args,
            receiver_convention,
            ..
        } => {
            let root_name = expression_root_name(object).map(str::to_string);
            if *receiver_convention == Some(ReceiverConvention::MutableBorrow) {
                if let Some(name) = root_name {
                    mutated.borrow_mut().insert(name);
                }
            }
            if let (Some(signatures), Type::Class { name, .. }) =
                (func_signatures, object.ty().resolve_alias())
            {
                let signature_name = format!("{name}::{method}");
                if let Some((params, _)) = signatures.get(&signature_name) {
                    for (arg, (_, convention)) in args.iter().zip(params) {
                        if convention.is_mut_borrow() {
                            if let Some(name) = expression_root_name(arg) {
                                mutated.borrow_mut().insert(name.to_string());
                            }
                        }
                    }
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
                if let Some(name) = args.first().and_then(expression_root_name) {
                    mutated.borrow_mut().insert(name.to_string());
                }
            }
        }
        HirExpr::IteratorCall { op, args, .. } => {
            if *op == HirIteratorOp::Next {
                if let Some(name) = args.first().and_then(expression_root_name) {
                    mutated.borrow_mut().insert(name.to_string());
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
        if let HirExpr::Name { name, ty, .. } = expr {
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
        if let HirExpr::Name { name, ty, .. } = node {
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
                    if let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding {
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
        HirStmt::NestedFunction { func, .. } => {
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
    pub needs_mul: bool,
    pub needs_div: bool,
    pub needs_rem: bool,
    pub needs_neg: bool,
    pub needs_partial_eq: bool,
    pub needs_partial_ord: bool,
}

fn type_mentions_type_var(ty: &Type, type_param_name: &str) -> bool {
    match ty.resolve_alias() {
        Type::TypeVar(name) => name == type_param_name,
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Newtype { inner, .. } => type_mentions_type_var(inner, type_param_name),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Coroutine(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            type_mentions_type_var(left, type_param_name)
                || type_mentions_type_var(right, type_param_name)
        }
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => items
            .iter()
            .any(|item| type_mentions_type_var(item, type_param_name)),
        Type::Callable(params, _, result) | Type::AsyncCallable(params, _, result) => {
            params
                .iter()
                .any(|param| type_mentions_type_var(param, type_param_name))
                || type_mentions_type_var(result, type_param_name)
        }
        Type::Function(function) | Type::AsyncFunction(function) => {
            function
                .params
                .iter()
                .any(|(_, param, _)| type_mentions_type_var(param, type_param_name))
                || type_mentions_type_var(&function.return_type, type_param_name)
        }
        Type::Class {
            fields, methods, ..
        } => {
            fields
                .iter()
                .any(|(_, field)| type_mentions_type_var(field, type_param_name))
                || methods.iter().any(|(_, method)| {
                    method
                        .params
                        .iter()
                        .any(|(_, param, _)| type_mentions_type_var(param, type_param_name))
                        || type_mentions_type_var(&method.return_type, type_param_name)
                })
        }
        _ => false,
    }
}

pub(crate) fn collect_typevar_operator_requirements(
    stmts: &[HirStmt],
    type_param_name: &str,
) -> TypeVarOpRequirements {
    let mut requirements = TypeVarOpRequirements::default();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| match expr {
        HirExpr::BinOp {
            left,
            op,
            right,
            ty,
        } => {
            let left_is_tp = type_mentions_type_var(left.ty(), type_param_name);
            let right_is_tp = type_mentions_type_var(right.ty(), type_param_name);
            let result_is_tp = type_mentions_type_var(ty, type_param_name);
            if left_is_tp || right_is_tp || result_is_tp {
                match op.as_str() {
                    "+" => requirements.needs_add = true,
                    "-" => requirements.needs_sub = true,
                    "*" => requirements.needs_mul = true,
                    "/" | "//" => requirements.needs_div = true,
                    "%" => requirements.needs_rem = true,
                    _ => {}
                }
            }
        }
        HirExpr::UnaryOp { op, operand, ty }
            if op == "-"
                && (type_mentions_type_var(operand.ty(), type_param_name)
                    || type_mentions_type_var(ty, type_param_name)) =>
        {
            requirements.needs_neg = true;
        }
        HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } if type_mentions_type_var(left.ty(), type_param_name)
            || comparators
                .iter()
                .any(|right| type_mentions_type_var(right.ty(), type_param_name)) =>
        {
            if ops.iter().any(|op| matches!(op.as_str(), "==" | "!=")) {
                requirements.needs_partial_eq = true;
            }
            if ops
                .iter()
                .any(|op| matches!(op.as_str(), "<" | "<=" | ">" | ">="))
            {
                requirements.needs_partial_ord = true;
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

pub(crate) fn collect_try_error_carriers(stmts: &[HirStmt]) -> Vec<Type> {
    let mut carriers = Vec::new();
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::TryExcept {
            body_error_types,
            handlers,
            ..
        } = stmt
        {
            if let Some(carrier) =
                crate::try_error_carrier::try_error_carrier(body_error_types, handlers)
            {
                carriers.push(carrier);
            }
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        stmts,
        TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut on_stmt,
        &mut on_expr,
    );
    carriers
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
