use super::{
    collect_locally_defined_vars, collect_referenced_vars_with_types, traversal, HashMap, HashSet,
    HirExpr, HirFunction, HirStmt, RustType, TraversalConfig, Type,
};
pub(super) fn hir_function_returns_sifr_int(
    func: &HirFunction,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
) -> bool {
    let local_int_bindings = func
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            HirStmt::Let { name, ty, .. }
                if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int) =>
            {
                Some(name.clone())
            }
            _ => None,
        })
        .collect::<HashSet<_>>();
    let shadowed_module_bindings = collect_function_local_shadow_names(func);
    let mut function_sifr_int_returns = function_sifr_int_returns.clone();
    let mut forced;
    loop {
        forced = collect_sifr_int_forced_locals(
            &func.body,
            &local_int_bindings,
            &shadowed_module_bindings,
            module_sifr_int_bindings,
            &function_sifr_int_returns,
        );
        let before = function_sifr_int_returns.len();
        function_sifr_int_returns.extend(collect_nested_sifr_int_function_returns(
            &func.body,
            module_sifr_int_bindings,
            &function_sifr_int_returns,
            &forced,
            &shadowed_module_bindings,
        ));
        if function_sifr_int_returns.len() == before {
            break;
        }
    }

    let mut returns_sifr_int = false;
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::Return { value: Some(value) } = stmt {
            returns_sifr_int |= hir_expr_needs_sifr_int_storage(
                value,
                &forced,
                &shadowed_module_bindings,
                module_sifr_int_bindings,
                &function_sifr_int_returns,
            );
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        &func.body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    returns_sifr_int
}

