use super::*;
use crate::RustExpr;
use sifr_ir::{HirExceptHandler, HirExpr, HirFunction, HirParam, HirStmt, MethodKind};
use sifr_type_system::{OwnershipKind, ParamConvention, Type};

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
                mutable_arg_places: Vec::new(),
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
fn option_target_coercion_flattens_only_excess_wrapper_layers() {
    let flat = Type::Union(vec![Type::Str, Type::None]);
    let nested = Type::Union(vec![flat.clone(), Type::None]);
    let twice_nested = Type::Union(vec![nested.clone(), Type::None]);
    let value = RustExpr::Ident("value".to_string());

    assert_eq!(
        flatten_option_value_for_target(&flat, &nested, value.clone()),
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(value.clone()))),
            method: "flatten".to_string(),
            args: Vec::new(),
        }
    );
    let twice_flattened = flatten_option_value_for_target(&flat, &twice_nested, value.clone());
    assert!(matches!(
        twice_flattened,
        RustExpr::MethodCall { method, receiver, .. }
            if method == "flatten"
                && matches!(receiver.as_ref(), RustExpr::Paren(inner)
                    if matches!(inner.as_ref(), RustExpr::MethodCall { method, .. } if method == "flatten"))
    ));
    assert_eq!(
        flatten_option_value_for_target(&nested, &nested, value.clone()),
        value.clone()
    );
    assert_eq!(
        flatten_option_value_for_target(&Type::Any, &nested, value.clone()),
        value.clone()
    );
    assert_eq!(
        flatten_option_value_for_target(&Type::Unknown, &nested, value.clone()),
        value
    );
}

#[test]
fn option_target_coercion_maps_outer_absence_to_nullable_union_variant() {
    let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let source = Type::Union(vec![target.clone(), Type::None]);
    let rendered = crate::render_expr(&flatten_option_value_for_target(
        &target,
        &source,
        RustExpr::Ident("value".to_string()),
    ));
    assert!(rendered.contains(".unwrap_or("), "{rendered}");
    assert!(rendered.contains(&target.union_enum_name()), "{rendered}");
    assert!(
        rendered.contains(&Type::None.union_variant_name()),
        "{rendered}"
    );
}

#[test]
fn collection_target_coercion_recovers_contextually_narrowed_safe_get() {
    let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let runtime_source = sifr_type_system::safe_optional_result(target.clone());
    let expr = HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "values".to_string(),
            binding_id: None,
            ty: Type::Dict(Box::new(Type::Str), Box::new(target.clone())),
        }),
        method: "get".to_string(),
        args: vec![HirExpr::StringLiteral("key".to_string())],
        receiver_convention: None,
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: None,
        ty: runtime_source,
    };
    let rendered = crate::render_expr(&adapt_collection_value_for_target(
        &target,
        &expr,
        RustExpr::Ident("value".to_string()),
    ));
    assert!(rendered.contains(".unwrap_or("), "{rendered}");
}

#[test]
fn collection_target_coercion_does_not_flatten_a_simple_optional_index() {
    let target = sifr_type_system::make_union(vec![Type::Str, Type::None]);
    let expr = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "maybe_values".to_string(),
            binding_id: None,
            ty: sifr_type_system::make_union(vec![Type::List(Box::new(Type::Str)), Type::None]),
        }),
        index: Box::new(HirExpr::IntLiteral(0)),
        ty: target.clone(),
    };
    let value = RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident("maybe_values".to_string())),
        method: "and_then".to_string(),
        args: Vec::new(),
    };
    assert_eq!(
        adapt_collection_value_for_target(&target, &expr, value.clone()),
        value
    );
}

#[test]
fn collection_storage_coercion_maps_nested_optional_elements() {
    let target_element = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let source_element = Type::Union(vec![target_element.clone(), Type::None]);
    let rendered = crate::render_expr(&adapt_collection_storage_for_target(
        &Type::List(Box::new(target_element.clone())),
        &Type::List(Box::new(source_element)),
        RustExpr::Ident("values".to_string()),
    ));
    assert!(rendered.contains(".into_iter().map("), "{rendered}");
    assert!(rendered.contains(".unwrap_or("), "{rendered}");
    assert!(
        rendered.contains(&target_element.union_enum_name()),
        "{rendered}"
    );
}

#[test]
fn safe_option_result_flattens_simple_option_and_preserves_nullable_union() {
    let value = RustExpr::Ident("value".to_string());
    assert_eq!(
        normalize_safe_option_result(&Type::Union(vec![Type::Str, Type::None]), value.clone()),
        RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(value.clone()))),
            method: "flatten".to_string(),
            args: Vec::new(),
        }
    );
    assert_eq!(
        normalize_safe_option_result(&Type::Str, value.clone()),
        value
    );

    let plain_union = sifr_type_system::make_union(vec![Type::Int, Type::Str]);
    assert_eq!(
        normalize_safe_option_result(&plain_union, value.clone()),
        value
    );

    let nullable_union = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    assert_eq!(
        normalize_safe_option_result(&nullable_union, value.clone()),
        value
    );
}

#[test]
fn union_member_wrapper_uses_unit_for_none_payload() {
    let target = sifr_type_system::make_union(vec![Type::Int, Type::Str, Type::None]);
    let wrapped = wrap_union_member_expr(
        &target,
        &Type::None,
        RustExpr::Literal(crate::RustLiteral::None),
    )
    .expect("none should be wrapped as an ordinary union member");
    let rendered = crate::render_expr(&wrapped);
    assert!(rendered.contains(&target.union_enum_name()), "{rendered}");
    assert!(rendered.ends_with("(())"), "{rendered}");
}
