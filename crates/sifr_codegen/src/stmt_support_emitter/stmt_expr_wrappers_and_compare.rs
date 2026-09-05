macro_rules! stmt_expr_wrappers_range_index {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::OkWrap { value, ty } = $expr {
            let Some(lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            let Type::Result(ok_ty, _) = crate::resolve_alias_type_for_plain_call(ty) else {
                return Ok(None);
            };
            let lowered_value = $emitter.coerce_local_value_for_target_type_for_ir(
                ok_ty,
                value,
                lowered_value,
            )?;
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![lowered_value],
            }));
        }
        if let HirExpr::ErrWrap { value, ty } = $expr {
            let Some(lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            let Type::Result(_, error_ty) = crate::resolve_alias_type_for_plain_call(ty) else {
                return Ok(None);
            };
            let lowered_value = $emitter.coerce_local_value_for_target_type_for_ir(
                error_ty,
                value,
                lowered_value,
            )?;
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Err".to_string()])),
                args: vec![lowered_value],
            }));
        }
        if let HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } = $expr
        {
            if let Some(lowered) = $emitter.lower_checked_if_expr_for_ir(
                condition,
                then_expr,
                else_expr,
            )? {
                return Ok(Some(lowered));
            }
            let Some(lowered_condition) = $emitter.lower_stmt_expr_for_ir(condition)? else {
                return Ok(None);
            };
            let Some(lowered_then) = $emitter.lower_stmt_expr_for_ir(then_expr)? else {
                return Ok(None);
            };
            let Some(lowered_else) = $emitter.lower_stmt_expr_for_ir(else_expr)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::If {
                cond: Box::new(lowered_condition),
                then_expr: Box::new(lowered_then),
                else_expr: Some(Box::new(lowered_else)),
            }));
        }
        if let HirExpr::RangeLiteral {
            start, end, step, ..
        } = $expr
        {
            return $emitter.try_lower_range_iter_expr_for_ir(start, end, step.as_deref());
        }
        if let HirExpr::Index {
            object, index, ty, ..
        } = $expr
        {
            if let Some(lowered) =
                $emitter.lower_proven_nonempty_head_read_for_ir(object, index, ty)
            {
                return Ok(Some(lowered));
            }
            if let Some(witness) = $emitter.checked_place_read_witness(object, index, ty) {
                return Ok(Some(witness));
            }
            if let Some(lowered) = $emitter.try_lower_nested_list_element_expr($expr) {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = $emitter.try_lower_list_indexed_dict_element_expr($expr) {
                return Ok(Some(lowered));
            }
            if let Some(lowered) = $emitter.try_lower_dict_indexed_list_element_expr($expr) {
                return Ok(Some(lowered));
            }
            if !crate::helpers::is_option_type(ty) {
                return $emitter.lower_non_option_index_expr_for_ir(object, index);
            }
            if let Some(lowered) = $emitter.try_lower_structured_index_expr(object, index, ty)? {
                return Ok(Some(lowered));
            }
            let index_returns_option = crate::helpers::is_option_type(ty);
            let option_inner_ty = object.ty().optional_member_type();
            if let Some(inner_ty) = option_inner_ty {
                let Some(lowered_object) = $emitter.lower_stmt_expr_for_ir(object)? else {
                    return Ok(None);
                };
                let Some(lowered_index) = $emitter.lower_stmt_expr_for_ir(index)? else {
                    return Ok(None);
                };
                let option_index_expr = match inner_ty {
                    Type::Dict(_, value_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                value_ty.as_ref(),
                            );
                        let key_arg = if matches!(index.as_ref(), HirExpr::StringLiteral(_)) {
                            lowered_index
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(lowered_index),
                            }
                        };
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        }
                    }
                    Type::List(element_ty) => {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                element_ty.as_ref(),
                            );
                        crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "get".to_string(),
                                args: vec![crate::RustExpr::Cast {
                                    expr: Box::new(lowered_index),
                                    ty: crate::RustType::Named("usize".to_string()),
                                }],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        }
                    }
                    Type::Bytes => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                            method: "get".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
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
                    },
                    Type::Str => $emitter.lower_string_index_option_with_cache(
                        object,
                        crate::RustExpr::Ident("__v".to_string()),
                        lowered_index,
                    ),
                    _ => return Ok(None),
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::MethodCall {
                        receiver: Box::new(lowered_object),
                        method: "as_ref".to_string(),
                        args: vec![],
                    }),
                    method: "and_then".to_string(),
                    args: vec![crate::RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "__v".to_string(),
                            ty: crate::RustType::Named("_".to_string()),
                        }],
                        body: Box::new(option_index_expr),
                        is_move: false,
                    }],
                }));
            }

            let Some(lowered_object) = $emitter.lower_stmt_expr_for_ir(object)? else {
                return Ok(None);
            };
            let Some(lowered_index) = $emitter.lower_stmt_expr_for_ir(index)? else {
                return Ok(None);
            };
            let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
            match object_ty {
                Type::Dict(_, value_ty) => {
                    let key_arg = if matches!(index.as_ref(), HirExpr::StringLiteral(_)) {
                        lowered_index
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(lowered_index),
                        }
                    };
                    if index_returns_option {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                value_ty.as_ref(),
                            );
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![key_arg],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        }));
                    }
                    return Ok(None);
                }
                Type::List(element_ty) => {
                    let list_index = crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    if index_returns_option {
                        let projection_method =
                            crate::helpers::option_projection_method_for_owned_type(
                                element_ty.as_ref(),
                            );
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![list_index],
                            }),
                            method: projection_method.to_string(),
                            args: vec![],
                        }));
                    }
                    return Ok(None);
                }
                Type::Bytes => {
                    let list_index = crate::RustExpr::Cast {
                        expr: Box::new(lowered_index),
                        ty: crate::RustType::Named("usize".to_string()),
                    };
                    if index_returns_option {
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(lowered_object),
                                method: "get".to_string(),
                                args: vec![list_index],
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
                        }));
                    }
                    return Ok(None);
                }
                Type::Str => {
                    if index_returns_option {
                        return Ok(Some($emitter.lower_string_index_option_with_cache(
                            object,
                            lowered_object,
                            lowered_index,
                        )));
                    }
                    return Err(crate::CodegenError::new(
                        "internal codegen invariant violated: string index produced non-optional result type",
                    ));
                }
                Type::Tuple(items) => {
                    let HirExpr::IntLiteral(idx) = index.as_ref() else {
                        return Ok(None);
                    };
                    let Ok(tuple_index) = usize::try_from(*idx) else {
                        return Ok(None);
                    };
                    let Some(element_ty) = items.get(tuple_index) else {
                        return Ok(None);
                    };
                    let field_expr = crate::RustExpr::Field {
                        expr: Box::new(lowered_object),
                        field: idx.to_string(),
                    };
                    return Ok(Some(
                        if crate::helpers::is_copy_type_for_codegen(element_ty)
                            || !element_ty.supports_derived_clone()
                        {
                            field_expr
                        } else {
                            crate::RustExpr::Clone(Box::new(field_expr))
                        },
                    ));
                }
                Type::Class { methods, .. } | Type::Protocol { methods, .. } => {
                    if let Some((_, getitem_ft)) = methods
                        .iter()
                        .find(|(name, ft)| name == "__getitem__" && ft.params.len() == 1)
                    {
                        let key_convention = getitem_ft.params[0].2;
                        let index_arg = if key_convention.is_shared_borrow()
                            || key_convention.is_mut_borrow()
                        {
                            crate::RustExpr::Ref {
                                mutable: key_convention.is_mut_borrow(),
                                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_index))),
                            }
                        } else {
                            lowered_index
                        };
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_object),
                            method: "__getitem__".to_string(),
                            args: vec![index_arg],
                        }));
                    }
                }
                _ => {}
            }
        }
    }};
}

