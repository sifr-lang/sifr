use super::{
    HandlerMatchCondition, HirExceptHandler, RustEmitter, RustExpr, RustStmt,
    io_error_kind_for_handler,
};
use sifr_type_system::Type;

impl RustEmitter {
    pub(crate) fn try_except_handler_condition_expr(
        handler: &HirExceptHandler,
        err_ident: &str,
        err_ty: &str,
        source_error_type: Option<&Type>,
    ) -> HandlerMatchCondition {
        let Some(error_type) = handler.error_type.as_deref() else {
            return HandlerMatchCondition::Always;
        };
        if crate::try_error_carrier::handler_is_catch_all(handler) {
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
        let resolved_error_type = handler
            .error_resolved_type
            .as_ref()
            .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)));
        if let Some((source, target)) = source_error_type.zip(handler.error_resolved_type.as_ref())
        {
            return if source.is_assignable_to(target) {
                HandlerMatchCondition::Always
            } else {
                HandlerMatchCondition::Unsupported
            };
        }
        if resolved_error_type.as_deref() == Some(err_ty) || error_type == err_ty {
            return HandlerMatchCondition::Always;
        }
        HandlerMatchCondition::Unsupported
    }

    pub(crate) fn lower_try_except_handler_chain_for_ir(
        &mut self,
        handlers: &[HirExceptHandler],
        err_ident: &str,
        carrier_type: Option<&Type>,
        err_ty: &str,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        if let Some(carrier @ Type::Union(members)) = carrier_type.map(Type::resolve_alias) {
            let mut arms = Vec::with_capacity(members.len());
            for member in members {
                let member_ident = "__sifr_try_variant_error";
                let member_ty = crate::render_type(&crate::sifr_type_to_rust_type(member));
                let Some(body) = self.lower_single_try_except_handler_chain_for_ir(
                    handlers,
                    member_ident,
                    &member_ty,
                    Some(member),
                )?
                else {
                    return Ok(None);
                };
                arms.push(crate::RustMatchArm {
                    pattern: format!(
                        "{}::{}({member_ident})",
                        carrier.union_enum_name(),
                        member.union_variant_name()
                    )
                    .into(),
                    bindings: vec![member_ident.to_string()],
                    guard: None,
                    body,
                });
            }
            return Ok(Some(vec![RustStmt::Match {
                expr: RustExpr::Ident(err_ident.to_string()),
                arms,
            }]));
        }
        self.lower_single_try_except_handler_chain_for_ir(handlers, err_ident, err_ty, carrier_type)
    }

    fn lower_single_try_except_handler_chain_for_ir(
        &mut self,
        handlers: &[HirExceptHandler],
        err_ident: &str,
        err_ty: &str,
        source_error_type: Option<&Type>,
    ) -> Result<Option<Vec<RustStmt>>, crate::CodegenError> {
        let mut branches: Vec<(Option<RustExpr>, Vec<RustStmt>)> = Vec::new();
        for handler in handlers {
            let condition = Self::try_except_handler_condition_expr(
                handler,
                err_ident,
                err_ty,
                source_error_type,
            );
            if matches!(condition, HandlerMatchCondition::Unsupported) {
                continue;
            }

            let handler_name = handler.name.as_deref().unwrap_or("_e");
            let handler_binding = if handler_name == "_" {
                None
            } else {
                let cloned_error = RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(err_ident.to_string())),
                    method: "clone".to_string(),
                    args: vec![],
                };
                let binding_value = source_error_type
                    .zip(handler.error_resolved_type.as_ref())
                    .map_or_else(
                        || cloned_error.clone(),
                        |(source, target)| {
                            if source.union_variant_name() == target.union_variant_name() {
                                return cloned_error.clone();
                            }
                            let target_name =
                                crate::render_type(&crate::sifr_type_to_rust_type(target));
                            if target_name == "Error" {
                                return RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        "Error".to_string(),
                                        "new".to_string(),
                                    ])),
                                    args: vec![RustExpr::Field {
                                        expr: Box::new(cloned_error.clone()),
                                        field: "message".to_string(),
                                    }],
                                };
                            }
                            let converted = self.consuming_value_conversion_for_ir(
                                target,
                                source,
                                cloned_error.clone(),
                            );
                            if converted != cloned_error {
                                return converted;
                            }
                            if super::can_construct_error_from_message_for_ir(&target_name) {
                                RustExpr::FnCall {
                                    func: Box::new(RustExpr::Path(vec![
                                        target_name,
                                        "new".to_string(),
                                    ])),
                                    args: vec![RustExpr::Field {
                                        expr: Box::new(cloned_error.clone()),
                                        field: "message".to_string(),
                                    }],
                                }
                            } else {
                                cloned_error.clone()
                            }
                        },
                    );
                Some(binding_value)
            };
            let lowered_handler_body = match self.try_lower_scoped_stmt_block_for_ir(&handler.body)
            {
                Ok(Some(lowered_handler_body)) => lowered_handler_body,
                Ok(None) => return Ok(None),
                Err(err) => return Err(err),
            };
            let mut handler_body = Vec::new();
            if let Some(binding_value) = handler_binding {
                handler_body.push(RustStmt::Let {
                    mutable: self.protected_mutable_place_roots.contains(handler_name),
                    name: handler_name.to_string(),
                    ty: None,
                    value: binding_value,
                });
            }
            handler_body.extend(lowered_handler_body);

            let cond_expr = match condition {
                HandlerMatchCondition::Always => None,
                HandlerMatchCondition::Expr(cond) => Some(cond),
                HandlerMatchCondition::Unsupported => continue,
            };
            branches.push((cond_expr, handler_body));
        }

        let has_unconditional_branch = branches.iter().any(|(cond, _)| cond.is_none());
        let mut current_else = if has_unconditional_branch {
            None
        } else {
            let Some(source_error_type) = source_error_type else {
                return Ok(None);
            };
            let Some(residual) =
                self.unmatched_try_error_return_for_ir(err_ident, source_error_type)
            else {
                return Ok(None);
            };
            Some(vec![residual])
        };
        if branches.is_empty() {
            return Ok(current_else);
        }
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

    fn unmatched_try_error_return_for_ir(
        &self,
        err_ident: &str,
        source_error_type: &Type,
    ) -> Option<RustStmt> {
        let target_error_type = self
            .try_closure_error_type_info
            .last()
            .and_then(Clone::clone)
            .or_else(
                || match self.current_return_type.as_ref()?.resolve_alias() {
                    Type::Result(_, error_type) => Some(error_type.as_ref().clone()),
                    _ => None,
                },
            )?;
        let error = RustExpr::Ident(err_ident.to_string());
        let converted =
            self.consuming_value_conversion_for_ir(&target_error_type, source_error_type, error);
        Some(RustStmt::Return(Some(RustExpr::FnCall {
            func: Box::new(RustExpr::Ident("Err".to_string())),
            args: vec![converted],
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(identity: &str, name: &str, parent_class: Option<&str>) -> Type {
        Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: name.to_string(),
            fields: vec![("message".to_string(), Type::Str)],
            methods: Vec::new(),
            parent_class: parent_class.map(str::to_string),
        }
    }

    #[test]
    fn handler_matching_uses_exact_user_error_ancestry() {
        let base = error("pkg.BaseError", "BaseError", Some("Error"));
        let child = error("pkg.ChildError", "ChildError", Some("pkg.BaseError|Error"));
        let unrelated = error("other.BaseError", "BaseError", Some("Error"));
        let handler = HirExceptHandler {
            error_type: Some("BaseError".to_string()),
            error_resolved_type: Some(base),
            name: Some("error".to_string()),
            body: Vec::new(),
        };

        assert!(matches!(
            RustEmitter::try_except_handler_condition_expr(
                &handler,
                "error",
                "ChildError",
                Some(&child),
            ),
            HandlerMatchCondition::Always
        ));
        assert!(matches!(
            RustEmitter::try_except_handler_condition_expr(
                &handler,
                "error",
                "BaseError",
                Some(&unrelated),
            ),
            HandlerMatchCondition::Unsupported
        ));
    }
}
