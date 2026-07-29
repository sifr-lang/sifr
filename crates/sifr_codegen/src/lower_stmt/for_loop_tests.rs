use super::*;
#[test]
fn lowers_simple_while_with_else() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::BoolLiteral(true),
        body: vec![HirStmt::Pass],
        else_body: Some(vec![HirStmt::Pass]),
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while with else lowered");
    assert_eq!(lowered.len(), 3);
    assert!(matches!(lowered[0], RustStmt::Let { .. }));
    assert!(matches!(lowered[1], RustStmt::While { .. }));
    assert!(matches!(lowered[2], RustStmt::If { .. }));
}

#[test]
fn lowers_simple_for_without_else() {
    let for_stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::Break],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(
        &for_stmt,
        true, // outer loop-else context should not leak into inner loop body
        &HashSet::new(),
        &HashSet::new(),
    )
    .expect("for lowered");
    assert_eq!(lowered.len(), 1);
    match &lowered[0] {
        RustStmt::For { body, .. } => {
            assert_eq!(body.len(), 1);
            assert!(matches!(body[0], RustStmt::Break));
        }
        _ => panic!("expected RustStmt::For"),
    }
}

#[test]
fn lowers_simple_for_with_name_iter_using_copy_mode() {
    let for_stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::Name {
            name: "items".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("for with name iter lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::For {
            var: ref var_name,
            iter: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if var_name == "i"
            && matches!(
                recv.as_ref(),
                RustExpr::MethodCall {
                    receiver: ref inner_recv,
                    ref method,
                    ref args,
                } if matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "items")
                    && method == "iter"
                    && args.is_empty()
            )
            && method == "copied"
            && args.is_empty()
    ));
}

#[test]
fn lowers_simple_for_over_task_handle_list_by_value() {
    let handle_ty = Type::Task(Box::new(Type::Int), Box::new(Type::Never));
    let for_stmt = HirStmt::For {
        target: "handle".to_string(),
        target_ty: handle_ty.clone(),
        iter: HirExpr::Name {
            name: "handles".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(handle_ty)),
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("for with task handle list lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::For {
            var: ref var_name,
            iter: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if var_name == "handle"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "handles")
            && method == "into_iter"
            && args.is_empty()
    ));
}

#[test]
fn lowers_simple_for_with_dict_iter_to_keys_cloned() {
    let for_stmt = HirStmt::For {
        target: "k".to_string(),
        target_ty: Type::Str,
        iter: HirExpr::Name {
            name: "m".to_string(),
            binding_id: None,
            ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    let lowered = try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("for with dict iter lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::For {
            iter: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if matches!(
            recv.as_ref(),
            RustExpr::MethodCall {
                receiver: ref inner_recv,
                ref method,
                ref args,
            } if matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "m")
                && method == "keys"
                && args.is_empty()
        )
            && method == "cloned"
            && args.is_empty()
    ));
}

#[test]
fn does_not_lower_for_with_non_leaf_iter() {
    let for_stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::Call {
            func: "items".to_string(),
            args: vec![],
            ty: Type::List(Box::new(Type::Int)),
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };

    assert!(try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_for_with_else() {
    let for_with_else = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::Pass],
        else_body: Some(vec![HirStmt::Pass]),
    };
    let lowered = try_lower_simple_stmt(&for_with_else, false, &HashSet::new(), &HashSet::new())
        .expect("for with else lowered");
    assert_eq!(lowered.len(), 3);
    assert!(matches!(lowered[0], RustStmt::Let { .. }));
    assert!(matches!(lowered[1], RustStmt::For { .. }));
    assert!(matches!(lowered[2], RustStmt::If { .. }));
}

