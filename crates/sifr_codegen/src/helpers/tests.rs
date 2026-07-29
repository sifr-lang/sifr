use super::*;
use sifr_ir::{HirExceptHandler, HirExpr, HirFunction, HirModule, HirParam, HirStmt, MethodKind};
use sifr_type_system::{OwnershipKind, ParamConvention, Type};
use std::collections::HashMap;

fn mk_function(name: &str, body: Vec<HirStmt>) -> HirFunction {
    HirFunction {
        name: name.to_string(),
        params: vec![],
        return_type: Type::None,
        body,
        is_async: false,
        method_kind: MethodKind::Regular,
        receiver: None,
        decorators: vec![],
        rust_interop: Vec::new(),
        python_interop: Vec::new(),
        compiler_intrinsic: None,
        type_params: vec![],
    }
}

fn mk_module_with_main(body: Vec<HirStmt>) -> HirModule {
    HirModule {
        functions: vec![mk_function("main", body)],
        classes: vec![],
        imports: vec![],
        constants: vec![],
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}

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
fn classify_value_category_marks_names_and_fields_as_places() {
    let name_expr = HirExpr::Name {
        name: "xs".to_string(),
        binding_id: None,
        ty: Type::List(Box::new(Type::Int)),
    };
    let field_expr = HirExpr::FieldAccess {
        object: Box::new(HirExpr::Name {
            name: "self".to_string(),
            binding_id: None,
            ty: Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "C".to_string(),
                fields: vec![("items".to_string(), Type::List(Box::new(Type::Int)))],
                methods: vec![],
                parent_class: None,
            },
        }),
        field: "items".to_string(),
        ty: Type::List(Box::new(Type::Int)),
    };
    let temp_expr = HirExpr::ListLiteral {
        elements: vec![HirExpr::IntLiteral(1)],
        ty: Type::List(Box::new(Type::Int)),
    };

    assert_eq!(classify_value_category(&name_expr), ValueCategory::Place);
    assert_eq!(classify_value_category(&field_expr), ValueCategory::Place);
    assert_eq!(
        classify_value_category(&temp_expr),
        ValueCategory::Temporary
    );
}

#[test]
fn classify_value_category_treats_copy_tuple_literal_of_places_as_place() {
    let tuple_expr = HirExpr::TupleLiteral {
        elements: vec![
            HirExpr::Name {
                name: "a".to_string(),
                binding_id: None,
                ty: Type::Int,
            },
            HirExpr::Name {
                name: "b".to_string(),
                binding_id: None,
                ty: Type::Bool,
            },
        ],
        ty: Type::Tuple(vec![Type::Int, Type::Bool]),
    };

    assert_eq!(classify_value_category(&tuple_expr), ValueCategory::Place);
}

#[test]
fn classify_value_category_treats_move_tuple_literal_as_temporary() {
    let tuple_expr = HirExpr::TupleLiteral {
        elements: vec![
            HirExpr::Name {
                name: "a".to_string(),
                binding_id: None,
                ty: Type::Int,
            },
            HirExpr::Name {
                name: "b".to_string(),
                binding_id: None,
                ty: Type::Str,
            },
        ],
        ty: Type::Tuple(vec![Type::Int, Type::Str]),
    };

    assert_eq!(
        classify_value_category(&tuple_expr),
        ValueCategory::Temporary
    );
}

#[test]
fn iterator_plan_preserves_named_copy_element_collection() {
    let source = HirExpr::Name {
        name: "xs".to_string(),
        binding_id: None,
        ty: Type::List(Box::new(Type::Int)),
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.value_category, ValueCategory::Place);
    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Copy);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
}

#[test]
fn iterator_plan_clones_named_move_element_collection() {
    let source = HirExpr::Name {
        name: "strings".to_string(),
        binding_id: None,
        ty: Type::List(Box::new(Type::Str)),
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.value_category, ValueCategory::Place);
    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Clone);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Move));
}

