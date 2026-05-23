macro_rules! stmt_expr_wrappers_range_index {
    ($emitter:ident, $expr:ident) => {{
        if let HirExpr::OkWrap { value, .. } = $expr {
            let Some(lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
            return Ok(Some(crate::RustExpr::FnCall {
                func: Box::new(crate::RustExpr::Path(vec!["Ok".to_string()])),
                args: vec![lowered_value],
            }));
        }
        if let HirExpr::ErrWrap { value, .. } = $expr {
            let Some(lowered_value) = $emitter.lower_stmt_expr_for_ir(value)? else {
                return Ok(None);
            };
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
            let Some(lowered_start) = $emitter.lower_stmt_expr_for_ir(start)? else {
                return Ok(None);
            };
            let Some(lowered_end) = $emitter.lower_stmt_expr_for_ir(end)? else {
                return Ok(None);
            };
            let lowered_range = crate::RustExpr::Range {
                start: Box::new(lowered_start),
                end: Box::new(lowered_end),
            };
            if let Some(step_expr) = step {
                let Some(lowered_step) = $emitter.lower_stmt_expr_for_ir(step_expr)? else {
                    return Ok(None);
                };
                return Ok(Some(crate::RustExpr::MethodCall {
                    receiver: Box::new(lowered_range),
                    method: "step_by".to_string(),
                    args: vec![crate::RustExpr::Cast {
                        expr: Box::new(lowered_step),
                        ty: crate::RustType::Named("usize".to_string()),
                    }],
                }));
            }
            return Ok(Some(lowered_range));
        }
        if let HirExpr::Index {
            object, index, ty, ..
        } = $expr
        {
            if !crate::helpers::is_option_type(ty) {
                if let Some(lowered) = $emitter.lower_non_option_index_expr_for_ir(object, index)? {
                    return Ok(Some(lowered));
                }
            }
            if let Some(lowered) = $emitter.try_lower_structured_index_expr(object, index, ty)? {
                return Ok(Some(lowered));
            }
            let object_ty = crate::resolve_alias_type_for_plain_call(object.ty());
            let index_returns_option = crate::helpers::is_option_type(ty);
            let option_inner_ty = if let Type::Union(members) = object_ty {
                let mut non_none = members.iter().filter(|m| !matches!(m, Type::None));
                let first = non_none.next();
                if non_none.next().is_none() && members.iter().any(|m| matches!(m, Type::None)) {
                    first
                } else {
                    None
                }
            } else {
                None
            };
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
                    Type::Str => crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(crate::RustExpr::MethodCall {
                                receiver: Box::new(crate::RustExpr::Ident("__v".to_string())),
                                method: "chars".to_string(),
                                args: vec![],
                            }),
                            method: "nth".to_string(),
                            args: vec![crate::RustExpr::Cast {
                                expr: Box::new(lowered_index),
                                ty: crate::RustType::Named("usize".to_string()),
                            }],
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
                    },
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
                    let indexed_expr = crate::RustExpr::Index {
                        expr: Box::new(lowered_object),
                        index: Box::new(key_arg),
                    };
                    return Ok(Some(
                        if crate::helpers::is_copy_type_for_codegen(value_ty.as_ref()) {
                            indexed_expr
                        } else {
                            crate::RustExpr::Clone(Box::new(indexed_expr))
                        },
                    ));
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
                    let indexed_expr = crate::RustExpr::Index {
                        expr: Box::new(lowered_object),
                        index: Box::new(list_index),
                    };
                    return Ok(Some(
                        if crate::helpers::is_copy_type_for_codegen(element_ty.as_ref()) {
                            indexed_expr
                        } else {
                            crate::RustExpr::Clone(Box::new(indexed_expr))
                        },
                    ));
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
                    return Ok(Some(crate::RustExpr::Cast {
                        expr: Box::new(crate::RustExpr::Index {
                            expr: Box::new(lowered_object),
                            index: Box::new(list_index),
                        }),
                        ty: crate::RustType::Named("u8".to_string()),
                    }));
                }
                Type::Str => {
                    let nth_expr = crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::MethodCall {
                            receiver: Box::new(lowered_object),
                            method: "chars".to_string(),
                            args: vec![],
                        }),
                        method: "nth".to_string(),
                        args: vec![crate::RustExpr::Cast {
                            expr: Box::new(lowered_index),
                            ty: crate::RustType::Named("usize".to_string()),
                        }],
                    };
                    if index_returns_option {
                        return Ok(Some(crate::RustExpr::MethodCall {
                            receiver: Box::new(nth_expr),
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
                        }));
                    }
                    return Err(crate::CodegenError::new(
                        "internal codegen invariant violated: string index produced non-optional result type",
                    ));
                }
                Type::Tuple(_) => {
                    let HirExpr::IntLiteral(idx) = index.as_ref() else {
                        return Ok(None);
                    };
                    return Ok(Some(crate::RustExpr::Field {
                        expr: Box::new(lowered_object),
                        field: idx.to_string(),
                    }));
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
            let Some(lowered_element) = $emitter.lower_stmt_expr_for_ir(element)? else {
                return Ok(None);
            };
            let Some(lowered_collection) = $emitter.lower_stmt_expr_for_ir(collection)? else {
                return Ok(None);
            };
            let lowered = match crate::resolve_alias_type_for_plain_call(collection.ty()) {
                Type::Dict(_, _) => {
                    let key_arg = if let HirExpr::StringLiteral(value) = element.as_ref() {
                        crate::RustExpr::Literal(crate::RustLiteral::Str(value.clone()))
                    } else if let HirExpr::Name { name, ty } = element.as_ref() {
                        if $emitter.borrowed_params.contains(name)
                            || $emitter.mut_borrowed_params.contains(name)
                        {
                            if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Str) {
                                crate::RustExpr::MethodCall {
                                    receiver: Box::new(crate::RustExpr::Paren(Box::new(
                                        lowered_element,
                                    ))),
                                    method: "as_str".to_string(),
                                    args: vec![],
                                }
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
                Type::List(_) | Type::Set(_) | Type::Range => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                    method: "contains".to_string(),
                    args: vec![crate::RustExpr::Ref {
                        mutable: false,
                        expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                    }],
                },
                Type::Str => crate::RustExpr::MethodCall {
                    receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_collection))),
                    method: "contains".to_string(),
                    args: vec![crate::RustExpr::MethodCall {
                        receiver: Box::new(crate::RustExpr::Paren(Box::new(lowered_element))),
                        method: "as_str".to_string(),
                        args: vec![],
                    }],
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
                    let Some(lowered_left) = $emitter.lower_stmt_expr_for_ir(lhs_expr)? else {
                        return Ok(None);
                    };
                    let Some(lowered_right) = $emitter.lower_stmt_expr_for_ir(rhs_expr)? else {
                        return Ok(None);
                    };
                    let lowered_left = if matches!(lhs_expr, HirExpr::Name { name, ty }
                        if ($emitter.borrowed_params.contains(name) || $emitter.mut_borrowed_params.contains(name))
                            && ty.ownership() != sifr_type_system::OwnershipKind::Copy)
                    {
                        crate::RustExpr::Clone(Box::new(lowered_left))
                    } else {
                        lowered_left
                    };
                    let lowered_right = if matches!(rhs_expr, HirExpr::Name { name, ty }
                        if ($emitter.borrowed_params.contains(name) || $emitter.mut_borrowed_params.contains(name))
                            && ty.ownership() != sifr_type_system::OwnershipKind::Copy)
                    {
                        crate::RustExpr::Clone(Box::new(lowered_right))
                    } else {
                        lowered_right
                    };
                    let left_is_option = crate::helpers::is_option_type(lhs_expr.ty());
                    let right_is_option = crate::helpers::is_option_type(rhs_expr.ty());
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
                    let left_ty = crate::resolve_alias_type_for_plain_call(lhs_expr.ty());
                    let right_ty = crate::resolve_alias_type_for_plain_call(rhs_expr.ty());
                    let (mut lowered_left, mut lowered_right) =
                        if left_is_option && !right_is_option && !right_none_like {
                            (
                                lowered_left,
                                crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![lowered_right],
                                },
                            )
                        } else if !left_is_option && right_is_option && !left_none_like {
                            (
                                crate::RustExpr::FnCall {
                                    func: Box::new(crate::RustExpr::Path(vec!["Some".to_string()])),
                                    args: vec![lowered_left],
                                },
                                lowered_right,
                            )
                        } else {
                            (lowered_left, lowered_right)
                        };
                    if !left_is_option
                        && !right_is_option
                        && matches!(left_ty, Type::Float)
                        && matches!(right_ty, Type::Int | Type::LiteralInt(_))
                    {
                        lowered_right = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_right))),
                            ty: crate::RustType::F64,
                        };
                    } else if !left_is_option
                        && !right_is_option
                        && matches!(right_ty, Type::Float)
                        && matches!(left_ty, Type::Int | Type::LiteralInt(_))
                    {
                        lowered_left = crate::RustExpr::Cast {
                            expr: Box::new(crate::RustExpr::Paren(Box::new(lowered_left))),
                            ty: crate::RustType::F64,
                        };
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
            let lower_boolop_operand =
                |this: &mut Self,
                 operand: &HirExpr|
                 -> Result<Option<crate::RustExpr>, crate::CodegenError> {
                    if matches!(crate::resolve_alias_type_for_plain_call(ty), Type::Bool) {
                        this.lower_condition_expr_for_ir(operand)
                    } else {
                        this.lower_stmt_expr_for_ir(operand)
                    }
                };
            let Some(mut acc) = lower_boolop_operand($emitter, first)? else {
                return Ok(None);
            };
            for value in iter {
                let Some(lowered_value) = lower_boolop_operand($emitter, value)? else {
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
