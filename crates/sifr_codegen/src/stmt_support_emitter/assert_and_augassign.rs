use super::{HirExpr, HirStmt, RustEmitter, RustStmt, Type};

impl RustEmitter {
    pub(crate) fn lower_exact_int_augassign_stmt_for_ir(
        &self,
        name: &str,
        op: &str,
        source_value: &HirExpr,
        value: crate::RustExpr,
    ) -> Option<crate::RustStmt> {
        if !self.is_registered_sifr_int_local(name) {
            return None;
        }
        let target = crate::RustExpr::Ident(name.to_string());
        let operand = self.coerce_expr_to_sifr_int_comparison_operand(
            self.rewrite_stdlib_constant_idents_in_expr(value),
        );
        let value = match op {
            "+=" | "-=" | "*=" | "&=" | "|=" | "^=" => crate::RustExpr::BinOp {
                left: Box::new(crate::RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(target.clone()),
                }),
                op: op.trim_end_matches('=').to_string(),
                right: Box::new(operand),
            },
            "/=" | "//=" | "%=" => self.sifr_int_known_nonzero_floor_expr(
                if op == "%=" { "%" } else { "/" },
                target.clone(),
                operand,
            ),
            "**=" | "<<=" | ">>=" => {
                let primitive_ty = if op == "**=" { "u32" } else { "usize" };
                let literal = crate::integer_literal_decimal(source_value)?
                    .parse::<i64>()
                    .ok()?;
                crate::RustExpr::MethodCall {
                    receiver: Box::new(target.clone()),
                    method: match op {
                        "**=" => "pow_known_valid",
                        "<<=" => "shl_known_valid",
                        ">>=" => "shr_known_valid",
                        _ => unreachable!(),
                    }
                    .to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(literal))),
                        ty: crate::RustType::Named(primitive_ty.to_string()),
                    }],
                }
            }
            _ => return None,
        };
        Some(crate::RustStmt::Assign { target, value })
    }

    pub(crate) fn try_lower_structured_assert_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Assert { test, msg } = stmt else {
            return Ok(false);
        };

        let Some(lowered_test) = self.lower_condition_expr_for_ir(test)? else {
            return Ok(false);
        };
        let lowered_msg = if let Some(msg_expr) = msg {
            let Some(lowered) = self.lower_rendered_expr_for_ir(msg_expr)? else {
                return Ok(false);
            };
            Some(lowered)
        } else {
            None
        };
        self.push_captured_stmt(&RustStmt::Assert {
            cond: lowered_test,
            msg: lowered_msg,
        });
        Ok(true)
    }

    pub(crate) fn try_lower_structured_aug_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AugAssign { name, op, value } = stmt else {
            return Ok(false);
        };
        let value_ty = Self::resolve_alias_type_for_loop_iter(value.ty());

        if self.is_registered_sifr_int_local(name) {
            let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(false);
            };
            let value_expr = self.rewrite_stdlib_constant_idents_in_expr(value_expr);
            let Some(lowered) =
                self.lower_exact_int_augassign_stmt_for_ir(name, op, value, value_expr)
            else {
                return Ok(false);
            };
            self.emit_lowered_stmts(std::slice::from_ref(&lowered));
            return Ok(true);
        }

        if op == "+=" {
            match value_ty {
                Type::Str => {
                    let arg_expr = if let HirExpr::StringLiteral(val) = value {
                        crate::RustExpr::Verbatim(format!("{val:?}"))
                    } else {
                        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                            return Ok(false);
                        };
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(value_expr))),
                            method: "as_str".to_string(),
                            args: vec![],
                        }
                    };
                    let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "push_str".to_string(),
                        args: vec![arg_expr],
                    });
                    self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                    return Ok(true);
                }
                Type::List(_) => {
                    let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(name.clone())),
                        method: "extend".to_string(),
                        args: vec![value_expr],
                    });
                    self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                    return Ok(true);
                }
                _ => {}
            }
        }

        if op == "**=" {
            let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
                return Ok(false);
            };
            let pow_value = crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(crate::RustExpr::Ident(
                    name.clone(),
                )))),
                method: "pow".to_string(),
                args: vec![crate::RustExpr::Cast {
                    expr: Box::new(value_expr),
                    ty: crate::RustType::Named("u32".to_string()),
                }],
            };
            let lowered = crate::RustStmt::Assign {
                target: crate::RustExpr::Ident(name.clone()),
                value: pow_value,
            };
            self.emit_lowered_stmts(std::slice::from_ref(&lowered));
            return Ok(true);
        }
        let rust_op = match op.as_str() {
            "+=" => "+=",
            "-=" => "-=",
            "*=" => "*=",
            "/=" | "//=" => "/=",
            "%=" => "%=",
            _ => return Ok(false),
        };

        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        let lowered = crate::RustStmt::AugAssign {
            target: crate::RustExpr::Ident(name.clone()),
            op: rust_op.to_string(),
            value: value_expr,
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }
}
