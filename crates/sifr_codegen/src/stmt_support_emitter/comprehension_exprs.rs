use super::{HirExpr, RustEmitter, RustExpr, RustStmt, Type};
impl RustEmitter {
    pub(crate) fn try_lower_comprehension_expr_for_ir(
        &mut self,
        expr: &HirExpr,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        match expr {
            HirExpr::ListComp {
                expr,
                generators,
                ty,
            } if matches!(
                Self::resolve_alias_type_for_loop_iter(ty),
                Type::Any | Type::List(_)
            ) =>
            {
                if generators.is_empty() || generators.iter().any(|(var, _, _)| var.contains(',')) {
                    return Ok(None);
                }
                if generators.iter().any(|(_, iter_expr, _)| {
                    Self::async_iterator_error_type_for_ir(iter_expr).is_some()
                }) {
                    return self.try_lower_async_list_comp_for_ir(expr, generators, ty);
                }

                let result_ident = "__sifr_list_comp".to_string();
                let Some(mut lowered_expr) = self.lower_stmt_expr_for_ir(expr)? else {
                    return Ok(None);
                };
                lowered_expr = crate::ownership_plan::materialize_comprehension_value(
                    expr,
                    lowered_expr,
                    generators,
                );
                if let Type::List(element_ty) = Self::resolve_alias_type_for_loop_iter(ty) {
                    lowered_expr = crate::helpers::adapt_collection_value_for_target(
                        element_ty.as_ref(),
                        expr,
                        lowered_expr,
                    );
                }
                let mut nested_body = vec![RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(result_ident.clone())),
                    method: "push".to_string(),
                    args: vec![lowered_expr],
                })];

