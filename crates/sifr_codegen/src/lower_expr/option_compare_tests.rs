use super::*;
#[test]
pub(super) fn lowers_option_is_none_compare_with_alias_option_name_operand() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
        }),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("alias option is-none compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_none"
            && args.is_empty()
    ));
}

#[test]
pub(super) fn lowers_option_is_not_none_compare_with_name_operand() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        }),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("option is-not-none compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_some"
            && args.is_empty()
    ));
}

#[test]
pub(super) fn lowers_option_is_not_none_compare_with_alias_option_name_operand() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
        }),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("alias option is-not-none compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_some"
            && args.is_empty()
    ));
}

#[test]
pub(super) fn lowers_option_is_none_compare_with_reversed_name_operand() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::NoneLiteral),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        }],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("reversed option is-none compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_none"
            && args.is_empty()
    ));
}

#[test]
pub(super) fn lowers_option_is_not_none_compare_with_reversed_alias_option_name_operand() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::NoneLiteral),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
        }],
        ty: Type::Bool,
    };

    let lowered =
        try_lower_leaf_expr(&cmp).expect("reversed alias option is-not-none compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            receiver: ref recv,
            ref method,
            ref args,
        } if matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_some"
            && args.is_empty()
    ));
}

#[test]
pub(super) fn lowers_simple_is_compare_as_eq() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::IntLiteral(1)),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::IntLiteral(1)],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("is compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, .. } if op == "=="
    ));
}

#[test]
pub(super) fn lowers_simple_is_not_compare_as_ne() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::IntLiteral(1)),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::IntLiteral(2)],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("is-not compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, .. } if op == "!="
    ));
}

#[test]
pub(super) fn lowers_bool_compare_with_literal_bool_name_operands() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: Type::LiteralBool(true),
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::Name {
            name: "rhs".to_string(),
            binding_id: None,
            ty: Type::Bool,
        }],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("bool/literal-bool compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, left, right }
            if op == "=="
                && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
    ));
}

#[test]
pub(super) fn does_not_lower_mismatched_bool_int_compare() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::BoolLiteral(true)),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::IntLiteral(1)],
        ty: Type::Bool,
    };

    assert!(try_lower_leaf_expr(&cmp).is_none());
}

#[test]
pub(super) fn lowers_string_literal_compare() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::StringLiteral("alpha".to_string())),
        ops: vec!["<".to_string()],
        comparators: vec![HirExpr::StringLiteral("beta".to_string())],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("string compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, .. } if op == "<"
    ));
}

#[test]
pub(super) fn does_not_lower_mismatched_string_int_compare() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::StringLiteral("x".to_string())),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::IntLiteral(1)],
        ty: Type::Bool,
    };

    assert!(try_lower_leaf_expr(&cmp).is_none());
}

#[test]
pub(super) fn lowers_enum_variant_equality_compare() {
    let enum_ty = Type::Enum {
        identity: None,
        name: "Color".to_string(),
        variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
    };
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "RED".to_string(),
            ty: enum_ty.clone(),
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "BLUE".to_string(),
            ty: enum_ty,
        }],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("enum equality compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, .. } if op == "=="
    ));
}

#[test]
pub(super) fn does_not_lower_enum_variant_ordering_compare() {
    let enum_ty = Type::Enum {
        identity: None,
        name: "Color".to_string(),
        variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
    };
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "RED".to_string(),
            ty: enum_ty.clone(),
        }),
        ops: vec!["<".to_string()],
        comparators: vec![HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "BLUE".to_string(),
            ty: enum_ty,
        }],
        ty: Type::Bool,
    };

    assert!(try_lower_leaf_expr(&cmp).is_none());
}

#[test]
pub(super) fn lowers_alias_wrapped_enum_variant_equality_compare() {
    let alias_enum_ty = Type::alias(
        "ColorAlias",
        Type::Enum {
            identity: None,
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
        },
    );
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "RED".to_string(),
            ty: alias_enum_ty.clone(),
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "BLUE".to_string(),
            ty: alias_enum_ty,
        }],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("alias enum equality compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, .. } if op == "=="
    ));
}

