use super::{
    collect_function_local_shadow_names, collect_function_sifr_int_forced_locals_with_extra,
    collect_locally_defined_vars, collect_mutated_vars_with_sigs,
    collect_nested_sifr_int_function_returns, collect_sifr_int_call_arg_function_params,
    collect_sifr_int_captured_forced_locals, collect_sifr_int_captured_shadowed_module_bindings,
    collect_sifr_int_forced_locals, collect_sifr_int_function_param_names,
    collect_sifr_int_result_call_arg_function_params,
    collect_sifr_int_result_call_arg_function_params_with_initial,
    collect_sifr_int_result_call_arg_method_params, collect_sifr_int_result_function_param_names,
    collect_sifr_int_result_method_param_names, function_returns_result_sifr_int,
    hir_function_returns_sifr_int_with_extra_forced,
    hir_function_returns_sifr_int_with_extra_forced_and_shadowed, is_result_int_type, make_union,
    nested_function_mutates_capture, result_int_return_type_to_sifr_int, result_method_key,
    traversal, HashMap, HashSet, HirExpr, HirFunction, HirModule, HirParam, HirStmt,
    NestedFnCapture, OwnershipKind, ParamConvention, RustEmitter, RustExpr, RustParam, RustStmt,
    RustType, RustTypeParam, TraversalConfig, Type,
};
impl RustEmitter {
    pub(crate) fn effective_nested_param_convention(
        param: &HirParam,
        mutated_vars: &std::collections::HashSet<String>,
    ) -> ParamConvention {
        if !mutated_vars.contains(&param.name) {
            return param.convention;
        }
        if param.ty.ownership() == OwnershipKind::Copy {
            return if param.convention.is_owned() {
                ParamConvention::own_mut()
            } else {
                param.convention
            };
        }
        if param.convention.is_borrowed() {
            ParamConvention::mut_borrow()
        } else {
            ParamConvention::own_mut()
        }
    }

    pub(crate) fn register_function_scope_binding(
        &mut self,
        name: &str,
        ty: &Type,
        convention: ParamConvention,
    ) {
        if convention.is_shared_borrow() && ty.ownership() != sifr_type_system::OwnershipKind::Copy
        {
            self.borrowed_params.insert(name.to_string());
        }
        if convention.is_mut_borrow() && ty.ownership() != sifr_type_system::OwnershipKind::Copy {
            self.mut_borrowed_params.insert(name.to_string());
        }
        if let Type::Callable(ref param_types, ref conventions, _) = ty {
            let conv_list: Vec<(Type, ParamConvention)> = param_types
                .iter()
                .zip(conventions.iter())
                .map(|(ty, convention)| (ty.clone(), *convention))
                .collect();
            self.callable_var_conventions
                .insert(name.to_string(), conv_list);
        }
        self.local_binding_types
            .insert(name.to_string(), ty.clone());
    }

    pub(crate) fn register_function_scope_params(&mut self, func_name: &str, params: &[HirParam]) {
        for (param_idx, param) in params.iter().enumerate() {
            self.register_function_scope_binding(&param.name, &param.ty, param.convention);
            if self.function_param_lowers_to_sifr_int(func_name, param_idx) {
                self.sifr_int_local_bindings
                    .borrow_mut()
                    .insert(param.name.clone());
            }
            if self.function_param_lowers_to_sifr_int_result(func_name, param_idx) {
                self.sifr_int_result_local_bindings
                    .borrow_mut()
                    .insert(param.name.clone());
            }
        }
    }

