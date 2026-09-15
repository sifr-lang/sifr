use super::*;

fn is_exact_float_literal(expr: &RustExpr, expected: f64) -> bool {
    matches!(expr, RustExpr::Literal(RustLiteral::Float(value)) if value.to_bits() == expected.to_bits())
}

#[test]
pub(super) fn lowers_leaf_expr_variants() {
    let int_expr = try_lower_leaf_expr(&HirExpr::IntLiteral(7)).expect("int lowered");
    let str_expr =
        try_lower_leaf_expr(&HirExpr::StringLiteral("ok".to_string())).expect("str lowered");
    let bool_expr = try_lower_leaf_expr(&HirExpr::BoolLiteral(true)).expect("bool lowered");
    let bool_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "ok".to_string(),
        binding_id: None,
        ty: Type::Bool,
    })
    .expect("bool name lowered");
    let none_expr = try_lower_leaf_expr(&HirExpr::NoneLiteral).expect("none lowered");
    let enum_expr = try_lower_leaf_expr(&HirExpr::EnumVariant {
        enum_name: "Color".to_string(),
        variant: "RED".to_string(),
        ty: sifr_type_system::Type::Enum {
            identity: None,
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1))],
        },
    })
    .expect("enum variant lowered");

    assert!(matches!(
        int_expr,
        RustExpr::FnCall { func, .. }
            if matches!(func.as_ref(), RustExpr::Path(path) if path == &["SifrInt", "from_i64"])
    ));
    assert!(matches!(str_expr, RustExpr::Literal(RustLiteral::Str(_))));
    assert!(matches!(
        bool_expr,
        RustExpr::Literal(RustLiteral::Bool(true))
    ));
    assert!(matches!(bool_name_expr, RustExpr::Ident(ref name) if name == "ok"));
    assert!(matches!(none_expr, RustExpr::Literal(RustLiteral::None)));
    assert!(matches!(enum_expr, RustExpr::Path(_)));
}

#[test]
pub(super) fn lowers_concrete_empty_list_with_explicit_rust_type() {
    let concrete = try_lower_leaf_expr(&HirExpr::ListLiteral {
        elements: Vec::new(),
        ty: Type::List(Box::new(Type::List(Box::new(Type::Int)))),
    })
    .expect("concrete empty list lowered");
    let unresolved = try_lower_leaf_expr(&HirExpr::ListLiteral {
        elements: Vec::new(),
        ty: Type::List(Box::new(Type::Any)),
    })
    .expect("unresolved empty list lowered");
    let nested_unresolved = try_lower_leaf_expr(&HirExpr::ListLiteral {
        elements: Vec::new(),
        ty: Type::List(Box::new(Type::List(Box::new(Type::Unknown)))),
    })
    .expect("nested unresolved empty list lowered");
    let union_unresolved = try_lower_leaf_expr(&HirExpr::ListLiteral {
        elements: Vec::new(),
        ty: Type::List(Box::new(Type::Union(vec![Type::None, Type::Any]))),
    })
    .expect("union unresolved empty list lowered");

    assert_eq!(crate::render_expr(&concrete), "Vec::<Vec<SifrInt>>::new()");
    assert_eq!(unresolved, RustExpr::Vec(Vec::new()));
    assert_eq!(nested_unresolved, RustExpr::Vec(Vec::new()));
    assert_eq!(union_unresolved, RustExpr::Vec(Vec::new()));
}

#[test]
pub(super) fn lowers_fixed_width_literal_for_target_type() {
    assert_eq!(
        fixed_width_literal_expr_for_target(
            &Type::FixedInt(sifr_type_system::FixedIntType::U8),
            &HirExpr::IntLiteral(255),
        ),
        Some(RustExpr::Verbatim("255u8".to_string()))
    );
    assert_eq!(
        fixed_width_literal_expr_for_target(
            &Type::FixedInt(sifr_type_system::FixedIntType::I8),
            &HirExpr::UnaryOp {
                op: "-".to_string(),
                operand: Box::new(HirExpr::IntLiteral(128)),
                ty: Type::Int,
            },
        ),
        Some(RustExpr::Verbatim("-128i8".to_string()))
    );
    assert_eq!(
        fixed_width_literal_expr_for_target(
            &Type::FixedInt(sifr_type_system::FixedIntType::U64),
            &HirExpr::LargeIntLiteral("18446744073709551615".to_string()),
        ),
        Some(RustExpr::Verbatim("18446744073709551615u64".to_string()))
    );
}

