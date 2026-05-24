use super::{HirExpr, RustEmitter, RustExpr, RustStmt, Type};
impl RustEmitter {
    pub(crate) fn lower_structured_nested_list_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        outer_index: &HirExpr,
        inner_index: &HirExpr,
        value: &HirExpr,
        target_elem_ty: &Type,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
            return Ok(None);
        };
        let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(inner_index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let lowered_value = if value.ty().rust_type().starts_with('&')
            && !target_elem_ty.rust_type().starts_with('&')
        {
            crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                method: "clone".to_string(),
                args: vec![],
            }
        } else {
            lowered_value
        };

        let value_is_option = if crate::helpers::is_option_type(value.ty()) {
            true
        } else if let HirExpr::Name { name, ty } = value {
            matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) && self
                .local_binding_types
                .get(name)
                .is_some_and(crate::helpers::is_option_type)
        } else {
            false
        };
        let assign_into_elem = if value_is_option && !crate::helpers::is_option_type(target_elem_ty)
        {
            RustStmt::IfLet {
                pattern: "Some(__nested_assign_value)".to_string(),
                expr: RustExpr::Ident("__nested_assign_value".to_string()),
                then_body: vec![RustStmt::Assign {
                    target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                    value: RustExpr::Ident("__nested_assign_value".to_string()),
                }],
                else_body: None,
            }
        } else {
            RustStmt::Assign {
                target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                value: RustExpr::Ident("__nested_assign_value".to_string()),
            }
        };
        let index_is_option = |expr: &HirExpr| {
            if crate::helpers::is_option_type(expr.ty()) {
                return true;
            }
            if let HirExpr::Name { name, ty } = expr {
                return matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Any | Type::Unknown
                ) && self
                    .local_binding_types
                    .get(name)
                    .is_some_and(crate::helpers::is_option_type);
            }
            false
        };
        let outer_index_is_option = index_is_option(outer_index);
        let inner_index_is_option = index_is_option(inner_index);

        let mut inner_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__ii_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(
                RustExpr::Ident("__row".to_string()),
                "__ii_raw",
            ),
        }];
        inner_then_body.push(RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
            },
            then_body: vec![RustStmt::IfLet {
                pattern: "Some(__elem)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__row".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                },
                then_body: vec![assign_into_elem],
                else_body: None,
            }],
            else_body: None,
        });
        let inner_body = if inner_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__ii_raw_opt".to_string(),
                    ty: None,
                    value: lowered_inner_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__ii_raw)".to_string(),
                    expr: RustExpr::Ident("__ii_raw_opt".to_string()),
                    then_body: inner_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut inner_body = vec![RustStmt::Let {
                mutable: false,
                name: "__ii_raw".to_string(),
                ty: None,
                value: lowered_inner_index,
            }];
            inner_body.extend(inner_then_body);
            inner_body
        };

        let mut outer_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__oi_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(
                RustExpr::Ident(object.to_string()),
                "__oi_raw",
            ),
        }];
        outer_then_body.push(RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
            },
            then_body: vec![RustStmt::IfLet {
                pattern: "Some(__row)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                },
                then_body: inner_body,
                else_body: None,
            }],
            else_body: None,
        });

        let outer_body = if outer_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__oi_raw_opt".to_string(),
                    ty: None,
                    value: lowered_outer_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__oi_raw)".to_string(),
                    expr: RustExpr::Ident("__oi_raw_opt".to_string()),
                    then_body: outer_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut outer_body = vec![RustStmt::Let {
                mutable: false,
                name: "__oi_raw".to_string(),
                ty: None,
                value: lowered_outer_index,
            }];
            outer_body.extend(outer_then_body);
            outer_body
        };

        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__nested_assign_value".to_string(),
                ty: None,
                value: lowered_value,
            },
            RustStmt::Block(outer_body),
        ])))
    }

    pub(crate) fn lower_structured_attribute_nested_list_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        field: &str,
        outer_index: &HirExpr,
        inner_index: &HirExpr,
        value: &HirExpr,
        field_ty: &Type,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Type::List(inner) = Self::resolve_alias_type_for_loop_iter(field_ty) else {
            return Ok(None);
        };
        let Type::List(target_elem_ty) = Self::resolve_alias_type_for_loop_iter(inner) else {
            return Ok(None);
        };
        let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
            return Ok(None);
        };
        let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(inner_index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let lowered_value = if value.ty().rust_type().starts_with('&')
            && !target_elem_ty.rust_type().starts_with('&')
        {
            crate::RustExpr::MethodCall {
                receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                method: "clone".to_string(),
                args: vec![],
            }
        } else {
            lowered_value
        };

        let value_is_option = if crate::helpers::is_option_type(value.ty()) {
            true
        } else if let HirExpr::Name { name, ty } = value {
            matches!(
                crate::resolve_alias_type_for_plain_call(ty),
                Type::Any | Type::Unknown
            ) && self
                .local_binding_types
                .get(name)
                .is_some_and(crate::helpers::is_option_type)
        } else {
            false
        };
        let assign_into_elem =
            if value_is_option && !crate::helpers::is_option_type(target_elem_ty.as_ref()) {
                RustStmt::IfLet {
                    pattern: "Some(__nested_assign_value)".to_string(),
                    expr: RustExpr::Ident("__nested_assign_value".to_string()),
                    then_body: vec![RustStmt::Assign {
                        target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                        value: RustExpr::Ident("__nested_assign_value".to_string()),
                    }],
                    else_body: None,
                }
            } else {
                RustStmt::Assign {
                    target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                    value: RustExpr::Ident("__nested_assign_value".to_string()),
                }
            };
        let field_receiver = || RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.to_string(),
        };
        let index_is_option = |expr: &HirExpr| {
            if crate::helpers::is_option_type(expr.ty()) {
                return true;
            }
            if let HirExpr::Name { name, ty } = expr {
                return matches!(
                    crate::resolve_alias_type_for_plain_call(ty),
                    Type::Any | Type::Unknown
                ) && self
                    .local_binding_types
                    .get(name)
                    .is_some_and(crate::helpers::is_option_type);
            }
            false
        };
        let outer_index_is_option = index_is_option(outer_index);
        let inner_index_is_option = index_is_option(inner_index);

        let mut inner_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__ii_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(
                RustExpr::Ident("__row".to_string()),
                "__ii_raw",
            ),
        }];
        inner_then_body.push(RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
            },
            then_body: vec![RustStmt::IfLet {
                pattern: "Some(__elem)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("__row".to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                },
                then_body: vec![assign_into_elem],
                else_body: None,
            }],
            else_body: None,
        });
        let inner_body = if inner_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__ii_raw_opt".to_string(),
                    ty: None,
                    value: lowered_inner_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__ii_raw)".to_string(),
                    expr: RustExpr::Ident("__ii_raw_opt".to_string()),
                    then_body: inner_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut inner_body = vec![RustStmt::Let {
                mutable: false,
                name: "__ii_raw".to_string(),
                ty: None,
                value: lowered_inner_index,
            }];
            inner_body.extend(inner_then_body);
            inner_body
        };

        let mut outer_then_body = vec![RustStmt::Let {
            mutable: false,
            name: "__oi_norm".to_string(),
            ty: None,
            value: crate::build_normalized_list_index_i64_expr(field_receiver(), "__oi_raw"),
        }];
        outer_then_body.push(RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(crate::RustLiteral::Int(0))),
            },
            then_body: vec![RustStmt::IfLet {
                pattern: "Some(__row)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(field_receiver()),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                },
                then_body: inner_body,
                else_body: None,
            }],
            else_body: None,
        });

        let outer_body = if outer_index_is_option {
            vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__oi_raw_opt".to_string(),
                    ty: None,
                    value: lowered_outer_index,
                },
                RustStmt::IfLet {
                    pattern: "Some(__oi_raw)".to_string(),
                    expr: RustExpr::Ident("__oi_raw_opt".to_string()),
                    then_body: outer_then_body,
                    else_body: None,
                },
            ]
        } else {
            let mut outer_body = vec![RustStmt::Let {
                mutable: false,
                name: "__oi_raw".to_string(),
                ty: None,
                value: lowered_outer_index,
            }];
            outer_body.extend(outer_then_body);
            outer_body
        };

        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__nested_assign_value".to_string(),
                ty: None,
                value: lowered_value,
            },
            RustStmt::Block(outer_body),
        ])))
    }

    pub(crate) fn lower_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        index: &HirExpr,
        value: &HirExpr,
        object_ty: &Type,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(lowered_index) = self.lower_stmt_expr_for_ir(index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let clone_non_copy_name = |expr: &HirExpr, lowered: crate::RustExpr| {
            if matches!(expr, HirExpr::Name { .. })
                && !crate::helpers::is_copy_type_for_codegen(expr.ty())
            {
                crate::RustExpr::Clone(Box::new(lowered))
            } else {
                lowered
            }
        };

        match Self::resolve_alias_type_for_loop_iter(object_ty) {
            Type::List(_) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__assign_value".to_string(),
                    ty: None,
                    value: clone_non_copy_name(value, lowered_value),
                },
                crate::build_list_subscript_assign_stmt(
                    RustExpr::Ident(object.to_string()),
                    lowered_index,
                    RustExpr::Ident("__assign_value".to_string()),
                ),
            ]))),
            Type::Dict(key_ty, _) => {
                let key_needs_clone = matches!(key_ty.as_ref(), Type::Str | Type::TypeVar(_))
                    && matches!(index, HirExpr::Name { name, .. }
                        if self.borrowed_params.contains(name.as_str())
                            || self.mut_borrowed_params.contains(name.as_str()));
                let lowered_index = if key_needs_clone {
                    RustExpr::Clone(Box::new(lowered_index))
                } else {
                    clone_non_copy_name(index, lowered_index)
                };
                Ok(Some(RustStmt::Block(vec![
                    RustStmt::Let {
                        mutable: false,
                        name: "__assign_key".to_string(),
                        ty: None,
                        value: lowered_index,
                    },
                    RustStmt::Let {
                        mutable: false,
                        name: "__assign_value".to_string(),
                        ty: None,
                        value: clone_non_copy_name(value, lowered_value),
                    },
                    crate::build_dict_subscript_assign_stmt(
                        RustExpr::Ident(object.to_string()),
                        RustExpr::Ident("__assign_key".to_string()),
                        RustExpr::Ident("__assign_value".to_string()),
                    ),
                ])))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn clone_non_copy_name_expr_for_ir(
        expr: &HirExpr,
        lowered: crate::RustExpr,
    ) -> crate::RustExpr {
        if matches!(expr, HirExpr::Name { .. })
            && !crate::helpers::is_copy_type_for_codegen(expr.ty())
        {
            crate::RustExpr::Clone(Box::new(lowered))
        } else {
            lowered
        }
    }

    pub(crate) fn build_dict_lookup_key_arg_for_ir(
        lowered_index: crate::RustExpr,
    ) -> crate::RustExpr {
        if matches!(
            lowered_index,
            crate::RustExpr::Literal(crate::RustLiteral::Str(_))
        ) {
            lowered_index
        } else {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_index),
            }
        }
    }

    pub(crate) fn build_subscript_augassign_elem_stmt_for_ir(
        op: &str,
        lowered_value: crate::RustExpr,
    ) -> Option<crate::RustStmt> {
        if op == "**=" {
            return Some(crate::RustStmt::Assign {
                target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                    "__elem".to_string(),
                ))),
                value: crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Ident("__elem".to_string())),
                    method: "pow".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_value),
                        ty: crate::RustType::Named("u32".to_string()),
                    }],
                },
            });
        }
        if op == "//=" {
            return Some(crate::RustStmt::Assign {
                target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                    "__elem".to_string(),
                ))),
                value: crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident(
                        "__elem".to_string(),
                    )))),
                    op: "/".to_string(),
                    right: Box::new(lowered_value),
                },
            });
        }
        let rust_op = op.strip_suffix('=')?;
        Some(crate::RustStmt::AugAssign {
            target: crate::RustExpr::Deref(Box::new(crate::RustExpr::Ident("__elem".to_string()))),
            op: rust_op.to_string(),
            value: lowered_value,
        })
    }
}
