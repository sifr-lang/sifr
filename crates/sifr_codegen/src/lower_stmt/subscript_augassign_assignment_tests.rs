use super::*;

fn key_error_type() -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "KeyError".to_string(),
        fields: vec![("message".to_string(), Type::Str)],
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

#[test]
fn lowers_simple_list_subscript_augassign_plus_equal_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "items".to_string(),
        index: HirExpr::Name {
            name: "i".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        op: "+=".to_string(),
        value: HirExpr::Name {
            name: "delta".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::List(Box::new(Type::Int)),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("list subscript augassign lowered");
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered list subscript augassign");
    };
    assert_eq!(stmts.len(), 3);
    assert!(matches!(
        &stmts[2],
        RustStmt::If {
            then_body,
            ..
        } if matches!(
            then_body.first(),
            Some(RustStmt::IfLet { then_body, .. }) if matches!(
                then_body.first(),
                Some(RustStmt::AugAssign {
                    target: RustExpr::Deref(target),
                    op,
                    value: RustExpr::Ident(rhs),
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && op == "+"
                    && rhs == "delta"
            )
        )
    ));
}

#[test]
fn lowers_simple_string_list_subscript_augassign_plus_equal_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "rows".to_string(),
        index: HirExpr::IntLiteral(0),
        op: "+=".to_string(),
        value: HirExpr::Name {
            name: "c".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        object_ty: Type::List(Box::new(Type::Str)),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("string list subscript augassign lowered");
    let RustStmt::Block(stmts) = &lowered[0] else {
        panic!("expected block-lowered string list subscript augassign");
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
                Some(RustStmt::Expr(RustExpr::MethodCall { receiver, method, args }))
                    if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && method == "push_str"
                        && matches!(
                            args.first(),
                            Some(RustExpr::MethodCall {
                                receiver: inner_receiver,
                                method: inner_method,
                                args: inner_args,
                            }) if matches!(
                                inner_receiver.as_ref(),
                                RustExpr::Paren(expr)
                                    if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "c")
                            ) && inner_method == "as_str" && inner_args.is_empty()
                        )
            )
        )
    ));
}

#[test]
fn lowers_simple_list_subscript_augassign_bitwise_and_shift_ops() {
    for (op, expected) in [
        ("&=", "&"),
        ("|=", "|"),
        ("^=", "^"),
        ("<<=", "<<"),
        (">>=", ">>"),
    ] {
        let stmt = HirStmt::SubscriptAugAssign {
            object: "items".to_string(),
            index: HirExpr::IntLiteral(0),
            op: op.to_string(),
            value: HirExpr::Name {
                name: "rhs".to_string(),
                binding_id: None,
                ty: Type::Int,
            },
            object_ty: Type::List(Box::new(Type::Int)),
            missing_key_error: None,
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("list subscript bitwise/shift augassign lowered");
        let RustStmt::Block(stmts) = &lowered[0] else {
            panic!("expected block-lowered list subscript bitwise/shift augassign");
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
                    Some(RustStmt::AugAssign {
                        target: RustExpr::Deref(target),
                        op,
                        value: RustExpr::Ident(rhs),
                    }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                        && op == expected
                        && rhs == "rhs"
                )
            )
        ));
    }
}

#[test]
fn lowers_simple_dict_subscript_augassign_with_name_key() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        op: "+=".to_string(),
        value: HirExpr::Name {
            name: "delta".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict subscript augassign lowered");
    assert!(matches!(
        lowered[0],
        RustStmt::IfLet {
            ref pattern,
            expr: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            then_body: ref body,
            else_body: None,
        } if pattern == "Some(__elem)"
            && method == "get_mut"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "mapping")
            && matches!(
                args.first(),
                Some(RustExpr::Ref {
                    mutable: false,
                    expr
                }) if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "key")
            )
            && matches!(
                body.first(),
                Some(RustStmt::AugAssign {
                    target: RustExpr::Deref(target),
                    op,
                    value: RustExpr::Ident(rhs),
                }) if matches!(target.as_ref(), RustExpr::Ident(name) if name == "__elem")
                    && op == "+"
                    && rhs == "delta"
            )
    ));
}

