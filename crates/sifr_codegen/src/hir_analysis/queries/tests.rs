use super::*;
use crate::ModuleFuncSignatures;
use sifr_ir::{HirExpr, HirFunction, HirIteratorOp, HirParam, HirPattern, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashMap;

#[test]
fn collect_mutated_vars_marks_mutborrow_call_argument() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::Call {
            func: "touch".to_string(),
            args: vec![HirExpr::Name {
                name: "items".to_string(),
                ty: Type::List(Box::new(Type::Int)),
            }],
            ty: Type::None,
        },
    }];

    let mut sigs: ModuleFuncSignatures = HashMap::new();
    sigs.insert(
        "touch".to_string(),
        (
            vec![(
                Type::List(Box::new(Type::Int)),
                ParamConvention::mut_borrow(),
            )],
            Type::None,
        ),
    );

    let mutated = collect_mutated_vars(&stmts, Some(&sigs));
    assert!(mutated.contains("items"));
}

#[test]
fn collect_mutated_vars_marks_local_nested_function_mutborrow_call_argument() {
    let nested = HirFunction {
        name: "touch_local".to_string(),
        params: vec![HirParam {
            name: "xs".to_string(),
            ty: Type::List(Box::new(Type::Int)),
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::None,
        body: vec![HirStmt::Expr {
            expr: HirExpr::MethodCall {
                object: Box::new(HirExpr::Name {
                    name: "xs".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                }),
                method: "append".to_string(),
                args: vec![HirExpr::IntLiteral(1)],
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: vec![],
        type_params: vec![],
    };

    let stmts = vec![
        HirStmt::NestedFunction { func: nested },
        HirStmt::Expr {
            expr: HirExpr::Call {
                func: "touch_local".to_string(),
                args: vec![HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                }],
                ty: Type::None,
            },
        },
    ];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("items"));
}

#[test]
fn collect_mutated_vars_marks_iterator_next_argument() {
    let iterator_ty = Type::Class {
        name: "CountdownIter".to_string(),
        fields: vec![],
        methods: vec![(
            "__next__".to_string(),
            sifr_type_system::FunctionType {
                params: vec![],
                return_type: Box::new(Type::Union(vec![Type::Int, Type::None])),
            },
        )],
        parent_class: None,
    };
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::IteratorCall {
            op: HirIteratorOp::Next,
            args: vec![HirExpr::Name {
                name: "it".to_string(),
                ty: iterator_ty,
            }],
            ty: Type::Union(vec![Type::Int, Type::None]),
        },
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("it"));
}

#[test]
fn collect_mutated_vars_marks_anext_argument() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::Call {
            func: "anext".to_string(),
            args: vec![HirExpr::Name {
                name: "agen".to_string(),
                ty: Type::AsyncGenerator(Box::new(Type::Int), Box::new(Type::Never)),
            }],
            ty: Type::Awaitable(Box::new(Type::Result(
                Box::new(Type::Union(vec![Type::Int, Type::None])),
                Box::new(Type::Never),
            ))),
        },
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("agen"));
}

#[test]
fn body_calls_function_ignores_nested_function_scope() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![HirParam {
            name: "n".to_string(),
            ty: Type::Int,
            default: None,
            keyword_only: false,
            convention: ParamConvention::own(),
        }],
        return_type: Type::Int,
        body: vec![HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "target".to_string(),
                args: vec![HirExpr::Name {
                    name: "n".to_string(),
                    ty: Type::Int,
                }],
                ty: Type::Int,
            }),
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: vec![],
        type_params: vec![],
    };
    let stmts = vec![HirStmt::NestedFunction { func: nested }];

    assert!(!body_calls_function(&stmts, "target"));
}

#[test]
fn body_contains_yield_detects_try_except_and_loop_else_paths() {
    let stmts = vec![HirStmt::TryExcept {
        body: vec![HirStmt::While {
            condition: HirExpr::BoolLiteral(false),
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Yield {
                value: HirExpr::IntLiteral(1),
            }]),
        }],
        handlers: vec![sifr_ir::HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: None,
            name: Some("e".to_string()),
            body: vec![HirStmt::Yield {
                value: HirExpr::IntLiteral(2),
            }],
        }],
        body_error_types: vec!["Error".to_string()],
    }];

    assert!(body_contains_yield(&stmts));
}

#[test]
fn collect_locally_defined_vars_includes_match_captures() {
    let stmts = vec![HirStmt::Match {
        subject: HirExpr::IntLiteral(3),
        subject_ty: Type::Int,
        arms: vec![sifr_ir::HirMatchArm {
            pattern: HirPattern::Capture {
                name: "x".to_string(),
                ty: Type::Int,
            },
            guard: None,
            body: vec![HirStmt::Pass],
        }],
    }];

    let defined = collect_locally_defined_vars(&stmts);
    assert!(defined.contains("x"));
}