#[test]
fn iterator_plan_consumes_temporary_collection() {
    let source = HirExpr::ListLiteral {
        elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
        ty: Type::List(Box::new(Type::Int)),
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.value_category, ValueCategory::Temporary);
    assert_eq!(plan.source_access_mode, SourceAccessMode::Consume);
    assert_eq!(plan.yield_mode, YieldMode::Move);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
}

#[test]
fn iterator_plan_defaults_to_borrow_for_conservative_unknown_elements() {
    let source = HirExpr::Name {
        name: "unknown".to_string(),
        binding_id: None,
        ty: Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Unknown".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        },
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.value_category, ValueCategory::Place);
    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Borrow);
    assert_eq!(plan.element_ownership, None);
}

#[test]
fn option_projection_method_prefers_copy_for_copy_types() {
    assert_eq!(
        option_projection_method_for_owned_type(&Type::Int),
        "copied"
    );
    assert_eq!(
        option_projection_method_for_owned_type(&Type::Str),
        "cloned"
    );
}

#[test]
fn iterator_plan_copy_hint_does_not_force_unknown_source_to_copy() {
    let source = HirExpr::Name {
        name: "x".to_string(),
        binding_id: None,
        ty: Type::Any,
    };
    let plan = plan_iterator_ownership_with_element_hint(&source, Some(&Type::Int));

    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Borrow);
    assert_eq!(plan.element_ownership, None);
}

#[test]
fn iterator_plan_preserved_list_any_uses_borrow_not_clone() {
    let source = HirExpr::Name {
        name: "items".to_string(),
        binding_id: None,
        ty: Type::List(Box::new(Type::Any)),
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Borrow);
    assert_eq!(plan.element_ownership, None);
}

#[test]
fn iterator_plan_typevar_hint_stays_conservative() {
    let source = HirExpr::Name {
        name: "xs".to_string(),
        binding_id: None,
        ty: Type::TypeVar("T".to_string()),
    };
    let plan = plan_iterator_ownership_with_element_hint(&source, Some(&Type::Int));

    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Borrow);
    assert_eq!(plan.element_ownership, None);
}

#[test]
fn iterator_plan_uses_hint_for_collection_with_unknown_elements() {
    let source = HirExpr::Name {
        name: "items".to_string(),
        binding_id: None,
        ty: Type::Set(Box::new(Type::Any)),
    };
    let plan = plan_iterator_ownership_with_element_hint(&source, Some(&Type::Str));

    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Clone);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Move));
}

#[test]
fn iterator_plan_list_typevar_uses_clone_yield() {
    let source = HirExpr::Name {
        name: "xs".to_string(),
        binding_id: None,
        ty: Type::List(Box::new(Type::TypeVar("T".to_string()))),
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
    assert_eq!(plan.yield_mode, YieldMode::Clone);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Move));
}

#[test]
fn iterator_plan_copies_tuple_of_copy_elements() {
    let source = HirExpr::Name {
        name: "pairs".to_string(),
        binding_id: None,
        ty: Type::List(Box::new(Type::Tuple(vec![Type::Int, Type::Int]))),
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.yield_mode, YieldMode::Copy);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
}

#[test]
fn iterator_plan_consumes_range_without_clone_rules() {
    let source = HirExpr::Name {
        name: "r".to_string(),
        binding_id: None,
        ty: Type::Range,
    };
    let plan = plan_iterator_ownership(&source);

    assert_eq!(plan.source_access_mode, SourceAccessMode::Consume);
    assert_eq!(plan.yield_mode, YieldMode::Move);
    assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
}

