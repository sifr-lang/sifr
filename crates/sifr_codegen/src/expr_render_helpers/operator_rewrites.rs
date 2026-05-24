use super::{HirExpr, RustEmitter, Type};
impl RustEmitter {
    pub(crate) fn try_lower_structured_field_access_expr(
        &mut self,
        object: &HirExpr,
        field: &str,
        ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let lowered_object =
            if let Some(lowered_leaf) = crate::try_lower_leaf_or_name_expr_result(object)? {
                lowered_leaf
            } else {
                let Some(lowered_object) = self.lower_stmt_expr_for_ir(object)? else {
                    return Ok(None);
                };
                lowered_object
            };

        Ok(Some(self.lower_field_access_expr_with_lowered_object(
            object,
            field,
            ty,
            lowered_object,
        )))
    }

    pub(crate) fn try_lower_structured_class_binop_expr(
        &mut self,
        left: &HirExpr,
        op: &str,
        right: &HirExpr,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let method_name = match op {
            "+" => "__add__",
            "-" => "__sub__",
            _ => return Ok(None),
        };
        let Type::Class { methods, .. } = crate::resolve_alias_type_for_plain_call(left.ty())
        else {
            return Ok(None);
        };
        let Some((_, method_sig)) = methods.iter().find(|(name, _)| name == method_name) else {
            return Ok(None);
        };

        let lowered_left = if let HirExpr::FieldAccess { object, field, ty } = left {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(left)?
        };
        let Some(lowered_left) = lowered_left else {
            return Ok(None);
        };

        let lowered_right = if let HirExpr::FieldAccess { object, field, ty } = right {
            self.try_lower_structured_field_access_expr(object, field, ty)?
        } else {
            crate::try_lower_leaf_or_name_expr_result(right)?
        };
        let Some(lowered_right) = lowered_right else {
            return Ok(None);
        };

        let left_expr = crate::RustExpr::Ref {
            mutable: false,
            expr: Box::new(lowered_left),
        };

        let right_expr = if method_sig
            .params
            .first()
            .is_some_and(|(_, _, conv)| conv.is_shared_borrow())
        {
            crate::RustExpr::Ref {
                mutable: false,
                expr: Box::new(lowered_right),
            }
        } else {
            lowered_right
        };

        Ok(Some(crate::RustExpr::BinOp {
            left: Box::new(left_expr),
            op: op.to_string(),
            right: Box::new(right_expr),
        }))
    }

