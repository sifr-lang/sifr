use super::*;
use crate::ModuleFuncSignatures;
use sifr_ir::{HirExpr, HirFunction, HirIteratorOp, HirParam, HirPattern, HirStmt, MethodKind};
use sifr_type_system::{ParamConvention, Type};
use std::collections::HashMap;

fn test_error_type(name: &str) -> Type {
    Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: name.to_string(),
        fields: Vec::new(),
        methods: Vec::new(),
        parent_class: Some("Error".to_string()),
    }
}

#[test]
fn collect_mutated_vars_marks_mutborrow_call_argument() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "touch".to_string(),
            args: vec![HirExpr::Name {
                name: "items".to_string(),
                binding_id: None,
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
fn collect_mutated_vars_marks_generic_mutborrow_call_argument() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::GenericCall {
            mutable_arg_places: Vec::new(),
            func: "touch".to_string(),
            type_args: vec![Type::Int],
            args: vec![HirExpr::Name {
                name: "items".to_string(),
                binding_id: None,
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
fn collect_mutated_vars_marks_method_mutborrow_argument() {
    let crate_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Crate".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "receiver".to_string(),
                binding_id: None,
                ty: crate_ty.clone(),
            }),
            method: "merge".to_string(),
            args: vec![HirExpr::Name {
                name: "other".to_string(),
                binding_id: None,
                ty: crate_ty.clone(),
            }],
            receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: None,
            ty: Type::None,
        },
    }];
    let mut sigs: ModuleFuncSignatures = HashMap::new();
    sigs.insert(
        "Crate::merge".to_string(),
        (vec![(crate_ty, ParamConvention::mut_borrow())], Type::None),
    );

    let mutated = collect_mutated_vars(&stmts, Some(&sigs));

    assert!(mutated.contains("other"));
}

#[test]
fn collect_mutated_vars_marks_method_mutborrow_field_argument_root() {
    let crate_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Crate".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    let depot_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "Depot".to_string(),
        fields: vec![("stock".to_string(), crate_ty.clone())],
        methods: vec![],
        parent_class: None,
    };
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "receiver".to_string(),
                binding_id: None,
                ty: crate_ty.clone(),
            }),
            method: "merge".to_string(),
            args: vec![HirExpr::FieldAccess {
                object: Box::new(HirExpr::Name {
                    name: "depot".to_string(),
                    binding_id: None,
                    ty: depot_ty,
                }),
                field: "stock".to_string(),
                ty: crate_ty.clone(),
            }],
            receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: None,
            ty: Type::None,
        },
    }];
    let mut sigs: ModuleFuncSignatures = HashMap::new();
    sigs.insert(
        "Crate::merge".to_string(),
        (vec![(crate_ty, ParamConvention::mut_borrow())], Type::None),
    );

    let mutated = collect_mutated_vars(&stmts, Some(&sigs));

    assert!(mutated.contains("depot"));
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
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                }),
                method: "append".to_string(),
                args: vec![HirExpr::IntLiteral(1)],
                receiver_convention: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
                receiver_target: None,
                mutable_arg_places: Vec::new(),
                source: None,
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };

    let stmts = vec![
        HirStmt::NestedFunction {
            func: nested,
            move_captures: false,
            capture_clones: Vec::new(),
        },
        HirStmt::Expr {
            expr: HirExpr::Call {
                mutable_arg_places: Vec::new(),
                func: "touch_local".to_string(),
                args: vec![HirExpr::Name {
                    name: "items".to_string(),
                    binding_id: None,
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
        identity: None,
        type_args: Vec::new(),
        name: "CountdownIter".to_string(),
        fields: vec![],
        methods: vec![(
            "__next__".to_string(),
            sifr_type_system::FunctionType {
                receiver: None,
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
                binding_id: None,
                ty: iterator_ty,
            }],
            mutable_arg_places: Vec::new(),
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
            mutable_arg_places: Vec::new(),
            func: "anext".to_string(),
            args: vec![HirExpr::Name {
                name: "agen".to_string(),
                binding_id: None,
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
                mutable_arg_places: Vec::new(),
                func: "target".to_string(),
                args: vec![HirExpr::Name {
                    name: "n".to_string(),
                    binding_id: None,
                    ty: Type::Int,
                }],
                ty: Type::Int,
            }),
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };
    let stmts = vec![HirStmt::NestedFunction {
        func: nested,
        move_captures: false,
        capture_clones: Vec::new(),
    }];

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
        body_error_types: vec![test_error_type("Error")],
    }];

    assert!(body_contains_yield(&stmts));
}

#[test]
fn collect_try_error_carriers_descends_into_nested_functions() {
    let first_error = test_error_type("FirstError");
    let second_error = test_error_type("SecondError");
    let nested = HirFunction {
        name: "inner".to_string(),
        params: Vec::new(),
        return_type: Type::None,
        body: vec![HirStmt::TryExcept {
            body: vec![HirStmt::Pass],
            handlers: Vec::new(),
            body_error_types: vec![first_error.clone(), second_error.clone()],
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: Vec::new(),
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: Vec::new(),
    };

    assert_eq!(
        collect_try_error_carriers(&[HirStmt::NestedFunction {
            func: nested,
            move_captures: false,
            capture_clones: Vec::new(),
        }]),
        vec![Type::Union(vec![first_error, second_error])]
    );
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
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };

    let stmts = vec![HirStmt::NestedFunction {
        func: nested,
        move_captures: false,
        capture_clones: Vec::new(),
    }];
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
            mutable_arg_places: Vec::new(),
            func: "id".to_string(),
            args: vec![HirExpr::MethodCall {
                object: Box::new(HirExpr::Name {
                    name: "x".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                }),
                method: "append".to_string(),
                args: vec![HirExpr::IntLiteral(1)],
                receiver_convention: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
                receiver_target: None,
                mutable_arg_places: Vec::new(),
                source: None,
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
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };

    let mutated = collect_mutated_vars(
        &[HirStmt::NestedFunction {
            func: nested,
            move_captures: false,
            capture_clones: Vec::new(),
        }],
        None,
    );
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
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };

    let mutated = collect_mutated_vars(
        &[HirStmt::NestedFunction {
            func: nested,
            move_captures: false,
            capture_clones: Vec::new(),
        }],
        None,
    );
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
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                }),
                method: "append".to_string(),
                args: vec![HirExpr::IntLiteral(1)],
                receiver_convention: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
                receiver_target: None,
                mutable_arg_places: Vec::new(),
                source: None,
                ty: Type::None,
            },
        }],
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    };

    let mutated = collect_mutated_vars(
        &[HirStmt::NestedFunction {
            func: nested,
            move_captures: false,
            capture_clones: Vec::new(),
        }],
        None,
    );
    assert!(mutated.contains("items"));
}