pub(super) fn function_returns_result_sifr_int(
    func: &HirFunction,
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    result_function_params: &HashMap<String, HashSet<usize>>,
    initial_result_bindings: HashSet<String>,
) -> bool {
    if !is_result_int_type(&func.return_type) {
        return false;
    }

    let mut result_function_returns = result_function_returns.clone();
    result_function_returns.extend(collect_nested_sifr_int_result_function_returns(
        &func.body,
        &result_function_returns,
        result_method_returns,
        result_function_params,
    ));
    let local_result_bindings = collect_sifr_int_result_local_bindings_with_initial(
        &func.body,
        &result_function_returns,
        result_method_returns,
        initial_result_bindings,
    );
    let mut returns_sifr_int_result = false;
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::Return { value: Some(value) } = stmt {
            returns_sifr_int_result |= hir_expr_returns_sifr_int_result(
                value,
                &result_function_returns,
                result_method_returns,
                &local_result_bindings,
            );
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        &func.body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    returns_sifr_int_result
}

pub(super) fn collect_nested_sifr_int_result_function_returns(
    body: &[HirStmt],
    inherited_result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    result_function_params: &HashMap<String, HashSet<usize>>,
) -> HashSet<String> {
    let mut nested_returns = HashSet::new();
    loop {
        let before = nested_returns.len();
        let mut available_result_returns = inherited_result_function_returns.clone();
        available_result_returns.extend(nested_returns.iter().cloned());
        let mut on_stmt = |stmt: &HirStmt| {
            if let HirStmt::NestedFunction { func, .. } = stmt {
                if function_returns_result_sifr_int(
                    func,
                    &available_result_returns,
                    result_method_returns,
                    result_function_params,
                    collect_sifr_int_result_function_param_names(func, result_function_params),
                ) {
                    nested_returns.insert(func.name.clone());
                }
            }
        };
        let mut on_expr = |_expr: &HirExpr| {};
        traversal::walk_stmts(
            body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        if nested_returns.len() == before {
            break;
        }
    }
    nested_returns
}

pub(super) fn collect_sifr_int_result_local_bindings(
    body: &[HirStmt],
    result_function_returns: &HashSet<String>,
) -> HashSet<String> {
    collect_sifr_int_result_local_bindings_with_initial(
        body,
        result_function_returns,
        &HashSet::new(),
        HashSet::new(),
    )
}

pub(super) fn collect_sifr_int_result_local_bindings_with_initial(
    body: &[HirStmt],
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    mut result_bindings: HashSet<String>,
) -> HashSet<String> {
    let mut on_stmt = |stmt: &HirStmt| match stmt {
        HirStmt::Let {
            name, ty, value, ..
        } if is_result_int_type(ty)
            && hir_expr_returns_sifr_int_result(
                value,
                result_function_returns,
                result_method_returns,
                &result_bindings,
            ) =>
        {
            result_bindings.insert(name.clone());
        }
        HirStmt::Assign { name, value }
            if result_bindings.contains(name)
                && !hir_expr_returns_sifr_int_result(
                    value,
                    result_function_returns,
                    result_method_returns,
                    &result_bindings,
                ) =>
        {
            result_bindings.remove(name);
        }
        _ => {}
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    result_bindings
}

pub(super) fn collect_sifr_int_result_function_param_names(
    func: &HirFunction,
    result_function_params: &HashMap<String, HashSet<usize>>,
) -> HashSet<String> {
    let Some(indexes) = result_function_params.get(&func.name) else {
        return HashSet::new();
    };
    func.params
        .iter()
        .enumerate()
        .filter(|(idx, _)| indexes.contains(idx))
        .map(|(_, param)| param.name.clone())
        .collect()
}

pub(super) fn collect_sifr_int_result_method_param_names(
    method: &HirFunction,
    method_key: &str,
    result_method_params: &HashMap<String, HashSet<usize>>,
) -> HashSet<String> {
    let Some(indexes) = result_method_params.get(method_key) else {
        return HashSet::new();
    };
    method
        .params
        .iter()
        .enumerate()
        .filter(|(idx, _)| indexes.contains(idx))
        .map(|(_, param)| param.name.clone())
        .collect()
}

pub(super) fn hir_expr_returns_sifr_int_result(
    expr: &HirExpr,
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    local_result_bindings: &HashSet<String>,
) -> bool {
    match expr {
        HirExpr::BinOp { op, ty, .. } => {
            matches!(op.as_str(), "//" | "%") && is_result_int_type(ty)
        }
        HirExpr::Call { func, .. } => result_function_returns.contains(func),
        HirExpr::MethodCall { object, method, .. } => {
            hir_expr_class_name(object).is_some_and(|class_name| {
                result_method_returns.contains(&result_method_key(&class_name, method))
            })
        }
        HirExpr::Name { name, .. } => local_result_bindings.contains(name),
        _ => false,
    }
}

pub(crate) fn is_result_int_type(ty: &Type) -> bool {
    let Type::Result(ok_ty, _) = crate::resolve_alias_type_for_plain_call(ty) else {
        return false;
    };
    matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Int | Type::LiteralInt(_)
    )
}

pub(crate) fn result_int_return_type_to_sifr_int(ty: &Type) -> RustType {
    let Type::Result(_, err_ty) = crate::resolve_alias_type_for_plain_call(ty) else {
        return RustType::Named(ty.rust_type());
    };
    RustType::Result(
        Box::new(RustType::Named("SifrInt".to_string())),
        Box::new(crate::sifr_type_to_rust_type(err_ty)),
    )
}

pub(crate) fn result_method_key(class_name: &str, method_name: &str) -> String {
    format!("{class_name}::{method_name}")
}

pub(super) fn hir_expr_class_name(expr: &HirExpr) -> Option<String> {
    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::Class { name, .. } => Some(name.clone()),
        _ => None,
    }
}

pub(super) fn hir_function_returns_sifr_int_with_extra_forced(
    func: &HirFunction,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
    extra_forced_locals: &HashSet<String>,
) -> bool {
    let extra_shadowed_module_bindings = HashSet::new();
    hir_function_returns_sifr_int_with_extra_forced_and_shadowed(
        func,
        module_sifr_int_bindings,
        function_sifr_int_returns,
        extra_forced_locals,
        &extra_shadowed_module_bindings,
    )
}

pub(super) fn hir_function_returns_sifr_int_with_extra_forced_and_shadowed(
    func: &HirFunction,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
    extra_forced_locals: &HashSet<String>,
    extra_shadowed_module_bindings: &HashSet<String>,
) -> bool {
    let mut shadowed_module_bindings = collect_function_local_shadow_names(func);
    shadowed_module_bindings.extend(extra_shadowed_module_bindings.iter().cloned());
    let forced = collect_function_sifr_int_forced_locals_with_extra_and_shadowed(
        func,
        module_sifr_int_bindings,
        function_sifr_int_returns,
        extra_forced_locals,
        extra_shadowed_module_bindings,
    );
    let mut function_sifr_int_returns = function_sifr_int_returns.clone();
    function_sifr_int_returns.extend(collect_nested_sifr_int_function_returns(
        &func.body,
        module_sifr_int_bindings,
        &function_sifr_int_returns,
        &forced,
        &shadowed_module_bindings,
    ));

    let mut returns_sifr_int = false;
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::Return { value: Some(value) } = stmt {
            returns_sifr_int |= hir_expr_needs_sifr_int_storage(
                value,
                &forced,
                &shadowed_module_bindings,
                module_sifr_int_bindings,
                &function_sifr_int_returns,
            );
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    traversal::walk_stmts(
        &func.body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    returns_sifr_int
}

pub(super) fn collect_function_sifr_int_forced_locals_with_extra(
    func: &HirFunction,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
    extra_forced_locals: &HashSet<String>,
) -> HashSet<String> {
    let extra_shadowed_module_bindings = HashSet::new();
    collect_function_sifr_int_forced_locals_with_extra_and_shadowed(
        func,
        module_sifr_int_bindings,
        function_sifr_int_returns,
        extra_forced_locals,
        &extra_shadowed_module_bindings,
    )
}

pub(super) fn collect_function_sifr_int_forced_locals_with_extra_and_shadowed(
    func: &HirFunction,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
    extra_forced_locals: &HashSet<String>,
    extra_shadowed_module_bindings: &HashSet<String>,
) -> HashSet<String> {
    let mut function_sifr_int_returns = function_sifr_int_returns.clone();
    let local_int_bindings = func
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            HirStmt::Let { name, ty, .. }
                if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int) =>
            {
                Some(name.clone())
            }
            _ => None,
        })
        .collect::<HashSet<_>>();
    let mut shadowed_module_bindings = collect_function_local_shadow_names(func);
    shadowed_module_bindings.extend(extra_shadowed_module_bindings.iter().cloned());
    let mut forced;
    loop {
        forced = collect_sifr_int_forced_locals_with_seed(
            &func.body,
            &local_int_bindings,
            &shadowed_module_bindings,
            module_sifr_int_bindings,
            &function_sifr_int_returns,
            extra_forced_locals,
        );
        let before = function_sifr_int_returns.len();
        function_sifr_int_returns.extend(collect_nested_sifr_int_function_returns(
            &func.body,
            module_sifr_int_bindings,
            &function_sifr_int_returns,
            &forced,
            &shadowed_module_bindings,
        ));
        if function_sifr_int_returns.len() == before {
            break;
        }
    }
    forced
}

