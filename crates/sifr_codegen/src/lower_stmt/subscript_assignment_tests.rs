use super::*;
#[test]
fn lowers_simple_attribute_list_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "items".to_string(),
        index: HirExpr::Name {
            name: "i".to_string(),
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            ty: Type::Int,
        },
        field_ty: Type::List(Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("attribute list subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered attribute list subscript assignment");
    };
    assert_eq!(stmts.len(), 3);
    assert!(matches!(
        &stmts[2],
        RustStmt::If {
            then_body,
            else_body: None,
            ..
        } if matches!(
            then_body.first(),
            Some(RustStmt::IfLet {
                pattern,
                expr: RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
                then_body,
                else_body: None,
            }) if pattern == "Some(__elem)"
                && method == "get_mut"
                && matches!(
                    receiver.as_ref(),
                    RustExpr::Field { expr, field }
                        if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                        && field == "items"
                )
                && matches!(
                    args.first(),
                    Some(RustExpr::Cast {
                        expr: idx,
                        ty: RustType::Named(usize_ty),
                    }) if matches!(idx.as_ref(), RustExpr::Ident(name) if name == "__idx_norm")
                        && usize_ty == "usize"
                )
                && matches!(
                    then_body.first(),
                    Some(RustStmt::Assign {
                        target: RustExpr::Deref(target),
                        value: RustExpr::Ident(rhs),
                    }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && rhs == "v"
                )
        )
    ));
}

#[test]
fn lowers_simple_alias_attribute_list_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        value: HirExpr::IntLiteral(1),
        field_ty: Type::alias("IntList", Type::List(Box::new(Type::Int))),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias attribute-list subscript assign lowered");
    assert!(matches!(lowered[0], RustStmt::Block(_)));
}

#[test]
fn lowers_simple_attribute_dict_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "val".to_string(),
            ty: Type::Int,
        },
        field_ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("attribute dict subscript assign lowered");
    assert!(matches!(
        lowered[0],
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        }) if method == "insert"
            && matches!(
                recv.as_ref(),
                RustExpr::Field { expr, field }
                    if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                    && field == "mapping"
            )
            && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "key")
            && matches!(args.get(1), Some(RustExpr::Ident(name)) if name == "val")
    ));
}

#[test]
fn lowers_simple_alias_attribute_dict_subscript_assign_stmt() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "mapping".to_string(),
        index: HirExpr::IntLiteral(1),
        value: HirExpr::IntLiteral(2),
        field_ty: Type::alias(
            "IntMap",
            Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
        ),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias attribute-dict subscript assign lowered");
    assert!(matches!(lowered[0], RustStmt::Expr(_)));
}

#[test]
fn does_not_lower_attribute_dict_subscript_assign_with_string_name_key() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "mapping".to_string(),
        index: HirExpr::Name {
            name: "k".to_string(),
            ty: Type::Str,
        },
        value: HirExpr::IntLiteral(1),
        field_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn does_not_lower_attribute_subscript_assign_with_non_leaf_index() {
    let stmt = HirStmt::AttributeSubscriptAssign {
        object: "self".to_string(),
        field: "items".to_string(),
        index: HirExpr::Call {
            func: "next_idx".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        value: HirExpr::IntLiteral(1),
        field_ty: Type::List(Box::new(Type::Int)),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_list_subscript_assign_stmt() {
    let stmt = HirStmt::SubscriptAssign {
        object: "items".to_string(),
        index: HirExpr::Name {
            name: "i".to_string(),
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("list subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered list subscript assignment");
    };
    assert_eq!(stmts.len(), 3);
    assert!(matches!(
        stmts[0],
        RustStmt::Let {
            mutable: false,
            ref name,
            value: RustExpr::Ident(ref inner),
            ..
        } if name == "__idx_raw" && inner == "i"
    ));
    assert!(matches!(
        stmts[2],
        RustStmt::If {
            then_body: ref outer_then,
            else_body: None,
            ..
        } if matches!(
            outer_then.first(),
            Some(RustStmt::IfLet {
                pattern,
                expr: RustExpr::MethodCall {
                    receiver: recv,
                    method,
                    args,
                },
                then_body: body,
                else_body: None,
            }) if pattern == "Some(__elem)"
                && method == "get_mut"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "items")
                && matches!(
                    args.first(),
                    Some(RustExpr::Cast {
                        expr: inner,
                        ty: RustType::Named(usize_ty),
                    }) if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "__idx_norm")
                        && usize_ty == "usize"
                )
                && matches!(
                    body.first(),
                    Some(RustStmt::Assign {
                        target: RustExpr::Deref(target),
                        value: RustExpr::Ident(rhs),
                    }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && rhs == "v"
                )
        )
    ));
}

#[test]
fn lowers_simple_alias_list_subscript_assign_stmt() {
    let stmt = HirStmt::SubscriptAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        value: HirExpr::IntLiteral(1),
        object_ty: Type::alias("IntList", Type::List(Box::new(Type::Int))),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias-list subscript assign lowered");
    assert!(matches!(lowered[0], RustStmt::Block(_)));
}

#[test]
fn lowers_simple_dict_subscript_assign_stmt() {
    let stmt = HirStmt::SubscriptAssign {
        object: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            ty: Type::Str,
        },
        value: HirExpr::Name {
            name: "val".to_string(),
            ty: Type::Int,
        },
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        }) if method == "insert"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "mapping")
            && matches!(args.first(), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "key"))
            && matches!(args.get(1), Some(RustExpr::Ident(name)) if name == "val")
    ));
}