macro_rules! stmt_expr_contains_unary_compare_bool {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::ContainsOp {
            element,
            collection,
            ..
        } = $expr
        {
            if let Some(lowered) =
                $emitter.try_lower_defaultdict_index_contains_expr(element, collection)
            {
                return Ok(Some(lowered));
            }
            let Some(mut lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                return Ok(None);
            };
            let collection_element_ty = collection.ty().contains_element_type();
            let union_wrapped_element = collection_element_ty.as_ref().and_then(|target_ty| {
                let owned = $emitter.materialize_reusable_value_for_ir(
                    element,
                    lowered_element.clone(),
                );
                crate::helpers::wrap_union_member_expr(target_ty, element.ty(), owned)
            });
            let element_was_union_wrapped = union_wrapped_element.is_some();
            if let Some(wrapped) = union_wrapped_element {
                lowered_element = wrapped;
            }
            if let Some(lowered) = $emitter.try_lower_list_indexed_dict_contains_expr(
                element,
                collection,
                lowered_element.clone(),
            ) {
                return Ok(Some(lowered));
            }
            // Membership only borrows the collection. Emit field storage directly
            // instead of lowering it as an owned field value and cloning it.
            let lowered_collection = if matches!(collection.as_ref(), HirExpr::FieldAccess { .. }) {
                $emitter.emit_storage_path(collection)
            } else {
                $emitter.lower_stmt_expr_for_ir(collection)?
            };
            let Some(lowered_collection) = lowered_collection else {
                return Ok(None);
            };
            let lowered = match crate::resolve_alias_type_for_plain_call(collection.ty()) {
                Type::Dict(_, _) => {
                    let key_arg = if element_was_union_wrapped {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                        }
                    } else if let HirExpr::StringLiteral(value) = element.as_ref() {
                        crate::RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
                    } else if let HirExpr::Name { name, ty, .. } = element.as_ref() {
                        if $emitter.borrowed_params.contains(name)
                            || $emitter.mut_borrowed_params.contains(name)
                        {
                            if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                                $emitter.string_view_expr(element, lowered_element)
                            } else {
                                lowered_element
                            }
                        } else {
                            crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                            }
                        }
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                        }
                    };
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                        method: "contains_key".to_string(),
                        args: vec![key_arg],
                    }
                }
                Type::List(_) | Type::Set(_) | Type::Range => {
                    let element_arg = if matches!(element.as_ref(), HirExpr::Name { name, .. }
                        if $emitter.borrowed_params.contains(name)
                            || $emitter.mut_borrowed_params.contains(name))
                    {
                        lowered_element
                    } else {
                        crate::RustExpr::Ref {
                            mutable: false,
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                        }
                    };
                    crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                        method: "contains".to_string(),
                        args: vec![element_arg],
                    }
                }
                Type::Str => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                    method: "contains".to_string(),
                    args: vec![$emitter.string_view_expr(element, lowered_element)],
                },
                Type::Bytes => crate::RustExpr::Block {
                    stmts: vec![crate::RustStmt::Let {
                        mutable: false,
                        name: "__byte_candidate".to_string(),
                        ty: None,
                        value: lowered_element,
                    }],
                    expr: Some(Box::new(crate::RustExpr::If {
                        cond: Box::new(crate::RustExpr::BinOp {
                            left: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ident(
                                    "__byte_candidate".to_string(),
                                )),
                                op: "<".to_string(),
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    0,
                                ))),
                            }),
                            op: "||".to_string(),
                            right: Box::new(crate::RustExpr::BinOp {
                                left: Box::new(crate::RustExpr::Ident(
                                    "__byte_candidate".to_string(),
                                )),
                                op: ">".to_string(),
                                right: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Int(
                                    255,
                                ))),
                            }),
                        }),
                        then_expr: Box::new(crate::RustExpr::Literal(crate::RustLiteral::Bool(
                            false,
                        ))),
                        else_expr: Some(Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                lowered_collection,
                            ))),
                            method: "contains".to_string(),
                            args: vec![crate::RustExpr::Ref {
                                mutable: false,
                                expr: Box::new(crate::RustExpr::Cast {
                                    expr: Box::new(crate::RustExpr::Ident(
                                        "__byte_candidate".to_string(),
                                    )),
                                    ty: crate::RustType::Named("u8".to_string()),
                                }),
                            }],
                        })),
                    })),
                },
                _ => return Ok(None),
            };
            return Ok(Some(lowered));
        }
        if let HirExpr::UnaryOp { op, operand, .. } = $expr {
            if op == "not" {
                if let Some(option_var) = crate::helpers::detect_option_truthiness(operand) {
                    return Ok(Some(crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Ident(option_var)),
                        method: "is_none".to_string(),
                        args: vec![],
                    }));
                }
                if let Some(lowered) = Self::try_lower_collection_truthiness_condition_for_ir($expr)
                {
                    return Ok(Some(lowered));
                }
            }
            let Some(lowered_operand) = $emitter.lower_stmt_expr_for_ir(operand)? else {
                return Ok(None);
            };
            let lowered = match op.as_str() {
                "not" => crate::RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "~" => crate::RustExpr::UnaryOp {
                    op: "!".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "-" => crate::RustExpr::UnaryOp {
                    op: "-".to_string(),
                    operand: Box::new(crate::RustExpr::Paren(Box::new(lowered_operand))),
                },
                "+" => crate::RustExpr::Paren(Box::new(lowered_operand)),
                _ => return Ok(None),
            };
            return Ok(Some(lowered));
        }
        if let HirExpr::Compare {
            left,
            ops,
            comparators,
            ..
        } = $expr
        {
            if !ops.is_empty() && ops.len() == comparators.len() {
                let mut lhs_expr = left.as_ref();
                let mut lowered_chain: Option<crate::RustExpr> = None;
                for (idx, op) in ops.iter().enumerate() {
                    let Some(rhs_expr) = comparators.get(idx) else {
                        unreachable!("compare ops/comparators lengths checked equal");
                    };
                    let lowered_op = match op.as_str() {
                        "==" | "!=" | "<" | "<=" | ">" | ">=" => op.clone(),
                        "is" => "==".to_string(),
                        "is not" => "!=".to_string(),
                        _ => return Ok(None),
                    };
                    if let Some(lowered_cmp) = $emitter.lower_static_none_comparison(lhs_expr, op, rhs_expr)? {
                        lowered_chain = Some(if let Some(existing) = lowered_chain {
                            crate::RustExpr::BinOp {
                                left: Box::new(existing), op: "&&".to_string(), right: Box::new(lowered_cmp),
                            }
                        } else { lowered_cmp });
                        lhs_expr = rhs_expr;
                        continue;
                    }
                    let left_none_like = matches!(lhs_expr, HirExpr::NoneLiteral)
                        || matches!(
                            crate::resolve_alias_type_for_plain_call(lhs_expr.ty()),
                            Type::None
                        );
                    let right_none_like = matches!(rhs_expr, HirExpr::NoneLiteral)
                        || matches!(
                            crate::resolve_alias_type_for_plain_call(rhs_expr.ty()),
                            Type::None
                        );
                    let option_expr = if left_none_like
                        && crate::helpers::is_option_type(rhs_expr.ty())
                    {
                        Some(rhs_expr)
                    } else if right_none_like && crate::helpers::is_option_type(lhs_expr.ty()) {
                        Some(lhs_expr)
                    } else {
                        None
                    };
                    if matches!(lowered_op.as_str(), "==" | "!=")
                        && let Some(option_expr) = option_expr
                    {
                        let Some(lowered_option) =
                            $emitter.lower_stmt_expr_for_ir(option_expr)?
                        else {
                            return Ok(None);
                        };
                        let lowered_cmp = crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_option),
                            method: if lowered_op == "==" {
                                "is_none".to_string()
                            } else {
                                "is_some".to_string()
                            },
                            args: Vec::new(),
                        };
                        lowered_chain = Some(if let Some(existing) = lowered_chain {
                            crate::RustExpr::BinOp {
                                left: Box::new(existing),
                                op: "&&".to_string(),
                                right: Box::new(lowered_cmp),
                            }
                        } else {
                            lowered_cmp
                        });
                        lhs_expr = rhs_expr;
                        continue;
                    }
                    if matches!(lowered_op.as_str(), "==" | "!=") {
                        if let Some(lowered_cmp) = $emitter
                            .try_lower_string_equality_for_compare(
                                lhs_expr,
                                rhs_expr,
                                &lowered_op,
                            )?
                        {
                            lowered_chain = Some(if let Some(existing) = lowered_chain {
                                crate::RustExpr::BinOp {
                                    left: Box::new(existing),
                                    op: "&&".to_string(),
                                    right: Box::new(lowered_cmp),
                                }
                            } else {
                                lowered_cmp
                            });
                            lhs_expr = rhs_expr;
                            continue;
                        }
                    }
                    let Some(lowered_left) = $emitter.lower_stmt_expr_for_ir(lhs_expr)? else {
                        return Ok(None);
                    };
                    let Some(lowered_right) = $emitter.lower_stmt_expr_for_ir(rhs_expr)? else {
                        return Ok(None);
                    };
                    let lowered_left = if matches!(lhs_expr, HirExpr::ListLiteral { elements, .. } if elements.is_empty()) {
                        crate::lower_expr::typed_empty_list_expr(rhs_expr.ty()).unwrap_or(lowered_left)
                    } else { lowered_left };
                    let lowered_right = if matches!(rhs_expr, HirExpr::ListLiteral { elements, .. } if elements.is_empty()) {
                        crate::lower_expr::typed_empty_list_expr(lhs_expr.ty()).unwrap_or(lowered_right)
                    } else { lowered_right };
                    let left_witness_ty = match lhs_expr {
                        HirExpr::Index {
                            object, index, ty, ..
                        } if $emitter.has_checked_place_read_witness(object, index) => {
                            ty.optional_member_type()
                        }
                        _ => None,
                    };
                    let right_witness_ty = match rhs_expr {
                        HirExpr::Index {
                            object, index, ty, ..
                        } if $emitter.has_checked_place_read_witness(object, index) => {
                            ty.optional_member_type()
                        }
                        _ => None,
                    };
                    let left_is_option = (crate::helpers::is_option_type(lhs_expr.ty())
                        && left_witness_ty.is_none())
                        || crate::stmt_support_emitter::compiler_verified_pop_lowers_as_option_for_ir(
                            lhs_expr,
                        );
                    let right_is_option = (crate::helpers::is_option_type(rhs_expr.ty())
                        && right_witness_ty.is_none())
                        || crate::stmt_support_emitter::compiler_verified_pop_lowers_as_option_for_ir(
                            rhs_expr,
                        );
                    let left_ty = crate::resolve_alias_type_for_plain_call(
                        left_witness_ty.as_ref().unwrap_or_else(|| lhs_expr.ty()),
                    );
                    let right_ty = crate::resolve_alias_type_for_plain_call(
                        right_witness_ty.as_ref().unwrap_or_else(|| rhs_expr.ty()),
                    );
                    let (mut lowered_left, mut lowered_right) =
                        if left_is_option && !right_is_option && !right_none_like {
                            (
                                lowered_left,
                                crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![$emitter.materialize_reusable_value_for_ir(rhs_expr, lowered_right)],
                                },
                            )
                        } else if !left_is_option && right_is_option && !left_none_like {
                            (
                                crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![$emitter.materialize_reusable_value_for_ir(lhs_expr, lowered_left)],
                                },
                                lowered_right,
                            )
                        } else {
                            (lowered_left, lowered_right)
                    };
                    if matches!(lowered_op.as_str(), "==" | "!=") {
                        let mut right_representation_was_wrapped = false;
                        let mut left_representation_was_wrapped = false;
                        if let Some(wrapped) = crate::helpers::wrap_union_member_expr(
                            lhs_expr.ty(),
                            rhs_expr.ty(),
                            lowered_right.clone(),
                        ) {
                            lowered_right = wrapped;
                            right_representation_was_wrapped = true;
                        } else if let Some(wrapped) = crate::helpers::wrap_union_member_expr(
                            rhs_expr.ty(),
                            lhs_expr.ty(),
                            lowered_left.clone(),
                        ) {
                            lowered_left = wrapped;
                            left_representation_was_wrapped = true;
                        }
                        let left_is_shared_borrow =
                            $emitter.comparison_operand_is_shared_borrow(lhs_expr);
                        let right_is_shared_borrow =
                            $emitter.comparison_operand_is_shared_borrow(rhs_expr);
                        if left_is_shared_borrow && !right_is_shared_borrow {
                            lowered_right = $emitter.borrow_comparison_operand(
                                rhs_expr,
                                lowered_right,
                                right_representation_was_wrapped,
                            );
                        } else if right_is_shared_borrow && !left_is_shared_borrow {
                            lowered_left = $emitter.borrow_comparison_operand(
                                lhs_expr,
                                lowered_left,
                                left_representation_was_wrapped,
                            );
                        }
                    } else {
                        let left_is_shared_borrow =
                            $emitter.comparison_operand_is_shared_borrow(lhs_expr);
                        let right_is_shared_borrow =
                            $emitter.comparison_operand_is_shared_borrow(rhs_expr);
                        if left_is_shared_borrow && !right_is_shared_borrow {
                            lowered_right = $emitter.borrow_comparison_operand(
                                rhs_expr,
                                lowered_right,
                                false,
                            );
                        } else if right_is_shared_borrow && !left_is_shared_borrow {
                            lowered_left = $emitter.borrow_comparison_operand(
                                lhs_expr,
                                lowered_left,
                                false,
                            );
                        }
                    }
                    if let Some(lowered_cmp) = crate::lower_exact_integer_float_compare(
                        left_ty,
                        right_ty,
                        &lowered_op,
                        lowered_left.clone(),
                        lowered_right.clone(),
                    ) {
                        lowered_chain = Some(if let Some(existing) = lowered_chain {
                            crate::RustExpr::BinOp {
                                left: Box::new(existing),
                                op: "&&".to_string(),
                                right: Box::new(lowered_cmp),
                            }
                        } else {
                            lowered_cmp
                        });
                        lhs_expr = rhs_expr;
                        continue;
                    }
                    if !left_is_option && !right_is_option {
                        if crate::fixed_width_int_type(left_ty)
                            && !$emitter.is_sifr_int_expr(&lowered_left)
                        {
                            lowered_left = crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "SifrInt".to_string(),
                                    "from".to_string(),
                                ])),
                                args: vec![lowered_left],
                            };
                        }
                        if crate::fixed_width_int_type(right_ty)
                            && !$emitter.is_sifr_int_expr(&lowered_right)
                        {
                            lowered_right = crate::RustExpr::FnCall {
                                func: Box::new(crate::RustExpr::Path(vec![
                                    "SifrInt".to_string(),
                                    "from".to_string(),
                                ])),
                                args: vec![lowered_right],
                            };
                        }
                    }
                    let lowered_cmp = crate::RustExpr::BinOp {
                        left: Box::new(lowered_left),
                        op: lowered_op,
                        right: Box::new(lowered_right),
                    };
                    lowered_chain = Some(if let Some(existing) = lowered_chain {
                        crate::RustExpr::BinOp {
                            left: Box::new(existing),
                            op: "&&".to_string(),
                            right: Box::new(lowered_cmp),
                        }
                    } else {
                        lowered_cmp
                    });
                    lhs_expr = rhs_expr;
                }
                return Ok(lowered_chain.map(|$expr| crate::RustExpr::Paren(Box::new($expr))));
            }
        }
        if let HirExpr::BoolOp { op, values, ty } = $expr {
            let lowered_op = match op.as_str() {
                "and" => "&&",
                "or" => "||",
                _ => return Ok(None),
            };
            if values.is_empty() {
                return Ok(None);
            }
            let mut iter = values.iter();
            let Some(first) = iter.next() else {
                return Ok(None);
            };
            let Some(mut acc) = $crate::stmt_support_emitter::boolop_operand::lower_boolop_operand(
                $emitter, first, ty,
            )? else {
                return Ok(None);
            };
            for value in iter {
                let Some(lowered_value) = $crate::stmt_support_emitter::boolop_operand::lower_boolop_operand(
                    $emitter, value, ty,
                )? else {
                    return Ok(None);
                };
                acc = crate::RustExpr::BinOp {
                    left: Box::new(crate::RustExpr::Paren(Box::new(acc))),
                    op: lowered_op.to_string(),
                    right: Box::new(crate::RustExpr::Paren(Box::new(lowered_value))),
                };
            }
            return Ok(Some(crate::RustExpr::Paren(Box::new(acc))));
        }
    }};
}