    pub(crate) fn register_local_body_binding_types(&mut self, body: &[HirStmt]) {
        let mut bindings = HashMap::new();
        let mut widened_bindings = HashSet::new();
        let mut on_stmt = |stmt: &HirStmt| match stmt {
            HirStmt::Let { name, ty, .. } => {
                bindings.entry(name.clone()).or_insert_with(|| ty.clone());
            }
            HirStmt::Assign { name, value }
                if matches!(value, HirExpr::NoneLiteral) || matches!(value.ty(), Type::None) =>
            {
                if let Some(existing) = bindings.get(name).cloned() {
                    if !crate::helpers::is_option_type(&existing) {
                        bindings.insert(name.clone(), make_union(vec![existing, Type::None]));
                        widened_bindings.insert(name.clone());
                    }
                }
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
        for (name, ty) in bindings {
            self.local_binding_types.entry(name).or_insert(ty);
        }
        self.none_widened_local_bindings.extend(widened_bindings);
        self.register_sifr_int_forced_local_bindings(body);
    }

    pub(crate) fn register_sifr_int_forced_local_bindings(&self, body: &[HirStmt]) {
        let local_int_bindings = self
            .local_binding_types
            .iter()
            .filter(|(_, ty)| matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int))
            .map(|(name, _)| name.clone())
            .collect::<HashSet<_>>();
        if local_int_bindings.is_empty() {
            return;
        }

        let module_sifr_int_bindings = self.module_sifr_int_bindings();
        let function_sifr_int_returns = self.function_sifr_int_returns_for_body(body);
        let shadowed_module_bindings = self.local_binding_types.keys().cloned().collect();

        let mut forced = self.sifr_int_forced_local_bindings.borrow().clone();
        forced.extend(collect_sifr_int_forced_locals(
            body,
            &local_int_bindings,
            &shadowed_module_bindings,
            &module_sifr_int_bindings,
            &function_sifr_int_returns,
        ));
        *self.sifr_int_forced_local_bindings.borrow_mut() = forced;
    }

    pub(crate) fn register_sifr_int_function_returns(&self, module: &HirModule) {
        let module_sifr_int_bindings = self.module_sifr_int_bindings();
        let mut function_returns = HashSet::new();
        let mut result_function_returns = HashSet::new();
        let mut result_method_returns = HashSet::new();
        let mut function_params: HashMap<String, HashSet<usize>> = HashMap::new();
        let mut result_function_params: HashMap<String, HashSet<usize>> = HashMap::new();
        let mut result_method_params: HashMap<String, HashSet<usize>> = HashMap::new();
        let module_function_params = module
            .functions
            .iter()
            .map(|func| {
                (
                    func.name.clone(),
                    func.params
                        .iter()
                        .map(|param| param.ty.clone())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<HashMap<_, _>>();
        let module_method_params = module
            .classes
            .iter()
            .flat_map(|class| {
                class.methods.iter().map(|method| {
                    (
                        result_method_key(&class.name, &method.name),
                        method
                            .params
                            .iter()
                            .map(|param| param.ty.clone())
                            .collect::<Vec<_>>(),
                    )
                })
            })
            .collect::<HashMap<_, _>>();
        loop {
            let before = function_returns.len();
            let before_result_returns = result_function_returns.len();
            let before_result_methods = result_method_returns.len();
            let before_params = function_params.values().map(HashSet::len).sum::<usize>();
            let before_result_params = result_function_params
                .values()
                .map(HashSet::len)
                .sum::<usize>();
            let before_result_method_params = result_method_params
                .values()
                .map(HashSet::len)
                .sum::<usize>();
            let discovered = module
                .functions
                .iter()
                .filter(|func| {
                    let extra_forced_params =
                        collect_sifr_int_function_param_names(func, &function_params);
                    matches!(
                        crate::resolve_alias_type_for_plain_call(&func.return_type),
                        Type::Int
                    ) && hir_function_returns_sifr_int_with_extra_forced(
                        func,
                        &module_sifr_int_bindings,
                        &function_returns,
                        &extra_forced_params,
                    )
                })
                .map(|func| func.name.clone())
                .collect::<Vec<_>>();
            function_returns.extend(discovered);
            let discovered_result_returns = module
                .functions
                .iter()
                .filter(|func| {
                    function_returns_result_sifr_int(
                        func,
                        &result_function_returns,
                        &result_method_returns,
                        &result_function_params,
                        collect_sifr_int_result_function_param_names(func, &result_function_params),
                    )
                })
                .map(|func| func.name.clone())
                .collect::<Vec<_>>();
            result_function_returns.extend(discovered_result_returns);
            let discovered_result_method_returns = module
                .classes
                .iter()
                .flat_map(|class| {
                    class
                        .methods
                        .iter()
                        .filter(|method| {
                            let method_key = result_method_key(&class.name, &method.name);
                            let result_param_bindings = collect_sifr_int_result_method_param_names(
                                method,
                                &method_key,
                                &result_method_params,
                            );
                            function_returns_result_sifr_int(
                                method,
                                &result_function_returns,
                                &result_method_returns,
                                &result_function_params,
                                result_param_bindings,
                            )
                        })
                        .map(|method| result_method_key(&class.name, &method.name))
                })
                .collect::<Vec<_>>();
            result_method_returns.extend(discovered_result_method_returns);
            for func in &module.functions {
                let extra_forced_params =
                    collect_sifr_int_function_param_names(func, &function_params);
                let forced = collect_function_sifr_int_forced_locals_with_extra(
                    func,
                    &module_sifr_int_bindings,
                    &function_returns,
                    &extra_forced_params,
                );
                let shadowed_module_bindings = collect_function_local_shadow_names(func);
                for (name, indexes) in collect_sifr_int_call_arg_function_params(
                    &func.body,
                    &module_function_params,
                    &forced,
                    &shadowed_module_bindings,
                    &module_sifr_int_bindings,
                    &function_returns,
                ) {
                    function_params.entry(name).or_default().extend(indexes);
                }
                for (name, indexes) in collect_sifr_int_result_call_arg_function_params(
                    func,
                    &module_function_params,
                    &result_function_returns,
                    &result_method_returns,
                    &result_function_params,
                ) {
                    result_function_params
                        .entry(name)
                        .or_default()
                        .extend(indexes);
                }
                for (name, indexes) in collect_sifr_int_result_call_arg_method_params(
                    &func.body,
                    &module_method_params,
                    &result_function_returns,
                    &result_method_returns,
                    collect_sifr_int_result_function_param_names(func, &result_function_params),
                ) {
                    result_method_params
                        .entry(name)
                        .or_default()
                        .extend(indexes);
                }
            }
            for class in &module.classes {
                for method in &class.methods {
                    let method_key = result_method_key(&class.name, &method.name);
                    let result_param_bindings = collect_sifr_int_result_method_param_names(
                        method,
                        &method_key,
                        &result_method_params,
                    );
                    for (name, indexes) in
                        collect_sifr_int_result_call_arg_function_params_with_initial(
                            &method.body,
                            &module_function_params,
                            &result_function_returns,
                            &result_method_returns,
                            result_param_bindings.clone(),
                        )
                    {
                        result_function_params
                            .entry(name)
                            .or_default()
                            .extend(indexes);
                    }
                    for (name, indexes) in collect_sifr_int_result_call_arg_method_params(
                        &method.body,
                        &module_method_params,
                        &result_function_returns,
                        &result_method_returns,
                        result_param_bindings,
                    ) {
                        result_method_params
                            .entry(name)
                            .or_default()
                            .extend(indexes);
                    }
                }
            }
            let after_params = function_params.values().map(HashSet::len).sum::<usize>();
            let after_result_params = result_function_params
                .values()
                .map(HashSet::len)
                .sum::<usize>();
            let after_result_method_params = result_method_params
                .values()
                .map(HashSet::len)
                .sum::<usize>();
            if function_returns.len() == before
                && result_function_returns.len() == before_result_returns
                && result_method_returns.len() == before_result_methods
                && after_params == before_params
                && after_result_params == before_result_params
                && after_result_method_params == before_result_method_params
            {
                break;
            }
        }
        *self.sifr_int_function_returns.borrow_mut() = function_returns;
        *self.sifr_int_result_function_returns.borrow_mut() = result_function_returns;
        *self.sifr_int_result_method_returns.borrow_mut() = result_method_returns;
        *self.sifr_int_function_params.borrow_mut() = function_params;
        *self.sifr_int_result_function_params.borrow_mut() = result_function_params;
        *self.sifr_int_result_method_params.borrow_mut() = result_method_params;
    }

    pub(crate) fn module_sifr_int_bindings(&self) -> HashSet<String> {
        self.module_constants
            .iter()
            .filter(|(_, (ty, rust_name))| {
                matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int)
                    && rust_name.ends_with("()")
            })
            .map(|(name, _)| name.clone())
            .collect::<HashSet<_>>()
    }

    pub(crate) fn function_sifr_int_returns_for_body(&self, body: &[HirStmt]) -> HashSet<String> {
        let module_sifr_int_bindings = self.module_sifr_int_bindings();
        let mut function_returns = self.sifr_int_function_returns.borrow().clone();
        let forced_locals = self.sifr_int_forced_local_bindings.borrow().clone();
        let mut shadowed_module_bindings = self
            .local_binding_types
            .keys()
            .cloned()
            .collect::<HashSet<_>>();
        shadowed_module_bindings.extend(collect_locally_defined_vars(body));
        function_returns.extend(collect_nested_sifr_int_function_returns(
            body,
            &module_sifr_int_bindings,
            &function_returns,
            &forced_locals,
            &shadowed_module_bindings,
        ));
        function_returns
    }

    pub(crate) fn recursive_capture_lowers_to_sifr_int(&self, capture: &NestedFnCapture) -> bool {
        matches!(
            crate::resolve_alias_type_for_plain_call(&capture.ty),
            Type::Int
        ) && (self.module_sifr_int_bindings().contains(&capture.name)
            && !self.local_binding_types.contains_key(&capture.name)
            || self.is_registered_sifr_int_local(&capture.name)
            || self.is_forced_sifr_int_local(&capture.name))
    }
}

impl RustEmitter {
    pub(crate) fn lower_recursive_capture_param_type(&self, capture: &NestedFnCapture) -> RustType {
        if self.recursive_capture_lowers_to_sifr_int(capture) {
            return RustType::Named("SifrInt".to_string());
        }
        self.lower_function_param_type(&capture.ty, capture.convention)
    }

    pub(crate) fn try_lower_structured_nested_function_stmt(&mut self, stmt: &HirStmt) -> bool {
        let HirStmt::NestedFunction { func } = stmt else {
            return false;
        };

        if func.method_kind != sifr_hir::MethodKind::Regular
            || !func.decorators.is_empty()
            || !func.type_params.is_empty()
            || func
                .params
                .iter()
                .any(|param| param.default.is_some() || param.keyword_only)
        {
            return false;
        }

        let is_recursive =
            crate::hir_analysis::queries::body_calls_function(&func.body, &func.name);
        let nested_mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        let effective_param_conventions = func
            .params
            .iter()
            .map(|param| {
                (
                    param.name.clone(),
                    Self::effective_nested_param_convention(param, &nested_mutated_vars),
                )
            })
            .collect::<std::collections::HashMap<_, _>>();
        let recursive_captures = self
            .nested_fn_captures
            .get(&func.name)
            .cloned()
            .unwrap_or_default();
        let outer_forced_locals = self.sifr_int_forced_local_bindings.borrow().clone();
        let sifr_int_captured_forced_locals =
            collect_sifr_int_captured_forced_locals(func, &outer_forced_locals);
        let outer_shadowed_module_bindings = self
            .local_binding_types
            .keys()
            .filter(|name| self.module_sifr_int_bindings().contains(*name))
            .cloned()
            .collect::<HashSet<_>>();
        let captured_shadowed_module_bindings = collect_sifr_int_captured_shadowed_module_bindings(
            func,
            &outer_shadowed_module_bindings,
        );
        let sifr_int_recursive_captures = recursive_captures
            .iter()
            .filter(|capture| self.recursive_capture_lowers_to_sifr_int(capture))
            .map(|capture| capture.name.clone())
            .collect::<HashSet<_>>();
        let mut sifr_int_nested_capture_bindings = sifr_int_recursive_captures.clone();
        sifr_int_nested_capture_bindings.extend(sifr_int_captured_forced_locals.iter().cloned());
        let nested_returns_sifr_int = matches!(
            crate::resolve_alias_type_for_plain_call(&func.return_type),
            Type::Int
        )
            && hir_function_returns_sifr_int_with_extra_forced_and_shadowed(
                func,
                &self.module_sifr_int_bindings(),
                &self.sifr_int_function_returns.borrow(),
                &sifr_int_nested_capture_bindings,
                &captured_shadowed_module_bindings,
            );
        let nested_returns_sifr_int_result = function_returns_result_sifr_int(
            func,
            &self.sifr_int_result_function_returns.borrow(),
            &self.sifr_int_result_method_returns.borrow(),
            &self.sifr_int_result_function_params.borrow(),
            collect_sifr_int_result_function_param_names(
                func,
                &self.sifr_int_result_function_params.borrow(),
            ),
        );
        let post_stmt_callable_conventions = {
            let mut conventions = self.callable_var_conventions.clone();
            let params = func
                .params
                .iter()
                .map(|param| {
                    (
                        param.ty.clone(),
                        *effective_param_conventions
                            .get(&param.name)
                            .unwrap_or(&param.convention),
                    )
                })
                .collect::<Vec<_>>();
            conventions.insert(func.name.clone(), params);
            conventions
        };

        let saved_return_type = self.current_return_type.clone();
        let saved_mutated_vars = self.mutated_vars.clone();
        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        let saved_callable_var_conventions = self.callable_var_conventions.clone();
        let saved_local_binding_types = self.local_binding_types.clone();
        let saved_none_widened_local_bindings = self.none_widened_local_bindings.clone();
        let saved_sifr_int_local_bindings = self.sifr_int_local_bindings.borrow().clone();
        let saved_sifr_int_forced_local_bindings =
            self.sifr_int_forced_local_bindings.borrow().clone();
        let saved_sifr_int_result_local_bindings =
            self.sifr_int_result_local_bindings.borrow().clone();
        let saved_current_sifr_int_return = self.current_sifr_int_return.get();
        let saved_current_sifr_int_result_return = self.current_sifr_int_result_return.get();
        let nested_binding_mutable = saved_mutated_vars.contains(&func.name)
            || nested_function_mutates_capture(func, &nested_mutated_vars);

        self.current_return_type = Some(func.return_type.clone());
        self.mutated_vars = nested_mutated_vars;
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.local_binding_types.clear();
        self.none_widened_local_bindings.clear();
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        self.sifr_int_result_local_bindings.borrow_mut().clear();
        self.current_sifr_int_return.set(nested_returns_sifr_int);
        self.current_sifr_int_result_return
            .set(nested_returns_sifr_int_result);
        if nested_returns_sifr_int {
            self.sifr_int_function_returns
                .borrow_mut()
                .insert(func.name.clone());
        }
        if nested_returns_sifr_int_result {
            self.sifr_int_result_function_returns
                .borrow_mut()
                .insert(func.name.clone());
        }
        self.callable_var_conventions
            .clone_from(&post_stmt_callable_conventions);
        for param in &func.params {
            self.register_function_scope_binding(
                &param.name,
                &param.ty,
                *effective_param_conventions
                    .get(&param.name)
                    .unwrap_or(&param.convention),
            );
        }
        for capture in &recursive_captures {
            self.register_function_scope_binding(&capture.name, &capture.ty, capture.convention);
            if sifr_int_nested_capture_bindings.contains(&capture.name) {
                self.sifr_int_local_bindings
                    .borrow_mut()
                    .insert(capture.name.clone());
            }
        }
        for name in &captured_shadowed_module_bindings {
            self.local_binding_types.insert(name.clone(), Type::Int);
        }
        self.sifr_int_local_bindings
            .borrow_mut()
            .extend(sifr_int_captured_forced_locals.iter().cloned());
        self.sifr_int_forced_local_bindings
            .borrow_mut()
            .extend(sifr_int_captured_forced_locals);
        self.register_local_body_binding_types(&func.body);

        let mut lowered_body = Vec::new();
        for body_stmt in &func.body {
            lowered_body.extend(self.lower_stmt_strict_for_function(
                body_stmt,
                "nested function body statement lowering",
            ));
        }

        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated_vars;
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.callable_var_conventions
            .clone_from(&saved_callable_var_conventions);
        self.local_binding_types = saved_local_binding_types;
        self.none_widened_local_bindings = saved_none_widened_local_bindings;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        *self.sifr_int_result_local_bindings.borrow_mut() = saved_sifr_int_result_local_bindings;
        self.current_sifr_int_return
            .set(saved_current_sifr_int_return);
        self.current_sifr_int_result_return
            .set(saved_current_sifr_int_result_return);

        let lowered_stmt = if is_recursive {
            let params = func
                .params
                .iter()
                .map(|param| RustParam::Named {
                    name: param.name.clone(),
                    ty: self.lower_function_param_type(
                        &param.ty,
                        *effective_param_conventions
                            .get(&param.name)
                            .unwrap_or(&param.convention),
                    ),
                })
                .chain(recursive_captures.iter().map(|capture| RustParam::Named {
                    name: capture.name.clone(),
                    ty: self.lower_recursive_capture_param_type(capture),
                }))
                .collect::<Vec<_>>();
            RustStmt::LocalFn {
                name: func.name.clone(),
                params,
                ret: if nested_returns_sifr_int {
                    Some(RustType::Named("SifrInt".to_string()))
                } else if nested_returns_sifr_int_result {
                    Some(result_int_return_type_to_sifr_int(&func.return_type))
                } else {
                    self.lower_function_return_type(func, false)
                },
                body: lowered_body,
            }
        } else {
            let params = func
                .params
                .iter()
                .map(|param| RustParam::Named {
                    name: param.name.clone(),
                    ty: self.lower_function_param_type(
                        &param.ty,
                        *effective_param_conventions
                            .get(&param.name)
                            .unwrap_or(&param.convention),
                    ),
                })
                .collect::<Vec<_>>();
            RustStmt::Let {
                mutable: nested_binding_mutable,
                name: func.name.clone(),
                ty: None,
                value: RustExpr::ClosureBlock {
                    params,
                    body: lowered_body,
                    is_move: false,
                    is_async: false,
                },
            }
        };

        self.callable_var_conventions = saved_callable_var_conventions;
        self.callable_var_conventions
            .extend(post_stmt_callable_conventions);
        self.push_captured_stmt(&lowered_stmt);
        true
    }

    pub(crate) fn lower_mutable_param_shadows(
        params: &[HirParam],
        reassigned_vars: &std::collections::HashSet<String>,
    ) -> Vec<(String, RustExpr)> {
        params
            .iter()
            .filter(|param| {
                if param.convention.is_owned() {
                    false
                } else {
                    reassigned_vars.contains(&param.name)
                }
            })
            .map(|param| {
                let value = if param.convention.is_borrowed()
                    && param.ty.ownership() != OwnershipKind::Copy
                {
                    RustExpr::Clone(Box::new(RustExpr::Ident(param.name.clone())))
                } else {
                    RustExpr::Ident(param.name.clone())
                };
                (param.name.clone(), value)
            })
            .collect()
    }

    pub(crate) fn apply_mutable_param_shadowing(
        &mut self,
        mutable_param_shadows: &[(String, RustExpr)],
    ) {
        for (param_name, _) in mutable_param_shadows {
            self.borrowed_params.remove(param_name);
            self.mut_borrowed_params.remove(param_name);
        }
    }

    pub(crate) fn emit_mutable_param_shadow_stmts(
        mutable_param_shadows: &[(String, RustExpr)],
    ) -> Vec<RustStmt> {
        mutable_param_shadows
            .iter()
            .map(|(param_name, value)| RustStmt::Let {
                mutable: true,
                name: param_name.clone(),
                ty: None,
                value: value.clone(),
            })
            .collect()
    }

    pub(crate) fn returns_result_none(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Result(ok_ty, _) => matches!(
                crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                Type::None
            ),
            _ => false,
        }
    }

    pub(crate) fn lower_function_type_params(func: &HirFunction) -> Vec<RustTypeParam> {
        if func.type_params.is_empty() {
            return Vec::new();
        }
        let needs_hash_eq = Self::func_needs_hash_eq(func);
        func.type_params
            .iter()
            .map(|tp| {
                let extra = Self::extra_bounds_for_type_param(tp, &func.body);
                let base = if needs_hash_eq {
                    "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq + 'static"
                } else {
                    "Clone + std::fmt::Display + PartialOrd + 'static"
                };
                RustTypeParam {
                    name: tp.clone(),
                    bounds: vec![format!("{base}{extra}")],
                }
            })
            .collect()
    }

    pub(crate) fn lower_function_param_type(
        &self,
        ty: &Type,
        convention: ParamConvention,
    ) -> RustType {
        let base = self.rust_ir_type_with_generics(ty);
        if ty.ownership() != sifr_type_system::OwnershipKind::Copy && convention.is_borrowed() {
            RustType::Ref {
                mutable: convention.is_mut_borrow(),
                inner: Box::new(base),
            }
        } else {
            base
        }
    }

    pub(crate) fn lower_module_function_param_type(
        &self,
        func_name: &str,
        param_idx: usize,
        param: &HirParam,
    ) -> RustType {
        if self.function_param_lowers_to_sifr_int(func_name, param_idx)
            && matches!(
                crate::resolve_alias_type_for_plain_call(&param.ty),
                Type::Int
            )
        {
            return RustType::Named("SifrInt".to_string());
        }
        if self.function_param_lowers_to_sifr_int_result(func_name, param_idx)
            && is_result_int_type(&param.ty)
        {
            return result_int_return_type_to_sifr_int(&param.ty);
        }
        self.lower_function_param_type(&param.ty, param.convention)
    }

    pub(crate) fn lower_function_return_type(
        &self,
        func: &HirFunction,
        is_generator: bool,
    ) -> Option<RustType> {
        if is_generator {
            return Some(self.rust_ir_type_with_generics(&func.return_type));
        }

        if func.return_type == Type::None {
            return None;
        }
        if self.function_returns_sifr_int(&func.name) {
            return Some(RustType::Named("SifrInt".to_string()));
        }
        if self
            .sifr_int_result_function_returns
            .borrow()
            .contains(&func.name)
        {
            return Some(result_int_return_type_to_sifr_int(&func.return_type));
        }
        Some(self.rust_ir_type_with_generics(&func.return_type))
    }

    pub(crate) fn lower_stmt_strict_for_function(
        &mut self,
        stmt: &HirStmt,
        _context: &str,
    ) -> Vec<RustStmt> {
        self.capture_structured_stmts(|inner| inner.emit_stmt(stmt))
    }
}
