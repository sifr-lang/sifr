use super::*;

#[test]
fn lowers_leaf_expression_statement() {
    let stmts = try_lower_expr_stmt(&HirExpr::IntLiteral(1)).expect("leaf stmt lowered");
    assert_eq!(stmts.len(), 1);
    assert!(matches!(stmts[0], RustStmt::Expr(_)));
}

#[test]
fn scope_result_reports_invalid_scope_context() {
    let stmt = HirStmt::Pass;
    let scope_ctx = ScopeContext {
        in_display_impl: true,
        in_generator_closure: true,
        ..ScopeContext::default()
    };

    let err = try_lower_simple_stmt_with_scope_result(
        &stmt,
        &HashSet::new(),
        &HashSet::new(),
        &scope_ctx,
    )
    .expect_err("expected invalid scope context to return lowering error");

    assert!(err
        .message
        .contains("display impl and generator closure cannot both be active"));
}

#[test]
fn scope_result_propagates_stmt_expr_shape_errors() {
    let stmt = HirStmt::Let {
        name: "ok".to_string(),
        ty: Type::Bool,
        value: HirExpr::Compare {
            left: Box::new(HirExpr::IntLiteral(1)),
            ops: vec!["==".to_string()],
            comparators: vec![],
            ty: Type::Bool,
        },
        is_mutable: false,
    };

    let err = try_lower_simple_stmt_with_scope_result(
        &stmt,
        &HashSet::new(),
        &HashSet::new(),
        &ScopeContext::default(),
    )
    .expect_err("invalid compare shape should return lowering error");

    assert!(err.message.contains("ops/comparators length mismatch"));
}

#[test]
fn lowers_pass_and_continue_and_break() {
    let pass = try_lower_simple_stmt(&HirStmt::Pass, false, &HashSet::new(), &HashSet::new())
        .expect("pass lowered");
    assert!(pass.is_empty());

    let cont = try_lower_simple_stmt(&HirStmt::Continue, false, &HashSet::new(), &HashSet::new())
        .expect("continue lowered");
    assert!(matches!(cont[0], RustStmt::Continue));

    let brk = try_lower_simple_stmt(&HirStmt::Break, true, &HashSet::new(), &HashSet::new())
        .expect("break lowered");
    assert_eq!(brk.len(), 2);
    assert!(matches!(brk[0], RustStmt::Assign { .. }));
    assert!(matches!(brk[1], RustStmt::Break));
}

#[test]
fn lowers_simple_let_and_assign() {
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Int,
        value: HirExpr::IntLiteral(1),
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(
        &let_stmt,
        false,
        &HashSet::from(["x".to_string()]),
        &HashSet::new(),
    )
    .expect("let lowered");
    assert!(matches!(lowered[0], RustStmt::Let { mutable: true, .. }));

    let assign_stmt = HirStmt::Assign {
        name: "x".to_string(),
        value: HirExpr::IntLiteral(2),
    };
    let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assign lowered");
    assert!(matches!(lowered[0], RustStmt::Assign { .. }));
}

#[test]
fn simple_let_declines_non_optional_list_index_to_allow_structured_lowering() {
    let let_stmt = HirStmt::Let {
        name: "first".to_string(),
        ty: Type::Int,
        value: HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "values".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }),
            index: Box::new(HirExpr::IntLiteral(0)),
            ty: Type::Int,
        },
        is_mutable: false,
    };

    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new());

    assert!(
        lowered.is_none(),
        "non-optional proven list indexes should bypass the simple stmt path"
    );
}

#[test]
fn simple_return_declines_non_optional_string_index_to_allow_structured_lowering() {
    let return_stmt = HirStmt::Return {
        value: Some(HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "text".to_string(),
                ty: Type::Str,
            }),
            index: Box::new(HirExpr::Name {
                name: "j".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Str,
        }),
    };

    let lowered = try_lower_simple_stmt_with_ctx(
        &return_stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx {
            return_type: Some(&Type::Str),
            ..SimpleStmtLoweringCtx::default()
        },
    );

    assert!(
        lowered.is_none(),
        "non-optional proven string indexes should bypass the simple return path"
    );
}

