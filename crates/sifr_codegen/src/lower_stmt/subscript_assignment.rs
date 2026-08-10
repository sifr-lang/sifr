use super::{
    is_option_like_type, resolve_alias_type, try_lower_attribute_dict_insert_key_expr,
    try_lower_leaf_expr, try_lower_leaf_or_name_expr, try_lower_name_ident_expr, HirExpr, RustExpr,
    RustLiteral, RustStmt, RustType, Type,
};

pub(crate) fn build_list_subscript_assign_stmt(
    receiver: RustExpr,
    lowered_index: RustExpr,
    lowered_value: RustExpr,
) -> RustStmt {
    build_list_get_mut_block_stmt(
        receiver,
        lowered_index,
        RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: lowered_value,
        },
    )
}

pub(super) fn build_list_get_mut_block_stmt(
    receiver: RustExpr,
    lowered_index: RustExpr,
    then_body_stmt: RustStmt,
) -> RustStmt {
    RustStmt::Block(vec![
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
            value: build_normalized_list_index_i64_expr(receiver.clone(), "__idx_raw"),
        },
        RustStmt::If {
            cond: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                op: ">=".to_string(),
                right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
            },
            then_body: vec![RustStmt::IfLet {
                pattern: "Some(__elem)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "get_mut".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                        ty: RustType::Named("usize".to_string()),
                    }],
                },
                then_body: vec![then_body_stmt],
                else_body: None,
            }],
            else_body: None,
        },
    ])
}

pub(crate) fn build_normalized_list_index_i64_expr(
    receiver: RustExpr,
    raw_index_name: &str,
) -> RustExpr {
    let raw_ident = || RustExpr::Ident(raw_index_name.to_string());
    RustExpr::If {
        cond: Box::new(RustExpr::BinOp {
            left: Box::new(raw_ident()),
            op: "<".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        }),
        then_expr: Box::new(RustExpr::BinOp {
            left: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(receiver),
                    method: "len".to_string(),
                    args: vec![],
                }),
                ty: RustType::I64,
            }),
            op: "+".to_string(),
            right: Box::new(raw_ident()),
        }),
        else_expr: Some(Box::new(raw_ident())),
    }
}

pub(crate) fn build_dict_subscript_assign_stmt(
    receiver: RustExpr,
    lowered_index: RustExpr,
    lowered_value: RustExpr,
) -> RustStmt {
    RustStmt::Expr(RustExpr::MethodCall {
        receiver: Box::new(receiver),
        method: "insert".to_string(),
        args: vec![lowered_index, lowered_value],
    })
}

pub(super) fn try_lower_simple_subscript_assign_stmt(
    object: &str,
    index: &HirExpr,
    value: &HirExpr,
    object_ty: &Type,
) -> Option<Vec<RustStmt>> {
    let lowered_index =
        maybe_clone_subscript_assignment_name(index, try_lower_leaf_or_name_expr(index)?);
    let lowered_value =
        maybe_clone_subscript_assignment_name(value, try_lower_leaf_or_name_expr(value)?);
    match resolve_alias_type(object_ty) {
        Type::List(_) => Some(vec![build_list_subscript_assign_stmt(
            RustExpr::Ident(object.to_string()),
            lowered_index,
            lowered_value,
        )]),
        Type::Dict(_, _) => Some(vec![build_dict_subscript_assign_stmt(
            RustExpr::Ident(object.to_string()),
            lowered_index,
            lowered_value,
        )]),
        _ => None,
    }
}

pub(super) fn maybe_clone_subscript_assignment_name(expr: &HirExpr, lowered: RustExpr) -> RustExpr {
    if !expr.ty().contains_affine_resource()
        && matches!(expr, HirExpr::Name { .. })
        && !crate::helpers::is_copy_type_for_codegen(expr.ty())
    {
        RustExpr::Clone(Box::new(lowered))
    } else {
        lowered
    }
}

