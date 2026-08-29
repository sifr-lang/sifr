use super::*;
use sifr_ir::HirIteratorOp;
#[test]
pub(super) fn lowers_contains_for_range_collection() {
    let expr = HirExpr::ContainsOp {
        element: Box::new(HirExpr::IntLiteral(3)),
        collection: Box::new(HirExpr::RangeLiteral {
            start: Box::new(HirExpr::IntLiteral(0)),
            end: Box::new(HirExpr::IntLiteral(5)),
            step: None,
            ty: Type::Range,
        }),
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("range contains lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            method,
            args,
            ..
        } if method == "contains" && matches!(args.first(), Some(RustExpr::Ref { .. }))
    ));
}

#[test]
pub(super) fn lowers_contains_for_string_collection_with_borrowed_arg() {
    let expr = HirExpr::ContainsOp {
        element: Box::new(HirExpr::StringLiteral("T".to_string())),
        collection: Box::new(HirExpr::Name {
            name: "current_iso".to_string(),
            binding_id: None,
            ty: Type::Str,
        }),
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("string contains lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            receiver,
            method,
            args
        } if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "current_iso")
            && method == "contains"
            && matches!(args.first(), Some(RustExpr::Ref { .. }))
    ));
}

#[test]
pub(super) fn lowers_question_mark_ok_err_wrap_variants() {
    let q = HirExpr::QuestionMark {
        expr: Box::new(HirExpr::Name {
            name: "res".to_string(),
            binding_id: None,
            ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
        }),
        ty: Type::Int,
    };
    let ok = HirExpr::OkWrap {
        value: Box::new(HirExpr::IntLiteral(1)),
        ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
    };
    let err = HirExpr::ErrWrap {
        value: Box::new(HirExpr::StringLiteral("boom".to_string())),
        ty: Type::Result(Box::new(Type::Int), Box::new(Type::Any)),
    };

    assert!(matches!(try_lower_leaf_expr(&q), Some(RustExpr::Try(_))));
    assert!(matches!(
        try_lower_leaf_expr(&ok),
        Some(RustExpr::FnCall { func, .. })
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Ok".to_string()])
    ));
    assert!(matches!(
        try_lower_leaf_expr(&err),
        Some(RustExpr::FnCall { func, .. })
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Err".to_string()])
    ));
}

#[test]
pub(super) fn lowers_walrus_expr_with_leaf_value() {
    let expr = HirExpr::WalrusExpr {
        name: "n".to_string(),
        value: Box::new(HirExpr::IntLiteral(3)),
        ty: Type::Int,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("walrus lowered");
    assert!(matches!(
        lowered,
        RustExpr::Block { stmts, expr: Some(inner) }
            if matches!(stmts.first(), Some(RustStmt::Let { name, .. }) if name == "n")
                && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "n")
    ));
}

#[test]
pub(super) fn lowers_super_call_with_leaf_args() {
    let expr = HirExpr::SuperCall {
        parent_class: "Base".to_string(),
        parent_type: Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Base".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        },
        method: "new".to_string(),
        args: vec![HirExpr::IntLiteral(1)],
        ty: Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Base".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        },
    };

    let lowered = try_lower_leaf_expr(&expr).expect("super call lowered");
    assert!(matches!(
        lowered,
        RustExpr::FnCall { func, args }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Base".to_string(), "new".to_string()])
                && args.len() == 1
    ));
}

#[test]
pub(super) fn lowers_regular_super_call_with_self_receiver() {
    let expr = HirExpr::SuperCall {
        parent_class: "Base".to_string(),
        parent_type: Type::Class {
            identity: None,
            type_args: Vec::new(),
            name: "Base".to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        },
        method: "count".to_string(),
        args: vec![HirExpr::IntLiteral(1)],
        ty: Type::Int,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("super call lowered");
    assert!(matches!(
        lowered,
        RustExpr::FnCall { args, .. }
            if args.len() == 2
                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "self")
    ));
}