#[test]
fn simple_compare_condition_wraps_proven_list_index_without_double_option() {
    let expr = HirExpr::Compare {
        left: Box::new(HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "actual".to_string(),
                ty: Type::List(Box::new(Type::Bool)),
            }),
            index: Box::new(HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Bool,
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::Index {
            object: Box::new(HirExpr::Name {
                name: "expected".to_string(),
                ty: Type::List(Box::new(Type::Bool)),
            }),
            index: Box::new(HirExpr::Name {
                name: "i".to_string(),
                ty: Type::Int,
            }),
            ty: Type::Union(vec![Type::Bool, Type::None]),
        }],
        ty: Type::Bool,
    };

    let lowered = try_lower_simple_condition_test_expr(&expr, &HashSet::new())
        .expect("compare condition should lower");

    assert!(matches!(
        lowered,
        RustExpr::BinOp { left, right, .. }
            if matches!(
                left.as_ref(),
                RustExpr::FnCall { func, args }
                    if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Some".to_string()])
                        && matches!(
                            args.as_slice(),
                            [RustExpr::Index { .. }]
                        )
            ) && matches!(
                right.as_ref(),
                RustExpr::MethodCall { method, .. } if method == "copied"
            )
    ));
}

#[test]
fn lowers_simple_field_assign_for_non_self_target() {
    let stmt = HirStmt::FieldAssign {
        object: "node".to_string(),
        field: "value".to_string(),
        field_ty: Type::Int,
        value: HirExpr::Name {
            name: "next_value".to_string(),
            ty: Type::Int,
        },
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("field assign lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assign {
            target: RustExpr::Field { ref expr, ref field },
            value: RustExpr::Ident(ref rhs),
        } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "node")
            && field == "value"
            && rhs == "next_value"
    ));
}

#[test]
fn does_not_lower_field_assign_on_self_target() {
    let stmt = HirStmt::FieldAssign {
        object: "self".to_string(),
        field: "value".to_string(),
        field_ty: Type::Int,
        value: HirExpr::IntLiteral(1),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn does_not_lower_field_assign_with_non_leaf_value() {
    let stmt = HirStmt::FieldAssign {
        object: "node".to_string(),
        field: "value".to_string(),
        field_ty: Type::Int,
        value: HirExpr::Call {
            func: "compute".to_string(),
            args: vec![],
            ty: Type::Int,
        },
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn lowers_simple_tuple_unpack_stmt() {
    let tuple_unpack = HirStmt::TupleUnpack {
        targets: vec![
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("a".to_string()),
                ty: Type::Int,
                rebind_existing: false,
            },
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("b".to_string()),
                ty: Type::Bool,
                rebind_existing: false,
            },
        ],
        value: HirExpr::TupleLiteral {
            elements: vec![HirExpr::IntLiteral(1), HirExpr::BoolLiteral(true)],
            ty: Type::Tuple(vec![Type::Int, Type::Bool]),
        },
    };
    let lowered = try_lower_simple_stmt(&tuple_unpack, false, &HashSet::new(), &HashSet::new())
        .expect("tuple unpack lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::LetPattern {
            ref pattern,
            value: RustExpr::Tuple(ref elements),
        } if pattern == "(a, b)" && elements.len() == 2
    ));
}

#[test]
fn lowers_simple_tuple_unpack_stmt_with_mutated_bindings() {
    let tuple_unpack = HirStmt::TupleUnpack {
        targets: vec![
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("l".to_string()),
                ty: Type::Int,
                rebind_existing: false,
            },
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("r".to_string()),
                ty: Type::Int,
                rebind_existing: false,
            },
        ],
        value: HirExpr::TupleLiteral {
            elements: vec![HirExpr::IntLiteral(0), HirExpr::IntLiteral(4)],
            ty: Type::Tuple(vec![Type::Int, Type::Int]),
        },
    };
    let mutated_vars = HashSet::from(["l".to_string(), "r".to_string(), "mid".to_string()]);
    let lowered = try_lower_simple_stmt(&tuple_unpack, false, &mutated_vars, &HashSet::new())
        .expect("tuple unpack lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::LetPattern {
            ref pattern,
            value: RustExpr::Tuple(ref elements),
        } if pattern == "(mut l, mut r)" && elements.len() == 2
    ));
}

#[test]
fn does_not_lower_tuple_unpack_with_non_leaf_value() {
    let tuple_unpack = HirStmt::TupleUnpack {
        targets: vec![
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("a".to_string()),
                ty: Type::Int,
                rebind_existing: false,
            },
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("b".to_string()),
                ty: Type::Bool,
                rebind_existing: false,
            },
        ],
        value: HirExpr::Call {
            func: "pair".to_string(),
            args: vec![],
            ty: Type::Tuple(vec![Type::Int, Type::Bool]),
        },
    };

    assert!(
        try_lower_simple_stmt(&tuple_unpack, false, &HashSet::new(), &HashSet::new(),).is_none()
    );
}