                for (var, iter_expr, maybe_filter) in generators.iter().rev() {
                    let Some(iter) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
                        return Ok(None);
                    };
                    let loop_body = if let Some(filter) = maybe_filter {
                        let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                            return Ok(None);
                        };
                        vec![RustStmt::If {
                            cond: lowered_filter,
                            then_body: nested_body,
                            else_body: None,
                        }]
                    } else {
                        nested_body
                    };
                    nested_body = vec![RustStmt::For {
                        var: var.clone(),
                        iter,
                        body: loop_body,
                    }];
                }

                let mut stmts = vec![RustStmt::Let {
                    mutable: true,
                    name: result_ident.clone(),
                    ty: None,
                    value: RustExpr::Vec(vec![]),
                }];
                stmts.extend(nested_body);

                Ok(Some(RustExpr::Block {
                    stmts,
                    expr: Some(Box::new(RustExpr::Ident(result_ident))),
                }))
            }
            HirExpr::DictComp {
                key_expr,
                val_expr,
                generators,
                ty,
            } if generators.len() == 1
                && matches!(
                    Self::resolve_alias_type_for_loop_iter(ty),
                    Type::Any | Type::Dict(_, _)
                ) =>
            {
                let Some((var, iter_expr, maybe_filter)) = generators.first() else {
                    return Ok(None);
                };
                if var.contains(',') {
                    return Ok(None);
                }
                if Self::async_iterator_error_type_for_ir(iter_expr).is_some() {
                    return self
                        .try_lower_async_dict_comp_for_ir(key_expr, val_expr, generators, ty);
                }
                let Some(iter) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
                    return Ok(None);
                };
                let Some(mut lowered_key) = self.lower_stmt_expr_for_ir(key_expr)? else {
                    return Ok(None);
                };
                lowered_key = crate::helpers::clone_dict_key_for_reused_value(
                    key_expr,
                    val_expr,
                    lowered_key,
                );
                let Some(mut lowered_value) = self.lower_stmt_expr_for_ir(val_expr)? else {
                    return Ok(None);
                };
                if let Type::Dict(key_ty, value_ty) = Self::resolve_alias_type_for_loop_iter(ty) {
                    lowered_key = crate::helpers::adapt_collection_value_for_target(
                        key_ty.as_ref(),
                        key_expr,
                        lowered_key,
                    );
                    lowered_value = crate::helpers::adapt_collection_value_for_target(
                        value_ty.as_ref(),
                        val_expr,
                        lowered_value,
                    );
                }

                let result_ident = "__sifr_dict_comp".to_string();
                let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(result_ident.clone())),
                    method: "insert".to_string(),
                    args: vec![lowered_key, lowered_value],
                });

                let loop_body = if let Some(filter) = maybe_filter {
                    let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                        return Ok(None);
                    };
                    vec![RustStmt::If {
                        cond: lowered_filter,
                        then_body: vec![insert_stmt],
                        else_body: None,
                    }]
                } else {
                    vec![insert_stmt]
                };

                Ok(Some(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: true,
                            name: result_ident.clone(),
                            ty: None,
                            value: RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "HashMap".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        RustStmt::For {
                            var: var.clone(),
                            iter,
                            body: loop_body,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::Ident(result_ident))),
                }))
            }
            HirExpr::SetComp {
                expr,
                generators,
                ty,
            } if generators.len() == 1
                && matches!(
                    Self::resolve_alias_type_for_loop_iter(ty),
                    Type::Any | Type::Set(_)
                ) =>
            {
                let Some((var, iter_expr, maybe_filter)) = generators.first() else {
                    return Ok(None);
                };
                if var.contains(',') {
                    return Ok(None);
                }
                if Self::async_iterator_error_type_for_ir(iter_expr).is_some() {
                    return self.try_lower_async_set_comp_for_ir(expr, generators, ty);
                }
                let Some(iter) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
                    return Ok(None);
                };
                let Some(mut lowered_expr) = self.lower_stmt_expr_for_ir(expr)? else {
                    return Ok(None);
                };
                if let Type::Set(element_ty) = Self::resolve_alias_type_for_loop_iter(ty) {
                    lowered_expr = crate::helpers::adapt_collection_value_for_target(
                        element_ty.as_ref(),
                        expr,
                        lowered_expr,
                    );
                }

                let result_ident = "__sifr_set_comp".to_string();
                let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(result_ident.clone())),
                    method: "insert".to_string(),
                    args: vec![lowered_expr],
                });

                let loop_body = if let Some(filter) = maybe_filter {
                    let Some(lowered_filter) = self.lower_stmt_expr_for_ir(filter)? else {
                        return Ok(None);
                    };
                    vec![RustStmt::If {
                        cond: lowered_filter,
                        then_body: vec![insert_stmt],
                        else_body: None,
                    }]
                } else {
                    vec![insert_stmt]
                };

                Ok(Some(RustExpr::Block {
                    stmts: vec![
                        RustStmt::Let {
                            mutable: true,
                            name: result_ident.clone(),
                            ty: None,
                            value: RustExpr::FnCall {
                                func: Box::new(RustExpr::Path(vec![
                                    "HashSet".to_string(),
                                    "new".to_string(),
                                ])),
                                args: vec![],
                            },
                        },
                        RustStmt::For {
                            var: var.clone(),
                            iter,
                            body: loop_body,
                        },
                    ],
                    expr: Some(Box::new(RustExpr::Ident(result_ident))),
                }))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn try_lower_generator_expr_for_ir(
        &mut self,
        value_expr: &HirExpr,
        var: &str,
        iter_expr: &HirExpr,
        filter: Option<&HirExpr>,
        result_ty: &Type,
    ) -> Result<Option<RustExpr>, crate::CodegenError> {
        if var.contains(',')
            || !matches!(
                Self::resolve_alias_type_for_loop_iter(result_ty),
                Type::Any | Type::Iterator(_)
            )
        {
            return Ok(None);
        }

        let Some(iter_chain) = self.lower_comprehension_iter_for_ir(iter_expr)? else {
            return Ok(None);
        };
        let Some(lowered_value_expr) = self.lower_stmt_expr_for_ir(value_expr)? else {
            return Ok(None);
        };
        let lowered_body = if let Some(filter_expr) = filter {
            let Some(lowered_filter_expr) = self.lower_stmt_expr_for_ir(filter_expr)? else {
                return Ok(None);
            };
            RustExpr::If {
                cond: Box::new(lowered_filter_expr),
                then_expr: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                    args: vec![lowered_value_expr],
                }),
                else_expr: Some(Box::new(RustExpr::Literal(crate::RustLiteral::None))),
            }
        } else {
            RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![lowered_value_expr],
            }
        };

        let generator_chain = RustExpr::MethodCall {
            receiver: Box::new(iter_chain),
            method: "filter_map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![crate::RustParam::Named {
                    name: var.to_string(),
                    ty: crate::RustType::Named("_".to_string()),
                }],
                body: Box::new(lowered_body),
                is_move: false,
            }],
        };
        if matches!(
            Self::resolve_alias_type_for_loop_iter(result_ty),
            Type::Iterator(_)
        ) {
            return Ok(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                args: vec![generator_chain],
            }));
        }
        Ok(Some(generator_chain))
    }
}