#[test]
fn lowers_simple_dict_subscript_augassign_with_string_literal_key() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "mapping".to_string(),
        index: HirExpr::StringLiteral("k".to_string()),
        op: "-=".to_string(),
        value: HirExpr::IntLiteral(1),
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("dict subscript augassign lowered");
    assert!(matches!(
        lowered[0],
        RustStmt::IfLet {
            expr: RustExpr::MethodCall { ref args, .. },
            ..
        } if matches!(
            args.first(),
            Some(RustExpr::Ref { mutable: false, expr })
                if matches!(expr.as_ref(), RustExpr::Literal(RustLiteral::Str(key)) if key == "k")
        )
    ));
}

#[test]
fn lowers_simple_alias_dict_subscript_augassign_stmt() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "mapping".to_string(),
        index: HirExpr::Name {
            name: "key".to_string(),
            binding_id: None,
            ty: Type::Str,
        },
        op: "|=".to_string(),
        value: HirExpr::IntLiteral(2),
        object_ty: Type::alias(
            "IntMap",
            Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        ),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias-dict subscript augassign lowered");
    assert!(matches!(lowered[0], RustStmt::IfLet { .. }));
}

#[test]
fn checked_dict_subscript_augassign_returns_key_error_when_missing() {
    let error_ty = key_error_type();
    let stmt = HirStmt::SubscriptAugAssign {
        object: "mapping".to_string(),
        index: HirExpr::StringLiteral("missing".to_string()),
        op: "+=".to_string(),
        value: HirExpr::IntLiteral(1),
        object_ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        missing_key_error: Some(error_ty.clone()),
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());

    let mut emitter = crate::RustEmitter::new();
    let HirStmt::SubscriptAugAssign {
        object,
        index,
        op,
        value,
        object_ty,
        ..
    } = &stmt
    else {
        panic!("expected subscript augassign");
    };
    let lowered = emitter
        .lower_subscript_augassign_stmt_for_ir(object, index, op, value, object_ty, Some(&error_ty))
        .expect("checked dict augassign lowering should succeed")
        .expect("checked dict augassign should lower");
    assert!(matches!(
        lowered,
        RustStmt::IfLet {
            expr: RustExpr::MethodCall { args: key_args, .. },
            then_body,
            else_body: Some(else_body),
            ..
        } if matches!(key_args.as_slice(), [RustExpr::Ref { mutable: false, .. }])
            && !then_body.is_empty()
            && matches!(
                else_body.as_slice(),
                [RustStmt::Return(Some(RustExpr::FnCall { func, args }))]
                    if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Err".to_string()])
                        && matches!(
                            args.as_slice(),
                            [RustExpr::FnCall { func, .. }]
                                if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["KeyError".to_string(), "new".to_string()])
                        )
            )
    ));
}

#[test]
fn annotated_defaultdict_alias_keeps_entry_insertion_codegen() {
    let stmt = HirStmt::SubscriptAugAssign {
        object: "mapping".to_string(),
        index: HirExpr::StringLiteral("missing".to_string()),
        op: "+=".to_string(),
        value: HirExpr::IntLiteral(3),
        object_ty: Type::alias(
            "__sifr_defaultdict_int",
            Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        ),
        missing_key_error: None,
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("defaultdict augassign should lower");
    assert!(matches!(
        &lowered[0],
        RustStmt::Block(stmts)
            if matches!(
                stmts.first(),
                Some(RustStmt::Let {
                    value: RustExpr::MethodCall { receiver, method, .. },
                    ..
                }) if method == "or_insert"
                    && matches!(
                        receiver.as_ref(),
                        RustExpr::MethodCall { method, .. } if method == "entry"
                    )
            )
    ));
}
