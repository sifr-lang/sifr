use crate::ModuleFuncSignatures;
use crate::hir_analysis::traversal::{self, TraversalConfig, TraversalControl};
#[cfg(test)]
pub(crate) use sifr_ir::{
    HirControlFlowEffect as ControlFlowEffect, reachable_top_level_stmt_indices,
    unreachable_top_level_stmt_indices,
};
use sifr_ir::{HirExpr, HirIteratorOp, HirPattern, HirStmt};
pub(crate) use sifr_ir::{
    block_control_flow_effect, body_contains_return, try_body_has_value_return,
};
use sifr_type_system::{ParamConvention, ReceiverConvention, Type};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

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
    stmts_reference_var_with_config(stmts, var_name, TraversalConfig::LOCAL_SCOPE_ONLY)
}

pub(crate) fn stmts_reference_var_including_nested_functions(
    stmts: &[HirStmt],
    var_name: &str,
) -> bool {
    stmts_reference_var_with_config(stmts, var_name, TraversalConfig::INCLUDE_NESTED_FUNCTIONS)
}

fn stmts_reference_var_with_config(
    stmts: &[HirStmt],
    var_name: &str,
    config: TraversalConfig,
) -> bool {
    let mut on_stmt = |stmt: &HirStmt| {
        if config.descend_nested_functions {
            if let HirStmt::NestedFunction { func, .. } = stmt {
                let parameter_shadows_var = func.params.iter().any(|param| param.name == var_name);
                if !parameter_shadows_var
                    && stmts_reference_var_with_config(&func.body, var_name, config)
                {
                    return TraversalControl::Stop;
                }
            }
        }
        let target_references_var = match stmt {
            HirStmt::Assign { name, .. } | HirStmt::AugAssign { name, .. } => name == var_name,
            HirStmt::FieldAssign { object, .. }
            | HirStmt::NestedFieldAssign { object, .. }
            | HirStmt::SubscriptAssign { object, .. }
            | HirStmt::NestedSubscriptAssign { object, .. }
            | HirStmt::AttributeNestedSubscriptAssign { object, .. }
            | HirStmt::SubscriptAugAssign { object, .. }
            | HirStmt::AttributeAugAssign { object, .. }
            | HirStmt::AttributeSubscriptAssign { object, .. } => object == var_name,
            HirStmt::TupleUnpack { targets, .. } => {
                targets.iter().any(|target| match &target.binding {
                    sifr_ir::HirTupleTargetBinding::Name(name) => {
                        target.rebind_existing && name == var_name
                    }
                    sifr_ir::HirTupleTargetBinding::Field { object, .. } => object == var_name,
                })
            }
            HirStmt::StarUnpack {
                before,
                star,
                after,
                ..
            } => before
                .iter()
                .chain(std::iter::once(star))
                .chain(after)
                .any(|target| {
                    matches!(
                        &target.binding,
                        sifr_ir::HirTupleTargetBinding::Name(name)
                            if target.rebind_existing && name == var_name
                    )
                }),
            _ => false,
        };
        if target_references_var {
            TraversalControl::Stop
        } else {
            TraversalControl::Continue
        }
    };
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
            let captured_mutated = collect_mutated_vars(&func.body, func_signatures)
                .into_iter()
                .filter(|name| !param_names.contains(name) && !locally_defined.contains(name))
                .collect::<Vec<_>>();
            mutated.borrow_mut().extend(captured_mutated);
        }
        HirStmt::TupleUnpack { targets, .. } => {
            mutated.borrow_mut().extend(targets.iter().filter_map(
                |target| match &target.binding {
                    sifr_ir::HirTupleTargetBinding::Name(name) if target.rebind_existing => {
                        Some(name.clone())
                    }
                    _ => None,
                },
            ));
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            ..
        } => {
            mutated.borrow_mut().extend(
                before
                    .iter()
                    .chain(std::iter::once(star))
                    .chain(after)
                    .filter_map(|target| match &target.binding {
                        sifr_ir::HirTupleTargetBinding::Name(name) if target.rebind_existing => {
                            Some(name.clone())
                        }
                        _ => None,
                    }),
            );
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
        HirExpr::Call { func, args, .. } | HirExpr::GenericCall { func, args, .. } => {
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
        HirExpr::IteratorCall { op, args, .. } if *op == HirIteratorOp::Next => {
            if let Some(name) = args.first().and_then(expression_root_name) {
                mutated.borrow_mut().insert(name.to_string());
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
    // Handler bindings exist only in their handler. Exclude those exact uses,
    // not all uses of the spelling: an enclosing capture can share the name.
    let mut handler_local_uses = HashSet::new();
    traversal::walk_stmts(
        stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut |stmt| {
            if let HirStmt::TryExcept { handlers, .. } = stmt {
                for handler in handlers {
                    if let Some(binding) = &handler.name {
                        traversal::walk_stmts(
                            &handler.body,
                            TraversalConfig::LOCAL_SCOPE_ONLY,
                            &mut |_| {},
                            &mut |expr| {
                                if matches!(expr, HirExpr::Name { name, .. } if name == binding) {
                                    handler_local_uses.insert(crate::body_analysis::expr_key(expr));
                                }
                            },
                        );
                    }
                }
            }
        },
        &mut |_| {},
    );
    let mut refs: HashMap<String, Type> = HashMap::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::Name { name, ty, .. } = expr
            && !handler_local_uses.contains(&crate::body_analysis::expr_key(expr))
        {
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
            for target in before.iter().chain(std::iter::once(star)).chain(after) {
                if target.rebind_existing {
                    continue;
                }
                if let sifr_ir::HirTupleTargetBinding::Name(name) = &target.binding {
                    defined.insert(name.clone());
                }
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
    pub needs_display: bool,
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
    let mut on_expr = |expr: &HirExpr| {
        match expr {
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
        HirExpr::Call { func, args, .. }
            if matches!(func.as_str(), "print" | "str")
                && args
                    .iter()
                    .any(|arg| type_mentions_type_var(arg.ty(), type_param_name)) =>
        {
            requirements.needs_display = true;
        }
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } if method == "contains"
            && (type_mentions_type_var(object.ty(), type_param_name)
                || args
                    .iter()
                    .any(|arg| type_mentions_type_var(arg.ty(), type_param_name))) =>
        {
            requirements.needs_partial_eq = true;
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } if type_mentions_type_var(element.ty(), type_param_name)
            || type_mentions_type_var(collection.ty(), type_param_name) =>
        {
            requirements.needs_partial_eq = true;
        }
        HirExpr::FString { parts, .. }
            if parts.iter().any(|part| {
                matches!(part, sifr_ir::HirFStringPart::Expr(value) if type_mentions_type_var(value.ty(), type_param_name))
            }) =>
        {
            requirements.needs_display = true;
        }
        _ => {}
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