#[test]
fn lowers_tuple_unpack_with_rebind_targets_to_temp_and_assigns() {
    let tuple_unpack = HirStmt::TupleUnpack {
        targets: vec![
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("left".to_string()),
                ty: Type::Int,
                rebind_existing: true,
            },
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Name("right".to_string()),
                ty: Type::Int,
                rebind_existing: false,
            },
        ],
        value: HirExpr::TupleLiteral {
            elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
            ty: Type::Tuple(vec![Type::Int, Type::Int]),
        },
    };

    let lowered = lower_tuple_unpack_targets(
        match &tuple_unpack {
            HirStmt::TupleUnpack { targets, .. } => targets,
            _ => unreachable!(),
        },
        RustExpr::Tuple(vec![
            RustExpr::Literal(RustLiteral::Int(1)),
            RustExpr::Literal(RustLiteral::Int(2)),
        ]),
        &HashSet::new(),
    );

    assert!(matches!(
        lowered.as_slice(),
        [
            RustStmt::LetPattern { pattern, .. },
            RustStmt::Assign { target: RustExpr::Ident(left), value: RustExpr::Ident(tmp0) },
            RustStmt::Let { name: right, value: RustExpr::Ident(tmp1), .. }
        ] if pattern == "(__sifr_tuple_unpack_0, __sifr_tuple_unpack_1)"
            && left == "left"
            && right == "right"
            && tmp0 == "__sifr_tuple_unpack_0"
            && tmp1 == "__sifr_tuple_unpack_1"
    ));
}

#[test]
fn lowers_tuple_unpack_with_field_targets_to_temp_and_field_assigns() {
    let tuple_unpack = HirStmt::TupleUnpack {
        targets: vec![
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Field {
                    object: "node".to_string(),
                    field: "left".to_string(),
                },
                ty: Type::Int,
                rebind_existing: false,
            },
            sifr_ir::HirTupleTarget {
                binding: sifr_ir::HirTupleTargetBinding::Field {
                    object: "node".to_string(),
                    field: "right".to_string(),
                },
                ty: Type::Int,
                rebind_existing: false,
            },
        ],
        value: HirExpr::TupleLiteral {
            elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
            ty: Type::Tuple(vec![Type::Int, Type::Int]),
        },
    };

    let lowered = lower_tuple_unpack_targets(
        match &tuple_unpack {
            HirStmt::TupleUnpack { targets, .. } => targets,
            _ => unreachable!(),
        },
        RustExpr::Tuple(vec![
            RustExpr::Literal(RustLiteral::Int(1)),
            RustExpr::Literal(RustLiteral::Int(2)),
        ]),
        &HashSet::new(),
    );

    assert_eq!(lowered.len(), 3);
    let RustStmt::LetPattern { pattern, .. } = &lowered[0] else {
        panic!("expected temp tuple pattern");
    };
    assert_eq!(pattern, "(__sifr_tuple_unpack_0, __sifr_tuple_unpack_1)");

    let RustStmt::Assign { target, value } = &lowered[1] else {
        panic!("expected first field assignment");
    };
    let RustExpr::Field { expr, field } = target else {
        panic!("expected first field target");
    };
    let RustExpr::Ident(object) = expr.as_ref() else {
        panic!("expected field base identifier");
    };
    assert_eq!(object, "node");
    assert_eq!(field, "left");
    let RustExpr::Ident(tmp0) = value else {
        panic!("expected first tuple temp value");
    };
    assert_eq!(tmp0, "__sifr_tuple_unpack_0");

    let RustStmt::Assign { target, value } = &lowered[2] else {
        panic!("expected second field assignment");
    };
    let RustExpr::Field { expr, field } = target else {
        panic!("expected second field target");
    };
    let RustExpr::Ident(object) = expr.as_ref() else {
        panic!("expected field base identifier");
    };
    assert_eq!(object, "node");
    assert_eq!(field, "right");
    let RustExpr::Ident(tmp1) = value else {
        panic!("expected second tuple temp value");
    };
    assert_eq!(tmp1, "__sifr_tuple_unpack_1");
}
