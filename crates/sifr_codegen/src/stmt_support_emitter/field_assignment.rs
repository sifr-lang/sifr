use super::{HirExpr, HirStmt, RustEmitter, RustExpr, RustStmt, Type};

impl RustEmitter {
    pub(crate) fn lower_field_assign_stmt_for_block(
        &mut self,
        object: &str,
        field: &str,
        field_ty: &Type,
        value: &HirExpr,
    ) -> Result<Option<RustStmt>, crate::CodegenError> {
        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(None);
        };
        let value_expr = self.clone_field_storage_name_expr_for_ir(value, value_expr);
        let value_expr = self.adapt_field_assign_value_for_recursive_storage(
            object,
            field,
            field_ty,
            value_expr,
            value.ty(),
        );
        Ok(Some(RustStmt::Assign {
            target: RustExpr::Field {
                expr: Box::new(Self::object_name_expr_for_ir(object)),
                field: field.to_string(),
            },
            value: value_expr,
        }))
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
        let value_expr = if crate::helpers::is_logically_copy_rust_move_type(field_ty)
            && matches!(value, HirExpr::Name { .. })
            && Self::rust_expr_is_reusable_place_for_ir(&value_expr)
        {
            crate::RustExpr::Clone(Box::new(value_expr))
        } else {
            self.clone_field_storage_name_expr_for_ir(value, value_expr)
        };
        let value_expr =
            crate::helpers::flatten_option_value_for_target(field_ty, value.ty(), value_expr);
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
        let value_expr = self.clone_field_storage_name_expr_for_ir(value, value_expr);
        let value_expr = crate::helpers::flatten_option_value_for_target(
            nested_field_ty,
            value.ty(),
            value_expr,
        );
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
        let member = field_ty.optional_member_type()?;
        let Type::Class { name, .. } = member.resolve_alias() else {
            return None;
        };
        Some(name.clone())
    }

    pub(crate) fn optional_class_name(ty: &Type) -> Option<String> {
        let member = ty.optional_member_type()?;
        let Type::Class { name, .. } = member.resolve_alias() else {
            return None;
        };
        Some(name.clone())
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
        let Some(value_expr) = self.lower_stmt_expr_for_ir(value)? else {
            return Ok(false);
        };

        let receiver = crate::RustExpr::Field {
            expr: Box::new(Self::object_name_expr_for_ir(object)),
            field: field.clone(),
        };
        let lowered = match field_ty {
            Type::List(element_ty) => {
                let Some(index_expr) = self.lower_stmt_expr_for_ir(index)? else {
                    return Ok(false);
                };
                let index_expr = Self::clone_non_copy_name_expr_for_ir(index, index_expr);
                crate::build_list_subscript_assign_stmt(
                    receiver,
                    index_expr,
                    crate::helpers::flatten_option_value_for_target(
                        element_ty.as_ref(),
                        value.ty(),
                        value_expr,
                    ),
                )
            }
            Type::Dict(key_ty, value_ty) => {
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
                crate::RustStmt::Expr(crate::RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "insert".to_string(),
                    args: vec![
                        index_expr,
                        crate::helpers::flatten_option_value_for_target(
                            value_ty.as_ref(),
                            value.ty(),
                            value_expr,
                        ),
                    ],
                })
            }
            _ => return Ok(false),
        };
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
}