#[test]
fn collect_locally_defined_vars_ignores_nested_function_body_bindings() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![],
        return_type: Type::None,
        body: vec![HirStmt::Let {
            name: "nested_local".to_string(),
            ty: Type::Int,
            value: HirExpr::IntLiteral(1),
            is_mutable: true,
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: vec![],
        type_params: vec![],
    };

    let stmts = vec![HirStmt::NestedFunction { func: nested }];
    let defined = collect_locally_defined_vars(&stmts);

    assert!(defined.contains("inner"));
    assert!(!defined.contains("nested_local"));
}

#[test]
fn collect_mutated_vars_handles_nested_exprs() {
    let stmts = vec![HirStmt::Let {
        name: "x".to_string(),
        ty: Type::List(Box::new(Type::Int)),
        value: HirExpr::Call {
            func: "id".to_string(),
            args: vec![HirExpr::MethodCall {
                object: Box::new(HirExpr::Name {
                    name: "x".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                }),
                method: "append".to_string(),
                args: vec![HirExpr::IntLiteral(1)],
                ty: Type::None,
            }],
            ty: Type::None,
        },
        is_mutable: true,
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("x"));
}

#[test]
fn collect_mutated_vars_ignores_nested_function_scope() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![],
        return_type: Type::None,
        body: vec![HirStmt::Assign {
            name: "inside".to_string(),
            value: HirExpr::IntLiteral(1),
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: vec![],
        type_params: vec![],
    };

    let mutated = collect_mutated_vars(&[HirStmt::NestedFunction { func: nested }], None);
    assert!(!mutated.contains("inside"));
}

#[test]
fn collect_mutated_vars_includes_captured_rebinds_from_nested_functions() {
    let nested = HirFunction {
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
        type_params: vec![],
    };

    let mutated = collect_mutated_vars(&[HirStmt::NestedFunction { func: nested }], None);
    assert!(mutated.contains("total"));
}

#[test]
fn collect_mutated_vars_marks_captured_outer_mutation_from_nested_function() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![],
        return_type: Type::None,
        body: vec![HirStmt::Expr {
            expr: HirExpr::MethodCall {
                object: Box::new(HirExpr::Name {
                    name: "items".to_string(),
                    ty: Type::List(Box::new(Type::Int)),
                }),
                method: "append".to_string(),
                args: vec![HirExpr::IntLiteral(1)],
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        decorators: vec![],
        type_params: vec![],
    };

    let mutated = collect_mutated_vars(&[HirStmt::NestedFunction { func: nested }], None);
    assert!(mutated.contains("items"));
}

#[test]
fn collect_mutated_vars_marks_dict_setdefault_receiver() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "data".to_string(),
                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
            }),
            method: "setdefault".to_string(),
            args: vec![
                HirExpr::StringLiteral("k".to_string()),
                HirExpr::IntLiteral(1),
            ],
            ty: Type::Int,
        },
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("data"));
}

#[test]
fn collect_mutated_vars_marks_set_update_receiver() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "seen".to_string(),
                ty: Type::Set(Box::new(Type::Int)),
            }),
            method: "intersection_update".to_string(),
            args: vec![HirExpr::ListLiteral {
                elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
                ty: Type::List(Box::new(Type::Int)),
            }],
            ty: Type::None,
        },
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("seen"));
}

#[test]
fn collect_mutated_vars_marks_self_for_delegated_field_class_method_call() {
    let writer_ty = Type::Class {
        name: "writer".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    let holder_ty = Type::Class {
        name: "DictWriter".to_string(),
        fields: vec![("_writer".to_string(), writer_ty.clone())],
        methods: vec![],
        parent_class: None,
    };
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::MethodCall {
            object: Box::new(HirExpr::FieldAccess {
                object: Box::new(HirExpr::Name {
                    name: "self".to_string(),
                    ty: holder_ty,
                }),
                field: "_writer".to_string(),
                ty: writer_ty,
            }),
            method: "writerow".to_string(),
            args: vec![],
            ty: Type::None,
        },
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("self"));
}