#[test]
pub(super) fn does_not_lower_non_path_call_with_leaf_args() {
    let expr = HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "compute".to_string(),
        args: vec![
            HirExpr::IntLiteral(1),
            HirExpr::Name {
                name: "n".to_string(),
                binding_id: None,
                ty: Type::Int,
            },
        ],
        ty: Type::Int,
    };

    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_simple_path_call_with_leaf_args() {
    let expr = HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "pkg::helper".to_string(),
        args: vec![HirExpr::BoolLiteral(true)],
        ty: Type::Int,
    };

    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_special_builtin_call() {
    let expr = HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "print".to_string(),
        args: vec![HirExpr::StringLiteral("x".to_string())],
        ty: Type::None,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn lowers_hash_builtin_call_with_leaf_arg() {
    let expr = HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "hash".to_string(),
        args: vec![HirExpr::Name {
            name: "item".to_string(),
            binding_id: None,
            ty: Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "Color".to_string(),
                fields: vec![],
                methods: vec![],
                parent_class: None,
            },
        }],
        ty: Type::Int,
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}

#[test]
pub(super) fn lowers_divmod_builtin_call_with_leaf_args() {
    let expr = HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "divmod".to_string(),
        args: vec![HirExpr::IntLiteral(17), HirExpr::IntLiteral(5)],
        ty: Type::Tuple(vec![Type::Int, Type::Int]),
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}

#[test]
pub(super) fn lowers_map_builtin_call_with_typed_lambda() {
    let expr = HirExpr::IteratorCall {
        op: HirIteratorOp::Map,
        mutable_arg_places: Vec::new(),
        args: vec![
            HirExpr::Lambda {
                params: vec![HirParam {
                    name: "x".to_string(),
                    ty: Type::Int,
                    default: None,
                    keyword_only: false,
                    convention: sifr_type_system::ParamConvention::borrow(),
                }],
                body: Box::new(HirExpr::BinOp {
                    left: Box::new(HirExpr::Name {
                        name: "x".to_string(),
                        binding_id: None,
                        ty: Type::Int,
                    }),
                    op: "*".to_string(),
                    right: Box::new(HirExpr::IntLiteral(2)),
                    ty: Type::Int,
                }),
                ty: Type::Callable(
                    vec![Type::Int],
                    vec![sifr_type_system::ParamConvention::own()],
                    Box::new(Type::Int),
                ),
            },
            HirExpr::Name {
                name: "nums".to_string(),
                binding_id: None,
                ty: Type::List(Box::new(Type::Int)),
            },
        ],
        ty: Type::Iterator(Box::new(Type::Int)),
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}

#[test]
pub(super) fn lowers_map_named_callable_with_optional_widening_closure() {
    let node_ty = Type::Class {
        identity: None,
        type_args: Vec::new(),
        name: "TreeNode".to_string(),
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    let optional_node_ty = Type::Union(vec![node_ty.clone(), Type::None]);
    let expr = HirExpr::IteratorCall {
        op: HirIteratorOp::Map,
        mutable_arg_places: Vec::new(),
        args: vec![
            HirExpr::Name {
                name: "format_node".to_string(),
                binding_id: None,
                ty: Type::Function(sifr_type_system::FunctionType {
                    receiver: None,
                    params: vec![(
                        "node".to_string(),
                        optional_node_ty,
                        sifr_type_system::ParamConvention::borrow(),
                    )],
                    return_type: Box::new(Type::Str),
                }),
            },
            HirExpr::Name {
                name: "nodes".to_string(),
                binding_id: None,
                ty: Type::List(Box::new(node_ty)),
            },
        ],
        ty: Type::Iterator(Box::new(Type::Str)),
    };

    let lowered = try_lower_leaf_expr(&expr).expect("map lowered");
    assert!(matches!(
        lowered,
        RustExpr::FnCall { args, .. }
            if matches!(
                args.first(),
                Some(RustExpr::MethodCall { method, args: map_args, .. })
                    if method == "map"
                        && matches!(
                            map_args.first(),
                            Some(RustExpr::Closure { body, .. })
                                if matches!(
                                    body.as_ref(),
                                    RustExpr::FnCall { func, args }
                                        if matches!(func.as_ref(), RustExpr::Ident(name) if name == "format_node")
                                            && matches!(
                                                args.first(),
                                                Some(RustExpr::Ref { mutable: false, expr })
                                                    if matches!(
                                                        expr.as_ref(),
                                                        RustExpr::FnCall { func, args }
                                                            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["Some".to_string()])
                                                                && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "__sifr_map_item")
                                                    )
                                            )
                                )
                        )
            )
    ));
}