#[test]
pub(super) fn does_not_lower_alias_wrapped_enum_variant_ordering_compare() {
    let alias_enum_ty = Type::alias(
        "ColorAlias",
        Type::Enum {
            identity: None,
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
        },
    );
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "RED".to_string(),
            ty: alias_enum_ty.clone(),
        }),
        ops: vec!["<".to_string()],
        comparators: vec![HirExpr::EnumVariant {
            enum_name: "Color".to_string(),
            variant: "BLUE".to_string(),
            ty: alias_enum_ty,
        }],
        ty: Type::Bool,
    };

    assert!(try_lower_leaf_expr(&cmp).is_none());
}

#[test]
pub(super) fn lowers_alias_wrapped_scalar_compare() {
    let alias_int = Type::alias("Meters", Type::Int);
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: alias_int.clone(),
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::Name {
            name: "y".to_string(),
            binding_id: None,
            ty: alias_int,
        }],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("alias scalar compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, .. } if op == "=="
    ));
}

#[test]
pub(super) fn does_not_lower_mismatched_alias_wrapped_scalar_compare() {
    let int_alias = Type::alias("Meters", Type::Int);
    let bool_alias = Type::alias("Flag", Type::Bool);
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "x".to_string(),
            binding_id: None,
            ty: int_alias,
        }),
        ops: vec!["==".to_string()],
        comparators: vec![HirExpr::Name {
            name: "ok".to_string(),
            binding_id: None,
            ty: bool_alias,
        }],
        ty: Type::Bool,
    };

    assert!(try_lower_leaf_expr(&cmp).is_none());
}

#[test]
pub(super) fn lowers_simple_chained_compare_variants() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::IntLiteral(1)),
        ops: vec!["<".to_string(), "<".to_string()],
        comparators: vec![HirExpr::IntLiteral(2), HirExpr::IntLiteral(3)],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("chained compare lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp {
            op: ref top_op,
            left: ref top_left,
            right: ref top_right,
        } if top_op == "&&"
            && matches!(top_left.as_ref(), RustExpr::BinOp { op, .. } if op == "<")
            && matches!(top_right.as_ref(), RustExpr::BinOp { op, .. } if op == "<")
    ));
}

#[test]
pub(super) fn does_not_lower_option_is_none_compare_with_non_leaf_left() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "maybe_x".to_string(),
            args: vec![],
            ty: Type::Union(vec![Type::Int, Type::None]),
        }),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };

    assert!(try_lower_leaf_expr(&cmp).is_none());
}

#[test]
pub(super) fn lowers_range_literal_with_step() {
    let range = HirExpr::RangeLiteral {
        start: Box::new(HirExpr::IntLiteral(1)),
        end: Box::new(HirExpr::IntLiteral(10)),
        step: Some(Box::new(HirExpr::IntLiteral(2))),
        ty: Type::Range,
    };

    let lowered = try_lower_leaf_expr(&range).expect("range with step lowered");
    assert_eq!(
        crate::render_expr(&lowered),
        "SifrRange::new_known_nonzero(SifrInt::from_i64(1), SifrInt::from_i64(10), SifrInt::from_i64(2))"
    );
}

#[test]
pub(super) fn lowers_none_identity_compare_with_none_typed_left() {
    let is_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: Type::None,
        }),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };
    let is_not_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: Type::None,
        }),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };

    let lowered_is = try_lower_leaf_expr(&is_cmp).expect("none identity is lowered");
    let lowered_is_not = try_lower_leaf_expr(&is_not_cmp).expect("none identity is-not lowered");

    assert!(matches!(
        lowered_is,
        RustExpr::Literal(RustLiteral::Bool(true))
    ));
    assert!(matches!(
        lowered_is_not,
        RustExpr::Literal(RustLiteral::Bool(false))
    ));
}

