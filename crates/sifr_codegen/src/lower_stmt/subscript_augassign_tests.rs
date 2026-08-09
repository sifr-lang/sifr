use super::*;
#[test]
fn lowers_simple_list_subscript_augassign_floor_div_equal_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "//=".to_string(),
        value: HirExpr::Name {
            name: "d".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("list subscript floor-div augassign lowered");
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered list subscript floor-div augassign");
    };
    assert!(matches!(
        &stmts[2],
        RustStmt::If {
            then_body,
            ..
        } if matches!(
            then_body.first(),
            Some(RustStmt::IfLet { then_body, .. }) if matches!(
                then_body.first(),
                Some(RustStmt::Assign {
                    target: RustExpr::Deref(target),
                    value: RustExpr::BinOp { left, op, right },
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && matches!(left.as_ref(), RustExpr::Deref(inner) if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "__elem"))
                    && op == "/"
                    && matches!(right.as_ref(), RustExpr::Ident(name) if name == "d")
            )
        )
    ));
}

#[test]
fn lowers_simple_list_subscript_augassign_power_equal_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "**=".to_string(),
        value: HirExpr::Name {
            name: "p".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("list subscript power augassign lowered");
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered list subscript power augassign");
    };
    assert!(matches!(
        &stmts[2],
        RustStmt::If {
            then_body,
            ..
        } if matches!(
            then_body.first(),
            Some(RustStmt::IfLet { then_body, .. }) if matches!(
                then_body.first(),
                Some(RustStmt::Assign {
                    target: RustExpr::Deref(target),
                    value: RustExpr::MethodCall { receiver, method, args },
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && method == "pow"
                    && matches!(
                        args.first(),
                        Some(RustExpr::Cast {
                            expr,
                            ty: RustType::Named(name),
                        }) if matches!(expr.as_ref(), RustExpr::Ident(v) if v == "p") && name == "u32"
                    )
            )
        )
    ));
}

#[test]
fn lowers_simple_alias_list_subscript_augassign_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "+=".to_string(),
        value: HirExpr::IntLiteral(1),
        object_ty: Type::alias("IntList", Type::List(Box::new(Type::Int))),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias-list subscript augassign lowered");
    assert!(matches!(lowered[0], RustStmt::Block(_)));
}

#[test]
fn does_not_lower_subscript_augassign_with_non_leaf_value() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "+=".to_string(),
        value: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "next_value".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
        missing_key_error: None,
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}