#[test]
pub(super) fn does_not_lower_call_with_non_leaf_arg() {
    let expr = HirExpr::Call {
        mutable_arg_places: Vec::new(),
        func: "compute".to_string(),
        args: vec![HirExpr::ListComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::List(Box::new(Type::Int)),
        }],
        ty: Type::Int,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_method_call_on_any_with_leaf_args() {
    let expr = HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "obj".to_string(),
            binding_id: None,
            ty: Type::Any,
        }),
        method: "work".to_string(),
        args: vec![HirExpr::IntLiteral(2)],
        receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: None,
        ty: Type::Any,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_method_call_on_typed_object() {
    let expr = HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "items".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        }),
        method: "append".to_string(),
        args: vec![HirExpr::IntLiteral(1)],
        receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: None,
        ty: Type::None,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_len_method_call_on_any_object() {
    let expr = HirExpr::MethodCall {
        object: Box::new(HirExpr::Name {
            name: "obj".to_string(),
            binding_id: None,
            ty: Type::Any,
        }),
        method: "len".to_string(),
        args: vec![],
        receiver_convention: Some(sifr_type_system::ReceiverConvention::SharedBorrow),
        receiver_target: None,
        mutable_arg_places: Vec::new(),
        source: None,
        ty: Type::Int,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn lowers_path_constructor_call_with_leaf_args() {
    let expr = HirExpr::ConstructorCall {
        class_name: "pkg::Widget".to_string(),
        args: vec![HirExpr::IntLiteral(1)],
        ty: Type::Any,
    };

    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_non_path_constructor_call() {
    let expr = HirExpr::ConstructorCall {
        class_name: "Widget".to_string(),
        args: vec![HirExpr::IntLiteral(1)],
        ty: Type::Any,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn lowers_simple_index_on_any_with_leaf_index() {
    let expr = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "data".to_string(),
            binding_id: None,
            ty: Type::Any,
        }),
        index: Box::new(HirExpr::IntLiteral(0)),
        ty: Type::Any,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("index lowered");
    assert!(matches!(
        lowered,
        RustExpr::Index { expr, index }
            if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "data")
                && matches!(index.as_ref(), RustExpr::FnCall { func, .. }
                    if matches!(func.as_ref(), RustExpr::Path(path) if path == &["SifrInt", "from_i64"]))
    ));
}

#[test]
pub(super) fn does_not_lower_index_on_typed_object() {
    let expr = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "items".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        }),
        index: Box::new(HirExpr::IntLiteral(0)),
        ty: Type::Union(vec![Type::Int, Type::None]),
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn lowers_dict_index_to_optional_projection_for_optional_hir_type() {
    let expr = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "table".to_string(),
            binding_id: None,
            ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
        }),
        index: Box::new(HirExpr::IntLiteral(1)),
        ty: Type::Union(vec![Type::Int, Type::None]),
    };

    let lowered = try_lower_leaf_expr(&expr).expect("dict index lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall { method, .. } if method == "cloned"
    ));
}

#[test]
pub(super) fn lowers_dict_index_to_proven_some_block_for_non_optional_hir_type() {
    let expr = HirExpr::Index {
        object: Box::new(HirExpr::Name {
            name: "table".to_string(),
            binding_id: None,
            ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
        }),
        index: Box::new(HirExpr::IntLiteral(1)),
        ty: Type::Int,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("dict index lowered");
    let RustExpr::Block { stmts, expr } = lowered else {
        panic!("expected block lowering");
    };
    assert!(matches!(
        stmts.first(),
        Some(RustStmt::LetElse {
            pattern,
            value: RustExpr::MethodCall { method, .. },
            else_body,
        }) if pattern == "Some(__sifr_proven_dict_value)"
            && method == "cloned"
            && matches!(
                else_body.first(),
                Some(RustStmt::Expr(RustExpr::FnCall { func, .. }))
                    if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec![
                        "std".to_string(),
                        "process".to_string(),
                        "abort".to_string()
                    ])
            )
    ));
    assert!(matches!(
        expr.as_deref(),
        Some(RustExpr::Ident(name)) if name == "__sifr_proven_dict_value"
    ));
}

