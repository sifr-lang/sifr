use super::*;
#[test]
fn lowers_simple_while_without_else() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::BoolLiteral(true),
        body: vec![HirStmt::Break],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(
        &while_stmt,
        true, // outer context has else, inner while should not inherit it
        &HashSet::new(),
        &HashSet::new(),
    )
    .expect("while lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::While { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], RustStmt::Break));
        }
        _ => panic!("expected RustStmt::While"),
    }
}

#[test]
fn lowers_simple_while_with_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Name {
            name: "ready".to_string(),
            binding_id: None,
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with name condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::Ident(ref name),
            ..
        } if name == "ready"
    ));
}

#[test]
fn lowers_simple_while_with_int_truthiness_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Name {
            name: "count".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with int truthiness condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::BinOp { ref op, .. },
            ..
        } if op == "!="
    ));
}

#[test]
fn lowers_simple_while_with_bigint_truthiness_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Name {
            name: "count".to_string(),
            binding_id: None,
            ty: Type::BigInt,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with bigint truthiness condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::BinOp { ref op, .. },
            ..
        } if op == "!="
    ));
}

#[test]
fn lowers_simple_while_with_not_bool_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ready".to_string(),
                binding_id: None,
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with not-bool name condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::UnaryOp {
                ref op,
                ref operand,
            },
            ..
        } if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ready")
    ));
}

#[test]
fn lowers_simple_while_with_not_int_truthiness_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "count".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with not-int truthiness condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::BinOp { ref op, .. },
            ..
        } if op == "=="
    ));
}

#[test]
fn lowers_simple_while_with_not_bigint_truthiness_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "count".to_string(),
                binding_id: None,
                ty: Type::BigInt,
            }),
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with not-bigint truthiness condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::BinOp { ref op, .. },
            ..
        } if op == "=="
    ));
}

#[test]
fn lowers_simple_while_with_not_option_truthiness_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                binding_id: None,
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with not-option truthiness name condition lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::While {
            cond:
                RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            ..
        } => {
            assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
            assert_eq!(method, "is_none");
            assert!(args.is_empty());
        }
        _ => panic!("expected while with method-call condition"),
    }
}

#[test]
fn lowers_simple_while_with_option_is_none_compare_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                binding_id: None,
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with option is-none compare condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_none"
            && args.is_empty()
    ));
}

#[test]
fn lowers_simple_while_with_option_is_not_none_compare_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                binding_id: None,
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with option is-not-none compare condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::While {
            cond: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_some"
            && args.is_empty()
    ));
}

#[test]
fn lowers_simple_while_with_option_truthiness_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with option truthiness name condition lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::While {
            cond:
                RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            ..
        } => {
            assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
            assert_eq!(method, "is_some");
            assert!(args.is_empty());
        }
        _ => panic!("expected while with method-call condition"),
    }
}

#[test]
fn lowers_simple_while_with_alias_option_truthiness_name_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with alias option truthiness name condition lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::While {
            cond:
                RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            ..
        } => {
            assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
            assert_eq!(method, "is_some");
            assert!(args.is_empty());
        }
        _ => panic!("expected while with method-call condition"),
    }
}

#[test]
fn does_not_lower_while_with_non_leaf_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "ready".to_string(),
            args: vec![],
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn does_not_lower_while_with_non_leaf_option_truthiness_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "maybe_x".to_string(),
            args: vec![],
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn does_not_lower_while_with_non_leaf_option_is_none_compare_condition() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Call {
                mutable_arg_places: Vec::new(),
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}
