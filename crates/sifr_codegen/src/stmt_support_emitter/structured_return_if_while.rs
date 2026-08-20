use super::{
    is_none_like_result_value, queries, HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, Type,
};
impl RustEmitter {
    pub(crate) fn try_lower_structured_return_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Return { value } = stmt else {
            return Ok(false);
        };
        let return_ty_snapshot = self.current_return_type.clone();

        if let Some(value) = value {
            if self.emission_ctx.in_display_impl && self.try_closure_depth == 0 {
                let Some(display_expr) =
                    self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                else {
                    return Ok(false);
                };
                self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::MacroCall {
                    name: "write".to_string(),
                    args: vec![
                        crate::RustExpr::Ident("f".to_string()),
                        crate::RustExpr::Literal(crate::RustLiteral::Str("{}".to_string())),
                        display_expr,
                    ],
                })));
                return Ok(true);
            }
            if self.try_closure_depth > 0 {
                let wrap_option = self
                    .try_closure_option_wrap
                    .last()
                    .copied()
                    .unwrap_or(false);

                let Some(mut lowered_return_value) =
                    self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                else {
                    return Ok(false);
                };

                if !wrap_option {
                    if let Some(return_ty) = return_ty_snapshot.as_ref() {
                        if let Type::Result(ok_ty, _) =
                            crate::resolve_alias_type_for_plain_call(return_ty)
                        {
                            let value_is_none_like = is_none_like_result_value(value);
                            if value_is_none_like
                                && matches!(
                                    crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                                    Type::None
                                )
                            {
                                lowered_return_value = crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                                    args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                                };
                            }
                        }
                    }
                }

                let try_payload = if wrap_option {
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![lowered_return_value],
                    }
                } else {
                    lowered_return_value
                };
                self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![try_payload],
                })));
                return Ok(true);
            }

            let Some(lowered_return_value) =
                self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
            else {
                return Ok(false);
            };
            self.push_captured_stmt(&RustStmt::Return(Some(lowered_return_value)));
            return Ok(true);
        }

        if self.try_closure_depth > 0 {
            let wrap_option = self
                .try_closure_option_wrap
                .last()
                .copied()
                .unwrap_or(false);
            if wrap_option {
                self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                    args: vec![crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    }],
                })));
            } else {
                let direct_result_none = return_ty_snapshot.as_ref().is_some_and(|ret_ty| {
                    match crate::resolve_alias_type_for_plain_call(ret_ty) {
                        Type::Result(ok_ty, _) => matches!(
                            crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                            Type::None
                        ),
                        _ => false,
                    }
                });
                if direct_result_none {
                    self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::FnCall {
                            func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                            args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                        }],
                    })));
                } else {
                    self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
                    })));
                }
            }
        } else if self.emission_ctx.in_display_impl {
            self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Unit)],
            })));
        } else {
            self.push_captured_stmt(&RustStmt::Return(None));
        }
        Ok(true)
    }

    pub(crate) fn try_lower_structured_raise_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Raise { value } = stmt else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        let lowered = self.coerce_raised_error_for_ir(value, lowered);
        self.push_captured_stmt(&RustStmt::Return(Some(crate::RustExpr::FnCall {
            func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
            args: vec![lowered],
        })));
        Ok(true)
    }

    pub(crate) fn coerce_raised_error_for_ir(
        &self,
        value: &HirExpr,
        lowered: RustExpr,
    ) -> RustExpr {
        self.coerce_error_type_for_ir(value.ty(), lowered)
    }

    pub(crate) fn coerce_error_type_for_ir(
        &self,
        source_type: &Type,
        lowered: RustExpr,
    ) -> RustExpr {
        let target = self
            .try_closure_error_type_info
            .last()
            .and_then(Option::as_ref)
            .or_else(|| {
                let Type::Result(_, error) = self.current_return_type.as_ref()?.resolve_alias()
                else {
                    return None;
                };
                Some(error.as_ref())
            });
        let Some(target) = target else {
            return lowered;
        };
        let converted = self.consuming_value_upcast_for_ir(target, source_type, lowered.clone());
        if converted != lowered {
            return converted;
        }
        let source_name = crate::render_type(&crate::sifr_type_to_rust_type(source_type));
        let target_name = crate::render_type(&crate::sifr_type_to_rust_type(target));
        if source_name == target_name {
            lowered
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(lowered),
                method: "into".to_string(),
                args: Vec::new(),
            }
        }
    }

    pub(crate) fn try_lower_structured_if_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } = stmt
        else {
            return Ok(false);
        };

        if elif_clauses.is_empty() && else_body.is_none() {
            if let HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } = condition
            {
                if let HirExpr::WalrusExpr { name, value, ty } = left.as_ref() {
                    let Some(lowered_value) = self.lower_rendered_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    let walrus_compare_expr = HirExpr::Compare {
                        left: Box::new(HirExpr::Name {
                            name: name.clone(),
                            binding_id: None,
                            ty: ty.clone(),
                        }),
                        ops: ops.clone(),
                        comparators: comparators.clone(),
                        ty: condition.ty().clone(),
                    };
                    let Some(lowered_cond) =
                        self.lower_rendered_expr_for_ir(&walrus_compare_expr)?
                    else {
                        return Ok(false);
                    };
                    let Some(lowered_then_body) = self.try_lower_if_branch_for_ir(then_body)?
                    else {
                        return Ok(false);
                    };

                    self.push_captured_stmt(&RustStmt::Let {
                        mutable: false,
                        name: name.clone(),
                        ty: None,
                        value: lowered_value,
                    });
                    self.push_captured_stmt(&RustStmt::If {
                        cond: lowered_cond,
                        then_body: lowered_then_body,
                        else_body: None,
                    });
                    return Ok(true);
                }
            }
        }

        if let Some((var_name, first_variant, first_enum_name, _)) =
            crate::helpers::detect_isinstance_union(condition)
        {
            let mut branch_specs: Vec<(String, &[HirStmt])> = vec![(first_variant, then_body)];
            let mut needed_variants = vec![branch_specs[0].0.clone()];
            let mut all_isinstance = true;
            for (elif_cond, elif_body) in elif_clauses {
                let Some((elif_var, elif_variant, _, _)) =
                    crate::helpers::detect_isinstance_union(elif_cond)
                else {
                    all_isinstance = false;
                    break;
                };
                if elif_var != var_name {
                    all_isinstance = false;
                    break;
                }
                needed_variants.push(elif_variant.clone());
                branch_specs.push((elif_variant, elif_body.as_slice()));
            }
            if all_isinstance {
                let enum_name = self.resolve_union_enum_name(&first_enum_name, &needed_variants);
                let mut nested_else = if let Some(else_body) = else_body {
                    let remaining_variants = self
                        .union_enums
                        .get(&enum_name)
                        .map(|members| {
                            members
                                .iter()
                                .map(Type::union_variant_name)
                                .filter(|variant| !needed_variants.contains(variant))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(false);
                    };
                    if remaining_variants.len() == 1 {
                        let else_mutated = queries::collect_mutated_vars(else_body, None);
                        let else_binding = if else_mutated.contains(&var_name) {
                            format!("mut {var_name}")
                        } else {
                            var_name.clone()
                        };
                        Some(vec![RustStmt::IfLet {
                            pattern: format!(
                                "{enum_name}::{}({else_binding})",
                                remaining_variants[0]
                            ),
                            expr: RustExpr::Ident(var_name.clone()),
                            then_body: lowered_else_body,
                            else_body: Some(vec![RustStmt::Expr(RustExpr::FormatMacro {
                                name: "unreachable".to_string(),
                                format_str:
                                    "sifr union narrowing fell through exhaustive branch chain"
                                        .to_string(),
                                args: vec![],
                            })]),
                        }])
                    } else {
                        Some(lowered_else_body)
                    }
                } else {
                    None
                };

                for (variant_name, body) in branch_specs.iter().rev() {
                    let mutated = queries::collect_mutated_vars(body, None);
                    let binding = if mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    let Some(lowered_body) = self.try_lower_stmt_block_for_ir(body)? else {
                        return Ok(false);
                    };
                    nested_else = Some(vec![RustStmt::IfLet {
                        pattern: format!("{enum_name}::{variant_name}({binding})"),
                        expr: RustExpr::Ident(var_name.clone()),
                        then_body: lowered_body,
                        else_body: nested_else,
                    }]);
                }

                let Some(root) = nested_else.and_then(|stmts| stmts.into_iter().next()) else {
                    return Ok(false);
                };
                self.push_captured_stmt(&root);
                return Ok(true);
            }
        }

        if elif_clauses.is_empty() {
            if let Some((var_name, variant_name, enum_name, other_variants)) =
                crate::helpers::detect_isinstance_union(condition)
            {
                let mut needed_variants = vec![variant_name.clone()];
                needed_variants.extend(other_variants.iter().map(|(variant, _)| variant.clone()));
                let enum_name = self.resolve_union_enum_name(&enum_name, &needed_variants);

                let then_mutated = queries::collect_mutated_vars(then_body, None);
                let then_binding = if then_mutated.contains(&var_name) {
                    format!("mut {var_name}")
                } else {
                    var_name.clone()
                };
                let Some(lowered_then_body) = self.try_lower_stmt_block_for_ir(then_body)? else {
                    return Ok(false);
                };

                let mut arms = vec![crate::RustMatchArm {
                    pattern: format!("{enum_name}::{variant_name}({then_binding})"),
                    bindings: vec![],
                    guard: None,
                    body: lowered_then_body,
                }];

                if let Some(else_body) = else_body {
                    let else_mutated = queries::collect_mutated_vars(else_body, None);
                    let else_binding = if else_mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    let Some(lowered_else_body) = self.try_lower_stmt_block_for_ir(else_body)?
                    else {
                        return Ok(false);
                    };
                    if other_variants.len() == 1 {
                        let (other_variant, _) = &other_variants[0];
                        arms.push(crate::RustMatchArm {
                            pattern: format!("{enum_name}::{other_variant}({else_binding})"),
                            bindings: vec![],
                            guard: None,
                            body: lowered_else_body,
                        });
                    } else {
                        arms.push(crate::RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: vec![],
                            guard: None,
                            body: lowered_else_body,
                        });
                    }
                } else {
                    arms.push(crate::RustMatchArm {
                        pattern: "_".to_string(),
                        bindings: vec![],
                        guard: None,
                        body: vec![],
                    });
                }

                self.push_captured_stmt(&RustStmt::Match {
                    expr: RustExpr::Ident(var_name),
                    arms,
                });
                return Ok(true);
            }
        }

        let Some(lowered_if_stmt) = self.try_lower_if_stmt_for_ir(
            condition,
            then_body,
            elif_clauses,
            else_body.as_deref(),
        )?
        else {
            return Ok(false);
        };
        self.push_captured_stmt(&lowered_if_stmt);
        Ok(true)
    }

    pub(crate) fn resolve_union_enum_name(
        &self,
        preferred: &str,
        needed_variants: &[String],
    ) -> String {
        if self.union_enums.contains_key(preferred) {
            return preferred.to_string();
        }
        let mut candidates = self.union_enums.iter().collect::<Vec<_>>();
        candidates.sort_by(|(left, _), (right, _)| left.cmp(right));
        for (candidate, members) in candidates {
            if needed_variants.iter().all(|needed| {
                members
                    .iter()
                    .any(|member| member.union_variant_name() == *needed)
            }) {
                return candidate.clone();
            }
        }
        preferred.to_string()
    }
}
