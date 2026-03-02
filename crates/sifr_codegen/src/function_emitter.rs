use crate::{
    body_contains_yield, collect_mutated_vars_with_sigs, type_contains_typevar, RustEmitter,
    RustExpr, RustItem, RustLiteral, RustParam, RustStmt, RustType, RustTypeParam, Visibility,
};
use sifr_hir::{HirExpr, HirFunction, HirStmt};
use sifr_type_system::{ParamConvention, Type};

impl RustEmitter {
    fn returns_result_none(ty: &Type) -> bool {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Result(ok_ty, _) => matches!(
                crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                Type::None
            ),
            _ => false,
        }
    }

    fn lower_function_type_params(&self, func: &HirFunction) -> Vec<RustTypeParam> {
        if func.type_params.is_empty() {
            return Vec::new();
        }
        let needs_hash_eq = Self::func_needs_hash_eq(func);
        func.type_params
            .iter()
            .map(|tp| {
                let extra = Self::extra_bounds_for_type_param(tp, &func.body);
                let base = if needs_hash_eq {
                    "Clone + std::fmt::Display + PartialOrd + std::hash::Hash + Eq"
                } else {
                    "Clone + std::fmt::Display + PartialOrd"
                };
                RustTypeParam {
                    name: tp.clone(),
                    bounds: vec![format!("{base}{extra}")],
                }
            })
            .collect()
    }

    fn lower_function_param_type(&self, ty: &Type, convention: ParamConvention) -> RustType {
        let base = crate::sifr_type_to_rust_type(ty);
        match convention {
            ParamConvention::Borrow if ty.ownership() != sifr_type_system::OwnershipKind::Copy => {
                RustType::Ref {
                    mutable: false,
                    inner: Box::new(base),
                }
            }
            ParamConvention::MutBorrow
                if ty.ownership() != sifr_type_system::OwnershipKind::Copy =>
            {
                RustType::Ref {
                    mutable: true,
                    inner: Box::new(base),
                }
            }
            _ => base,
        }
    }

    fn lower_function_return_type(
        &self,
        func: &HirFunction,
        is_generator: bool,
    ) -> Option<RustType> {
        if is_generator {
            let yield_ty = if let Type::List(elem) = &func.return_type {
                crate::sifr_type_to_rust_type(elem)
            } else {
                RustType::I64
            };
            if matches!(func.return_type, Type::List(_)) {
                return Some(RustType::Vec(Box::new(yield_ty)));
            }
            return Some(RustType::Impl(format!(
                "Iterator<Item = {}>",
                crate::render_type(&yield_ty)
            )));
        }

        if func.return_type == Type::None {
            return None;
        }
        if let Type::Class {
            name: ref ret_name, ..
        } = func.return_type
        {
            if self.generic_classes.contains(ret_name) && !func.type_params.is_empty() {
                let type_params_in_ret = func
                    .type_params
                    .iter()
                    .filter(|tp| type_contains_typevar(&func.return_type, tp))
                    .collect::<Vec<_>>();
                if !type_params_in_ret.is_empty() {
                    return Some(RustType::Named(format!(
                        "{}<{}>",
                        ret_name,
                        type_params_in_ret
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }
            }
        }
        Some(crate::sifr_type_to_rust_type(&func.return_type))
    }

    fn lower_stmt_expr_strict_for_function(&mut self, expr: &HirExpr, context: &str) -> RustExpr {
        match self.lower_stmt_expr_for_ir(expr) {
            Ok(Some(lowered)) => self.rewrite_stdlib_constant_idents_in_expr(lowered),
            Ok(None) => panic!(
                "structured expression lowering missing for function IR emission ({context}): {expr:?}"
            ),
            Err(err) => {
                self.lowering_stats.expr_lowering_errors += 1;
                panic!(
                    "structured expression lowering failed for function IR emission ({context}): {}; expr={expr:?}",
                    err.message
                );
            }
        }
    }

    fn lower_stmt_strict_for_function(&mut self, stmt: &HirStmt, context: &str) -> Vec<RustStmt> {
        let output_len = self.output.len();
        let lowered = self.capture_structured_stmts(|inner| inner.emit_stmt(stmt));
        if self.output.len() > output_len {
            let emitted = self.output[output_len..].to_string();
            self.output.truncate(output_len);
            panic!(
                "string statement emission leaked during function IR emission ({context}): {}",
                emitted.trim()
            );
        }
        lowered
    }

    fn lower_generator_function_body(
        &mut self,
        func: &HirFunction,
        mutable_param_shadows: &[String],
    ) -> Vec<RustStmt> {
        let yield_ty = if let Type::List(elem) = &func.return_type {
            crate::sifr_type_to_rust_type(elem)
        } else {
            RustType::I64
        };

        let mut body = Vec::new();
        for param_name in mutable_param_shadows {
            body.push(RustStmt::Let {
                mutable: true,
                name: param_name.clone(),
                ty: None,
                value: RustExpr::Ident(param_name.clone()),
            });
        }

        let mut init_stmts: Vec<&HirStmt> = Vec::new();
        let mut while_stmt: Option<(&HirExpr, &Vec<HirStmt>)> = None;
        for stmt in &func.body {
            if while_stmt.is_none() {
                if let HirStmt::While {
                    condition, body, ..
                } = stmt
                {
                    while_stmt = Some((condition, body));
                } else {
                    init_stmts.push(stmt);
                }
            }
        }

        for stmt in init_stmts {
            body.extend(
                self.lower_stmt_strict_for_function(stmt, "generator init statement lowering"),
            );
        }

        let mut closure_body = Vec::new();
        if let Some((condition, while_body_hir)) = while_stmt {
            let has_conditional_yield = !while_body_hir
                .iter()
                .any(|stmt| matches!(stmt, HirStmt::Yield { .. }))
                && while_body_hir.iter().any(|stmt| {
                    if let HirStmt::If { then_body, .. } = stmt {
                        body_contains_yield(then_body)
                    } else {
                        false
                    }
                });

            if has_conditional_yield {
                let mut lowered_while_body = Vec::new();
                lowered_while_body.push(RustStmt::Let {
                    mutable: true,
                    name: "__yielded".to_string(),
                    ty: Some(RustType::Option(Box::new(yield_ty.clone()))),
                    value: RustExpr::Literal(RustLiteral::None),
                });

                for stmt in while_body_hir {
                    if let HirStmt::If {
                        condition: if_cond,
                        then_body,
                        ..
                    } = stmt
                    {
                        if body_contains_yield(then_body) {
                            let mut lowered_then = Vec::new();
                            for then_stmt in then_body {
                                if let HirStmt::Yield { value } = then_stmt {
                                    lowered_then.push(RustStmt::Assign {
                                        target: RustExpr::Ident("__yielded".to_string()),
                                        value: RustExpr::FnCall {
                                            func: Box::new(RustExpr::Path(
                                                vec!["Some".to_string()],
                                            )),
                                            args: vec![self.lower_stmt_expr_strict_for_function(
                                                value,
                                                "conditional generator yield value lowering",
                                            )],
                                        },
                                    });
                                } else {
                                    lowered_then.extend(self.lower_stmt_strict_for_function(
                                        then_stmt,
                                        "conditional generator branch stmt lowering",
                                    ));
                                }
                            }
                            lowered_while_body.push(RustStmt::If {
                                cond: self.lower_stmt_expr_strict_for_function(
                                    if_cond,
                                    "conditional generator if condition lowering",
                                ),
                                then_body: lowered_then,
                                else_body: None,
                            });
                            continue;
                        }
                    }
                    lowered_while_body.extend(self.lower_stmt_strict_for_function(
                        stmt,
                        "conditional generator while stmt lowering",
                    ));
                }

                lowered_while_body.push(RustStmt::IfLet {
                    pattern: "Some(__v)".to_string(),
                    expr: RustExpr::Ident("__yielded".to_string()),
                    then_body: vec![RustStmt::Return(Some(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![RustExpr::Ident("__v".to_string())],
                    }))],
                    else_body: None,
                });

                closure_body.push(RustStmt::While {
                    cond: self.lower_stmt_expr_strict_for_function(
                        condition,
                        "conditional generator while condition lowering",
                    ),
                    body: lowered_while_body,
                });
                closure_body.push(RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None))));
            } else {
                let mut pre_yield: Vec<&HirStmt> = Vec::new();
                let mut yield_expr: Option<&HirExpr> = None;
                let mut post_yield: Vec<&HirStmt> = Vec::new();
                let mut found_yield = false;
                for stmt in while_body_hir {
                    if found_yield {
                        post_yield.push(stmt);
                    } else if let HirStmt::Yield { value } = stmt {
                        yield_expr = Some(value);
                        found_yield = true;
                    } else {
                        pre_yield.push(stmt);
                    }
                }

                let Some(yield_expr) = yield_expr else {
                    panic!(
                        "generator lowering expected a yield statement in while body: {:?}",
                        while_body_hir
                    );
                };

                let mut then_body = Vec::new();
                for stmt in pre_yield {
                    then_body.extend(
                        self.lower_stmt_strict_for_function(
                            stmt,
                            "generator pre-yield stmt lowering",
                        ),
                    );
                }
                then_body.push(RustStmt::Let {
                    mutable: false,
                    name: "__yield_val".to_string(),
                    ty: None,
                    value: self.lower_stmt_expr_strict_for_function(
                        yield_expr,
                        "generator yield value lowering",
                    ),
                });
                for stmt in post_yield {
                    then_body.extend(self.lower_stmt_strict_for_function(
                        stmt,
                        "generator post-yield stmt lowering",
                    ));
                }
                then_body.push(RustStmt::Return(Some(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![RustExpr::Ident("__yield_val".to_string())],
                })));

                closure_body.push(RustStmt::If {
                    cond: self.lower_stmt_expr_strict_for_function(
                        condition,
                        "generator while condition lowering",
                    ),
                    then_body,
                    else_body: Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                        RustLiteral::None,
                    )))]),
                });
            }
        } else {
            closure_body.push(RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None))));
        }

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

        let return_expr = if matches!(func.return_type, Type::List(_)) {
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "Vec".to_string(),
                    "from_iter".to_string(),
                ])),
                args: vec![from_fn_expr],
            }
        } else {
            from_fn_expr
        };
        body.push(RustStmt::Return(Some(return_expr)));
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
        for param in &func.params {
            if param.convention == ParamConvention::Borrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.borrowed_params.insert(param.name.clone());
            }
            if param.convention == ParamConvention::MutBorrow
                && param.ty.ownership() != sifr_type_system::OwnershipKind::Copy
            {
                self.mut_borrowed_params.insert(param.name.clone());
            }
            // Register Callable-typed params for convention-aware call emission
            if let Type::Callable(ref param_types, ref conventions, _) = param.ty {
                let conv_list: Vec<(Type, ParamConvention)> = param_types
                    .iter()
                    .zip(conventions.iter())
                    .map(|(t, c)| (t.clone(), *c))
                    .collect();
                self.callable_var_conventions
                    .insert(param.name.clone(), conv_list);
            }
        }

        let visibility = if !test_mode && module_public && func.name != "main" {
            Visibility::Pub
        } else {
            Visibility::Private
        };
        let is_generator = body_contains_yield(&func.body);
        if is_generator {
            self.generator_functions.insert(func.name.clone());
        }

        let mutable_param_shadows = func
            .params
            .iter()
            .filter(|param| {
                param.convention == ParamConvention::Own && self.mutated_vars.contains(&param.name)
            })
            .map(|param| param.name.clone())
            .collect::<Vec<_>>();

        let params = func
            .params
            .iter()
            .map(|param| RustParam::Named {
                name: param.name.clone(),
                ty: self.lower_function_param_type(&param.ty, param.convention),
            })
            .collect::<Vec<_>>();

        let mut lowered_body = if is_generator {
            self.lower_generator_function_body(func, &mutable_param_shadows)
        } else {
            let mut lowered = Vec::new();
            for param_name in &mutable_param_shadows {
                lowered.push(RustStmt::Let {
                    mutable: true,
                    name: param_name.clone(),
                    ty: None,
                    value: RustExpr::Ident(param_name.clone()),
                });
            }
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
            type_params: self.lower_function_type_params(func),
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
