use super::*;
#[test]
fn lowers_simple_match_with_literal_and_wildcard_patterns() {
    let stmt = HirStmt::Match {
        subject: HirExpr::Name {
            name: "n".to_string(),
            ty: Type::Int,
        },
        subject_ty: Type::Int,
        arms: vec![
            sifr_ir::HirMatchArm {
                pattern: HirPattern::Literal {
                    value: HirExpr::IntLiteral(1),
                },
                guard: None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::IntLiteral(10),
                }],
            },
            sifr_ir::HirMatchArm {
                pattern: HirPattern::Wildcard,
                guard: None,
                body: vec![HirStmt::Expr {
                    expr: HirExpr::IntLiteral(0),
                }],
            },
        ],
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("match lowered");
    assert!(matches!(lowered[0], RustStmt::Match { .. }));
}

#[test]
fn lowers_match_with_class_patterns_and_captures() {
    let point_ty = Type::Class {
        name: "Point".to_string(),
        fields: vec![("x".to_string(), Type::Int), ("y".to_string(), Type::Int)],
        methods: vec![],
        parent_class: None,
    };
    let stmt = HirStmt::Match {
        subject: HirExpr::Name {
            name: "p".to_string(),
            ty: point_ty.clone(),
        },
        subject_ty: point_ty,
        arms: vec![
            sifr_ir::HirMatchArm {
                pattern: HirPattern::Class {
                    class_name: "Point".to_string(),
                    fields: vec![
                        (
                            "x".to_string(),
                            HirPattern::Literal {
                                value: HirExpr::IntLiteral(0),
                            },
                        ),
                        (
                            "y".to_string(),
                            HirPattern::Capture {
                                name: "py".to_string(),
                                ty: Type::Int,
                            },
                        ),
                    ],
                },
                guard: None,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::StringLiteral("axis".to_string())),
                }],
            },
            sifr_ir::HirMatchArm {
                pattern: HirPattern::Wildcard,
                guard: None,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::StringLiteral("other".to_string())),
                }],
            },
        ],
    };

    let lowered = try_lower_simple_stmt_with_ctx(
        &stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx {
            return_type: Some(&Type::Str),
            in_display_impl: false,
            in_class_scope: false,
            in_generator_closure: false,
        },
    )
    .expect("class match lowered");

    assert!(matches!(
        lowered[0],
        RustStmt::Match { ref arms, .. }
            if arms.len() == 2
                && arms[0].pattern.contains("Point { x: 0, y: py")
                && arms[0].bindings.iter().any(|name| name == "py")
    ));
}

#[test]
fn lowers_match_with_string_literal_patterns() {
    let stmt = HirStmt::Match {
        subject: HirExpr::Name {
            name: "method".to_string(),
            ty: Type::Str,
        },
        subject_ty: Type::Str,
        arms: vec![
            sifr_ir::HirMatchArm {
                pattern: HirPattern::Literal {
                    value: HirExpr::StringLiteral("GET".to_string()),
                },
                guard: None,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::StringLiteral("read".to_string())),
                }],
            },
            sifr_ir::HirMatchArm {
                pattern: HirPattern::Wildcard,
                guard: None,
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::StringLiteral("other".to_string())),
                }],
            },
        ],
    };

    let lowered = try_lower_simple_stmt_with_ctx(
        &stmt,
        false,
        &HashSet::new(),
        &HashSet::new(),
        SimpleStmtLoweringCtx {
            return_type: Some(&Type::Str),
            in_display_impl: false,
            in_class_scope: false,
            in_generator_closure: false,
        },
    )
    .expect("string match lowered");

    assert!(matches!(
        lowered[0],
        RustStmt::Match { ref arms, .. }
            if arms.len() == 2
                && arms[0].pattern == "__s"
                && arms[0].guard.is_some()
    ));
}

#[test]
fn lowers_simple_nested_function_to_closure_block() {
    let stmt = HirStmt::NestedFunction {
        func: HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        },
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("nested function lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            ref name,
            value: RustExpr::ClosureBlock { .. },
            ..
        } if name == "inner"
    ));
}

#[test]
fn lowers_recursive_nested_function_without_captures_to_local_fn() {
    let stmt = HirStmt::NestedFunction {
        func: HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "inner".to_string(),
                    args: vec![],
                    ty: Type::Int,
                },
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        },
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("recursive nested function lowered");
    assert!(matches!(
        lowered[0],
        RustStmt::LocalFn { ref name, .. } if name == "inner"
    ));
}

#[test]
fn lowers_mutating_capture_nested_function_to_mutable_closure_binding() {
    let stmt = HirStmt::NestedFunction {
        func: HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::None,
            body: vec![HirStmt::AugAssign {
                name: "total".to_string(),
                op: "+=".to_string(),
                value: HirExpr::IntLiteral(1),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: vec![],
        },
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("mutating nested function lowered");

    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: true,
            ref name,
            value: RustExpr::ClosureBlock { .. },
            ..
        } if name == "inner"
    ));
}

#[test]
fn lowers_simple_try_except_catch_all_with_result_flow() {
    let stmt = HirStmt::TryExcept {
        body: vec![HirStmt::Expr {
            expr: HirExpr::QuestionMark {
                expr: Box::new(HirExpr::Name {
                    name: "res".to_string(),
                    ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
                }),
                ty: Type::Int,
            },
        }],
        handlers: vec![HirExceptHandler {
            error_type: None,
            error_resolved_type: None,
            name: None,
            body: vec![HirStmt::Pass],
        }],
        body_error_types: vec!["Error".to_string()],
    };
    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("try/except lowered");
    assert_eq!(lowered.len(), 2);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            ref name,
            value: RustExpr::FnCall { .. },
            ..
        } if name == "__sifr_try_res"
    ));
    assert!(matches!(
        lowered[1],
        RustStmt::IfLet {
            ref pattern,
            expr: RustExpr::Ident(ref expr_name),
            ..
        } if pattern == "Err(_e)" && expr_name == "__sifr_try_res"
    ));
}

#[test]
fn does_not_lower_try_except_with_typed_handler() {
    let stmt = HirStmt::TryExcept {
        body: vec![HirStmt::Expr {
            expr: HirExpr::QuestionMark {
                expr: Box::new(HirExpr::Name {
                    name: "res".to_string(),
                    ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
                }),
                ty: Type::Int,
            },
        }],
        handlers: vec![HirExceptHandler {
            error_type: Some("IOError".to_string()),
            error_resolved_type: Some(Type::Class {
                name: "IOError".to_string(),
                fields: vec![],
                methods: vec![],
                parent_class: None,
            }),
            name: None,
            body: vec![HirStmt::Pass],
        }],
        body_error_types: vec!["IOError".to_string()],
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}

#[test]
fn does_not_lower_try_except_without_result_flow() {
    let stmt = HirStmt::TryExcept {
        body: vec![HirStmt::Pass],
        handlers: vec![HirExceptHandler {
            error_type: None,
            error_resolved_type: None,
            name: None,
            body: vec![HirStmt::Pass],
        }],
        body_error_types: vec!["Error".to_string()],
    };
    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
}
