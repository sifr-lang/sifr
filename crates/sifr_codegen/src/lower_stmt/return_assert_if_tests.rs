use super::*;
#[test]
fn lowers_simple_assert_with_not_option_truthiness_name_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        },
        msg: None,
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert not-option truthiness name test lowered");

    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::Assert {
            cond:
                RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            msg: None,
        } => {
            assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
            assert_eq!(method, "is_none");
            assert!(args.is_empty());
        }
        _ => panic!("expected assert with method-call condition"),
    }
}

#[test]
fn lowers_simple_assert_with_option_truthiness_name_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::Name {
            name: "maybe_x".to_string(),
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        msg: None,
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert option truthiness name test lowered");

    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::Assert {
            cond:
                RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            msg: None,
        } => {
            assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "maybe_x"));
            assert_eq!(method, "is_some");
            assert!(args.is_empty());
        }
        _ => panic!("expected assert with method-call condition"),
    }
}

#[test]
fn lowers_simple_assert_with_option_is_none_compare_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        msg: None,
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert option is-none compare lowered");

    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            cond: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            msg: None,
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_none"
            && args.is_empty()
    ));
}

#[test]
fn lowers_simple_assert_with_option_is_not_none_compare_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        msg: None,
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert option is-not-none compare lowered");

    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            cond: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            msg: None,
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_some"
            && args.is_empty()
    ));
}

#[test]
fn does_not_lower_assert_with_non_leaf_not_bool_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Call {
                func: "is_ok".to_string(),
                args: vec![],
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        },
        msg: None,
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn does_not_lower_assert_with_non_leaf_not_option_truthiness_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        },
        msg: None,
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn does_not_lower_assert_with_non_leaf_option_is_none_compare_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::Compare {
            left: Box::new(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        msg: None,
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn does_not_lower_assert_with_non_leaf_option_truthiness_test() {
    let stmt = HirStmt::Assert {
        test: HirExpr::Call {
            func: "maybe_x".to_string(),
            args: vec![],
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        msg: None,
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_assert_with_option_name_msg() {
    let stmt = HirStmt::Assert {
        test: HirExpr::BoolLiteral(true),
        msg: Some(HirExpr::Name {
            name: "msg".to_string(),
            ty: Type::Union(vec![Type::Str, Type::None]),
        }),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert with option msg lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            msg: Some(RustExpr::MethodCall { ref method, .. }),
            ..
        } if method == "map_or"
    ));
}

#[test]
fn lowers_simple_assert_with_alias_option_name_msg() {
    let stmt = HirStmt::Assert {
        test: HirExpr::BoolLiteral(true),
        msg: Some(HirExpr::Name {
            name: "msg".to_string(),
            ty: Type::alias("MaybeStr", Type::Union(vec![Type::Str, Type::None])),
        }),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assert with alias option msg lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assert {
            msg: Some(RustExpr::MethodCall { ref method, .. }),
            ..
        } if method == "map_or"
    ));
}

#[test]
fn does_not_lower_assert_with_non_leaf_option_msg() {
    let stmt = HirStmt::Assert {
        test: HirExpr::BoolLiteral(true),
        msg: Some(HirExpr::Call {
            func: "maybe_msg".to_string(),
            args: vec![],
            ty: Type::Union(vec![Type::Str, Type::None]),
        }),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_if_without_elif() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Expr {
            expr: HirExpr::IntLiteral(1),
        }],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Expr {
            expr: HirExpr::IntLiteral(0),
        }]),
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(lowered[0], RustStmt::If { .. }));
}

#[test]
fn lowers_simple_if_with_name_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Name {
            name: "ok".to_string(),
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with name condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::If {
            cond: RustExpr::Ident(ref name),
            ..
        } if name == "ok"
    ));
}

#[test]
fn lowers_simple_if_with_not_bool_name_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with not-bool name condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::If {
            cond: RustExpr::UnaryOp {
                ref op,
                ref operand,
            },
            ..
        } if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
    ));
}

#[test]
fn does_not_lower_if_with_non_leaf_not_bool_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Call {
                func: "ok".to_string(),
                args: vec![],
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_if_with_not_option_truthiness_name_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with not-option truthiness condition lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::If {
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
        _ => panic!("expected if with method-call condition"),
    }
}

#[test]
fn lowers_simple_if_with_option_is_none_compare_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with option is-none compare condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::If {
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
fn lowers_option_is_none_if_with_exiting_body_to_let_else_without_unwrap() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(0)),
        }],
        elif_clauses: vec![],
        else_body: None,
    };

    let ret_ty = Type::Int;
    let lowered = try_lower_simple_stmt_with_ctx(
        &if_stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx {
            return_type: Some(&ret_ty),
            in_display_impl: false,
            in_class_scope: false,
            in_generator_closure: false,
        },
    )
    .expect("if with exiting body lowered");

    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::LetElse {
            pattern: ref p,
            value: RustExpr::Ident(ref name),
            ref else_body,
        }
        if p == "Some(maybe_x)"
            && name == "maybe_x"
            && !else_body.is_empty()
    ));
}