#[test]
fn lowers_simple_dict_subscript_assign_clones_non_copy_name_value() {
    let stmt = HirStmt::SubscriptAssign {
        object: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            ty: Type::Str,
        },
        value: HirExpr::Name {
            name: "val".to_string(),
            ty: Type::Str,
        },
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Str)),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict subscript assign lowered");
    assert!(matches!(
        lowered[0],
        RustStmt::Expr(RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        }) if method == "insert"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "mapping")
            && matches!(args.first(), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "key"))
            && matches!(args.get(1), Some(RustExpr::Clone(inner))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "val"))
    ));
}

#[test]
fn does_not_lower_subscript_assign_with_non_leaf_index() {
    let stmt = HirStmt::SubscriptAssign {
        object: "items".to_string(),
        index: HirExpr::Call {
            func: "next_idx".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        value: HirExpr::IntLiteral(1),
        object_ty: Type::List(Box::new(Type::Int)),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_list_delete_stmt() {
    let stmt = HirStmt::Delete {
        object: HirExpr::Name {
            name: "items".to_string(),
            ty: Type::List(Box::new(Type::Int)),
        },
        index: HirExpr::Name {
            name: "i".to_string(),
            ty: Type::Int,
        },
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("list delete lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Block(ref stmts)
            if matches!(
                stmts.get(2),
                Some(RustStmt::If {
                    then_body,
                    else_body: None,
                    ..
                }) if matches!(
                    then_body.first(),
                    Some(RustStmt::Let {
                        mutable: false,
                        name,
                        value: RustExpr::MethodCall {
                            receiver: recv,
                            method,
                            args,
                        },
                        ..
                    }) if name == "_"
                        && method == "remove"
                        && matches!(recv.as_ref(), RustExpr::Ident(obj) if obj == "items")
                        && matches!(
                            args.first(),
                            Some(RustExpr::Cast {
                                expr: inner,
                                ty: RustType::Named(usize_ty),
                            }) if matches!(inner.as_ref(), RustExpr::Ident(idx) if idx == "__idx_norm")
                                && usize_ty == "usize"
                        )
                )
            )
    ));
}

#[test]
fn lowers_simple_dict_delete_with_string_literal_key_stmt() {
    let stmt = HirStmt::Delete {
        object: HirExpr::Name {
            name: "mapping".to_string(),
            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        },
        index: HirExpr::StringLiteral("key".to_string()),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict delete lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            ref name,
            value: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if name == "_"
            && method == "remove"
            && matches!(recv.as_ref(), RustExpr::Ident(obj) if obj == "mapping")
            && matches!(
                args.first(),
                Some(RustExpr::Ref {
                    mutable: false,
                    expr: inner,
                }) if matches!(inner.as_ref(), RustExpr::Literal(RustLiteral::Str(key)) if key == "key")
            )
    ));
}

#[test]
fn does_not_lower_dict_delete_with_name_key() {
    let stmt = HirStmt::Delete {
        object: HirExpr::Name {
            name: "mapping".to_string(),
            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        },
        index: HirExpr::Name {
            name: "k".to_string(),
            ty: Type::Str,
        },
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_nested_subscript_assign_stmt() {
    let stmt = HirStmt::NestedSubscriptAssign {
        object: "matrix".to_string(),
        outer_index: HirExpr::Name {
            name: "i".to_string(),
            ty: Type::Int,
        },
        inner_index: HirExpr::Name {
            name: "j".to_string(),
            ty: Type::Int,
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("nested subscript assign lowered");
    assert_eq!(lowered.len(), 1);
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered nested subscript assignment");
    };
    assert_eq!(stmts.len(), 3);
    assert!(matches!(
        &stmts[0],
        RustStmt::Let {
            ref name,
            value: RustExpr::Ident(ref idx),
            ..
        } if name == "__oi_raw" && idx == "i"
    ));
    assert!(matches!(
        &stmts[1],
        RustStmt::Let {
            ref name,
            value: RustExpr::If { .. },
            ..
        } if name == "__oi_norm"
    ));
    assert!(matches!(
        &stmts[2],
        RustStmt::If {
            then_body: ref outer_then,
            else_body: None,
            ..
        } if matches!(
            outer_then.first(),
            Some(RustStmt::IfLet {
                pattern,
                expr: RustExpr::MethodCall {
                    receiver: recv,
                    method,
                    args,
                },
                then_body: outer_body,
                else_body: None,
            }) if pattern == "Some(__row)"
                && method == "get_mut"
                && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "matrix")
                && matches!(
                    args.first(),
                    Some(RustExpr::Cast {
                        expr,
                        ty: RustType::Named(usize_ty),
                    }) if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "__oi_norm")
                        && usize_ty == "usize"
                )
            && matches!(
                outer_body.last(),
                Some(RustStmt::If {
                    then_body: inner_outer_then,
                    ..
                }) if matches!(
                    inner_outer_then.first(),
                    Some(RustStmt::IfLet {
                        pattern: inner_pattern,
                        expr: RustExpr::MethodCall {
                            receiver: inner_recv,
                            method: inner_method,
                            args: inner_args,
                        },
                        then_body: inner_then,
                        else_body: None,
                    }) if inner_pattern == "Some(__elem)"
                        && inner_method == "get_mut"
                        && matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "__row")
                        && matches!(
                            inner_args.first(),
                            Some(RustExpr::Cast {
                                expr,
                                ty: RustType::Named(usize_ty),
                            }) if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "__ii_norm")
                                && usize_ty == "usize"
                        )
                        && matches!(
                            inner_then.first(),
                            Some(RustStmt::Assign {
                                target: RustExpr::Deref(target),
                                value: RustExpr::Ident(rhs),
                            }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                                && rhs == "v"
                        )
                )
            )
        )
    ));
}

#[test]
fn lowers_simple_nested_subscript_assign_stmt_with_optional_indices() {
    let stmt = HirStmt::NestedSubscriptAssign {
        object: "matrix".to_string(),
        outer_index: HirExpr::Name {
            name: "oi".to_string(),
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        inner_index: HirExpr::Name {
            name: "ii".to_string(),
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        value: HirExpr::Name {
            name: "v".to_string(),
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("nested subscript assign lowered");
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered nested subscript assignment");
    };
    assert!(matches!(
        stmts.first(),
        Some(RustStmt::Let { name, value: RustExpr::Ident(idx), .. })
            if name == "__oi_raw_opt" && idx == "oi"
    ));
    assert!(matches!(
        stmts.get(1),
        Some(RustStmt::IfLet {
            pattern,
            expr: RustExpr::Ident(raw_opt),
            then_body,
            else_body: None,
        }) if pattern == "Some(__oi_raw)"
            && raw_opt == "__oi_raw_opt"
            && matches!(
                then_body.get(1),
                Some(RustStmt::If {
                    then_body: outer_then,
                    ..
                }) if matches!(
                    outer_then.first(),
                    Some(RustStmt::IfLet { then_body: inner_body, .. })
                        if matches!(
                            inner_body.first(),
                            Some(RustStmt::Let {
                                name,
                                value: RustExpr::Ident(ii_idx),
                                ..
                            }) if name == "__ii_raw_opt" && ii_idx == "ii"
                        )
                )
            )
    ));
}

#[test]
fn does_not_lower_nested_subscript_assign_with_non_leaf_inner_index() {
    let stmt = HirStmt::NestedSubscriptAssign {
        object: "matrix".to_string(),
        outer_index: HirExpr::IntLiteral(0),
        inner_index: HirExpr::Call {
            func: "inner_idx".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        value: HirExpr::IntLiteral(1),
        object_ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}