pub(super) fn collect_sifr_int_function_param_names(
    func: &HirFunction,
    function_params: &HashMap<String, HashSet<usize>>,
) -> HashSet<String> {
    let Some(indexes) = function_params.get(&func.name) else {
        return HashSet::new();
    };
    func.params
        .iter()
        .enumerate()
        .filter(|(idx, _)| indexes.contains(idx))
        .map(|(_, param)| param.name.clone())
        .collect()
}

pub(super) fn collect_function_local_shadow_names(func: &HirFunction) -> HashSet<String> {
    let mut shadowed = collect_locally_defined_vars(&func.body);
    shadowed.extend(func.params.iter().map(|param| param.name.clone()));
    shadowed
}

pub(super) fn collect_sifr_int_call_arg_function_params(
    body: &[HirStmt],
    module_function_params: &HashMap<String, Vec<Type>>,
    forced_locals: &HashSet<String>,
    shadowed_module_bindings: &HashSet<String>,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
) -> HashMap<String, HashSet<usize>> {
    let mut discovered: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        let HirExpr::Call { func, args, .. } = expr else {
            return;
        };
        let Some(params) = module_function_params.get(func) else {
            return;
        };
        for (idx, arg) in args.iter().enumerate() {
            let Some(param_ty) = params.get(idx) else {
                continue;
            };
            if matches!(
                crate::resolve_alias_type_for_plain_call(param_ty),
                Type::Int
            ) && hir_expr_needs_sifr_int_storage(
                arg,
                forced_locals,
                shadowed_module_bindings,
                module_sifr_int_bindings,
                function_sifr_int_returns,
            ) {
                discovered.entry(func.clone()).or_default().insert(idx);
            }
        }
    };
    traversal::walk_stmts(
        body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    discovered
}

