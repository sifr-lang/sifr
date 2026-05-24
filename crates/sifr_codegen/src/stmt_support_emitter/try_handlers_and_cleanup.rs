use super::{
    io_error_kind_for_handler, HandlerMatchCondition, HirExceptHandler, HirExpr, HirStmt,
    RustEmitter, RustExpr, RustStmt, Type,
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

    pub(crate) fn try_lower_structured_field_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::FieldAssign {
            object,
            field,
            field_ty,
            value,
        } = stmt
        else {
            return Ok(false);
        };

        let target = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.clone(),
        };

        if self.current_class_name.as_deref() == Some("deque") && field == "_data" {
            if let HirExpr::ListLiteral { elements, .. } = value {
                let value_expr = if elements.is_empty() {
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "VecDeque".to_string(),
                            "new".to_string(),
                        ])),
                        args: vec![],
                    }
                } else {
                    let Some(list_expr) = self.lower_stmt_expr_for_ir(value)? else {
                        return Ok(false);
                    };
                    crate::RustExpr::FnCall {
                        func: Box::new(crate::RustExpr::Path(vec![
                            "VecDeque".to_string(),
                            "from".to_string(),
                        ])),
                        args: vec![list_expr],
                    }
                };
                let lowered = crate::RustStmt::Assign {
                    target,
                    value: value_expr,
                };
                self.emit_lowered_stmts(std::slice::from_ref(&lowered));
                return Ok(true);
            }
        }

        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };
        let value_expr = self.adapt_field_assign_value_for_recursive_storage(
            object,
            field,
            field_ty,
            value_expr,
            value.ty(),
        );
        let lowered = crate::RustStmt::Assign {
            target,
            value: value_expr,
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn lower_nested_field_assign_stmt_for_ir(
        &mut self,
        object: &str,
        field: &str,
        field_ty: &Type,
        nested_field: &str,
        nested_field_ty: &Type,
        value: &HirExpr,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let value_expr = self.adapt_field_assign_value_for_recursive_storage(
            object,
            nested_field,
            nested_field_ty,
            value_expr,
            value.ty(),
        );
        let outer_target = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.to_string(),
        };
        if crate::helpers::is_option_type(field_ty) {
            return Ok(Some(crate::RustStmt::IfLet {
                pattern: "Some(__nested_obj)".to_string(),
                expr: crate::RustExpr::MethodCall {
                    receiver: Box::new(outer_target),
                    method: "as_mut".to_string(),
                    args: vec![],
                },
                then_body: vec![RustStmt::Assign {
                    target: crate::RustExpr::Field {
                        expr: Box::new(crate::RustExpr::Ident("__nested_obj".to_string())),
                        field: nested_field.to_string(),
                    },
                    value: value_expr,
                }],
                else_body: None,
            }));
        }
        Ok(Some(RustStmt::Assign {
            target: crate::RustExpr::Field {
                expr: Box::new(outer_target),
                field: nested_field.to_string(),
            },
            value: value_expr,
        }))
    }

    pub(crate) fn try_lower_structured_nested_field_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::NestedFieldAssign {
            object,
            field,
            field_ty,
            nested_field,
            nested_field_ty,
            value,
        } = stmt
        else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_nested_field_assign_stmt_for_ir(
            object,
            field,
            field_ty,
            nested_field,
            nested_field_ty,
            value,
        )?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn optional_recursive_class_name(field_ty: &Type) -> Option<String> {
        let Type::Union(members) = field_ty.resolve_alias() else {
            return None;
        };
        let mut class_name: Option<String> = None;
        let mut has_none = false;
        for member in members {
            match member.resolve_alias() {
                Type::Class { name, .. } => class_name = Some(name.clone()),
                Type::None => has_none = true,
                _ => {}
            }
        }
        if has_none {
            class_name
        } else {
            None
        }
    }

    pub(crate) fn optional_class_name(ty: &Type) -> Option<String> {
        let Type::Union(members) = ty.resolve_alias() else {
            return None;
        };
        if members.len() != 2 {
            return None;
        }
        let mut class_name: Option<String> = None;
        let mut has_none = false;
        for member in members {
            match member.resolve_alias() {
                Type::Class { name, .. } => class_name = Some(name.clone()),
                Type::None => has_none = true,
                _ => return None,
            }
        }
        if has_none {
            class_name
        } else {
            None
        }
    }

    pub(crate) fn recursive_field_needs_boxing(
        &self,
        object: &str,
        field: &str,
        field_ty: &Type,
    ) -> bool {
        if object == "self"
            && self.current_class_name.as_ref().is_some_and(|class_name| {
                self.recursive_fields
                    .contains(&(class_name.clone(), field.to_string()))
            })
        {
            return true;
        }
        if let Some(class_name) = Self::optional_recursive_class_name(field_ty) {
            return self
                .recursive_fields
                .contains(&(class_name, field.to_string()));
        }
        false
    }

    pub(crate) fn adapt_field_assign_value_for_recursive_storage(
        &self,
        object: &str,
        field: &str,
        field_ty: &Type,
        value_expr: RustExpr,
        value_ty: &Type,
    ) -> RustExpr {
        if !self.recursive_field_needs_boxing(object, field, field_ty) {
            return value_expr;
        }

        let Some(class_name) = Self::optional_recursive_class_name(field_ty) else {
            if !Self::is_box_new_call_expr_for_ir(&value_expr) {
                return RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![value_expr],
                };
            }
            return value_expr;
        };

        let value_class_matches = matches!(
            value_ty.resolve_alias(),
            Type::Class { name, .. } if name == &class_name
        );

        if value_class_matches {
            let boxed_expr = if Self::is_box_new_call_expr_for_ir(&value_expr) {
                value_expr
            } else {
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                    args: vec![value_expr],
                }
            };
            return RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec!["Some".to_string()])),
                args: vec![boxed_expr],
            };
        }

        if Self::optional_class_name(value_ty).as_deref() == Some(class_name.as_str()) {
            return RustExpr::MethodCall {
                receiver: Box::new(value_expr),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![crate::RustParam::Named {
                        name: "__sifr_recursive_value".to_string(),
                        ty: crate::RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::FnCall {
                        func: Box::new(RustExpr::Path(vec!["Box".to_string(), "new".to_string()])),
                        args: vec![RustExpr::Ident("__sifr_recursive_value".to_string())],
                    }),
                    is_move: false,
                }],
            };
        }

        value_expr
    }

    pub(crate) fn try_lower_structured_attribute_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AttributeSubscriptAssign {
            object,
            field,
            index,
            value,
            field_ty,
        } = stmt
        else {
            return Ok(false);
        };
        let Type::Dict(key_ty, _) = field_ty else {
            return Ok(false);
        };

        let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
            && matches!(index, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name.as_str()) || self.mut_borrowed_params.contains(name.as_str()));
        let key_is_non_copy_name = matches!(index, HirExpr::Name { .. })
            && matches!(
                crate::resolve_alias_type_for_plain_call(index.ty()),
                Type::Str | Type::LiteralStr(_)
            );

        let Some(mut index_expr) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(false);
        };
        if key_needs_clone || key_is_non_copy_name {
            index_expr = crate::RustExpr::Clone(Box::new(index_expr));
        }
        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };

        let receiver = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.clone(),
        };
        let lowered = crate::RustStmt::Expr(crate::RustExpr::MethodCall {
            receiver: Box::new(receiver),
            method: "insert".to_string(),
            args: vec![index_expr, value_expr],
        });
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_attribute_nested_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::AttributeNestedSubscriptAssign {
            object,
            field,
            outer_index,
            inner_index,
            value,
            field_ty,
        } = stmt
        else {
            return Ok(false);
        };

        let Some(lowered) = self
            .lower_structured_attribute_nested_list_subscript_assign_stmt_for_ir(
                object,
                field,
                outer_index,
                inner_index,
                value,
                field_ty,
            )?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_assert_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Assert { test, msg } = stmt else {
            return Ok(false);
        };

        let Some(lowered_test) = self.lower_rendered_expr_for_ir(test)? else {
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

        if op == "+=" {
            match value_ty {
                Type::Str => {
                    let arg_expr = if let HirExpr::StringLiteral(val) = value {
                        crate::RustExpr::Ident(format!("{val:?}"))
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

pub(super) fn is_result_int_division_error_type(ty: &Type) -> bool {
    let Type::Result(ok_ty, err_ty) = ty else {
        return false;
    };
    matches!(
        crate::resolve_alias_type_for_plain_call(ok_ty.as_ref()),
        Type::Int | Type::LiteralInt(_)
    ) && matches!(
        crate::resolve_alias_type_for_plain_call(err_ty.as_ref()),
        Type::Class { name, .. } if name == "DivisionError"
    )
}

pub(super) fn result_int_to_sifr_int_rust_type(ty: &Type) -> crate::RustType {
    let Type::Result(_, err_ty) = ty else {
        return crate::RustType::Named(ty.rust_type());
    };
    crate::RustType::Result(
        Box::new(crate::RustType::Named("SifrInt".to_string())),
        Box::new(crate::sifr_type_to_rust_type(err_ty)),
    )
}

pub(super) fn inject_async_with_return_cleanup(
    stmts: &[RustStmt],
    receiver: &RustExpr,
) -> Vec<RustStmt> {
    stmts
        .iter()
        .flat_map(|stmt| inject_async_with_return_cleanup_stmt(stmt, receiver))
        .collect()
}

pub(super) fn inject_async_with_return_cleanup_stmt(
    stmt: &RustStmt,
    receiver: &RustExpr,
) -> Vec<RustStmt> {
    match stmt {
        RustStmt::Return(Some(value)) => vec![RustStmt::Return(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_async_with_return".to_string(),
                    ty: None,
                    value: value.clone(),
                },
                RustStmt::Expr(async_with_exit_call(receiver.clone(), "Return")),
            ],
            expr: Some(Box::new(RustExpr::Ident(
                "__sifr_async_with_return".to_string(),
            ))),
        }))],
        RustStmt::Return(None) => vec![
            RustStmt::Expr(async_with_exit_call(receiver.clone(), "Return")),
            RustStmt::Return(None),
        ],
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => vec![RustStmt::If {
            cond: cond.clone(),
            then_body: inject_async_with_return_cleanup(then_body, receiver),
            else_body: else_body
                .as_ref()
                .map(|body| inject_async_with_return_cleanup(body, receiver)),
        }],
        RustStmt::IfLet {
            pattern,
            expr,
            then_body,
            else_body,
        } => vec![RustStmt::IfLet {
            pattern: pattern.clone(),
            expr: expr.clone(),
            then_body: inject_async_with_return_cleanup(then_body, receiver),
            else_body: else_body
                .as_ref()
                .map(|body| inject_async_with_return_cleanup(body, receiver)),
        }],
        RustStmt::Match { expr, arms } => vec![RustStmt::Match {
            expr: expr.clone(),
            arms: arms
                .iter()
                .map(|arm| crate::RustMatchArm {
                    pattern: arm.pattern.clone(),
                    bindings: arm.bindings.clone(),
                    guard: arm.guard.clone(),
                    body: inject_async_with_return_cleanup(&arm.body, receiver),
                })
                .collect(),
        }],
        RustStmt::With { items, body } => vec![RustStmt::With {
            items: items.clone(),
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        RustStmt::Block(body) => {
            vec![RustStmt::Block(inject_async_with_return_cleanup(
                body, receiver,
            ))]
        }
        RustStmt::For { var, iter, body } => vec![RustStmt::For {
            var: var.clone(),
            iter: iter.clone(),
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        RustStmt::While { cond, body } => vec![RustStmt::While {
            cond: cond.clone(),
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        RustStmt::Loop { body } => vec![RustStmt::Loop {
            body: inject_async_with_return_cleanup(body, receiver),
        }],
        _ => vec![stmt.clone()],
    }
}

pub(super) fn async_with_exit_call(receiver: RustExpr, cause_variant: &str) -> RustExpr {
    RustExpr::Try(Box::new(RustExpr::Await(Box::new(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "__aexit__".to_string(),
        args: vec![RustExpr::Ref {
            mutable: false,
            expr: Box::new(RustExpr::Ident(format!("AsyncExitCause::{cause_variant}"))),
        }],
    }))))
}

pub(super) fn inject_async_for_early_exit_cleanup(
    stmts: &[RustStmt],
    receiver: &RustExpr,
    close_error_ty: &Type,
) -> Vec<RustStmt> {
    inject_async_for_early_exit_cleanup_with_breaks(stmts, receiver, close_error_ty, true)
}

pub(super) fn inject_async_for_early_exit_cleanup_with_breaks(
    stmts: &[RustStmt],
    receiver: &RustExpr,
    close_error_ty: &Type,
    include_breaks: bool,
) -> Vec<RustStmt> {
    stmts
        .iter()
        .flat_map(|stmt| {
            inject_async_for_early_exit_cleanup_stmt(stmt, receiver, close_error_ty, include_breaks)
        })
        .collect()
}

pub(super) fn inject_async_for_early_exit_cleanup_stmt(
    stmt: &RustStmt,
    receiver: &RustExpr,
    close_error_ty: &Type,
    include_breaks: bool,
) -> Vec<RustStmt> {
    match stmt {
        RustStmt::Break if include_breaks => vec![
            RustStmt::Expr(async_for_close_call(receiver.clone(), close_error_ty)),
            RustStmt::Break,
        ],
        RustStmt::Return(Some(value)) => vec![RustStmt::Return(Some(RustExpr::Block {
            stmts: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_async_for_return".to_string(),
                    ty: None,
                    value: value.clone(),
                },
                RustStmt::Expr(async_for_close_call(receiver.clone(), close_error_ty)),
            ],
            expr: Some(Box::new(RustExpr::Ident(
                "__sifr_async_for_return".to_string(),
            ))),
        }))],
        RustStmt::Return(None) => vec![
            RustStmt::Expr(async_for_close_call(receiver.clone(), close_error_ty)),
            RustStmt::Return(None),
        ],
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => vec![RustStmt::If {
            cond: cond.clone(),
            then_body: inject_async_for_early_exit_cleanup_with_breaks(
                then_body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
            else_body: else_body.as_ref().map(|body| {
                inject_async_for_early_exit_cleanup_with_breaks(
                    body,
                    receiver,
                    close_error_ty,
                    include_breaks,
                )
            }),
        }],
        RustStmt::IfLet {
            pattern,
            expr,
            then_body,
            else_body,
        } => vec![RustStmt::IfLet {
            pattern: pattern.clone(),
            expr: expr.clone(),
            then_body: inject_async_for_early_exit_cleanup_with_breaks(
                then_body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
            else_body: else_body.as_ref().map(|body| {
                inject_async_for_early_exit_cleanup_with_breaks(
                    body,
                    receiver,
                    close_error_ty,
                    include_breaks,
                )
            }),
        }],
        RustStmt::Match { expr, arms } => vec![RustStmt::Match {
            expr: expr.clone(),
            arms: arms
                .iter()
                .map(|arm| crate::RustMatchArm {
                    pattern: arm.pattern.clone(),
                    bindings: arm.bindings.clone(),
                    guard: arm.guard.clone(),
                    body: inject_async_for_early_exit_cleanup_with_breaks(
                        &arm.body,
                        receiver,
                        close_error_ty,
                        include_breaks,
                    ),
                })
                .collect(),
        }],
        RustStmt::With { items, body } => vec![RustStmt::With {
            items: items.clone(),
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
        }],
        RustStmt::Block(body) => vec![RustStmt::Block(
            inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                include_breaks,
            ),
        )],
        RustStmt::For { var, iter, body } => vec![RustStmt::For {
            var: var.clone(),
            iter: iter.clone(),
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                false,
            ),
        }],
        RustStmt::While { cond, body } => vec![RustStmt::While {
            cond: cond.clone(),
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                false,
            ),
        }],
        RustStmt::Loop { body } => vec![RustStmt::Loop {
            body: inject_async_for_early_exit_cleanup_with_breaks(
                body,
                receiver,
                close_error_ty,
                false,
            ),
        }],
        _ => vec![stmt.clone()],
    }
}

pub(super) fn async_for_close_call(receiver: RustExpr, close_error_ty: &Type) -> RustExpr {
    let close_call = RustExpr::Await(Box::new(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "aclose".to_string(),
        args: vec![],
    }));
    if matches!(close_error_ty.resolve_alias(), Type::Never) {
        close_call
    } else {
        RustExpr::Try(Box::new(close_call))
    }
}