pub(super) fn try_lower_simple_delete_stmt(
    object: &HirExpr,
    index: &HirExpr,
) -> Option<Vec<RustStmt>> {
    let receiver = try_lower_name_ident_expr(object)?;
    let lowered_index = try_lower_leaf_or_name_expr(index)?;
    match resolve_alias_type(object.ty()) {
        Type::List(_) => Some(vec![RustStmt::Block(vec![
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
                value: build_normalized_list_index_i64_expr(receiver.clone(), "__idx_raw"),
            },
            RustStmt::If {
                cond: RustExpr::BinOp {
                    left: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                        op: ">=".to_string(),
                        right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                    }),
                    op: "&&".to_string(),
                    right: Box::new(RustExpr::BinOp {
                        left: Box::new(RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                            ty: RustType::Named("usize".to_string()),
                        }),
                        op: "<".to_string(),
                        right: Box::new(RustExpr::MethodCall {
                            receiver: Box::new(receiver.clone()),
                            method: "len".to_string(),
                            args: vec![],
                        }),
                    }),
                },
                then_body: vec![RustStmt::Let {
                    mutable: false,
                    name: "_".to_string(),
                    ty: None,
                    value: RustExpr::MethodCall {
                        receiver: Box::new(receiver),
                        method: "remove".to_string(),
                        args: vec![RustExpr::Cast {
                            expr: Box::new(RustExpr::Ident("__idx_norm".to_string())),
                            ty: RustType::Named("usize".to_string()),
                        }],
                    },
                }],
                else_body: None,
            },
        ])]),
        Type::Dict(_, _) => Some(vec![RustStmt::Let {
            mutable: false,
            name: "_".to_string(),
            ty: None,
            value: RustExpr::MethodCall {
                receiver: Box::new(receiver),
                method: "remove".to_string(),
                args: vec![build_dict_delete_key_arg(index)?],
            },
        }]),
        _ => None,
    }
}

pub(super) fn build_dict_delete_key_arg(index: &HirExpr) -> Option<RustExpr> {
    if matches!(index, HirExpr::Name { .. }) {
        // Preserve name-key borrowing behavior.
        return None;
    }
    let lowered_index = try_lower_leaf_expr(index)?;
    Some(RustExpr::Ref {
        mutable: false,
        expr: Box::new(lowered_index),
    })
}

