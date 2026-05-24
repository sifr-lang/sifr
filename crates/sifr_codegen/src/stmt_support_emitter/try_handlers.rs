use super::{
    io_error_kind_for_handler, HandlerMatchCondition, HirExceptHandler, RustEmitter, RustExpr,
    RustStmt,
};

impl RustEmitter {
    pub(crate) fn try_except_handler_condition_expr(
        handler: &HirExceptHandler,
        err_ident: &str,
        err_ty: &str,
    ) -> HandlerMatchCondition {
        let Some(error_type) = handler.error_type.as_deref() else {
            return HandlerMatchCondition::Always;
        };
        if error_type == "Error" {
            return HandlerMatchCondition::Always;
        }
        if err_ty == "IOError" {
            if error_type == "IOError" {
                return HandlerMatchCondition::Always;
            }
            if let Some(kind) = io_error_kind_for_handler(error_type) {
                return HandlerMatchCondition::Expr(RustExpr::BinOp {
                    left: Box::new(RustExpr::Field {
                        expr: Box::new(RustExpr::Ident(err_ident.to_string())),
                        field: "kind".to_string(),
                    }),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Literal(crate::RustLiteral::Str(
                            kind.to_string(),
                        ))),
                        method: "to_string".to_string(),
                        args: vec![],
                    }),
                });
            }
            return HandlerMatchCondition::Unsupported;
        }
        if error_type == err_ty {
            return HandlerMatchCondition::Always;
        }
        HandlerMatchCondition::Unsupported
    }

    pub(crate) fn lower_try_except_handler_chain_for_ir(
        &mut self,
        handlers: &[HirExceptHandler],
        err_ident: &str,
        err_ty: &str,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let mut branches: Vec<(Option<RustExpr>, Vec<RustStmt>)> = Vec::new();
        for handler in handlers {
            let condition = Self::try_except_handler_condition_expr(handler, err_ident, err_ty);
            if matches!(condition, HandlerMatchCondition::Unsupported) {
                continue;
            }

            let mut handler_body = Vec::new();
            let handler_name = handler.name.as_deref().unwrap_or("_e");
            if handler_name != "_" {
                handler_body.push(RustStmt::Let {
                    mutable: false,
                    name: handler_name.to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(err_ident.to_string())),
                        method: "clone".to_string(),
                        args: vec![],
                    },
                });
            }
            match self.try_lower_stmt_block_for_ir(&handler.body) {
                Ok(Some(lowered_handler_body)) => handler_body.extend(lowered_handler_body),
                Ok(None) => return Ok(None),
                Err(err) => return Err(err),
            }

            let cond_expr = match condition {
                HandlerMatchCondition::Always => None,
                HandlerMatchCondition::Expr(cond) => Some(cond),
                HandlerMatchCondition::Unsupported => continue,
            };
            branches.push((cond_expr, handler_body));
        }

        if branches.is_empty() {
            return Ok(Some(vec![RustStmt::Let {
                mutable: false,
                name: "_".to_string(),
                ty: None,
                value: RustExpr::Ref {
                    mutable: false,
                    expr: Box::new(RustExpr::Ident(err_ident.to_string())),
                },
            }]));
        }

        let mut current_else: Option<Vec<RustStmt>> = None;
        for (cond, body) in branches.into_iter().rev() {
            if let Some(cond) = cond {
                current_else = Some(vec![RustStmt::If {
                    cond,
                    then_body: body,
                    else_body: current_else,
                }]);
            } else {
                current_else = Some(body);
            }
        }
        Ok(Some(current_else.unwrap_or_default()))
    }
}