#[test]
pub(super) fn leaf_expr_result_reports_invalid_compare_shape() {
    let expr = HirExpr::Compare {
        left: Box::new(HirExpr::IntLiteral(1)),
        ops: vec!["==".to_string()],
        comparators: vec![],
        ty: Type::Bool,
    };
    let err = try_lower_leaf_expr_result(&expr).expect_err("invalid compare shape should error");
    assert!(err.message.contains("ops/comparators length mismatch"));
}

#[test]
pub(super) fn lowers_numeric_name_leaf_expr_variants() {
    let int_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "count".to_string(),
        binding_id: None,
        ty: Type::Int,
    })
    .expect("int name lowered");
    let float_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "ratio".to_string(),
        binding_id: None,
        ty: Type::Float,
    })
    .expect("float name lowered");
    let alias_int_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "index".to_string(),
        binding_id: None,
        ty: Type::alias("Index", Type::Int),
    })
    .expect("alias-int name lowered");
    let alias_float_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "weight".to_string(),
        binding_id: None,
        ty: Type::alias("Weight", Type::Float),
    })
    .expect("alias-float name lowered");

    assert!(matches!(int_name_expr, RustExpr::Ident(name) if name == "count"));
    assert!(matches!(float_name_expr, RustExpr::Ident(name) if name == "ratio"));
    assert!(matches!(alias_int_name_expr, RustExpr::Ident(name) if name == "index"));
    assert!(matches!(alias_float_name_expr, RustExpr::Ident(name) if name == "weight"));
}

#[test]
pub(super) fn lowers_bool_and_enum_name_leaf_expr_variants() {
    let alias_bool_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "ready".to_string(),
        binding_id: None,
        ty: Type::alias("ReadyFlag", Type::Bool),
    })
    .expect("alias-bool name lowered");
    let enum_ty = Type::Enum {
        identity: None,
        name: "Mode".to_string(),
        variants: vec![("A".to_string(), Some(1)), ("B".to_string(), Some(2))],
    };
    let enum_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "mode".to_string(),
        binding_id: None,
        ty: enum_ty.clone(),
    })
    .expect("enum name lowered");
    let alias_enum_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "mode_alias".to_string(),
        binding_id: None,
        ty: Type::alias("ModeAlias", enum_ty),
    })
    .expect("alias-enum name lowered");

    assert!(matches!(alias_bool_name_expr, RustExpr::Ident(name) if name == "ready"));
    assert!(matches!(enum_name_expr, RustExpr::Ident(name) if name == "mode"));
    assert!(matches!(alias_enum_name_expr, RustExpr::Ident(name) if name == "mode_alias"));
}

#[test]
pub(super) fn lowers_string_name_leaf_expr_variants() {
    let string_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "label".to_string(),
        binding_id: None,
        ty: Type::Str,
    })
    .expect("string name lowered");
    let alias_string_name_expr = try_lower_leaf_expr(&HirExpr::Name {
        name: "title".to_string(),
        binding_id: None,
        ty: Type::alias("Title", Type::Str),
    })
    .expect("alias-string name lowered");

    assert!(matches!(string_name_expr, RustExpr::Ident(name) if name == "label"));
    assert!(matches!(
        alias_string_name_expr,
        RustExpr::Ident(name) if name == "title"
    ));
}

#[test]
pub(super) fn lowers_simple_compound_expr_variants() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::IntLiteral(1)),
        op: "+".to_string(),
        right: Box::new(HirExpr::IntLiteral(2)),
        ty: Type::Int,
    };
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::IntLiteral(3)),
        ops: vec![">".to_string()],
        comparators: vec![HirExpr::IntLiteral(1)],
        ty: Type::Bool,
    };
    let cond = HirExpr::IfExpr {
        condition: Box::new(HirExpr::BoolLiteral(true)),
        then_expr: Box::new(HirExpr::IntLiteral(1)),
        else_expr: Box::new(HirExpr::IntLiteral(0)),
        ty: Type::Int,
    };

    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { .. })
    ));
    assert!(matches!(
        try_lower_leaf_expr(&cmp),
        Some(RustExpr::BinOp { .. })
    ));
    assert!(matches!(
        try_lower_leaf_expr(&cond),
        Some(RustExpr::If { .. })
    ));
}

#[test]
pub(super) fn lowers_simple_float_division_binop() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::FloatLiteral(6.0)),
        op: "/".to_string(),
        right: Box::new(HirExpr::FloatLiteral(2.0)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, .. }) if op == "/"
    ));
}