#[test]
pub(super) fn lowers_simple_slice_on_any_without_step() {
    let expr = HirExpr::Slice {
        object: Box::new(HirExpr::Name {
            name: "values".to_string(),
            binding_id: None,
            ty: Type::Any,
        }),
        start: Some(Box::new(HirExpr::IntLiteral(1))),
        stop: Some(Box::new(HirExpr::IntLiteral(3))),
        step: None,
        ty: Type::Any,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("slice lowered");
    assert!(matches!(
        lowered,
        RustExpr::Slice { expr, start, stop }
            if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "values")
                && matches!(start.as_ref(), Some(s) if matches!(s.as_ref(), RustExpr::FnCall { .. }))
                && matches!(stop.as_ref(), Some(s) if matches!(s.as_ref(), RustExpr::FnCall { .. }))
    ));
}

#[test]
pub(super) fn does_not_lower_slice_with_step_on_any() {
    let expr = HirExpr::Slice {
        object: Box::new(HirExpr::Name {
            name: "values".to_string(),
            binding_id: None,
            ty: Type::Any,
        }),
        start: None,
        stop: None,
        step: Some(Box::new(HirExpr::IntLiteral(2))),
        ty: Type::Any,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_slice_on_typed_object() {
    let expr = HirExpr::Slice {
        object: Box::new(HirExpr::Name {
            name: "items".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        }),
        start: Some(Box::new(HirExpr::IntLiteral(1))),
        stop: Some(Box::new(HirExpr::IntLiteral(3))),
        step: None,
        ty: Type::List(Box::new(Type::Int)),
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn lowers_simple_dict_literal_with_leaf_entries() {
    let expr = HirExpr::DictLiteral {
        keys: vec![HirExpr::StringLiteral("k".to_string())],
        values: vec![HirExpr::IntLiteral(1)],
        ty: Type::Any,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("dict literal lowered");
    assert!(matches!(
        lowered,
        RustExpr::FnCall { func, args }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashMap".to_string(), "from".to_string()])
                && matches!(args.first(), Some(RustExpr::Array(entries)) if !entries.is_empty())
    ));
}

#[test]
pub(super) fn lowers_dict_literal_with_nested_lowerable_entry() {
    let expr = HirExpr::DictLiteral {
        keys: vec![HirExpr::StringLiteral("k".to_string())],
        values: vec![HirExpr::ListComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::List(Box::new(Type::Int)),
        }],
        ty: Type::Any,
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}

#[test]
pub(super) fn lowers_dict_literal_on_typed_dict() {
    let expr = HirExpr::DictLiteral {
        keys: vec![HirExpr::StringLiteral("k".to_string())],
        values: vec![HirExpr::IntLiteral(1)],
        ty: Type::Dict(Box::new(Type::Str), Box::new(Type::Int)),
    };
    let lowered = try_lower_leaf_expr(&expr).expect("typed dict literal lowered");
    assert!(matches!(
        lowered,
        RustExpr::FnCall { func, .. }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashMap".to_string(), "from".to_string()])
    ));
}

#[test]
pub(super) fn lowers_simple_set_literal_with_leaf_entries() {
    let expr = HirExpr::SetLiteral {
        elements: vec![HirExpr::IntLiteral(1)],
        ty: Type::Any,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("set literal lowered");
    assert!(matches!(
        lowered,
        RustExpr::FnCall { func, args }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashSet".to_string(), "from".to_string()])
                && matches!(args.first(), Some(RustExpr::Array(entries)) if !entries.is_empty())
    ));
}

#[test]
pub(super) fn lowers_set_literal_with_nested_lowerable_entry() {
    let expr = HirExpr::SetLiteral {
        elements: vec![HirExpr::ListComp {
            expr: Box::new(HirExpr::Name {
                name: "x".to_string(),
                binding_id: None,
                ty: Type::Int,
            }),
            generators: vec![(
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            )],
            ty: Type::List(Box::new(Type::Int)),
        }],
        ty: Type::Any,
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}

#[test]
pub(super) fn lowers_set_literal_on_typed_set() {
    let expr = HirExpr::SetLiteral {
        elements: vec![HirExpr::IntLiteral(1)],
        ty: Type::Set(Box::new(Type::Int)),
    };
    let lowered = try_lower_leaf_expr(&expr).expect("typed set literal lowered");
    assert!(matches!(
        lowered,
        RustExpr::FnCall { func, .. }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &vec!["HashSet".to_string(), "from".to_string()])
    ));
}

#[test]
pub(super) fn lowers_simple_list_comp_with_single_generator() {
    let expr = HirExpr::ListComp {
        expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        generators: vec![(
            "x".to_string(),
            HirExpr::Name {
                name: "items".to_string(),
                binding_id: None,
                ty: Type::List(Box::new(Type::Int)),
            },
            None,
        )],
        ty: Type::Any,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("list comp lowered");
    assert!(matches!(
        lowered,
        RustExpr::Block { stmts, expr: Some(result) }
            if matches!(stmts.first(), Some(RustStmt::Let { name, mutable, .. }) if name == "__sifr_list_comp" && *mutable)
                && matches!(stmts.get(1), Some(RustStmt::For { var, .. }) if var == "x")
                && matches!(result.as_ref(), RustExpr::Ident(name) if name == "__sifr_list_comp")
    ));
}

#[test]
pub(super) fn lowers_list_comp_with_multiple_generators() {
    let expr = HirExpr::ListComp {
        expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        generators: vec![
            (
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            ),
            (
                "y".to_string(),
                HirExpr::Name {
                    name: "other".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            ),
        ],
        ty: Type::Any,
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}

#[test]
pub(super) fn lowers_list_comp_on_typed_list() {
    let expr = HirExpr::ListComp {
        expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        generators: vec![(
            "x".to_string(),
            HirExpr::Name {
                name: "items".to_string(),
                binding_id: None,
                ty: Type::List(Box::new(Type::Int)),
            },
            None,
        )],
        ty: Type::List(Box::new(Type::Int)),
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}

#[test]
pub(super) fn lowers_simple_dict_comp_with_single_generator() {
    let expr = HirExpr::DictComp {
        key_expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        val_expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        generators: vec![(
            "x".to_string(),
            HirExpr::Name {
                name: "items".to_string(),
                binding_id: None,
                ty: Type::List(Box::new(Type::Int)),
            },
            None,
        )],
        ty: Type::Any,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("dict comp lowered");
    assert!(matches!(
        lowered,
        RustExpr::Block { stmts, expr: Some(result) }
            if matches!(stmts.first(), Some(RustStmt::Let { name, mutable, .. }) if name == "__sifr_dict_comp" && *mutable)
                && matches!(stmts.get(1), Some(RustStmt::For { var, .. }) if var == "x")
                && matches!(result.as_ref(), RustExpr::Ident(name) if name == "__sifr_dict_comp")
    ));
}

#[test]
pub(super) fn does_not_lower_dict_comp_with_multiple_generators() {
    let expr = HirExpr::DictComp {
        key_expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        val_expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        generators: vec![
            (
                "x".to_string(),
                HirExpr::Name {
                    name: "items".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            ),
            (
                "y".to_string(),
                HirExpr::Name {
                    name: "other".to_string(),
                    binding_id: None,
                    ty: Type::List(Box::new(Type::Int)),
                },
                None,
            ),
        ],
        ty: Type::Any,
    };
    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn lowers_dict_comp_on_typed_dict() {
    let expr = HirExpr::DictComp {
        key_expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        val_expr: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        generators: vec![(
            "x".to_string(),
            HirExpr::Name {
                name: "items".to_string(),
                binding_id: None,
                ty: Type::List(Box::new(Type::Int)),
            },
            None,
        )],
        ty: Type::Dict(Box::new(Type::Int), Box::new(Type::Int)),
    };
    assert!(try_lower_leaf_expr(&expr).is_some());
}
