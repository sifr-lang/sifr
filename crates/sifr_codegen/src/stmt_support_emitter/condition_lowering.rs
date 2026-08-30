use super::{HirExpr, RustEmitter, Type};
impl RustEmitter {
    pub(crate) fn lower_condition_expr_for_ir(
        &mut self,
        condition: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        if let HirExpr::BoolOp { op, values, .. } = condition {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return Ok(None),
            };
            let mut lowered_values = Vec::with_capacity(values.len());
            for value in values {
                let Some(lowered_value) = self.lower_condition_expr_for_ir(value)? else {
                    return Ok(None);
                };
                lowered_values.push(lowered_value);
            }
            let mut lowered_values_iter = lowered_values.into_iter();
            let Some(mut iter_expr) = lowered_values_iter.next() else {
                return Ok(None);
            };
            for rhs in lowered_values_iter {
                iter_expr = crate::RustExpr::BinOp {
                    left: Box::new(iter_expr),
                    op: lowered_op.to_string(),
                    right: Box::new(rhs),
                };
            }
            return Ok(Some(iter_expr));
        }
        if let Some(option_var) = crate::helpers::detect_option_truthiness(condition) {
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(option_var)),
                method: "is_some".to_string(),
                args: vec![],
            }));
        }
        if let Some(option_var) = crate::helpers::detect_not_option_truthiness(condition) {
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Ident(option_var)),
                method: "is_none".to_string(),
                args: vec![],
            }));
        }
        if let HirExpr::UnaryOp { op, operand, .. } = condition {
            if op == "not" && Self::option_inner_type_for_ir(operand.ty()).is_some() {
                let Some(lowered_option_expr) = self.lower_stmt_expr_for_ir(operand)? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_option_expr),
                    method: "is_none".to_string(),
                    args: vec![],
                }));
            }
        }
        if let Some(option_inner_ty) = Self::option_inner_type_for_ir(condition.ty()) {
            let Some(lowered_option_expr) = self.lower_stmt_expr_for_ir(condition)? else {
                return Ok(None);
            };
            if matches!(
                crate::resolve_alias_type_for_plain_call(&option_inner_ty),
                Type::Bool | Type::LiteralBool(_)
            ) {
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_option_expr),
                    method: "is_some_and".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                        is_move: false,
                    }],
                }));
            }
            if matches!(
                crate::resolve_alias_type_for_plain_call(&option_inner_ty),
                Type::Int | Type::LiteralInt(_) | Type::Float
            ) {
                let Some(zero_literal) =
                    Self::zero_literal_for_numeric_truthiness_type_for_ir(&option_inner_ty)
                else {
                    unreachable!("numeric Option truthiness guard must have a zero literal");
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_option_expr),
                    method: "is_some_and".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            op: "!=".to_string(),
                            right: Box::new(zero_literal),
                        }),
                        is_move: false,
                    }],
                }));
            }
            return Ok(Some(crate::RustExpr::MethodCall {
                receiver: Box::new(lowered_option_expr),
                method: "is_some".to_string(),
                args: vec![],
            }));
        }
        if let Some(lowered) = Self::try_lower_collection_truthiness_condition_for_ir(condition) {
            return Ok(Some(lowered));
        }
        if let Some(lowered) = Self::try_lower_numeric_truthiness_condition_for_ir(condition) {
            return Ok(Some(lowered));
        }
        if let Some(lowered) = self.try_lower_borrowed_name_compare_condition_for_ir(condition) {
            return Ok(Some(lowered));
        }
        if self.condition_uses_borrowed_name_for_ir(condition) {
            if let Some(lowered) = self.lower_stmt_expr_for_ir(condition)? {
                return Ok(Some(self.rewrite_stdlib_constant_idents_in_expr(lowered)));
            }
        }
        Ok(self
            .lower_rendered_expr_for_ir(condition)?
            .map(|lowered| self.rewrite_stdlib_constant_idents_in_expr(lowered)))
    }

    pub(crate) fn option_binding_value_expr_for_ir(&self, option_var: &str) -> crate::RustExpr {
        let base = crate::RustExpr::Ident(option_var.to_string());
        if self.borrowed_params.contains(option_var)
            || self.mut_borrowed_params.contains(option_var)
        {
            crate::RustExpr::MethodCall {
                receiver: Box::new(base),
                method: "as_ref".to_string(),
                args: vec![],
            }
        } else if self
            .local_binding_types
            .get(option_var)
            .is_some_and(crate::helpers::is_logically_copy_rust_move_type)
        {
            crate::RustExpr::Clone(Box::new(base))
        } else {
            base
        }
    }

    pub(crate) fn option_binding_pattern_for_ir(&self, option_var: &str) -> String {
        if crate::option_binding_mutability::option_binding_requires_mut(
            option_var,
            &self.mutated_vars,
            &self.borrowed_params,
            &self.mut_borrowed_params,
            &self.local_binding_types,
            &self.recursive_fields,
        ) {
            format!("Some(mut {option_var})")
        } else {
            format!("Some({option_var})")
        }
    }

    pub(crate) fn try_lower_collection_truthiness_condition_for_ir(
        condition: &HirExpr,
    ) -> Option<crate::RustExpr> {
        fn is_collection_truthy_type(ty: &Type) -> bool {
            matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::List(_) | Type::Dict(_, _) | Type::Set(_) | Type::Str | Type::Tuple(_)
            )
        }

        if let HirExpr::Name { name, ty, .. } = condition {
            if is_collection_truthy_type(ty) {
                return Some(crate::RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "is_empty".to_string(),
                        args: vec![],
                    }),
                });
            }
        }

        if let HirExpr::UnaryOp { op, operand, .. } = condition {
            if op == "not" {
                if let HirExpr::Name { name, ty, .. } = operand.as_ref() {
                    if is_collection_truthy_type(ty) {
                        return Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                            method: "is_empty".to_string(),
                            args: vec![],
                        });
                    }
                }
            }
        }

        None
    }

    pub(crate) fn try_lower_numeric_truthiness_condition_for_ir(
        condition: &HirExpr,
    ) -> Option<crate::RustExpr> {
        match condition {
            HirExpr::Name { name, ty, .. } => Some(crate::RustExpr::BinOp {
                left: Box::new(crate::RustExpr::Ident(name.clone())),
                op: "!=".to_string(),
                right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
            }),
            HirExpr::MethodCall {
                object,
                method,
                args,
                ty,
                ..
            } if method == "len" && args.is_empty() => {
                let HirExpr::Name { name, .. } = object.as_ref() else {
                    return None;
                };
                let lhs = crate::RustExpr::FnCall {
                    func: Box::new(crate::RustExpr::Path(vec![
                        "SifrInt".to_string(),
                        "from".to_string(),
                    ])),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "len".to_string(),
                        args: vec![],
                    }],
                };
                Some(crate::RustExpr::BinOp {
                    left: Box::new(lhs),
                    op: "!=".to_string(),
                    right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
                })
            }
            HirExpr::UnaryOp { op, operand, .. } if op == "not" => match operand.as_ref() {
                HirExpr::Name { name, ty, .. } => Some(crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Ident(name.clone())),
                    op: "==".to_string(),
                    right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
                }),
                HirExpr::MethodCall {
                    object,
                    method,
                    args,
                    ty,
                    ..
                } if method == "len" && args.is_empty() => {
                    let HirExpr::Name { name, .. } = object.as_ref() else {
                        return None;
                    };
                    let lhs = crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "SifrInt".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                            method: "len".to_string(),
                            args: vec![],
                        }],
                    };
                    Some(crate::RustExpr::BinOp {
                        left: Box::new(lhs),
                        op: "==".to_string(),
                        right: Box::new(Self::zero_literal_for_numeric_truthiness_type_for_ir(ty)?),
                    })
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn zero_literal_for_numeric_truthiness_type_for_ir(
        ty: &Type,
    ) -> Option<crate::RustExpr> {
        match crate::resolve_alias_type_for_plain_call(ty) {
            Type::Int | Type::LiteralInt(_) => Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec![
                    "SifrInt".to_string(),
                    "from_i64".to_string(),
                ])),
                args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
            }),
            Type::Float => Some(crate::RustExpr::Cast {
                expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Float(0.0))),
                ty: crate::RustType::F64,
            }),
            _ => None,
        }
    }
}
