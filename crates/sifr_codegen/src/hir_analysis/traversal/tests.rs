use super::{walk_expr_until, walk_stmts, walk_stmts_until, TraversalConfig, TraversalControl};
use sifr_ir::{
    HirExceptHandler, HirExpr, HirFunction, HirMatchArm, HirParam, HirPattern, HirStmt, MethodKind,
};
use sifr_type_system::{ParamConvention, Type};

#[test]
fn walk_stmts_covers_try_handlers_loop_else_and_match_patterns() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![HirParam {
            name: "p".to_string(),
            ty: Type::Int,
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::None,
        body: vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "nested_only".to_string(),
                args: vec![],
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: vec![],
        rust_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };

    let stmts = vec![
        HirStmt::TryExcept {
            body: vec![HirStmt::For {
                target: "item".to_string(),
                target_ty: Type::Int,
                iter: HirExpr::ListLiteral {
                    elements: vec![HirExpr::IntLiteral(1)],
                    ty: Type::List(Box::new(Type::Int)),
                },
                body: vec![HirStmt::Pass],
                else_body: Some(vec![HirStmt::Expr {
                    expr: HirExpr::Call {
                        func: "loop_else_call".to_string(),
                        args: vec![],
                        ty: Type::None,
                    },
                }]),
            }],
            handlers: vec![HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Match {
                    subject: HirExpr::Name {
                        name: "value".to_string(),
                        ty: Type::Int,
                    },
                    subject_ty: Type::Int,
                    arms: vec![HirMatchArm {
                        pattern: HirPattern::Literal {
                            value: HirExpr::Call {
                                func: "pattern_expr".to_string(),
                                args: vec![],
                                ty: Type::Int,
                            },
                        },
                        guard: Some(HirExpr::Call {
                            func: "guard_expr".to_string(),
                            args: vec![],
                            ty: Type::Bool,
                        }),
                        body: vec![HirStmt::Expr {
                            expr: HirExpr::Call {
                                func: "arm_body_call".to_string(),
                                args: vec![],
                                ty: Type::None,
                            },
                        }],
                    }],
                }],
            }],
            body_error_types: vec!["Error".to_string()],
        },
        HirStmt::NestedFunction { func: nested },
    ];

    let mut calls = Vec::<String>::new();
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::Call { func, .. } = expr {
            calls.push(func.clone());
        }
    };
    walk_stmts(
        &stmts,
        TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut on_stmt,
        &mut on_expr,
    );

    assert!(calls.contains(&"loop_else_call".to_string()));
    assert!(calls.contains(&"pattern_expr".to_string()));
    assert!(calls.contains(&"guard_expr".to_string()));
    assert!(calls.contains(&"arm_body_call".to_string()));
    assert!(calls.contains(&"nested_only".to_string()));
}

#[test]
fn walk_stmts_respects_nested_function_scope_boundary() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![],
        return_type: Type::None,
        body: vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "nested_only".to_string(),
                args: vec![],
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: vec![],
        rust_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };
    let stmts = vec![HirStmt::NestedFunction { func: nested }];

    let mut saw_nested_call = false;
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::Call { func, .. } = expr {
            if func == "nested_only" {
                saw_nested_call = true;
            }
        }
    };

    walk_stmts(
        &stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut on_stmt,
        &mut on_expr,
    );

    assert!(!saw_nested_call);
}

#[test]
fn walk_expr_until_stops_at_first_match() {
    let expr = HirExpr::TupleLiteral {
        elements: vec![
            HirExpr::Call {
                func: "first".to_string(),
                args: vec![],
                ty: Type::None,
            },
            HirExpr::Call {
                func: "second".to_string(),
                args: vec![],
                ty: Type::None,
            },
        ],
        ty: Type::Tuple(vec![Type::None, Type::None]),
    };

    let mut seen_calls = Vec::new();
    let result = walk_expr_until(&expr, &mut |node| {
        if let HirExpr::Call { func, .. } = node {
            seen_calls.push(func.clone());
            if func == "first" {
                return TraversalControl::Stop;
            }
        }
        TraversalControl::Continue
    });

    assert_eq!(result, TraversalControl::Stop);
    assert_eq!(seen_calls, vec!["first".to_string()]);
}

#[test]
fn walk_stmts_until_stops_before_later_statements() {
    let stmts = vec![
        HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        },
        HirStmt::Expr {
            expr: HirExpr::Call {
                func: "later".to_string(),
                args: vec![],
                ty: Type::None,
            },
        },
    ];

    let mut seen_stmt_kinds = Vec::new();
    let mut seen_later_call = false;
    let result = walk_stmts_until(
        &stmts,
        TraversalConfig::LOCAL_SCOPE_ONLY,
        &mut |stmt| {
            if matches!(stmt, HirStmt::Return { .. }) {
                seen_stmt_kinds.push("return");
                return TraversalControl::Stop;
            }
            seen_stmt_kinds.push("other");
            TraversalControl::Continue
        },
        &mut |expr| {
            if let HirExpr::Call { func, .. } = expr {
                if func == "later" {
                    seen_later_call = true;
                }
            }
            TraversalControl::Continue
        },
    );

    assert_eq!(result, TraversalControl::Stop);
    assert_eq!(seen_stmt_kinds, vec!["return"]);
    assert!(!seen_later_call);
}
