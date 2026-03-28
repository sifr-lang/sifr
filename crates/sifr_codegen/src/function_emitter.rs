use crate::helpers::collect_reassigned_vars;
use crate::{
    body_contains_yield, collect_mutated_vars_with_sigs, RustEmitter, RustExpr, RustItem,
    RustLiteral, RustParam, RustStmt, RustType, RustTypeParam, Visibility,
};
use sifr_hir::{HirFunction, HirParam, HirStmt};
use sifr_type_system::{OwnershipKind, ParamConvention, Type};

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
    }

    fn register_function_scope_params(&mut self, params: &[HirParam]) {
        for param in params {
            self.register_function_scope_binding(&param.name, &param.ty, param.convention);
        }
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
        let nested_binding_mutable = saved_mutated_vars.contains(&func.name);

        self.current_return_type = Some(func.return_type.clone());
        self.mutated_vars = nested_mutated_vars;
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
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
        }

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
            .clone_from(&post_stmt_callable_conventions);

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
                    ty: self.lower_function_param_type(&capture.ty, capture.convention),
                }))
                .collect::<Vec<_>>();
            RustStmt::LocalFn {
                name: func.name.clone(),
                params,
                ret: self.lower_function_return_type(func, false),
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

        self.current_return_type = Some(func.return_type.clone());
        self.mutated_vars = collect_mutated_vars_with_sigs(&func.body, &self.func_signatures);
        self.borrowed_params.clear();
        self.mut_borrowed_params.clear();
        self.callable_var_conventions.clear();
        self.register_function_scope_params(&func.params);

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
            .map(|param| {
                let rust_ty = self.lower_function_param_type(&param.ty, param.convention);
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
    }
}
