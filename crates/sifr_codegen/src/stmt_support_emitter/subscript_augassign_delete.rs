impl RustEmitter {
    fn lower_subscript_augassign_stmt_for_ir(
        &mut self,
        object: &str,
        index: &HirExpr,
        op: &str,
        value: &HirExpr,
        object_ty: &Type,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        if !matches!(
            op,
            "+=" | "-=" | "*=" | "/=" | "%=" | "//=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>="
        ) {
            return Ok(None);
        }
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        if op == "+="
            && matches!(
                Self::resolve_alias_type_for_loop_iter(object_ty),
                Type::List(elem_ty)
                    if matches!(
                        crate::resolve_alias_type_for_plain_call(elem_ty.as_ref()),
                        Type::Str | Type::LiteralStr(_)
                    )
            )
        {
            let push_arg = if let HirExpr::StringLiteral(val) = value {
                crate::RustExpr::Ident(format!("{val:?}"))
            } else {
                crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                    method: "as_str".to_string(),
                    args: vec![],
                }
            };
            return Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        crate::RustExpr::Ident(object.to_string()),
                        "__idx_raw",
                    ),
                },
                RustStmt::If {
                    cond: crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                        op: ">=".to_string(),
                        right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                    },
                    then_body: vec![RustStmt::IfLet {
                        pattern: "Some(__elem)".to_string(),
                        expr: crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                            method: "get_mut".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        },
                        then_body: vec![RustStmt::Expr(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                            method: "push_str".to_string(),
                            args: vec![push_arg],
                        })],
                        else_body: None,
                    }],
                    else_body: None,
                },
            ])));
        }
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let Some(lowered_body_stmt) =
            Self::build_subscript_augassign_elem_stmt_for_ir(op, lowered_value)
        else {
            return Ok(None);
        };

        if matches!(
            object_ty,
            Type::Alias { name: alias_name, .. } if alias_name == "__compat_defaultdict_int"
        ) {
            let lowered_index = Self::clone_non_copy_name_expr_for_ir(index, lowered_index);
            return Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__elem".to_string(),
                    ty: None,
                    value: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                            method: "entry".to_string(),
                            args: vec![lowered_index],
                        }),
                        method: "or_insert".to_string(),
                        args: vec![crate::RustExpr::Literal(crate::RustLiteral::Int(0))],
                    },
                },
                lowered_body_stmt,
            ])));
        }

        match Self::resolve_alias_type_for_loop_iter(object_ty) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        crate::RustExpr::Ident(object.to_string()),
                        "__idx_raw",
                    ),
                },
                RustStmt::IfLet {
                    pattern: "Some(__elem)".to_string(),
                    expr: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                        method: "get_mut".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                            ty: crate::RustType::Named("usize".to_string()),
                        }],
                    },
                    then_body: vec![lowered_body_stmt],
                    else_body: None,
                },
            ]))),
            Type::Dict(_, _) => {
                let key_arg = Self::build_dict_lookup_key_arg_for_ir(
                    Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                );
                Ok(Some(RustStmt::IfLet {
                    pattern: "Some(__elem)".to_string(),
                    expr: crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(object.to_string())),
                        method: "get_mut".to_string(),
                        args: vec![key_arg],
                    },
                    then_body: vec![lowered_body_stmt],
                    else_body: None,
                }))
            }
            _ => Ok(None),
        }
    }

    fn lower_delete_stmt_for_ir(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
            return Ok(None);
        };
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        match Self::resolve_alias_type_for_loop_iter(object.ty()) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__delete_target".to_string(),
                    ty: None,
                    value: crate::RustExpr::Ref {
                        mutable: true,
                        expr: Box::new(lowered_object),
                    },
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_raw".to_string(),
                    ty: None,
                    value: lowered_index,
                },
                RustStmt::Let {
                    mutable: false,
                    name: "__idx_norm".to_string(),
                    ty: None,
                    value: crate::build_normalized_list_index_i64_expr(
                        crate::RustExpr::Ident("__delete_target".to_string()),
                        "__idx_raw",
                    ),
                },
                RustStmt::If {
                    cond: crate::RustExpr::BinOp {
                        left: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                            op: ">=".to_string(),
                            right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(0))),
                        }),
                        op: "&&".to_string(),
                        right: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }),
                            op: "<".to_string(),
                            right: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident(
                                    "__delete_target".to_string(),
                                )),
                                method: "len".to_string(),
                                args: vec![],
                            }),
                        }),
                    },
                    then_body: vec![RustStmt::Let {
                        mutable: false,
                        name: "_".to_string(),
                        ty: None,
                        value: crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident(
                                "__delete_target".to_string(),
                            )),
                            method: "remove".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(crate::RustExpr::Ident("__idx_norm".to_string())),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
                        },
                    }],
                    else_body: None,
                },
            ]))),
            Type::Dict(_, _) => Ok(Some(RustStmt::Let {
                mutable: false,
                name: "_".to_string(),
                ty: None,
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_object),
                    method: "remove".to_string(),
                    args: vec![Self::build_dict_lookup_key_arg_for_ir(
                        Self::clone_non_copy_name_expr_for_ir(index, lowered_index),
                    )],
                },
            })),
            _ => Ok(None),
        }
    }

    pub(crate) fn try_lower_structured_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::SubscriptAssign {
            object,
            index,
            value,
            object_ty,
        } = stmt
        else {
            return Ok(false);
        };

        let Some(lowered) =
            self.lower_subscript_assign_stmt_for_ir(object, index, value, object_ty)?
        else {
            return Ok(false);
        };

        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_nested_subscript_assign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::NestedSubscriptAssign {
            object,
            outer_index,
            inner_index,
            value,
            object_ty,
        } = stmt
        else {
            return Ok(false);
        };

        let Type::List(inner) = Self::resolve_alias_type_for_loop_iter(object_ty) else {
            return Ok(false);
        };
        let Type::List(elem) = Self::resolve_alias_type_for_loop_iter(inner) else {
            return Ok(false);
        };

        let Some(lowered) = self.lower_structured_nested_list_subscript_assign_stmt_for_ir(
            object,
            outer_index,
            inner_index,
            value,
            elem,
        )?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_subscript_augassign_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::SubscriptAugAssign {
            object,
            index,
            op,
            value,
            object_ty,
        } = stmt
        else {
            return Ok(false);
        };
        let Some(lowered) =
            self.lower_subscript_augassign_stmt_for_ir(object, index, op, value, object_ty)?
        else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

    pub(crate) fn try_lower_structured_delete_stmt(
        &mut self,
        stmt: &HirStmt,
    ) -> Result<bool, crate::CodegenError> {
        let HirStmt::Delete { object, index } = stmt else {
            return Ok(false);
        };
        let Some(lowered) = self.lower_delete_stmt_for_ir(object, index)? else {
            return Ok(false);
        };
        self.emit_lowered_stmts(std::slice::from_ref(&lowered));
        Ok(true)
    }

}