#[test]
pub(super) fn lowers_none_identity_compare_with_alias_none_typed_left() {
    let alias_none = Type::alias("Nothing", Type::None);
    let is_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: alias_none.clone(),
        }),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };
    let is_not_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: alias_none,
        }),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };

    let lowered_is = try_lower_leaf_expr(&is_cmp).expect("alias-none identity is lowered");
    let lowered_is_not =
        try_lower_leaf_expr(&is_not_cmp).expect("alias-none identity is-not lowered");

    assert!(matches!(
        lowered_is,
        RustExpr::Literal(RustLiteral::Bool(true))
    ));
    assert!(matches!(
        lowered_is_not,
        RustExpr::Literal(RustLiteral::Bool(false))
    ));
}

#[test]
pub(super) fn lowers_none_identity_compare_with_none_typed_right() {
    let is_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::NoneLiteral),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: Type::None,
        }],
        ty: Type::Bool,
    };
    let is_not_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::NoneLiteral),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: Type::None,
        }],
        ty: Type::Bool,
    };

    let lowered_is = try_lower_leaf_expr(&is_cmp).expect("none identity reversed is lowered");
    let lowered_is_not =
        try_lower_leaf_expr(&is_not_cmp).expect("none identity reversed is-not lowered");

    assert!(matches!(
        lowered_is,
        RustExpr::Literal(RustLiteral::Bool(true))
    ));
    assert!(matches!(
        lowered_is_not,
        RustExpr::Literal(RustLiteral::Bool(false))
    ));
}

#[test]
pub(super) fn lowers_none_identity_compare_with_alias_none_typed_right() {
    let alias_none = Type::alias("Nothing", Type::None);
    let is_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::NoneLiteral),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: alias_none.clone(),
        }],
        ty: Type::Bool,
    };
    let is_not_cmp = HirExpr::Compare {
        left: Box::new(HirExpr::NoneLiteral),
        ops: vec!["is not".to_string()],
        comparators: vec![HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: alias_none,
        }],
        ty: Type::Bool,
    };

    let lowered_is = try_lower_leaf_expr(&is_cmp).expect("alias-none identity reversed is lowered");
    let lowered_is_not =
        try_lower_leaf_expr(&is_not_cmp).expect("alias-none identity reversed is-not lowered");

    assert!(matches!(
        lowered_is,
        RustExpr::Literal(RustLiteral::Bool(true))
    ));
    assert!(matches!(
        lowered_is_not,
        RustExpr::Literal(RustLiteral::Bool(false))
    ));
}