pub(super) fn try_lower_simple_nested_subscript_assign_stmt(
    object: &str,
    outer_index: &HirExpr,
    inner_index: &HirExpr,
    value: &HirExpr,
    object_ty: &Type,
) -> Option<Vec<RustStmt>> {
    let Type::List(inner_ty) = resolve_alias_type(object_ty) else {
        return None;
    };
    let Type::List(target_elem_ty) = resolve_alias_type(inner_ty) else {
        return None;
    };
    let lowered_outer_index = try_lower_leaf_or_name_expr(outer_index)?;
    let lowered_inner_index = try_lower_leaf_or_name_expr(inner_index)?;
    let outer_index_is_option = is_option_like_type(outer_index.ty());
    let inner_index_is_option = is_option_like_type(inner_index.ty());
    let target_elem_is_option = is_option_like_type(target_elem_ty);
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    let assign_elem_stmt = if is_option_like_type(value.ty()) && !target_elem_is_option {
        RustStmt::IfLet {
            pattern: "Some(__nested_assign_value)".to_string(),
            expr: lowered_value,
            then_body: vec![RustStmt::Assign {
                target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                value: RustExpr::Ident("__nested_assign_value".to_string()),
            }],
            else_body: None,
        }
    } else {
        RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: lowered_value,
        }
    };

    let mut inner_then_body = vec![RustStmt::Let {
        mutable: false,
        name: "__ii_norm".to_string(),
        ty: None,
        value: build_normalized_list_index_i64_expr(
            RustExpr::Ident("__row".to_string()),
            "__ii_raw",
        ),
    }];
    inner_then_body.push(RustStmt::If {
        cond: RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("__ii_norm".to_string())),
            op: ">=".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        },
        then_body: vec![RustStmt::IfLet {
            pattern: "Some(__elem)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__row".to_string())),
                method: "get_mut".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                    ty: RustType::Named("usize".to_string()),
                }],
            },
            then_body: vec![assign_elem_stmt],
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
        value: build_normalized_list_index_i64_expr(
            RustExpr::Ident(object.to_string()),
            "__oi_raw",
        ),
    }];
    outer_then_body.push(RustStmt::If {
        cond: RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("__oi_norm".to_string())),
            op: ">=".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        },
        then_body: vec![RustStmt::IfLet {
            pattern: "Some(__row)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "get_mut".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                    ty: RustType::Named("usize".to_string()),
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

    Some(vec![RustStmt::Block(outer_body)])
}

pub(super) fn try_lower_simple_attribute_subscript_assign_stmt(
    object: &str,
    field: &str,
    index: &HirExpr,
    value: &HirExpr,
    field_ty: &Type,
) -> Option<Vec<RustStmt>> {
    let lowered_value = try_lower_leaf_or_name_expr(value)?;

    match resolve_alias_type(field_ty) {
        Type::List(_) => Some(vec![build_list_subscript_assign_stmt(
            RustExpr::Field {
                expr: Box::new(RustExpr::Ident(object.to_string())),
                field: field.to_string(),
            },
            try_lower_leaf_or_name_expr(index)?,
            lowered_value,
        )]),
        Type::Dict(_, _) => Some(vec![build_dict_subscript_assign_stmt(
            RustExpr::Field {
                expr: Box::new(RustExpr::Ident(object.to_string())),
                field: field.to_string(),
            },
            try_lower_attribute_dict_insert_key_expr(index, field_ty)?,
            lowered_value,
        )]),
        _ => None,
    }
}

pub(super) fn try_lower_simple_attribute_nested_subscript_assign_stmt(
    object: &str,
    field: &str,
    outer_index: &HirExpr,
    inner_index: &HirExpr,
    value: &HirExpr,
    field_ty: &Type,
) -> Option<Vec<RustStmt>> {
    let Type::List(inner_ty) = resolve_alias_type(field_ty) else {
        return None;
    };
    let Type::List(target_elem_ty) = resolve_alias_type(inner_ty) else {
        return None;
    };
    let lowered_outer_index = try_lower_leaf_or_name_expr(outer_index)?;
    let lowered_inner_index = try_lower_leaf_or_name_expr(inner_index)?;
    let outer_index_is_option = is_option_like_type(outer_index.ty());
    let inner_index_is_option = is_option_like_type(inner_index.ty());
    let target_elem_is_option = is_option_like_type(target_elem_ty);
    let lowered_value = try_lower_leaf_or_name_expr(value)?;
    let assign_elem_stmt = if is_option_like_type(value.ty()) && !target_elem_is_option {
        RustStmt::IfLet {
            pattern: "Some(__nested_assign_value)".to_string(),
            expr: lowered_value,
            then_body: vec![RustStmt::Assign {
                target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
                value: RustExpr::Ident("__nested_assign_value".to_string()),
            }],
            else_body: None,
        }
    } else {
        RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: lowered_value,
        }
    };
    let receiver = || RustExpr::Field {
        expr: Box::new(RustExpr::Ident(object.to_string())),
        field: field.to_string(),
    };

    let mut inner_then_body = vec![RustStmt::Let {
        mutable: false,
        name: "__ii_norm".to_string(),
        ty: None,
        value: build_normalized_list_index_i64_expr(
            RustExpr::Ident("__row".to_string()),
            "__ii_raw",
        ),
    }];
    inner_then_body.push(RustStmt::If {
        cond: RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("__ii_norm".to_string())),
            op: ">=".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        },
        then_body: vec![RustStmt::IfLet {
            pattern: "Some(__elem)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__row".to_string())),
                method: "get_mut".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__ii_norm".to_string())),
                    ty: RustType::Named("usize".to_string()),
                }],
            },
            then_body: vec![assign_elem_stmt],
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
        value: build_normalized_list_index_i64_expr(receiver(), "__oi_raw"),
    }];
    outer_then_body.push(RustStmt::If {
        cond: RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("__oi_norm".to_string())),
            op: ">=".to_string(),
            right: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
        },
        then_body: vec![RustStmt::IfLet {
            pattern: "Some(__row)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(receiver()),
                method: "get_mut".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Ident("__oi_norm".to_string())),
                    ty: RustType::Named("usize".to_string()),
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

    Some(vec![RustStmt::Block(outer_body)])
}

