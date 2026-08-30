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
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    let mut emitter = crate::RustEmitter::new();
    let lowered = emitter
        .lower_subscript_augassign_stmt_for_ir(
            "items",
            match &stmt {
                HirStmt::SubscriptAugAssign { index, .. } => index,
                _ => unreachable!(),
            },
            "//=",
            match &stmt {
                HirStmt::SubscriptAugAssign { value, .. } => value,
                _ => unreachable!(),
            },
            &Type::List(Box::new(Type::Int)),
            None,
        )
        .expect("typed floor-div lowering should succeed")
        .expect("typed floor-div augassign should lower");
    let RustStmt::Block(stmts) = &lowered else {
        panic!("expected block-lowered list subscript floor-div augassign");
    };
    assert!(matches!(
        &stmts[2],
        RustStmt::IfLet { then_body, .. } if matches!(
            then_body.first(),
            Some(RustStmt::Assign {
                target: RustExpr::Deref(target),
                value: RustExpr::MethodCall { receiver, method, args },
            }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                && matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "__elem")
                && method == "floor_div_known_nonzero"
                && matches!(args.first(), Some(RustExpr::Ref { expr, .. }) if matches!(expr.as_ref(), RustExpr::Clone(inner) if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "d")))
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
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    let mut emitter = crate::RustEmitter::new();
    let lowered = emitter
        .lower_subscript_augassign_stmt_for_ir(
            "items",
            match &stmt {
                HirStmt::SubscriptAugAssign { index, .. } => index,
                _ => unreachable!(),
            },
            "**=",
            match &stmt {
                HirStmt::SubscriptAugAssign { value, .. } => value,
                _ => unreachable!(),
            },
            &Type::List(Box::new(Type::Int)),
            None,
        )
        .expect("typed power lowering should succeed")
        .expect("typed power augassign should lower");
    let RustStmt::Block(stmts) = &lowered else {
        panic!("expected block-lowered list subscript power augassign");
    };
    assert!(matches!(
        &stmts[2],
        RustStmt::IfLet { then_body, .. } if matches!(
            then_body.first(),
            Some(RustStmt::Assign {
                target: RustExpr::Deref(target),
                value: RustExpr::MethodCall { receiver, method, args },
            }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                && matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "__elem")
                && method == "pow_known_valid"
                && matches!(
                    args.first(),
                    Some(RustExpr::Ref { expr, .. })
                        if matches!(expr.as_ref(), RustExpr::Clone(inner) if matches!(inner.as_ref(), RustExpr::Ident(v) if v == "p"))
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
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
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
