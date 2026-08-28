use super::{HirExpr, RustEmitter, RustExpr, RustStmt, Type};

#[derive(Clone, Copy)]
struct AttributeNestedListDictAssign<'a> {
    object: &'a str,
    field: &'a str,
    outer_index: &'a HirExpr,
    inner_index: &'a HirExpr,
    value: &'a HirExpr,
    key_ty: &'a Type,
    target_elem_ty: &'a Type,
}
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
        let adapted_value = crate::helpers::flatten_option_value_for_target(
            target_elem_ty,
            value.ty(),
            lowered_value.clone(),
        );
        let option_value_adapted = adapted_value != lowered_value;
        let lowered_value = adapted_value;

        let value_is_option = if crate::helpers::is_option_type(value.ty()) {
            true
        } else if let HirExpr::Name { name, ty, .. } = value {
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
        let assign_into_elem = if value_is_option
            && !crate::helpers::is_option_type(target_elem_ty)
            && !option_value_adapted
        {
            RustStmt::IfLet {
                pattern: "Some(__nested_assign_value)".into(),
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
            if let HirExpr::Name { name, ty, .. } = expr {
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
                pattern: "Some(__elem)".into(),
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
                    pattern: "Some(__ii_raw)".into(),
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
                pattern: "Some(__row)".into(),
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
                    pattern: "Some(__oi_raw)".into(),
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

    pub(crate) fn lower_structured_nested_list_dict_subscript_assign_stmt_for_ir(
        &mut self,
        object: &str,
        outer_index: &HirExpr,
        inner_index: &HirExpr,
        value: &HirExpr,
        key_ty: &Type,
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

        let key_needs_clone = matches!(key_ty, Type::Str | Type::TypeVar(_))
            && matches!(inner_index, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name.as_str())
                    || self.mut_borrowed_params.contains(name.as_str()));
        let lowered_inner_index = if key_needs_clone {
            RustExpr::Clone(Box::new(lowered_inner_index))
        } else {
            Self::clone_non_copy_name_expr_for_ir(inner_index, lowered_inner_index)
        };
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let lowered_value = crate::helpers::flatten_option_value_for_target(
            target_elem_ty,
            value.ty(),
            lowered_value,
        );

        let index_is_option = |expr: &HirExpr| {
            if crate::helpers::is_option_type(expr.ty()) {
                return true;
            }
            if let HirExpr::Name { name, ty, .. } = expr {
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

        let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__row".to_string())),
            method: "insert".to_string(),
            args: vec![
                RustExpr::Ident("__nested_assign_key".to_string()),
                RustExpr::Ident("__nested_assign_value".to_string()),
            ],
        });
        let row_body = if inner_index_is_option {
            vec![RustStmt::IfLet {
                pattern: "Some(__nested_assign_key)".into(),
                expr: RustExpr::Ident("__nested_assign_key_opt".to_string()),
                then_body: vec![insert_stmt],
                else_body: None,
            }]
        } else {
            vec![insert_stmt]
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
                pattern: "Some(__row)".into(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(object.to_string())),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                },
                then_body: row_body,
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
                    pattern: "Some(__oi_raw)".into(),
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

        let key_binding_name = if inner_index_is_option {
            "__nested_assign_key_opt"
        } else {
            "__nested_assign_key"
        };
        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: key_binding_name.to_string(),
                ty: None,
                value: lowered_inner_index,
            },
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
        if let Type::Dict(key_ty, target_elem_ty) = Self::resolve_alias_type_for_loop_iter(inner) {
            return self.lower_structured_attribute_nested_list_dict_subscript_assign_stmt_for_ir(
                AttributeNestedListDictAssign {
                    object,
                    field,
                    outer_index,
                    inner_index,
                    value,
                    key_ty,
                    target_elem_ty,
                },
            );
        }
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
        let adapted_value = crate::helpers::flatten_option_value_for_target(
            target_elem_ty.as_ref(),
            value.ty(),
            lowered_value.clone(),
        );
        let option_value_adapted = adapted_value != lowered_value;
        let lowered_value = adapted_value;

        let value_is_option = if crate::helpers::is_option_type(value.ty()) {
            true
        } else if let HirExpr::Name { name, ty, .. } = value {
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
        let assign_into_elem = if value_is_option
            && !crate::helpers::is_option_type(target_elem_ty.as_ref())
            && !option_value_adapted
        {
            RustStmt::IfLet {
                pattern: "Some(__nested_assign_value)".into(),
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
            if let HirExpr::Name { name, ty, .. } = expr {
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
                pattern: "Some(__elem)".into(),
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
                    pattern: "Some(__ii_raw)".into(),
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
                pattern: "Some(__row)".into(),
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
                    pattern: "Some(__oi_raw)".into(),
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

    fn lower_structured_attribute_nested_list_dict_subscript_assign_stmt_for_ir(
        &mut self,
        assign: AttributeNestedListDictAssign<'_>,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let AttributeNestedListDictAssign {
            object,
            field,
            outer_index,
            inner_index,
            value,
            key_ty,
            target_elem_ty,
        } = assign;
        let Some(lowered_outer_index) = self.lower_stmt_expr_for_ir(outer_index)? else {
            return Ok(None);
        };
        let Some(lowered_inner_index) = self.lower_stmt_expr_for_ir(inner_index)? else {
            return Ok(None);
        };
        let Some(lowered_value) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };

        let key_needs_clone = matches!(key_ty, Type::Str | Type::TypeVar(_))
            && matches!(inner_index, HirExpr::Name { name, .. }
                if self.borrowed_params.contains(name.as_str())
                    || self.mut_borrowed_params.contains(name.as_str()));
        let lowered_inner_index = if key_needs_clone {
            RustExpr::Clone(Box::new(lowered_inner_index))
        } else {
            Self::clone_non_copy_name_expr_for_ir(inner_index, lowered_inner_index)
        };
        let lowered_value = Self::clone_non_copy_name_expr_for_ir(value, lowered_value);
        let lowered_value = crate::helpers::flatten_option_value_for_target(
            target_elem_ty,
            value.ty(),
            lowered_value,
        );

        let field_receiver = || RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.to_string(),
        };
        let index_is_option = |expr: &HirExpr| {
            if crate::helpers::is_option_type(expr.ty()) {
                return true;
            }
            if let HirExpr::Name { name, ty, .. } = expr {
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

        let insert_stmt = RustStmt::Expr(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident("__row".to_string())),
            method: "insert".to_string(),
            args: vec![
                RustExpr::Ident("__nested_assign_key".to_string()),
                RustExpr::Ident("__nested_assign_value".to_string()),
            ],
        });
        let row_body = if inner_index_is_option {
            vec![RustStmt::IfLet {
                pattern: "Some(__nested_assign_key)".into(),
                expr: RustExpr::Ident("__nested_assign_key_opt".to_string()),
                then_body: vec![insert_stmt],
                else_body: None,
            }]
        } else {
            vec![insert_stmt]
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
                pattern: "Some(__row)".into(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(field_receiver()),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                },
                then_body: row_body,
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
                    pattern: "Some(__oi_raw)".into(),
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

        let key_binding_name = if inner_index_is_option {
            "__nested_assign_key_opt"
        } else {
            "__nested_assign_key"
        };
        Ok(Some(RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: key_binding_name.to_string(),
                ty: None,
                value: lowered_inner_index,
            },
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
            if !expr.ty().contains_affine_resource()
                && matches!(expr, HirExpr::Name { .. })
                && !crate::helpers::is_copy_type_for_codegen(expr.ty())
            {
                crate::RustExpr::Clone(Box::new(lowered))
            } else {
                lowered
            }
        };

        match Self::resolve_alias_type_for_loop_iter(object_ty) {
            Type::List(element_ty) => Ok(Some(RustStmt::Block(vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__assign_value".to_string(),
                    ty: None,
                    value: crate::helpers::flatten_option_value_for_target(
                        element_ty.as_ref(),
                        value.ty(),
                        clone_non_copy_name(value, lowered_value),
                    ),
                },
                crate::build_list_subscript_assign_stmt(
                    RustExpr::Ident(object.to_string()),
                    lowered_index,
                    RustExpr::Ident("__assign_value".to_string()),
                ),
            ]))),
            Type::Dict(key_ty, value_ty) => {
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
                        value: crate::helpers::flatten_option_value_for_target(
                            value_ty.as_ref(),
                            value.ty(),
                            clone_non_copy_name(value, lowered_value),
                        ),
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
}