#[test]
fn body_calls_function_detects_calls_in_for_else() {
    let stmts = vec![HirStmt::For {
        target: "i".to_string(),
        target_ty: Type::Int,
        iter: HirExpr::ListLiteral {
            elements: vec![],
            ty: Type::List(Box::new(Type::Int)),
        },
        body: vec![HirStmt::Pass],
        else_body: Some(vec![HirStmt::Expr {
            expr: HirExpr::Call {
                func: "rec".to_string(),
                args: vec![HirExpr::IntLiteral(1)],
                ty: Type::Int,
            },
        }]),
    }];

    assert!(body_calls_function(&stmts, "rec"));
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
fn collect_locally_defined_vars_includes_else_and_star_unpack() {
    let stmts = vec![
        HirStmt::For {
            target: "item".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::ListLiteral {
                elements: vec![],
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Let {
                name: "from_else".to_string(),
                ty: Type::Int,
                value: HirExpr::IntLiteral(1),
                is_mutable: true,
            }]),
        },
        HirStmt::StarUnpack {
            before: vec![("first".to_string(), Type::Int)],
            star: ("rest".to_string(), Type::List(Box::new(Type::Int))),
            after: vec![("last".to_string(), Type::Int)],
            value: HirExpr::ListLiteral {
                elements: vec![HirExpr::IntLiteral(1)],
                ty: Type::List(Box::new(Type::Int)),
            },
        },
    ];

    let defined = collect_locally_defined_vars(&stmts);
    assert!(defined.contains("item"));
    assert!(defined.contains("from_else"));
    assert!(defined.contains("first"));
    assert!(defined.contains("rest"));
    assert!(defined.contains("last"));
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
        handlers: vec![HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: None,
            name: Some("e".to_string()),
            body: vec![HirStmt::Yield {
                value: HirExpr::IntLiteral(2),
            }],
        }],
        body_error_types: vec![test_error_type("Error")],
    }];

    assert!(body_contains_yield_inner(&stmts));
}

#[test]
fn body_contains_return_detects_try_handlers_and_loop_else_paths() {
    let stmts = vec![HirStmt::TryExcept {
        body: vec![HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::ListLiteral {
                elements: vec![],
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }]),
        }],
        handlers: vec![HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: None,
            name: Some("e".to_string()),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(2)),
            }],
        }],
        body_error_types: vec![test_error_type("Error")],
    }];

    assert!(body_contains_return_stmt(&stmts));
}

#[test]
fn body_contains_return_ignores_nested_function_scope() {
    let nested = HirFunction {
        name: "inner".to_string(),
        params: vec![],
        return_type: Type::Int,
        body: vec![HirStmt::Return {
            value: Some(HirExpr::IntLiteral(1)),
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

    assert!(!body_contains_return_stmt(&stmts));
}

#[test]
fn body_contains_field_assign_detects_delegated_self_field_class_mutation() {
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
            source: None,
            ty: Type::None,
        },
    }];

    assert!(body_contains_field_assign_codegen(&stmts));
}

#[test]
fn try_body_has_value_return_detects_loop_else_and_try_handler_returns() {
    let stmts = vec![HirStmt::TryExcept {
        body: vec![HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::ListLiteral {
                elements: vec![],
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(9)),
            }]),
        }],
        handlers: vec![HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: None,
            name: Some("e".to_string()),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(7)),
            }],
        }],
        body_error_types: vec![test_error_type("Error")],
    }];

    assert!(try_body_has_value_return(&stmts));
}

#[test]
fn try_body_has_value_return_ignores_return_none() {
    let stmts = vec![HirStmt::Return {
        value: Some(HirExpr::NoneLiteral),
    }];
    assert!(!try_body_has_value_return(&stmts));
}

#[test]
fn module_uses_bigint_detects_try_handler_branches() {
    let module = mk_module_with_main(vec![HirStmt::TryExcept {
        body: vec![HirStmt::Pass],
        handlers: vec![HirExceptHandler {
            error_type: Some("Error".to_string()),
            error_resolved_type: None,
            name: Some("e".to_string()),
            body: vec![HirStmt::Let {
                name: "n".to_string(),
                ty: Type::BigInt,
                value: HirExpr::Call {
                    func: "bigint".to_string(),
                    args: vec![HirExpr::IntLiteral(3)],
                    ty: Type::BigInt,
                },
                is_mutable: true,
            }],
        }],
        body_error_types: vec![test_error_type("Error")],
    }]);

    assert!(module_uses_bigint(&module));
}

#[test]
fn module_uses_bigint_false_without_bigint() {
    let module = mk_module_with_main(vec![HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Int,
        value: HirExpr::IntLiteral(1),
        is_mutable: true,
    }]);

    assert!(!module_uses_bigint(&module));
}