    pub(crate) fn try_lower_structured_index_expr(
        &mut self,
        object: &HirExpr,
        index: &HirExpr,
        result_ty: &Type,
    ) -> Result<Option<crate::RustExpr>, crate::CodegenError> {
        let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
        let option_inner_ty = if let Type::Union(members) = object_ty {
            let mut inner = None;
            for member in members {
                let member = crate::resolve_alias_type_for_plain_call(member);
                if matches!(member, Type::None) {
                    continue;
                }
                if inner.is_some() {
                    inner = None;
                    break;
                }
                inner = Some(member);
            }
            inner
        } else {
            None
        };
        let index_base_ty = if matches!(
            object_ty,
            Type::Dict(_, _) | Type::List(_) | Type::Str | Type::Tuple(_)
        ) {
            Some(object_ty)
        } else if matches!(
            option_inner_ty,
            Some(Type::Dict(_, _) | Type::List(_) | Type::Str | Type::Tuple(_))
        ) {
            option_inner_ty
        } else {
            None
        };
        let Some(index_base_ty) = index_base_ty else {
            return Ok(None);
        };

        let suppress_self_field_clone = matches!(object, HirExpr::FieldAccess { object: inner, .. }
            if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"))
            && self.pending_self_field_clone_suppression == 0;
        if suppress_self_field_clone {
            self.pending_self_field_clone_suppression += 1;
        }

        let lowered = (|| -> Result<Option<crate::RustExpr>, crate::CodegenError> {
            let lowered_object = if let HirExpr::FieldAccess {
                object: inner,
                field,
                ty,
            } = object
            {
                self.try_lower_structured_field_access_expr(inner, field, ty)?
            } else {
                crate::try_lower_leaf_or_name_expr_result(object)?
            };
            let lowered_object = match lowered_object {
                Some(expr) => expr,
                None => match self.try_lower_registry_expr_strict(object) {
                    Some(expr) => expr,
                    None => return Ok(None),
                },
            };

            let lowered_index = match crate::try_lower_leaf_or_name_expr_result(index)? {
                Some(expr) => expr,
                None => match self.try_lower_registry_expr_strict(index) {
                    Some(expr) => expr,
                    None => return Ok(None),
                },
            };

            let build_inner_index = |container_expr: crate::RustExpr| -> Option<crate::RustExpr> {
                let lowered_expr = match index_base_ty {
                    Type::Dict(key_ty, value_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                value_ty.as_ref(),
                            );
                        let key_is_string_like = matches!(
                            crate::resolve_alias_type_for_plain_call(key_ty.as_ref()),
                            Type::Str | Type::LiteralStr(_)
                        );
                        let key_arg = if let HirExpr::StringLiteral(value) = index {
                            crate::RustExpr::Ident(format!("{value:?}"))
                        } else if key_is_string_like {
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                    lowered_index.clone(),
                                ))),
                                method: "as_str".to_string(),
                                args: vec![],
                            }
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_index.clone()),
                            }
                        };
                        let lowered_get = crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(container_expr),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        };
                        if crate::helpers::is_option_type(value_ty.as_ref()) {
                            crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_get),
                                method: "and_then".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__v".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                    is_move: false,
                                }],
                            }
                        } else {
                            lowered_get
                        }
                    }
                    Type::List(element_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                element_ty.as_ref(),
                            );
                        let object_name = "__sifr_index_list".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            object_name.clone(),
                                                        )),
                                                        method: "len".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
                                }),
                                method: projection_method.to_string(),
                                args: vec![],
                            })),
                        }
                    }
                    Type::Bytes => {
                        let object_name = "__sifr_index_bytes".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(crate::RustExpr::Ident(
                                                            object_name.clone(),
                                                        )),
                                                        method: "len".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                    method: "get".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
                                }),
                                method: "map".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "__byte".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::Cast {
                                        expr: Box::new(crate::RustExpr::Deref(Box::new(
                                            crate::RustExpr::Ident("__byte".to_string()),
                                        ))),
                                        ty: crate::RustType::Named("u8".to_string()),
                                    }),
                                    is_move: false,
                                }],
                            })),
                        }
                    }
                    Type::Str => {
                        let object_name = "__sifr_index_str".to_string();
                        let index_name = "__sifr_index_i".to_string();
                        let normalized_name = "__sifr_index_norm".to_string();
                        crate::RustExpr::Block {
                            stmts: vec![
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: object_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::Ref {
                                        mutable: false,
                                        expr: Box::new(container_expr),
                                    },
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: index_name.clone(),
                                    ty: None,
                                    value: lowered_index.clone(),
                                },
                                crate::RustStmt::Let {
                                    mutable: false,
                                    name: normalized_name.clone(),
                                    ty: None,
                                    value: crate::RustExpr::If {
                                        cond: Box::new(crate::RustExpr::BinOp {
                                            left: Box::new(crate::RustExpr::Ident(
                                                index_name.clone(),
                                            )),
                                            op: "<".to_string(),
                                            right: Box::new(crate::RustExpr::Literal(
                                                crate::RustLiteral::Int(0),
                                            )),
                                        }),
                                        then_expr: Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::BinOp {
                                                left: Box::new(crate::RustExpr::Cast {
                                                    expr: Box::new(crate::RustExpr::MethodCall {
                                                        receiver: Box::new(
                                                            crate::RustExpr::MethodCall {
                                                                receiver: Box::new(
                                                                    crate::RustExpr::Ident(
                                                                        object_name.clone(),
                                                                    ),
                                                                ),
                                                                method: "chars".to_string(),
                                                                args: vec![],
                                                            },
                                                        ),
                                                        method: "count".to_string(),
                                                        args: vec![],
                                                    }),
                                                    ty: crate::RustType::I64,
                                                }),
                                                op: "+".to_string(),
                                                right: Box::new(crate::RustExpr::Ident(
                                                    index_name.clone(),
                                                )),
                                            }),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        }),
                                        else_expr: Some(Box::new(crate::RustExpr::Cast {
                                            expr: Box::new(crate::RustExpr::Ident(index_name)),
                                            ty: crate::RustType::Named("usize".to_string()),
                                        })),
                                    },
                                },
                            ],
                            expr: Some(Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident(object_name)),
                                        method: "chars".to_string(),
                                        args: vec![],
                                    }),
                                    method: "nth".to_string(),
                                    args: vec![crate::RustExpr::Ident(normalized_name)],
                                }),
                                method: "map".to_string(),
                                args: vec![crate::RustExpr::Closure {
                                    params: vec![crate::RustParam::Named {
                                        name: "c".to_string(),
                                        ty: crate::RustType::Named("_".to_string()),
                                    }],
                                    body: Box::new(crate::RustExpr::MethodCall {
                                        receiver: Box::new(crate::RustExpr::Ident("c".to_string())),
                                        method: "to_string".to_string(),
                                        args: vec![],
                                    }),
                                    is_move: false,
                                }],
                            })),
                        }
                    }
                    Type::Tuple(elements) => {
                        let HirExpr::IntLiteral(idx) = index else {
                            return None;
                        };
                        let Ok(idx) = usize::try_from(*idx) else {
                            return None;
                        };
                        if idx >= elements.len() {
                            return None;
                        }
                        crate::RustExpr::Field {
                            expr: Box::new(container_expr),
                            field: idx.to_string(),
                        }
                    }
                    _ => return None,
                };
                Some(lowered_expr)
            };

            if let Some(inner_ty) = option_inner_ty {
                if !matches!(
                    inner_ty,
                    Type::Dict(_, _) | Type::List(_) | Type::Bytes | Type::Str | Type::Tuple(_)
                ) {
                    return Ok(None);
                }
                let Some(mut inner_expr) =
                    build_inner_index(crate::RustExpr::Ident("__v".to_string()))
                else {
                    return Ok(None);
                };
                if let (Type::Tuple(elements), HirExpr::IntLiteral(raw_idx)) = (inner_ty, index) {
                    if let Ok(idx) = usize::try_from(*raw_idx) {
                        if let Some(element_ty) = elements.get(idx) {
                            if !crate::helpers::is_copy_type_for_codegen(element_ty) {
                                inner_expr = crate::RustExpr::MethodCall {
                                    receiver: Box::new(inner_expr),
                                    method: "clone".to_string(),
                                    args: vec![],
                                };
                            }
                        }
                    }
                }
                let projection_method = if matches!(inner_ty, Type::Tuple(_)) {
                    "map"
                } else {
                    "and_then"
                };
                let option_expr = crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_object))),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }),
                    method: projection_method.to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(inner_expr),
                        is_move: false,
                    }],
                };
                if crate::helpers::is_option_type(result_ty) {
                    return Ok(Some(option_expr));
                }
                if matches!(inner_ty, Type::Tuple(_)) {
                    return Ok(Some(Self::lower_proven_index_option_expr_for_ir(
                        option_expr,
                        "__sifr_index_value",
                        "compiler-verified tuple index should be in range",
                    )));
                }
                return Err(crate::CodegenError::new(
                    "internal codegen invariant violated: index on optional list/dict/bytes/str produced non-optional result type",
                ));
            }

            let Some(lowered_expr) = build_inner_index(lowered_object) else {
                return Ok(None);
            };
            if crate::helpers::is_option_type(result_ty) || matches!(index_base_ty, Type::Tuple(_))
            {
                return Ok(Some(lowered_expr));
            }
            match index_base_ty {
                Type::List(_) | Type::Bytes | Type::Str => Ok(Some(Self::lower_proven_index_option_expr_for_ir(
                    lowered_expr,
                    "__sifr_index_value",
                    "compiler-verified index should be in range",
                ))),
                Type::Dict(_, _) => Err(crate::CodegenError::new(
                    "internal codegen invariant violated: dict index produced non-optional result type",
                )),
                _ => Err(crate::CodegenError::new(
                    "internal codegen invariant violated: list/dict/bytes/str index produced non-optional result type",
                )),
            }
        })();

        if suppress_self_field_clone && self.pending_self_field_clone_suppression > 0 {
            self.pending_self_field_clone_suppression -= 1;
        }
        lowered
    }
}