#[test]
pub(super) fn lowers_simple_numeric_binop_with_name_operands() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        op: "+".to_string(),
        right: Box::new(HirExpr::Name {
            name: "rhs".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        ty: Type::Int,
    };

    let lowered = try_lower_leaf_expr(&bin).expect("int-name binop lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, left, right }
            if op == "+"
                && matches!(left.as_ref(), RustExpr::Ref { expr, .. } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "lhs"))
                && matches!(right.as_ref(), RustExpr::Ref { expr, .. } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "rhs"))
    ));
}

#[test]
pub(super) fn lowers_simple_mixed_int_float_division_with_name_operands() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        op: "/".to_string(),
        right: Box::new(HirExpr::Name {
            name: "rhs".to_string(),
            binding_id: None,
            ty: Type::Float,
        }),
        ty: Type::Float,
    };

    assert!(try_lower_leaf_expr(&bin).is_none());
}

#[test]
pub(super) fn lowers_alias_wrapped_numeric_binop_with_name_operands() {
    let alias_int = Type::alias("Meters", Type::Int);
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: alias_int.clone(),
        }),
        op: "+".to_string(),
        right: Box::new(HirExpr::Name {
            name: "rhs".to_string(),
            binding_id: None,
            ty: alias_int.clone(),
        }),
        ty: alias_int,
    };

    let lowered = try_lower_leaf_expr(&bin).expect("alias int-name binop lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, left, right }
            if op == "+"
                && matches!(left.as_ref(), RustExpr::Ref { expr, .. } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "lhs"))
                && matches!(right.as_ref(), RustExpr::Ref { expr, .. } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "rhs"))
    ));
}

#[test]
pub(super) fn lowers_simple_alias_base_int_binop_with_name_operands() {
    let alias_int = Type::alias("Meters", Type::Int);
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: alias_int,
        }),
        op: "+".to_string(),
        right: Box::new(HirExpr::Name {
            name: "rhs".to_string(),
            binding_id: None,
            ty: Type::Int,
        }),
        ty: Type::Int,
    };

    let lowered = try_lower_leaf_expr(&bin).expect("alias/base int-name binop lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, left, right }
            if op == "+"
                && matches!(left.as_ref(), RustExpr::Ref { expr, .. } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "lhs"))
                && matches!(right.as_ref(), RustExpr::Ref { expr, .. } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "rhs"))
    ));
}

#[test]
pub(super) fn lowers_alias_wrapped_mixed_int_float_division_with_name_operands() {
    let alias_int = Type::alias("Count", Type::Int);
    let alias_float = Type::alias("Ratio", Type::Float);
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: alias_int,
        }),
        op: "/".to_string(),
        right: Box::new(HirExpr::Name {
            name: "rhs".to_string(),
            binding_id: None,
            ty: alias_float.clone(),
        }),
        ty: alias_float,
    };

    assert!(try_lower_leaf_expr(&bin).is_none());
}

#[test]
pub(super) fn lowers_simple_alias_base_float_division_with_name_operands() {
    let alias_float = Type::alias("Ratio", Type::Float);
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::Name {
            name: "lhs".to_string(),
            binding_id: None,
            ty: alias_float,
        }),
        op: "/".to_string(),
        right: Box::new(HirExpr::Name {
            name: "rhs".to_string(),
            binding_id: None,
            ty: Type::Float,
        }),
        ty: Type::Float,
    };

    let lowered = try_lower_leaf_expr(&bin).expect("alias/base float-name division lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { op, left, right }
            if op == "/"
                && matches!(left.as_ref(), RustExpr::Ident(name) if name == "lhs")
                && matches!(right.as_ref(), RustExpr::Ident(name) if name == "rhs")
    ));
}

#[test]
pub(super) fn does_not_lower_simple_int_division_binop_with_non_float_result() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::IntLiteral(6)),
        op: "/".to_string(),
        right: Box::new(HirExpr::IntLiteral(2)),
        ty: Type::Int,
    };
    assert!(try_lower_leaf_expr(&bin).is_none());
}

#[test]
pub(super) fn defers_exact_integer_floor_division_to_proof_aware_lowering() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::IntLiteral(7)),
        op: "//".to_string(),
        right: Box::new(HirExpr::IntLiteral(2)),
        ty: Type::Int,
    };
    assert!(try_lower_leaf_expr(&bin).is_none());
}

#[test]
pub(super) fn lowers_simple_floor_division_float_binop_as_div() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::FloatLiteral(7.0)),
        op: "//".to_string(),
        right: Box::new(HirExpr::FloatLiteral(2.0)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, .. }) if op == "/"
    ));
}

#[test]
pub(super) fn does_not_lower_simple_floor_division_float_binop_with_non_float_result() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::FloatLiteral(7.0)),
        op: "//".to_string(),
        right: Box::new(HirExpr::FloatLiteral(2.0)),
        ty: Type::Int,
    };
    assert!(try_lower_leaf_expr(&bin).is_none());
}

