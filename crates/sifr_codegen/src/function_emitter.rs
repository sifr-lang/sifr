use crate::NestedFnCapture;
use crate::{
    body_contains_yield, collect_mutated_vars_with_sigs, RustEmitter, RustExpr, RustItem,
    RustLiteral, RustParam, RustStmt, RustType, RustTypeParam, Visibility,
};
use crate::{
    helpers::{
        collect_locally_defined_vars, collect_reassigned_vars, collect_referenced_vars_with_types,
    },
    hir_analysis::traversal::{self, TraversalConfig},
};
use sifr_hir::{HirExpr, HirFunction, HirModule, HirParam, HirStmt};
use sifr_type_system::{make_union, OwnershipKind, ParamConvention, Type};
use std::collections::{HashMap, HashSet};

impl RustEmitter {
    fn effective_nested_param_convention(
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

    fn register_function_scope_binding(
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

    fn register_function_scope_params(&mut self, func_name: &str, params: &[HirParam]) {
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

    pub(super) fn register_local_body_binding_types(&mut self, body: &[HirStmt]) {
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

    fn register_sifr_int_forced_local_bindings(&self, body: &[HirStmt]) {
        let local_int_bindings = self
            .local_binding_types
            .iter()
            .filter_map(|(name, ty)| {
                matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int)
                    .then(|| name.clone())
            })
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

    pub(super) fn register_sifr_int_function_returns(&self, module: &HirModule) {
        let module_sifr_int_bindings = self.module_sifr_int_bindings();
        let mut function_returns = HashSet::new();
        let mut result_function_returns = HashSet::new();
        let mut result_method_returns = HashSet::new();
        let mut function_params: HashMap<String, HashSet<usize>> = HashMap::new();
        let mut result_function_params: HashMap<String, HashSet<usize>> = HashMap::new();
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
        loop {
            let before = function_returns.len();
            let before_result_returns = result_function_returns.len();
            let before_result_methods = result_method_returns.len();
            let before_params = function_params.values().map(HashSet::len).sum::<usize>();
            let before_result_params = result_function_params
                .values()
                .map(HashSet::len)
                .sum::<usize>();
            let discovered = module
                .functions
                .iter()
                .filter_map(|func| {
                    let extra_forced_params =
                        collect_sifr_int_function_param_names(func, &function_params);
                    (matches!(
                        crate::resolve_alias_type_for_plain_call(&func.return_type),
                        Type::Int
                    ) && hir_function_returns_sifr_int_with_extra_forced(
                        func,
                        &module_sifr_int_bindings,
                        &function_returns,
                        &extra_forced_params,
                    ))
                    .then(|| func.name.clone())
                })
                .collect::<Vec<_>>();
            function_returns.extend(discovered);
            let discovered_result_returns = module
                .functions
                .iter()
                .filter_map(|func| {
                    (function_returns_result_sifr_int(
                        func,
                        &result_function_returns,
                        &result_method_returns,
                        &result_function_params,
                    ))
                    .then(|| func.name.clone())
                })
                .collect::<Vec<_>>();
            result_function_returns.extend(discovered_result_returns);
            let discovered_result_method_returns = module
                .classes
                .iter()
                .flat_map(|class| {
                    class.methods.iter().filter_map(|method| {
                        function_returns_result_sifr_int(
                            method,
                            &result_function_returns,
                            &result_method_returns,
                            &result_function_params,
                        )
                        .then(|| result_method_key(&class.name, &method.name))
                    })
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
            }
            let after_params = function_params.values().map(HashSet::len).sum::<usize>();
            let after_result_params = result_function_params
                .values()
                .map(HashSet::len)
                .sum::<usize>();
            if function_returns.len() == before
                && result_function_returns.len() == before_result_returns
                && result_method_returns.len() == before_result_methods
                && after_params == before_params
                && after_result_params == before_result_params
            {
                break;
            }
        }
        *self.sifr_int_function_returns.borrow_mut() = function_returns;
        *self.sifr_int_result_function_returns.borrow_mut() = result_function_returns;
        *self.sifr_int_result_method_returns.borrow_mut() = result_method_returns;
        *self.sifr_int_function_params.borrow_mut() = function_params;
        *self.sifr_int_result_function_params.borrow_mut() = result_function_params;
    }

    fn module_sifr_int_bindings(&self) -> HashSet<String> {
        self.module_constants
            .iter()
            .filter_map(|(name, (ty, rust_name))| {
                (matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Int)
                    && rust_name.ends_with("()"))
                .then(|| name.clone())
            })
            .collect::<HashSet<_>>()
    }

    fn function_sifr_int_returns_for_body(&self, body: &[HirStmt]) -> HashSet<String> {
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

    pub(super) fn recursive_capture_lowers_to_sifr_int(&self, capture: &NestedFnCapture) -> bool {
        matches!(
            crate::resolve_alias_type_for_plain_call(&capture.ty),
            Type::Int
        ) && (self.module_sifr_int_bindings().contains(&capture.name)
            && !self.local_binding_types.contains_key(&capture.name)
            || self.is_registered_sifr_int_local(&capture.name)
            || self.is_forced_sifr_int_local(&capture.name))
    }

    fn lower_recursive_capture_param_type(&self, capture: &NestedFnCapture) -> RustType {
        if self.recursive_capture_lowers_to_sifr_int(capture) {
            return RustType::Named("SifrInt".to_string());
        }
        self.lower_function_param_type(&capture.ty, capture.convention)
    }

    pub(super) fn try_lower_structured_nested_function_stmt(&mut self, stmt: &HirStmt) -> bool {
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
            .filter_map(|capture| {
                self.recursive_capture_lowers_to_sifr_int(capture)
                    .then(|| capture.name.clone())
            })
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
        let nested_binding_mutable = saved_mutated_vars.contains(&func.name);

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
                },
            }
        };

        self.callable_var_conventions = saved_callable_var_conventions;
        self.callable_var_conventions
            .extend(post_stmt_callable_conventions);
        self.push_captured_stmt(&lowered_stmt);
        true
    }

    pub(super) fn lower_mutable_param_shadows(
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

    pub(super) fn apply_mutable_param_shadowing(
        &mut self,
        mutable_param_shadows: &[(String, RustExpr)],
    ) {
        for (param_name, _) in mutable_param_shadows {
            self.borrowed_params.remove(param_name);
            self.mut_borrowed_params.remove(param_name);
        }
    }

    pub(super) fn emit_mutable_param_shadow_stmts(
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

    fn returns_result_none(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Result(ok_ty, _) => matches!(
                crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                Type::None
            ),
            _ => false,
        }
    }

    fn lower_function_type_params(func: &HirFunction) -> Vec<RustTypeParam> {
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

    fn lower_function_param_type(&self, ty: &Type, convention: ParamConvention) -> RustType {
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

    fn lower_module_function_param_type(
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

    fn lower_function_return_type(
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

    fn lower_stmt_strict_for_function(&mut self, stmt: &HirStmt, _context: &str) -> Vec<RustStmt> {
        self.capture_structured_stmts(|inner| inner.emit_stmt(stmt))
    }

    fn lower_generator_function_body(
        &mut self,
        func: &HirFunction,
        mutable_param_shadows: &[(String, RustExpr)],
    ) -> Vec<RustStmt> {
        let yield_ty = if let Type::Iterator(elem) = func.return_type.resolve_alias() {
            self.rust_ir_type_with_generics(elem)
        } else {
            RustType::I64
        };

        let mut body = Self::emit_mutable_param_shadow_stmts(mutable_param_shadows);
        for param in &func.params {
            if mutable_param_shadows
                .iter()
                .any(|(name, _)| name == &param.name)
                || !param.convention.is_borrowed()
                || param.ty.ownership() == OwnershipKind::Copy
            {
                continue;
            }
            body.push(RustStmt::Let {
                mutable: false,
                name: param.name.clone(),
                ty: None,
                value: RustExpr::Clone(Box::new(RustExpr::Ident(param.name.clone()))),
            });
        }
        let generator_iter_ty = RustType::Generic {
            base: "std::vec::IntoIter".to_string(),
            params: vec![yield_ty.clone()],
        };
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_generator_initialized".to_string(),
            ty: Some(RustType::Bool),
            value: RustExpr::Literal(RustLiteral::Bool(false)),
        });
        body.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_generator_iter".to_string(),
            ty: Some(generator_iter_ty),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                    args: vec![],
                }),
                method: "into_iter".to_string(),
                args: vec![],
            },
        });

        let cloned_borrowed_param_names: Vec<String> = func
            .params
            .iter()
            .filter(|param| {
                !mutable_param_shadows
                    .iter()
                    .any(|(name, _)| name == &param.name)
                    && param.convention.is_borrowed()
                    && param.ty.ownership() != OwnershipKind::Copy
            })
            .map(|param| param.name.clone())
            .collect();

        let saved_borrowed_params = self.borrowed_params.clone();
        let saved_mut_borrowed_params = self.mut_borrowed_params.clone();
        for name in &cloned_borrowed_param_names {
            self.borrowed_params.remove(name);
            self.mut_borrowed_params.remove(name);
        }

        let mut materialize_body = vec![RustStmt::Let {
            mutable: true,
            name: "_yields".to_string(),
            ty: Some(RustType::Vec(Box::new(yield_ty))),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Vec".to_string(), "new".to_string()])),
                args: vec![],
            },
        }];
        for stmt in &func.body {
            materialize_body.extend(self.lower_stmt_strict_for_function(
                stmt,
                "generator materialization statement lowering",
            ));
        }
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        materialize_body.push(RustStmt::Assign {
            target: RustExpr::Ident("__sifr_generator_iter".to_string()),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("_yields".to_string())),
                method: "into_iter".to_string(),
                args: vec![],
            },
        });
        materialize_body.push(RustStmt::Assign {
            target: RustExpr::Ident("__sifr_generator_initialized".to_string()),
            value: RustExpr::Literal(RustLiteral::Bool(true)),
        });

        let closure_body = vec![
            RustStmt::If {
                cond: RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(RustExpr::Ident("__sifr_generator_initialized".to_string())),
                },
                then_body: materialize_body,
                else_body: None,
            },
            RustStmt::Return(Some(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_generator_iter".to_string())),
                method: "next".to_string(),
                args: vec![],
            })),
        ];

        let from_fn_expr = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "std".to_string(),
                "iter".to_string(),
                "from_fn".to_string(),
            ])),
            args: vec![RustExpr::ClosureBlock {
                params: vec![],
                body: closure_body,
                is_move: true,
            }],
        };

        body.push(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
            args: vec![from_fn_expr],
        })));
        body
    }

    pub(super) fn emit_function(
        &mut self,
        func: &HirFunction,
        module_public: bool,
        test_mode: bool,
    ) {
        // In test mode, skip the main function
        if test_mode && func.name == "main" {
            return;
        }

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
        let saved_sifr_int_function_returns = self.sifr_int_function_returns.borrow().clone();
        let saved_sifr_int_result_function_returns =
            self.sifr_int_result_function_returns.borrow().clone();
        let saved_current_sifr_int_return = self.current_sifr_int_return.get();
        let saved_current_sifr_int_result_return = self.current_sifr_int_result_return.get();

        self.current_return_type = Some(func.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.callable_var_conventions.clear();
        self.local_binding_types.clear();
        self.none_widened_local_bindings.clear();
        self.sifr_int_local_bindings.borrow_mut().clear();
        self.sifr_int_forced_local_bindings.borrow_mut().clear();
        self.sifr_int_result_local_bindings.borrow_mut().clear();
        self.current_sifr_int_return
            .set(self.function_returns_sifr_int(&func.name));
        self.current_sifr_int_result_return.set(
            self.sifr_int_result_function_returns
                .borrow()
                .contains(&func.name),
        );
        self.register_function_scope_params(&func.name, &func.params);
        let active_function_returns = self.function_sifr_int_returns_for_body(&func.body);
        *self.sifr_int_function_returns.borrow_mut() = active_function_returns;
        self.register_local_body_binding_types(&func.body);

        let visibility = if !test_mode && module_public && func.name != "main" {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let is_generator = body_contains_yield(&func.body);
        if is_generator {
            self.generator_functions.insert(func.name.clone());
        }

        let reassigned_vars = collect_reassigned_vars(&func.body);
        let mutable_param_shadows =
            Self::lower_mutable_param_shadows(&func.params, &reassigned_vars);
        self.apply_mutable_param_shadowing(&mutable_param_shadows);

        let params = func
            .params
            .iter()
            .enumerate()
            .map(|(param_idx, param)| {
                let rust_ty = self.lower_module_function_param_type(&func.name, param_idx, param);
                if param.convention.is_owned() && param.convention.is_mutable() {
                    RustParam::NamedMut {
                        name: param.name.clone(),
                        ty: rust_ty,
                    }
                } else {
                    RustParam::Named {
                        name: param.name.clone(),
                        ty: rust_ty,
                    }
                }
            })
            .collect::<Vec<_>>();

        let mut lowered_body = if is_generator {
            self.lower_generator_function_body(func, &mutable_param_shadows)
        } else {
            let mut lowered = Self::emit_mutable_param_shadow_stmts(&mutable_param_shadows);
            for stmt in &func.body {
                lowered.extend(
                    self.lower_stmt_strict_for_function(stmt, "function body statement lowering"),
                );
            }
            lowered
        };

        if !is_generator
            && Self::returns_result_none(&func.return_type)
            && !matches!(
                func.body.last(),
                Some(HirStmt::Return { .. } | HirStmt::Raise { .. })
            )
        {
            lowered_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::Literal(RustLiteral::Unit)],
            })));
        }
        if lowered_body.is_empty() {
            if self
                .lower_function_return_type(func, is_generator)
                .is_none()
            {
                lowered_body.push(RustStmt::Return(None));
            } else {
                panic!(
                    "function IR lowering produced empty body for non-unit return: {}",
                    func.name
                );
            }
        }

        for decorator in &func.decorators {
            self.body_items
                .push(RustItem::Attr(format!("// @{decorator}")));
        }
        if test_mode && func.name.starts_with("test_") {
            self.body_items.push(RustItem::Attr("#[test]".to_string()));
        }

        self.body_items.push(RustItem::Fn {
            name: func.name.clone(),
            visibility,
            type_params: Self::lower_function_type_params(func),
            params,
            ret: self.lower_function_return_type(func, is_generator),
            body: lowered_body,
            is_async: false,
        });

        self.current_return_type = saved_return_type;
        self.mutated_vars = saved_mutated_vars;
        self.borrowed_params = saved_borrowed_params;
        self.mut_borrowed_params = saved_mut_borrowed_params;
        self.callable_var_conventions = saved_callable_var_conventions;
        self.local_binding_types = saved_local_binding_types;
        self.none_widened_local_bindings = saved_none_widened_local_bindings;
        *self.sifr_int_local_bindings.borrow_mut() = saved_sifr_int_local_bindings;
        *self.sifr_int_forced_local_bindings.borrow_mut() = saved_sifr_int_forced_local_bindings;
        *self.sifr_int_result_local_bindings.borrow_mut() = saved_sifr_int_result_local_bindings;
        *self.sifr_int_function_returns.borrow_mut() = saved_sifr_int_function_returns;
        *self.sifr_int_result_function_returns.borrow_mut() =
            saved_sifr_int_result_function_returns;
        self.current_sifr_int_return
            .set(saved_current_sifr_int_return);
        self.current_sifr_int_result_return
            .set(saved_current_sifr_int_result_return);
    }
}

