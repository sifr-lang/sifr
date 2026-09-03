use super::{HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, Type, is_none_like_result_value};
impl RustEmitter {
    pub(crate) fn try_closure_return_value_for_ir(
        &self,
        mut value: RustExpr,
        source_value_is_none_like: bool,
        return_ty: Option<&Type>,
    ) -> RustExpr {
        let wrap = self
            .try_closure_return_wrap
            .last()
            .cloned()
            .unwrap_or(crate::TryClosureReturnWrap::Direct);
        if source_value_is_none_like
            && return_ty.is_some_and(|return_ty| {
                matches!(
                    crate::resolve_alias_type_for_plain_call(return_ty),
                    Type::Result(ok_ty, _)
                        if matches!(
                            crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
                            Type::None
                        )
                )
            })
        {
            value = RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            };
        }
        let payload = match wrap {
            crate::TryClosureReturnWrap::Direct => value,
            crate::TryClosureReturnWrap::Optional => RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![value],
            },
            crate::TryClosureReturnWrap::ControlFlow { .. } => RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "std".to_string(),
                    "ops".to_string(),
                    "ControlFlow".to_string(),
                    "Break".to_string(),
                ])),
                args: vec![value],
            },
        };
        RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec!["Ok".to_string()])),
            args: vec![payload],
        }
    }

    pub(crate) fn try_closure_unit_return_for_ir(&self, return_ty: Option<&Type>) -> RustExpr {
        self.try_closure_return_value_for_ir(
            RustExpr::Literal(crate::RustLiteral::Unit),
            true,
            return_ty,
        )
    }

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
                let Some(lowered_return_value) =
                    self.lower_return_value_expr_for_ir(value, return_ty_snapshot.as_ref())?
                else {
                    return Ok(false);
                };
                let captured_return = self.try_closure_return_value_for_ir(
                    lowered_return_value,
                    is_none_like_result_value(value),
                    return_ty_snapshot.as_ref(),
                );
                self.push_captured_stmt(&RustStmt::Return(Some(captured_return)));
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
            let captured_return = self.try_closure_unit_return_for_ir(return_ty_snapshot.as_ref());
            self.push_captured_stmt(&RustStmt::Return(Some(captured_return)));
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
        let source_name = crate::render_type(&crate::sifr_type_to_rust_type(source_type));
        let target_name = crate::render_type(&crate::sifr_type_to_rust_type(target));
        if source_name == target_name {
            return lowered;
        }
        let converted =
            self.consuming_value_conversion_for_ir(target, source_type, lowered.clone());
        if converted != lowered {
            return converted;
        }
        RustExpr::MethodCall {
            receiver: Box::new(lowered),
            method: "into".to_string(),
            args: Vec::new(),
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

        if let Some(lowered_union) = self.try_lower_isinstance_union_chain_for_ir(
            condition,
            then_body,
            elif_clauses,
            else_body.as_deref(),
        )? {
            self.push_captured_stmt(&lowered_union);
            return Ok(true);
        }

        if elif_clauses.is_empty() {
            if let Some((var_name, variant_name, enum_name, other_variants)) =
                crate::helpers::detect_isinstance_union(condition)
            {
                let mut needed_variants = vec![variant_name.clone()];
                needed_variants.extend(other_variants.iter().map(|(variant, _)| variant.clone()));
                let enum_name = self.resolve_union_enum_name(&enum_name, &needed_variants);

                let then_mutated = self.body_analysis.mutated_in(then_body);
                let then_binding = if then_mutated.contains(&var_name) {
                    format!("mut {var_name}")
                } else {
                    var_name.clone()
                };
                let Some(lowered_then_body) = self.try_lower_if_branch_for_ir(then_body)? else {
                    return Ok(false);
                };

                let mut arms = vec![crate::RustMatchArm {
                    pattern: format!("{enum_name}::{variant_name}({then_binding})"),
                    bindings: vec![],
                    guard: None,
                    body: lowered_then_body,
                }];

                if let Some(else_body) = else_body {
                    let else_mutated = self.body_analysis.mutated_in(else_body);
                    let else_binding = if else_mutated.contains(&var_name) {
                        format!("mut {var_name}")
                    } else {
                        var_name.clone()
                    };
                    let Some(lowered_else_body) = self.try_lower_if_branch_for_ir(else_body)?
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
        candidates.sort_by_key(|(left, _)| (*left).clone());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_none_bare_return_uses_unit_inside_control_flow_carrier() {
        let mut emitter = RustEmitter::new();
        emitter
            .try_closure_return_wrap
            .push(crate::TryClosureReturnWrap::ControlFlow {
                continue_type: "(SifrInt,)".to_string(),
            });
        let return_ty = Type::Result(Box::new(Type::None), Box::new(Type::Str));

        let lowered = emitter.try_closure_unit_return_for_ir(Some(&return_ty));

        assert_eq!(
            crate::render_expr(&lowered),
            "Ok(::std::ops::ControlFlow::Break(Ok(())))"
        );
    }
}