#[test]
pub(super) fn lowers_simple_mixed_int_float_floor_division_binop_as_div_with_casts() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::IntLiteral(7)),
        op: "//".to_string(),
        right: Box::new(HirExpr::FloatLiteral(2.0)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, left, right })
            if op == "/"
                && is_exact_float_literal(left.as_ref(), 7.0)
                && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
    ));
}

#[test]
pub(super) fn lowers_simple_mixed_float_int_floor_division_binop_as_div_with_casts() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::FloatLiteral(7.0)),
        op: "//".to_string(),
        right: Box::new(HirExpr::IntLiteral(2)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, left, right })
            if op == "/"
                && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                && is_exact_float_literal(right.as_ref(), 2.0)
    ));
}

#[test]
pub(super) fn lowers_simple_mixed_int_float_division_binop() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::IntLiteral(7)),
        op: "/".to_string(),
        right: Box::new(HirExpr::FloatLiteral(2.0)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, left, right })
            if op == "/"
                && is_exact_float_literal(left.as_ref(), 7.0)
                && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
    ));
}

#[test]
pub(super) fn lowers_simple_mixed_float_int_division_binop() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::FloatLiteral(7.0)),
        op: "/".to_string(),
        right: Box::new(HirExpr::IntLiteral(2)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, left, right })
            if op == "/"
                && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                && is_exact_float_literal(right.as_ref(), 2.0)
    ));
}

#[test]
pub(super) fn lowers_simple_mixed_int_float_addition_binop() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::IntLiteral(7)),
        op: "+".to_string(),
        right: Box::new(HirExpr::FloatLiteral(2.0)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, left, right })
            if op == "+"
                && is_exact_float_literal(left.as_ref(), 7.0)
                && matches!(right.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
    ));
}

#[test]
pub(super) fn lowers_simple_mixed_float_int_modulo_binop() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::FloatLiteral(7.0)),
        op: "%".to_string(),
        right: Box::new(HirExpr::IntLiteral(2)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, left, right })
            if op == "%"
                && matches!(left.as_ref(), RustExpr::Cast { ty: RustType::F64, .. })
                && is_exact_float_literal(right.as_ref(), 2.0)
    ));
}

#[test]
pub(super) fn lowers_simple_int_true_division_binop_with_float_casts() {
    let bin = HirExpr::BinOp {
        left: Box::new(HirExpr::IntLiteral(7)),
        op: "/".to_string(),
        right: Box::new(HirExpr::IntLiteral(2)),
        ty: Type::Float,
    };
    assert!(matches!(
        try_lower_leaf_expr(&bin),
        Some(RustExpr::BinOp { op, left, right })
            if op == "/"
                && is_exact_float_literal(left.as_ref(), 7.0)
                && is_exact_float_literal(right.as_ref(), 2.0)
    ));
}

#[test]
pub(super) fn lowers_multi_operand_boolop_variants() {
    let and_expr = HirExpr::BoolOp {
        op: "and".to_string(),
        values: vec![
            HirExpr::BoolLiteral(true),
            HirExpr::BoolLiteral(false),
            HirExpr::BoolLiteral(true),
        ],
        ty: Type::Bool,
    };
    let or_expr = HirExpr::BoolOp {
        op: "or".to_string(),
        values: vec![
            HirExpr::BoolLiteral(true),
            HirExpr::BoolLiteral(false),
            HirExpr::BoolLiteral(true),
        ],
        ty: Type::Bool,
    };

    assert!(matches!(
        try_lower_leaf_expr(&and_expr),
        Some(RustExpr::BinOp { op, .. }) if op == "&&"
    ));
    assert!(matches!(
        try_lower_leaf_expr(&or_expr),
        Some(RustExpr::BinOp { op, .. }) if op == "||"
    ));
}

#[test]
pub(super) fn guarded_non_option_compare_does_not_wrap_rhs_in_some() {
    let expr = HirExpr::BoolOp {
        op: "and".to_string(),
        values: vec![
            HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "first".to_string(),
                    binding_id: None,
                    ty: Type::Str,
                }),
                ops: vec!["is not".to_string()],
                comparators: vec![HirExpr::NoneLiteral],
                ty: Type::Bool,
            },
            HirExpr::Compare {
                left: Box::new(HirExpr::Name {
                    name: "first".to_string(),
                    binding_id: None,
                    ty: Type::Str,
                }),
                ops: vec!["==".to_string()],
                comparators: vec![HirExpr::StringLiteral("-".to_string())],
                ty: Type::Bool,
            },
        ],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&expr).expect("guarded bool op lowered");
    assert!(matches!(
        lowered,
        RustExpr::BinOp { right, .. }
            if matches!(
                right.as_ref(),
                RustExpr::BinOp { left, op, right }
                    if op == "=="
                        && matches!(left.as_ref(), RustExpr::Ident(name) if name == "first")
                        && matches!(right.as_ref(), RustExpr::Literal(RustLiteral::Str(s)) if s == "-")
            )
    ));
}