fn hir_function_returns_sifr_int(
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

fn function_returns_result_sifr_int(
    func: &HirFunction,
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    result_function_params: &HashMap<String, HashSet<usize>>,
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
    let result_param_bindings =
        collect_sifr_int_result_function_param_names(func, result_function_params);
    let local_result_bindings = collect_sifr_int_result_local_bindings_with_initial(
        &func.body,
        &result_function_returns,
        result_method_returns,
        result_param_bindings,
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

fn collect_nested_sifr_int_result_function_returns(
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
            if let HirStmt::NestedFunction { func } = stmt {
                if function_returns_result_sifr_int(
                    func,
                    &available_result_returns,
                    result_method_returns,
                    result_function_params,
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

fn collect_sifr_int_result_local_bindings(
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

fn collect_sifr_int_result_local_bindings_with_initial(
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

fn collect_sifr_int_result_function_param_names(
    func: &HirFunction,
    result_function_params: &HashMap<String, HashSet<usize>>,
) -> HashSet<String> {
    let Some(indexes) = result_function_params.get(&func.name) else {
        return HashSet::new();
    };
    func.params
        .iter()
        .enumerate()
        .filter_map(|(idx, param)| indexes.contains(&idx).then(|| param.name.clone()))
        .collect()
}

fn hir_expr_returns_sifr_int_result(
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

fn hir_expr_class_name(expr: &HirExpr) -> Option<String> {
    match crate::resolve_alias_type_for_plain_call(expr.ty()) {
        Type::Class { name, .. } => Some(name.clone()),
        _ => None,
    }
}

fn hir_function_returns_sifr_int_with_extra_forced(
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

fn hir_function_returns_sifr_int_with_extra_forced_and_shadowed(
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

fn collect_function_sifr_int_forced_locals_with_extra(
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

fn collect_function_sifr_int_forced_locals_with_extra_and_shadowed(
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

fn collect_sifr_int_function_param_names(
    func: &HirFunction,
    function_params: &HashMap<String, HashSet<usize>>,
) -> HashSet<String> {
    let Some(indexes) = function_params.get(&func.name) else {
        return HashSet::new();
    };
    func.params
        .iter()
        .enumerate()
        .filter_map(|(idx, param)| indexes.contains(&idx).then(|| param.name.clone()))
        .collect()
}

fn collect_function_local_shadow_names(func: &HirFunction) -> HashSet<String> {
    let mut shadowed = collect_locally_defined_vars(&func.body);
    shadowed.extend(func.params.iter().map(|param| param.name.clone()));
    shadowed
}

fn collect_sifr_int_call_arg_function_params(
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

fn collect_sifr_int_result_call_arg_function_params(
    caller: &HirFunction,
    module_function_params: &HashMap<String, Vec<Type>>,
    result_function_returns: &HashSet<String>,
    result_method_returns: &HashSet<String>,
    result_function_params: &HashMap<String, HashSet<usize>>,
) -> HashMap<String, HashSet<usize>> {
    let result_param_bindings =
        collect_sifr_int_result_function_param_names(caller, result_function_params);
    let local_result_bindings = collect_sifr_int_result_local_bindings_with_initial(
        &caller.body,
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
        &caller.body,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );
    discovered
}

fn collect_nested_sifr_int_function_returns(
    body: &[HirStmt],
    module_sifr_int_bindings: &HashSet<String>,
    outer_function_returns: &HashSet<String>,
    outer_forced_locals: &HashSet<String>,
    outer_shadowed_module_bindings: &HashSet<String>,
) -> HashSet<String> {
    let nested_functions = body
        .iter()
        .filter_map(|stmt| match stmt {
            HirStmt::NestedFunction { func } => Some(func),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut function_returns = outer_function_returns.clone();
    loop {
        let before = function_returns.len();
        let discovered = nested_functions
            .iter()
            .filter_map(|func| {
                let captured_forced =
                    collect_sifr_int_captured_forced_locals(func, outer_forced_locals);
                let captured_shadowed = collect_sifr_int_captured_shadowed_module_bindings(
                    func,
                    outer_shadowed_module_bindings,
                );
                (matches!(
                    crate::resolve_alias_type_for_plain_call(&func.return_type),
                    Type::Int
                ) && hir_function_returns_sifr_int_with_extra_forced_and_shadowed(
                    func,
                    module_sifr_int_bindings,
                    &function_returns,
                    &captured_forced,
                    &captured_shadowed,
                ))
                .then(|| func.name.clone())
            })
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

fn collect_sifr_int_captured_forced_locals(
    func: &HirFunction,
    outer_forced_locals: &HashSet<String>,
) -> HashSet<String> {
    collect_captured_outer_names_transitively(func, outer_forced_locals)
}

fn collect_sifr_int_captured_shadowed_module_bindings(
    func: &HirFunction,
    outer_shadowed_module_bindings: &HashSet<String>,
) -> HashSet<String> {
    collect_captured_outer_names_transitively(func, outer_shadowed_module_bindings)
}

fn collect_captured_outer_names(
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
        .filter_map(|(name, _)| {
            (!param_names.contains(&name)
                && !locally_defined.contains(&name)
                && outer_names.contains(&name))
            .then_some(name)
        })
        .collect()
}

fn collect_captured_outer_names_transitively(
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
        HirStmt::NestedFunction { func } => Some(func),
        _ => None,
    }) {
        captured.extend(collect_captured_outer_names_transitively(
            nested,
            &visible_outer_names,
        ));
    }
    captured
}

fn collect_sifr_int_forced_locals(
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

fn collect_sifr_int_forced_locals_with_seed(
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

fn hir_expr_needs_sifr_int_storage(
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

fn is_sifr_int_augassign_op(op: &str) -> bool {
    matches!(op, "+=" | "-=" | "*=")
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_hir::MethodKind;

    fn int_binop_name(name: &str) -> HirExpr {
        HirExpr::BinOp {
            left: Box::new(HirExpr::Name {
                name: name.to_string(),
                ty: Type::Int,
            }),
            op: "+".to_string(),
            right: Box::new(HirExpr::IntLiteral(1)),
            ty: Type::Int,
        }
    }

    fn regular_int_function(params: Vec<HirParam>, body: Vec<HirStmt>) -> HirFunction {
        HirFunction {
            name: "f".to_string(),
            params,
            return_type: Type::Int,
            body,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }
    }

    fn helper_returning_name(name: &str) -> HirFunction {
        HirFunction {
            name: "helper".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(int_binop_name(name)),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }
    }

    fn middle_with_inner_returning_name(name: &str) -> HirFunction {
        HirFunction {
            name: "middle".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![
                HirStmt::NestedFunction {
                    func: helper_returning_name(name),
                },
                HirStmt::Return {
                    value: Some(HirExpr::Call {
                        func: "helper".to_string(),
                        args: vec![],
                        ty: Type::Int,
                    }),
                },
            ],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }
    }

    #[test]
    fn shadowed_module_const_local_does_not_promote_return_to_sifr_int() {
        let func = regular_int_function(
            vec![],
            vec![
                HirStmt::Let {
                    name: "BIG_LIMIT".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(5),
                    is_mutable: false,
                },
                HirStmt::Return {
                    value: Some(int_binop_name("BIG_LIMIT")),
                },
            ],
        );
        let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

        assert!(!hir_function_returns_sifr_int(
            &func,
            &module_sifr_int_bindings,
            &HashSet::new(),
        ));
    }

    #[test]
    fn shadowed_module_const_param_does_not_promote_return_to_sifr_int() {
        let func = regular_int_function(
            vec![HirParam {
                name: "BIG_LIMIT".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::own(),
            }],
            vec![HirStmt::Return {
                value: Some(int_binop_name("BIG_LIMIT")),
            }],
        );
        let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

        assert!(!hir_function_returns_sifr_int(
            &func,
            &module_sifr_int_bindings,
            &HashSet::new(),
        ));
    }

    #[test]
    fn nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int() {
        let func = regular_int_function(
            vec![],
            vec![
                HirStmt::Let {
                    name: "BIG_LIMIT".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(5),
                    is_mutable: false,
                },
                HirStmt::NestedFunction {
                    func: helper_returning_name("BIG_LIMIT"),
                },
                HirStmt::Return {
                    value: Some(HirExpr::Call {
                        func: "helper".to_string(),
                        args: vec![],
                        ty: Type::Int,
                    }),
                },
            ],
        );
        let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

        assert!(!hir_function_returns_sifr_int(
            &func,
            &module_sifr_int_bindings,
            &HashSet::new(),
        ));
    }

    #[test]
    fn multilevel_nested_helper_captures_outer_shadow_without_promoting_return_to_sifr_int() {
        let func = regular_int_function(
            vec![],
            vec![
                HirStmt::Let {
                    name: "BIG_LIMIT".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(5),
                    is_mutable: false,
                },
                HirStmt::NestedFunction {
                    func: middle_with_inner_returning_name("BIG_LIMIT"),
                },
                HirStmt::Return {
                    value: Some(HirExpr::Call {
                        func: "middle".to_string(),
                        args: vec![],
                        ty: Type::Int,
                    }),
                },
            ],
        );
        let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

        assert!(!hir_function_returns_sifr_int(
            &func,
            &module_sifr_int_bindings,
            &HashSet::new(),
        ));
    }

    #[test]
    fn multilevel_nested_helper_captures_forced_local_and_promotes_return_to_sifr_int() {
        let func = middle_with_inner_returning_name("big");
        let forced_locals = HashSet::from(["big".to_string()]);

        assert_eq!(
            collect_sifr_int_captured_forced_locals(&func, &forced_locals),
            forced_locals
        );
        assert!(hir_function_returns_sifr_int_with_extra_forced(
            &func,
            &HashSet::new(),
            &HashSet::new(),
            &forced_locals,
        ));
    }

    #[test]
    fn unshadowed_module_const_still_promotes_return_to_sifr_int() {
        let func = regular_int_function(
            vec![],
            vec![HirStmt::Return {
                value: Some(int_binop_name("BIG_LIMIT")),
            }],
        );
        let module_sifr_int_bindings = HashSet::from(["BIG_LIMIT".to_string()]);

        assert!(hir_function_returns_sifr_int(
            &func,
            &module_sifr_int_bindings,
            &HashSet::new(),
        ));
    }
}