pub(super) fn collect_sifr_int_result_call_arg_function_params(
    caller: &HirFunction,
    module_function_params: &HashMap<String, Vec<Type>>,
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    result_function_params: &HashMap<String, HashSet<usize>>,
) -> HashMap<String, HashSet<usize>> {
    let result_param_bindings =
        collect_sifr_int_result_function_param_names(caller, result_function_params);
    collect_sifr_int_result_call_arg_function_params_with_initial(
        &caller.body,
        module_function_params,
        result_function_returns,
        result_method_returns,
        result_param_bindings,
    )
}

pub(super) fn collect_sifr_int_result_call_arg_function_params_with_initial(
    body: &[HirStmt],
    module_function_params: &HashMap<String, Vec<Type>>,
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    result_param_bindings: HashSet<String>,
) -> HashMap<String, HashSet<usize>> {
    let local_result_bindings = collect_sifr_int_result_local_bindings_with_initial(
        body,
        result_function_returns,
        result_method_returns,
        result_param_bindings,
    );
    let mut discovered: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        let HirExpr::Call { func, args, .. } = expr else {
            return;
        };
        let Some(params) = module_function_params.get(func) else {
            return;
        };
        for (idx, arg) in args.iter().enumerate() {
            let Some(param_ty) = params.get(idx) else {
                continue;
            };
            if is_result_int_type(param_ty)
                && hir_expr_returns_sifr_int_result(
                    arg,
                    result_function_returns,
                    result_method_returns,
                    &local_result_bindings,
                )
            {
                discovered.entry(func.clone()).or_default().insert(idx);
            }
        }
    };
    traversal::walk_stmts(
        body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    discovered
}

pub(super) fn collect_sifr_int_result_call_arg_method_params(
    body: &[HirStmt],
    module_method_params: &HashMap<String, Vec<Type>>,
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    result_param_bindings: HashSet<String>,
) -> HashMap<String, HashSet<usize>> {
    let local_result_bindings = collect_sifr_int_result_local_bindings_with_initial(
        body,
        result_function_returns,
        result_method_returns,
        result_param_bindings,
    );
    let mut discovered: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } = expr
        else {
            return;
        };
        let Some(class_name) = hir_expr_class_name(object) else {
            return;
        };
        let method_key = result_method_key(&class_name, method);
        let Some(params) = module_method_params.get(&method_key) else {
            return;
        };
        for (idx, arg) in args.iter().enumerate() {
            let Some(param_ty) = params.get(idx) else {
                continue;
            };
            if is_result_int_type(param_ty)
                && hir_expr_returns_sifr_int_result(
                    arg,
                    result_function_returns,
                    result_method_returns,
                    &local_result_bindings,
                )
            {
                discovered
                    .entry(method_key.clone())
                    .or_default()
                    .insert(idx);
            }
        }
    };
    traversal::walk_stmts(
        body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    discovered
}