#[test]
pub(super) fn lowers_range_literal_with_name_bounds() {
    let range = HirExpr::RangeLiteral {
        start: Box::new(HirExpr::Name {
            name: "start".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        end: Box::new(HirExpr::Name {
            name: "end".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        step: None,
        ty: Type::Range,
    };

    let lowered = try_lower_leaf_expr(&range).expect("range with name bounds lowered");
    assert_eq!(
        crate::render_expr(&lowered),
        "SifrRange::new_known_nonzero(start.clone(), end.clone(), SifrInt::from_i64(1))"
    );
}

#[test]
pub(super) fn lowers_range_literal_with_name_step() {
    let range = HirExpr::RangeLiteral {
        start: Box::new(HirExpr::IntLiteral(1)),
        end: Box::new(HirExpr::IntLiteral(10)),
        step: Some(Box::new(HirExpr::Name {
            name: "step".to_string(),
            binding_id: None,
            ty: Type::Int,
        })),
        ty: Type::Range,
    };

    let lowered = try_lower_leaf_expr(&range).expect("range with name step lowered");
    assert_eq!(
        crate::render_expr(&lowered),
        "SifrRange::new_known_nonzero(SifrInt::from_i64(1), SifrInt::from_i64(10), step.clone())"
    );
}

#[test]
pub(super) fn lowers_range_literal_with_alias_name_bounds() {
    let alias_int = Type::alias("Index", Type::Int);
    let range = HirExpr::RangeLiteral {
        start: Box::new(HirExpr::Name {
            name: "start".to_string(),
            binding_id: None,
            ty: alias_int.clone(),
        }),
        end: Box::new(HirExpr::Name {
            name: "end".to_string(),
            binding_id: None,
            ty: alias_int,
        }),
        step: None,
        ty: Type::Range,
    };

    let lowered = try_lower_leaf_expr(&range).expect("range with alias-name bounds lowered");
    assert_eq!(
        crate::render_expr(&lowered),
        "SifrRange::new_known_nonzero(start.clone(), end.clone(), SifrInt::from_i64(1))"
    );
}

#[test]
pub(super) fn lowers_range_literal_with_alias_name_step() {
    let alias_int = Type::alias("Step", Type::Int);
    let range = HirExpr::RangeLiteral {
        start: Box::new(HirExpr::IntLiteral(1)),
        end: Box::new(HirExpr::IntLiteral(10)),
        step: Some(Box::new(HirExpr::Name {
            name: "step".to_string(),
            binding_id: None,
            ty: alias_int,
        })),
        ty: Type::Range,
    };

    let lowered = try_lower_leaf_expr(&range).expect("range with alias-name step lowered");
    assert_eq!(
        crate::render_expr(&lowered),
        "SifrRange::new_known_nonzero(SifrInt::from_i64(1), SifrInt::from_i64(10), step.clone())"
    );
}

#[test]
pub(super) fn does_not_lower_range_literal_with_non_int_name_operand() {
    let range = HirExpr::RangeLiteral {
        start: Box::new(HirExpr::Name {
            name: "start".to_string(),
            binding_id: None,
            ty: Type::Bool,
        }),
        end: Box::new(HirExpr::IntLiteral(10)),
        step: None,
        ty: Type::Range,
    };

    assert!(try_lower_leaf_expr(&range).is_none());
}

#[test]
pub(super) fn does_not_lower_field_access_for_non_self_name() {
    let expr = HirExpr::FieldAccess {
        object: Box::new(HirExpr::Name {
            name: "point".to_string(),
            binding_id: None,
            ty: Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "Point".to_string(),
                fields: vec![],
                methods: vec![],
                parent_class: None,
            },
        }),
        field: "x".to_string(),
        ty: Type::Int,
    };

    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_self_field_access() {
    let expr = HirExpr::FieldAccess {
        object: Box::new(HirExpr::Name {
            name: "self".to_string(),
            binding_id: None,
            ty: Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "Point".to_string(),
                fields: vec![],
                methods: vec![],
                parent_class: None,
            },
        }),
        field: "x".to_string(),
        ty: Type::Int,
    };

    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn does_not_lower_subclass_field_access() {
    let expr = HirExpr::FieldAccess {
        object: Box::new(HirExpr::Name {
            name: "dog".to_string(),
            binding_id: None,
            ty: Type::Class {
                identity: None,
                type_args: Vec::new(),
                name: "Dog".to_string(),
                fields: vec![],
                methods: vec![],
                parent_class: Some("Animal".to_string()),
            },
        }),
        field: "name".to_string(),
        ty: Type::Str,
    };

    assert!(try_lower_leaf_expr(&expr).is_none());
}

#[test]
pub(super) fn lowers_contains_for_list_name_collection() {
    let expr = HirExpr::ContainsOp {
        element: Box::new(HirExpr::Name {
            name: "needle".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        collection: Box::new(HirExpr::Name {
            name: "haystack".to_string(),
            binding_id: None,
            ty: Type::List(Box::new(Type::Int)),
        }),
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("contains lowered");
    assert!(matches!(
        lowered,
        RustExpr::MethodCall {
            receiver,
            method,
            args
        } if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "haystack")
            && method == "contains"
            && matches!(
                args.first(),
                Some(RustExpr::Ref { expr, .. })
                    if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "needle")
            )
    ));
}