#[test]
fn collect_mutated_vars_marks_dict_setdefault_receiver() {
    let stmts = vec![HirStmt::Expr {
        expr: HirExpr::MethodCall {
            object: Box::new(HirExpr::Name {
                name: "data".to_string(),
                binding_id: None,
                ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
            }),
            method: "setdefault".to_string(),
            args: vec![
                HirExpr::StringLiteral("k".to_string()),
                HirExpr::IntLiteral(1),
            ],
            receiver_convention: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: None,
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
                binding_id: None,
                ty: Type::Set(Box::new(Type::Int)),
            }),
            method: "intersection_update".to_string(),
            args: vec![HirExpr::ListLiteral {
                elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
                ty: Type::List(Box::new(Type::Int)),
            }],
            receiver_convention: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: None,
            ty: Type::None,
        },
    }];

    let mutated = collect_mutated_vars(&stmts, None);
    assert!(mutated.contains("seen"));
}

#[test]
fn collect_mutated_vars_marks_self_for_delegated_field_class_method_call() {
    let writer_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "writer".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    let holder_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
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
                    binding_id: None,
                    ty: holder_ty,
                }),
                field: "_writer".to_string(),
                ty: writer_ty,
            }),
            method: "writerow".to_string(),
            args: vec![],
            receiver_convention: Some(sifr_type_system::ReceiverConvention::MutableBorrow),
            receiver_target: None,
            mutable_arg_places: Vec::new(),
            source: None,
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
                binding_id: None,
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
                    mutable_arg_places: Vec::new(),
                    func: "ValueError".to_string(),
                    args: vec![HirExpr::StringLiteral("bad".to_string())],
                    ty: Type::Unknown,
                },
            }],
        }],
        body_error_types: vec![test_error_type("Error")],
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
                mutable_arg_places: Vec::new(),
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
fn python_async_context_suppression_keeps_following_return_reachable() {
    let error_type = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "PythonError".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    let stmts = vec![
        HirStmt::AsyncWith {
            kind: sifr_ir::HirAsyncWithKind::Python {
                context: HirExpr::Name {
                    name: "manager".to_string(),
                    binding_id: None,
                    ty: Type::Unknown,
                },
                manager_class: "Manager".to_string(),
                entered_type: Type::Unknown,
                enter_error_type: error_type.clone(),
                exit_error_type: error_type.clone(),
                entered_is_opaque_borrow: false,
                active_error_type: error_type,
            },
            target: None,
            body: vec![HirStmt::Raise {
                value: HirExpr::Name {
                    name: "error".to_string(),
                    binding_id: None,
                    ty: Type::Unknown,
                },
            }],
        },
        HirStmt::Return {
            value: Some(HirExpr::BoolLiteral(false)),
        },
    ];

    assert!(body_contains_return(&stmts));
    assert_eq!(
        block_control_flow_effect(&stmts),
        ControlFlowEffect::AlwaysExits
    );
}

#[test]
fn try_body_has_value_return_ignores_unreachable_value_return() {
    let stmts = vec![
        HirStmt::Raise {
            value: HirExpr::Call {
                mutable_arg_places: Vec::new(),
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