pub(super) fn collect_nested_sifr_int_function_returns(
    body: &[HirStmt],
    module_sifr_int_bindings: &HashSet<String>,
    outer_function_returns: &HashSet<String>,
    outer_forced_locals: &HashSet<String>,
    outer_shadowed_module_bindings: &HashSet<String>,
) -> HashSet<String> {
    let nested_functions = body
        .iter()
        .filter_map(|stmt| match stmt {
            HirStmt::NestedFunction { func, .. } => Some(func),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut function_returns = outer_function_returns.clone();
    loop {
        let before = function_returns.len();
        let discovered = nested_functions
            .iter()
            .filter(|func| {
                let captured_forced =
                    collect_sifr_int_captured_forced_locals(func, outer_forced_locals);
                let captured_shadowed = collect_sifr_int_captured_shadowed_module_bindings(
                    func,
                    outer_shadowed_module_bindings,
                );
                matches!(
                    crate::resolve_alias_type_for_plain_call(&func.return_type),
                    Type::Int
                ) && hir_function_returns_sifr_int_with_extra_forced_and_shadowed(
                    func,
                    module_sifr_int_bindings,
                    &function_returns,
                    &captured_forced,
                    &captured_shadowed,
                )
            })
            .map(|func| func.name.clone())
            .collect::<Vec<_>>();
        function_returns.extend(discovered);
        if function_returns.len() == before {
            break;
        }
    }
    function_returns
        .difference(outer_function_returns)
        .cloned()
        .collect()
}

pub(super) fn collect_sifr_int_captured_forced_locals(
    func: &HirFunction,
    outer_forced_locals: &HashSet<String>,
) -> HashSet<String> {
    collect_captured_outer_names_transitively(func, outer_forced_locals)
}

pub(super) fn collect_sifr_int_captured_shadowed_module_bindings(
    func: &HirFunction,
    outer_shadowed_module_bindings: &HashSet<String>,
) -> HashSet<String> {
    collect_captured_outer_names_transitively(func, outer_shadowed_module_bindings)
}

pub(super) fn collect_captured_outer_names(
    func: &HirFunction,
    outer_names: &HashSet<String>,
) -> HashSet<String> {
    if outer_names.is_empty() {
        return HashSet::new();
    }
    let param_names = func
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    let locally_defined = collect_locally_defined_vars(&func.body);
    collect_referenced_vars_with_types(&func.body)
        .into_iter()
        .filter(|(name, _)| {
            !param_names.contains(name)
                && !locally_defined.contains(name)
                && outer_names.contains(name)
        })
        .map(|(name, _)| name)
        .collect()
}

pub(super) fn nested_function_mutates_capture(
    func: &HirFunction,
    nested_mutated_vars: &HashSet<String>,
) -> bool {
    let param_names = func
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    let locally_defined = collect_locally_defined_vars(&func.body);
    nested_mutated_vars
        .iter()
        .any(|name| !param_names.contains(name) && !locally_defined.contains(name))
}

pub(super) fn collect_captured_outer_names_transitively(
    func: &HirFunction,
    outer_names: &HashSet<String>,
) -> HashSet<String> {
    if outer_names.is_empty() {
        return HashSet::new();
    }
    let param_names = func
        .params
        .iter()
        .map(|param| param.name.clone())
        .collect::<HashSet<_>>();
    let locally_defined = collect_locally_defined_vars(&func.body);
    let shadowed_in_func = param_names
        .union(&locally_defined)
        .cloned()
        .collect::<HashSet<_>>();
    let visible_outer_names = outer_names
        .difference(&shadowed_in_func)
        .cloned()
        .collect::<HashSet<_>>();
    if visible_outer_names.is_empty() {
        return HashSet::new();
    }

    let mut captured = collect_captured_outer_names(func, &visible_outer_names);
    for nested in func.body.iter().filter_map(|stmt| match stmt {
        HirStmt::NestedFunction { func, .. } => Some(func),
        _ => None,
    }) {
        captured.extend(collect_captured_outer_names_transitively(
            nested,
            &visible_outer_names,
        ));
    }
    captured
}

pub(super) fn collect_sifr_int_forced_locals(
    body: &[HirStmt],
    local_int_bindings: &HashSet<String>,
    shadowed_module_bindings: &HashSet<String>,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
) -> HashSet<String> {
    collect_sifr_int_forced_locals_with_seed(
        body,
        local_int_bindings,
        shadowed_module_bindings,
        module_sifr_int_bindings,
        function_sifr_int_returns,
        &HashSet::new(),
    )
}

pub(super) fn collect_sifr_int_forced_locals_with_seed(
    body: &[HirStmt],
    local_int_bindings: &HashSet<String>,
    shadowed_module_bindings: &HashSet<String>,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
    seed: &HashSet<String>,
) -> HashSet<String> {
    let mut forced = seed.clone();
    if local_int_bindings.is_empty() {
        return forced;
    }
    loop {
        let before = forced.len();
        let mut on_stmt = |stmt: &HirStmt| match stmt {
            HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value }
                if local_int_bindings.contains(name)
                    && hir_expr_needs_sifr_int_storage(
                        value,
                        &forced,
                        shadowed_module_bindings,
                        module_sifr_int_bindings,
                        function_sifr_int_returns,
                    ) =>
            {
                forced.insert(name.clone());
            }
            HirStmt::AugAssign { name, op, value }
                if local_int_bindings.contains(name)
                    && is_sifr_int_augassign_op(op)
                    && (forced.contains(name)
                        || hir_expr_needs_sifr_int_storage(
                            value,
                            &forced,
                            shadowed_module_bindings,
                            module_sifr_int_bindings,
                            function_sifr_int_returns,
                        )) =>
            {
                forced.insert(name.clone());
            }
            _ => {}
        };
        let mut on_expr = |_expr: &HirExpr| {};
        traversal::walk_stmts(
            body,
            TraversalConfig::LOCAL_SCOPE_ONLY,
            &mut on_stmt,
            &mut on_expr,
        );
        if forced.len() == before {
            break;
        }
    }
    forced
}

pub(super) fn hir_expr_needs_sifr_int_storage(
    expr: &HirExpr,
    forced_locals: &HashSet<String>,
    shadowed_module_bindings: &HashSet<String>,
    module_sifr_int_bindings: &HashSet<String>,
    function_sifr_int_returns: &HashSet<String>,
) -> bool {
    match expr {
        HirExpr::LargeIntLiteral(_) => true,
        HirExpr::Name { name, .. } => {
            forced_locals.contains(name)
                || (module_sifr_int_bindings.contains(name)
                    && !shadowed_module_bindings.contains(name))
        }
        HirExpr::Call { func, .. } => function_sifr_int_returns.contains(func),
        HirExpr::BinOp {
            left,
            op,
            right,
            ty,
            ..
        } if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int)
            && matches!(op.as_str(), "+" | "-" | "*") =>
        {
            hir_expr_needs_sifr_int_storage(
                left,
                forced_locals,
                shadowed_module_bindings,
                module_sifr_int_bindings,
                function_sifr_int_returns,
            ) || hir_expr_needs_sifr_int_storage(
                right,
                forced_locals,
                shadowed_module_bindings,
                module_sifr_int_bindings,
                function_sifr_int_returns,
            )
        }
        HirExpr::UnaryOp { op, operand, ty }
            if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int)
                && matches!(op.as_str(), "+" | "-") =>
        {
            hir_expr_needs_sifr_int_storage(
                operand,
                forced_locals,
                shadowed_module_bindings,
                module_sifr_int_bindings,
                function_sifr_int_returns,
            )
        }
        _ => false,
    }
}

pub(super) fn is_sifr_int_augassign_op(op: &str) -> bool {
    matches!(op, "+=" | "-=" | "*=")
}