#[test]
fn collect_mutated_vars_marks_field_assign_object() {
    let stmts = vec![HirStmt::FieldAssign {
        object: "root".to_string(),
        field: "left".to_string(),
        field_ty: Type::Int,
        value: HirExpr::IntLiteral(1),
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("root"));
}

#[test]
fn collect_typed_refs_in_expr_includes_fstring_interpolations() {
    let expr = HirExpr::FString {
        parts: vec![
            sifr_ir::HirFStringPart::Literal("value=".to_string()),
            sifr_ir::HirFStringPart::Expr(HirExpr::Name {
                name: "n".to_string(),
                ty: Type::Int,
            }),
        ],
        ty: Type::Str,
    };
    let mut refs = HashMap::new();
    collect_typed_refs_in_expr(&expr, &mut refs);

    assert_eq!(refs.get("n"), Some(&Type::Int));
}

#[test]
fn block_control_flow_effect_reports_always_returns_for_exhaustive_if() {
    let effect = block_control_flow_effect(&[HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        }],
        elif_clauses: vec![],
        else_body: Some(vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(2)),
        }]),
    }]);

    assert_eq!(effect, ControlFlowEffect::AlwaysReturns);
    assert!(effect.always_exits());
}

#[test]
fn block_control_flow_effect_reports_fallthrough_for_non_exhaustive_if() {
    let effect = block_control_flow_effect(&[HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        }],
        elif_clauses: vec![],
        else_body: None,
    }]);

    assert_eq!(effect, ControlFlowEffect::FallsThrough);
    assert!(!effect.always_exits());
}

#[test]
fn block_control_flow_effect_reports_always_exits_for_mixed_return_raise() {
    let effect = block_control_flow_effect(&[HirStmt::TryExcept {
        body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        }],
        handlers: vec![sifr_ir::HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: None,
            name: Some("e".to_string()),
            body: vec![HirStmt::Raise {
                value: HirExpr::Call {
                    func: "ValueError".to_string(),
                    args: vec![HirExpr::StringLiteral("bad".to_string())],
                    ty: Type::Unknown,
                },
            }],
        }],
        body_error_types: vec!["Error".to_string()],
    }]);

    assert_eq!(effect, ControlFlowEffect::AlwaysExits);
    assert!(effect.always_exits());
}

#[test]
fn reachable_stmt_indices_omit_unreachable_tail_after_return() {
    let stmts = vec![
        HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        },
        HirStmt::Expr {
            expr: HirExpr::IntLiteral(2),
        },
    ];
    assert_eq!(reachable_top_level_stmt_indices(&stmts), vec![0]);
    assert_eq!(unreachable_top_level_stmt_indices(&stmts), vec![1]);
}

#[test]
fn body_contains_return_ignores_unreachable_return() {
    let stmts = vec![
        HirStmt::Raise {
            value: HirExpr::Call {
                func: "ValueError".to_string(),
                args: vec![HirExpr::StringLiteral("bad".to_string())],
                ty: Type::Unknown,
            },
        },
        HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
        },
    ];
    assert!(!body_contains_return(&stmts));
}

#[test]
fn try_body_has_value_return_ignores_unreachable_value_return() {
    let stmts = vec![
        HirStmt::Raise {
            value: HirExpr::Call {
                func: "ValueError".to_string(),
                args: vec![HirExpr::StringLiteral("bad".to_string())],
                ty: Type::Unknown,
            },
        },
        HirStmt::Return {
            value: Some(HirExpr::IntLiteral(99)),
        },
    ];
    assert!(!try_body_has_value_return(&stmts));
}

#[test]
fn collect_typevar_operator_requirements_detects_add_and_sub() {
    let stmts = vec![
        HirStmt::Expr {
            expr: HirExpr::BinOp {
                left: Box::new(HirExpr::Name {
                    name: "a".to_string(),
                    ty: Type::TypeVar("T".to_string()),
                }),
                op: "+".to_string(),
                right: Box::new(HirExpr::Name {
                    name: "b".to_string(),
                    ty: Type::TypeVar("T".to_string()),
                }),
                ty: Type::TypeVar("T".to_string()),
            },
        },
        HirStmt::Expr {
            expr: HirExpr::BinOp {
                left: Box::new(HirExpr::Name {
                    name: "a".to_string(),
                    ty: Type::TypeVar("T".to_string()),
                }),
                op: "-".to_string(),
                right: Box::new(HirExpr::Name {
                    name: "b".to_string(),
                    ty: Type::TypeVar("T".to_string()),
                }),
                ty: Type::TypeVar("T".to_string()),
            },
        },
    ];

    let req = collect_typevar_operator_requirements(&stmts, "T");
    assert!(req.needs_add);
    assert!(req.needs_sub);
}

#[test]
fn collect_let_declared_types_covers_nested_blocks() {
    let stmts = vec![HirStmt::If {
        condition: HirExpr::BoolLiteral(true),
        then_body: vec![HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Union(vec![Type::Int, Type::Str]),
            value: HirExpr::IntLiteral(1),
            is_mutable: true,
        }],
        elif_clauses: vec![],
        else_body: None,
    }];

    let declared = collect_let_declared_types(&stmts);
    assert_eq!(declared.len(), 1);
    assert!(matches!(declared[0], Type::Union(_)));
}