pub(super) fn try_lower_simple_subscript_augassign_stmt(
    object: &str,
    index: &HirExpr,
    op: &str,
    value: &HirExpr,
    object_ty: &Type,
    missing_key_error: Option<&Type>,
) -> Option<Vec<RustStmt>> {
    if missing_key_error.is_some() {
        return None;
    }
    if !is_supported_subscript_augassign_op(op) {
        return None;
    }
    let lowered_index = try_lower_leaf_or_name_expr(index)?;
    let lowered_value = try_lower_leaf_or_name_expr(value)?;

    if op == "+="
        && matches!(
            resolve_alias_type(object_ty),
            Type::List(element_ty)
                if matches!(resolve_alias_type(element_ty.as_ref()), Type::Str | Type::LiteralStr(_))
        )
    {
        let push_arg = if matches!(value, HirExpr::StringLiteral(_)) {
            lowered_value
        } else {
            RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Paren(Box::new(lowered_value))),
                method: "as_str".to_string(),
                args: vec![],
            }
        };
        return Some(vec![build_list_get_mut_block_stmt(
            RustExpr::Ident(object.to_string()),
            lowered_index,
            RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__elem".to_string())),
                method: "push_str".to_string(),
                args: vec![push_arg],
            }),
        )]);
    }

    let lowered_body_stmt = build_subscript_augassign_elem_stmt(op, lowered_value);

    if matches!(
        object_ty,
        Type::Alias { name: alias_name, .. } if alias_name == "__sifr_defaultdict_int"
    ) {
        return Some(vec![RustStmt::Block(vec![
            RustStmt::Let {
                mutable: false,
                name: "__elem".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident(object.to_string())),
                        method: "entry".to_string(),
                        args: vec![if matches!(
                            &lowered_index,
                            RustExpr::Literal(RustLiteral::Str(_))
                        ) {
                            lowered_index
                        } else {
                            RustExpr::Clone(Box::new(lowered_index))
                        }],
                    }),
                    method: "or_insert".to_string(),
                    args: vec![RustExpr::Literal(RustLiteral::Int(0))],
                },
            },
            lowered_body_stmt,
        ])]);
    }

    match resolve_alias_type(object_ty) {
        Type::List(_) => Some(vec![build_list_get_mut_block_stmt(
            RustExpr::Ident(object.to_string()),
            lowered_index,
            lowered_body_stmt,
        )]),
        Type::Dict(_, _) => Some(vec![RustStmt::IfLet {
            pattern: "Some(__elem)".to_string(),
            expr: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident(object.to_string())),
                method: "get_mut".to_string(),
                args: vec![build_dict_get_mut_key_arg(lowered_index)],
            },
            then_body: vec![lowered_body_stmt],
            else_body: None,
        }]),
        _ => None,
    }
}

pub(super) fn build_dict_get_mut_key_arg(lowered_index: RustExpr) -> RustExpr {
    RustExpr::Ref {
        mutable: false,
        expr: Box::new(lowered_index),
    }
}

pub(super) fn is_supported_subscript_augassign_op(op: &str) -> bool {
    matches!(
        op,
        "+=" | "-=" | "*=" | "/=" | "%=" | "//=" | "**=" | "&=" | "|=" | "^=" | "<<=" | ">>="
    )
}

pub(super) fn build_subscript_augassign_elem_stmt(op: &str, lowered_value: RustExpr) -> RustStmt {
    if op == "**=" {
        return RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__elem".to_string())),
                method: "pow".to_string(),
                args: vec![RustExpr::Cast {
                    expr: Box::new(lowered_value),
                    ty: RustType::Named("u32".to_string()),
                }],
            },
        };
    }
    if op == "//=" {
        return RustStmt::Assign {
            target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
            value: RustExpr::BinOp {
                left: Box::new(RustExpr::Deref(Box::new(RustExpr::Ident(
                    "__elem".to_string(),
                )))),
                op: "/".to_string(),
                right: Box::new(lowered_value),
            },
        };
    }
    RustStmt::AugAssign {
        target: RustExpr::Deref(Box::new(RustExpr::Ident("__elem".to_string()))),
        op: op.strip_suffix('=').unwrap_or(op).to_string(),
        value: lowered_value,
    }
}