#[test]
fn lowers_simple_for_with_else_and_name_iter() {
    let for_with_else = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::Name {
            name: "items".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        },
        body: vec![HirStmt::Pass],
        else_body: Some(vec![HirStmt::Pass]),
    };
    let lowered = try_lower_simple_stmt(&for_with_else, false, &HashSet::new(), &HashSet::new())
        .expect("for with else and name iter lowered");
    assert_eq!(lowered.len(), 3);
    assert!(matches!(lowered[0], RustStmt::Let { .. }));
    assert!(matches!(
        lowered[1],
        RustStmt::For {
            iter: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if matches!(
            recv.as_ref(),
            RustExpr::MethodCall {
                receiver: ref inner_recv,
                ref method,
                ref args,
            } if matches!(inner_recv.as_ref(), RustExpr::Ident(name) if name == "items")
                && method == "iter"
                && args.is_empty()
        )
            && method == "copied"
            && args.is_empty()
    ));
    assert!(matches!(lowered[2], RustStmt::If { .. }));
}

#[test]
fn does_not_lower_for_with_else_and_non_leaf_iter() {
    let for_with_else = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::Call {
            func: "items".to_string(),
            args: vec![],
            ty: Type::List(Box::new(Type::Int)),
        },
        body: vec![HirStmt::Pass],
        else_body: Some(vec![HirStmt::Pass]),
    };
    assert!(
        try_lower_simple_stmt(&for_with_else, false, &HashSet::new(), &HashSet::new(),).is_none()
    );
}

#[test]
fn does_not_lower_for_with_tuple_target() {
    let for_tuple_target = HirStmt::For {
        target: "i,v".to_string(),
        target_ty: Type::Tuple(vec![Type::Int, Type::Int]),
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::Pass],
        else_body: None,
    };
    assert!(
        try_lower_simple_stmt(&for_tuple_target, false, &HashSet::new(), &HashSet::new(),)
            .is_none()
    );
}

#[test]
fn lowers_for_else_with_broke_marker_in_loop_body() {
    let for_stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::Break],
        else_body: Some(vec![HirStmt::Expr {
            expr: HirExpr::IntLiteral(1),
        }]),
    };

    let lowered = try_lower_simple_stmt(&for_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("for else lowered");

    match &lowered[1] {
        RustStmt::For { body, .. } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], RustStmt::Assign { .. }));
            assert!(matches!(body[1], RustStmt::Break));
        }
        _ => panic!("expected for stmt"),
    }
}

#[test]
fn for_else_body_break_uses_outer_loop_else_context() {
    let for_stmt = HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(3)),
            step: None,
            ty: Type::Range,
        },
        body: vec![HirStmt::Pass],
        else_body: Some(vec![HirStmt::Break]),
    };

    let lowered = try_lower_simple_stmt(&for_stmt, true, &HashSet::new(), &HashSet::new())
        .expect("for else lowered");

    match &lowered[2] {
        RustStmt::If { then_body, .. } => {
            assert_eq!(then_body.len(), 2);
            assert!(matches!(then_body[0], RustStmt::Assign { .. }));
            assert!(matches!(then_body[1], RustStmt::Break));
        }
        _ => panic!("expected if stmt"),
    }
}

#[test]
fn lowers_while_else_with_broke_marker_in_loop_body() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::BoolLiteral(true),
        body: vec![HirStmt::Break],
        else_body: Some(vec![HirStmt::Expr {
            expr: HirExpr::IntLiteral(1),
        }]),
    };

    let lowered = try_lower_simple_stmt(&while_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("while else lowered");

    match &lowered[1] {
        RustStmt::While { body, .. } => {
            assert_eq!(body.len(), 2);
            assert!(matches!(body[0], RustStmt::Assign { .. }));
            assert!(matches!(body[1], RustStmt::Break));
        }
        _ => panic!("expected while stmt"),
    }
}

#[test]
fn while_else_body_break_uses_outer_loop_else_context() {
    let while_stmt = HirStmt::While {
        condition: HirExpr::BoolLiteral(false),
        body: vec![HirStmt::Pass],
        else_body: Some(vec![HirStmt::Break]),
    };

    let lowered = try_lower_simple_stmt(&while_stmt, true, &HashSet::new(), &HashSet::new())
        .expect("while else lowered");

    match &lowered[2] {
        RustStmt::If { then_body, .. } => {
            assert_eq!(then_body.len(), 2);
            assert!(matches!(then_body[0], RustStmt::Assign { .. }));
            assert!(matches!(then_body[1], RustStmt::Break));
        }
        _ => panic!("expected if stmt"),
    }
}