#[test]
pub(super) fn lowers_unary_not_with_bool_name_operand() {
    let unary = HirExpr::UnaryOp {
        op: "not".to_string(),
        operand: Box::new(HirExpr::Name {
            name: "ok".to_string(),
            binding_id: None,
            ty: Type::Bool,
        }),
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&unary).expect("unary not bool-name lowered");
    assert!(matches!(
        lowered,
        RustExpr::UnaryOp {
            op: ref operator,
            operand: ref inner,
        } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "ok")
    ));
}

#[test]
pub(super) fn lowers_unary_not_with_option_name_operand() {
    let unary = HirExpr::UnaryOp {
        op: "not".to_string(),
        operand: Box::new(HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        }),
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&unary).expect("unary not option-name lowered");
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
pub(super) fn lowers_unary_not_with_alias_option_name_operand() {
    let unary = HirExpr::UnaryOp {
        op: "not".to_string(),
        operand: Box::new(HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None])),
        }),
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&unary).expect("unary not alias-option-name lowered");
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
pub(super) fn lowers_unary_not_with_alias_bool_name_operand() {
    let alias_bool = Type::alias("Decision", Type::Bool);
    let unary = HirExpr::UnaryOp {
        op: "not".to_string(),
        operand: Box::new(HirExpr::Name {
            name: "ok".to_string(),
            binding_id: None,
            ty: alias_bool,
        }),
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&unary).expect("unary not alias-bool-name lowered");
    assert!(matches!(
        lowered,
        RustExpr::UnaryOp {
            op: ref operator,
            operand: ref inner,
        } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ident(name) if name == "ok")
    ));
}

#[test]
pub(super) fn lowers_unary_bitwise_invert_with_int_operand() {
    let unary = HirExpr::UnaryOp {
        op: "~".to_string(),
        operand: Box::new(HirExpr::IntLiteral(7)),
        ty: Type::Int,
    };

    let lowered = try_lower_leaf_expr(&unary).expect("unary invert int lowered");
    assert!(matches!(
        lowered,
        RustExpr::UnaryOp {
            op: ref operator,
            operand: ref inner,
        } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ref { mutable: false, expr }
            if matches!(expr.as_ref(), RustExpr::FnCall { func, .. }
                if matches!(func.as_ref(), RustExpr::Path(path) if path == &["SifrInt", "from_i64"])))
    ));
}

#[test]
pub(super) fn lowers_unary_bitwise_invert_with_alias_int_name_operand() {
    let alias_int = Type::alias("Bits", Type::Int);
    let unary = HirExpr::UnaryOp {
        op: "~".to_string(),
        operand: Box::new(HirExpr::Name {
            name: "mask".to_string(),
            binding_id: None,
            ty: alias_int,
        }),
        ty: Type::Int,
    };

    let lowered = try_lower_leaf_expr(&unary).expect("unary invert alias-int-name lowered");
    assert!(matches!(
        lowered,
        RustExpr::UnaryOp {
            op: ref operator,
            operand: ref inner,
        } if operator == "!" && matches!(inner.as_ref(), RustExpr::Ref { mutable: false, expr }
            if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "mask"))
    ));
}

#[test]
pub(super) fn does_not_lower_unary_bitwise_invert_with_non_int_operand() {
    let unary = HirExpr::UnaryOp {
        op: "~".to_string(),
        operand: Box::new(HirExpr::BoolLiteral(true)),
        ty: Type::Bool,
    };

    assert!(try_lower_leaf_expr(&unary).is_none());
}

#[test]
pub(super) fn lowers_option_is_none_compare_with_name_operand() {
    let cmp = HirExpr::Compare {
        left: Box::new(HirExpr::Name {
            name: "maybe_x".to_string(),
            binding_id: None,
            ty: Type::Union(vec![Type::Int, Type::None]),
        }),
        ops: vec!["is".to_string()],
        comparators: vec![HirExpr::NoneLiteral],
        ty: Type::Bool,
    };

    let lowered = try_lower_leaf_expr(&cmp).expect("option is-none compare lowered");
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
