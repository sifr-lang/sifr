use super::*;

#[test]
fn lowers_simple_raise_with_leaf_expr() {
    let stmt = HirStmt::Raise {
        value: HirExpr::IntLiteral(7),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("raise lowered");

    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
            assert!(
                matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()])
            );
        }
        _ => panic!("expected return Err(...)"),
    }
}

#[test]
fn does_not_lower_raise_with_non_leaf_expr() {
    let stmt = HirStmt::Raise {
        value: HirExpr::Call {
            func: "err".to_string(),
            args: vec![],
            ty: Type::Int,
        },
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_raise_with_name_expr() {
    let stmt = HirStmt::Raise {
        value: HirExpr::Name {
            name: "e".to_string(),
            ty: Type::Int,
        },
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("raise name lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::Return(Some(RustExpr::FnCall { func, args })) => {
            assert!(
                matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()])
            );
            assert!(matches!(args.first(), Some(RustExpr::Ident(name)) if name == "e"));
        }
        _ => panic!("expected return Err(e)"),
    }
}

#[test]
fn lowers_simple_assert_without_msg() {
    let stmt = HirStmt::Assert {
        test: HirExpr::BoolLiteral(true),
        msg: None,
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert lowered");

    assert_eq!(lowered.len(), 1);
    assert!(matches!(lowered[0], RustStmt::Assert { msg: None, .. }));
}

#[test]
fn lowers_simple_assert_with_leaf_msg() {
    let stmt = HirStmt::Assert {
        test: HirExpr::BoolLiteral(true),
        msg: Some(HirExpr::StringLiteral("boom".to_string())),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert with msg lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            msg: Some(RustExpr::Literal(RustLiteral::Str(_))),
            ..
        }
    ));
}

#[test]
fn lowers_simple_assert_with_name_msg() {
    let stmt = HirStmt::Assert {
        test: HirExpr::BoolLiteral(true),
        msg: Some(HirExpr::Name {
            name: "msg".to_string(),
            ty: Type::Str,
        }),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert with name msg lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            msg: Some(RustExpr::Ident(ref name)),
            ..
        } if name == "msg"
    ));
}

#[test]
fn does_not_lower_assert_with_non_leaf_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::Call {
            func: "is_ok".to_string(),
            args: vec![],
            ty: Type::Bool,
        },
        msg: None,
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_assert_with_bool_name_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::Name {
            name: "ok".to_string(),
            ty: Type::Bool,
        },
        msg: None,
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert bool name test lowered");

    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            cond: RustExpr::Ident(ref name),
            msg: None,
        } if name == "ok"
    ));
}

#[test]
fn lowers_simple_assert_with_not_bool_name_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        },
        msg: None,
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert not-bool name test lowered");

    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            cond: RustExpr::UnaryOp {
                ref op,
                ref operand,
            },
            msg: None,
        } if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
    ));
}