#[test]
fn lowers_simple_if_with_option_is_not_none_compare_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is not".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with option is-not-none compare condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::IfLet {
            ref pattern,
            expr: RustExpr::Ident(ref name),
            ..
        } if pattern == "Some(maybe_x)" && name == "maybe_x"
    ));
}

#[test]
fn lowers_simple_if_with_option_and_not_none_chain_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::BoolOp {
            op: "and".to_string(),
            values: vec![
                HirExpr::Compare {
                    left: Box::new(HirExpr::Name {
                        name: "a".to_string(),
                        ty: Type::Union(vec![Type::Int, Type::None]),
                    }),
                    ops: vec!["is not".to_string()],
                    comparators: vec![HirExpr::NoneLiteral],
                    ty: Type::Bool,
                },
                HirExpr::Compare {
                    left: Box::new(HirExpr::Name {
                        name: "b".to_string(),
                        ty: Type::Union(vec![Type::Int, Type::None]),
                    }),
                    ops: vec!["is not".to_string()],
                    comparators: vec![HirExpr::NoneLiteral],
                    ty: Type::Bool,
                },
            ],
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Pass]),
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with option and-not-none chain condition lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::IfLet {
            ref pattern,
            expr: RustExpr::Ident(ref name),
            ..
        } if pattern == "Some(a)" && name == "a"
    ));
}

#[test]
fn does_not_lower_if_with_non_leaf_option_is_none_compare_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Compare {
            left: Box::new(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ops: vec!["is".to_string()],
            comparators: vec![HirExpr::NoneLiteral],
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn does_not_lower_if_with_non_leaf_not_option_truthiness_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_if_with_option_truthiness_name_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Name {
            name: "maybe_x".to_string(),
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Pass]),
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if option truthiness lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::IfLet {
            pattern: ref p,
            expr: RustExpr::Ident(ref n),
            else_body: Some(_),
            ..
        } if p == "Some(maybe_x)" && n == "maybe_x"
    ));
}

#[test]
fn lowers_simple_if_with_alias_option_truthiness_name_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Name {
            name: "maybe_x".to_string(),
            ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Pass]),
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if alias option truthiness lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::IfLet {
            pattern: ref p,
            expr: RustExpr::Ident(ref n),
            else_body: Some(_),
            ..
        } if p == "Some(maybe_x)" && n == "maybe_x"
    ));
}

#[test]
fn lowers_if_option_truthiness_with_elif() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::Name {
            name: "maybe_x".to_string(),
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![(HirExpr::BoolLiteral(true), vec![HirStmt::Pass])],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if option truthiness with elif lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::IfLet { else_body, .. } => {
            assert!(else_body.is_some());
            if let Some(else_body) = else_body {
                assert_eq!(else_body.len(), 1);
                assert!(matches!(else_body[0], RustStmt::If { .. }));
            }
        }
        _ => panic!("expected if let stmt"),
    }
}

#[test]
fn lowers_if_with_option_truthiness_elif_clause() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::BoolLiteral(false),
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![(
            HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            },
            vec![HirStmt::Pass],
        )],
        else_body: Some(vec![HirStmt::Pass]),
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with option truthiness elif lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::If { else_body, .. } => {
            assert!(else_body.is_some());
            if let Some(else_body) = else_body {
                assert_eq!(else_body.len(), 1);
                assert!(matches!(
                    else_body[0],
                    RustStmt::IfLet {
                        pattern: ref p,
                        expr: RustExpr::Ident(ref n),
                        else_body: Some(_),
                        ..
                    } if p == "Some(maybe_x)" && n == "maybe_x"
                ));
            }
        }
        _ => panic!("expected if stmt"),
    }
}

#[test]
fn lowers_simple_if_with_elif() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Expr {
            expr: HirExpr::IntLiteral(1),
        }],
        elif_clauses: vec![(
            HirExpr::BoolLiteral(false),
            vec![HirStmt::Expr {
                expr: HirExpr::IntLiteral(2),
            }],
        )],
        else_body: Some(vec![HirStmt::Expr {
            expr: HirExpr::IntLiteral(3),
        }]),
    };

    let lowered = try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("if with elif lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::If { else_body, .. } => {
            assert!(else_body.is_some());
            if let Some(else_body) = else_body {
                assert_eq!(else_body.len(), 1);
                assert!(matches!(else_body[0], RustStmt::If { .. }));
            }
        }
        _ => panic!("expected if stmt"),
    }
}

#[test]
fn does_not_lower_if_with_non_leaf_elif_condition() {
    let if_stmt = HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Pass],
        elif_clauses: vec![(
            HirExpr::Call {
                func: "flag".to_string(),
                args: vec![],
                ty: Type::Bool,
            },
            vec![HirStmt::Pass],
        )],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&if_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}
