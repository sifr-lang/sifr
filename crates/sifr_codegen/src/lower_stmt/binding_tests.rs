use super::*;
#[test]
fn lowers_simple_let_with_not_bool_name_rhs() {
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Bool,
        value: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                binding_id: None,
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        },
        is_mutable: false,
    };

    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let not-bool name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            name: ref let_name,
            value: RustExpr::UnaryOp {
                ref op,
                ref operand,
            },
            ..
        } if let_name == "x"
            && op == "!"
            && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
    ));
}

#[test]
fn lowers_simple_assign_with_not_bool_name_rhs() {
    let assign_stmt = HirStmt::Assign {
        name: "x".to_string(),
        value: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "ok".to_string(),
                binding_id: None,
                ty: Type::Bool,
            }),
            ty: Type::Bool,
        },
    };

    let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assign not-bool name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assign {
            target: RustExpr::Ident(ref target_name),
            value: RustExpr::UnaryOp {
                ref op,
                ref operand,
            },
        } if target_name == "x"
            && op == "!"
            && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
    ));
}

#[test]
fn lowers_simple_let_with_not_option_name_rhs() {
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Bool,
        value: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                binding_id: None,
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        },
        is_mutable: false,
    };

    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let not-option name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            name: ref let_name,
            value: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
            ..
        } if let_name == "x"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_none"
            && args.is_empty()
    ));
}

#[test]
fn lowers_simple_assign_with_not_option_name_rhs() {
    let assign_stmt = HirStmt::Assign {
        name: "x".to_string(),
        value: HirExpr::UnaryOp {
            op: "not".to_string(),
            operand: Box::new(HirExpr::Name {
                name: "maybe_x".to_string(),
                binding_id: None,
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
            ty: Type::Bool,
        },
    };

    let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("assign not-option name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assign {
            target: RustExpr::Ident(ref target_name),
            value: RustExpr::MethodCall {
                receiver: ref recv,
                ref method,
                ref args,
            },
        } if target_name == "x"
            && matches!(recv.as_ref(), RustExpr::Ident(name) if name == "maybe_x")
            && method == "is_none"
            && args.is_empty()
    ));
}

#[test]
fn lowers_simple_let_name_rhs() {
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::Int,
        value: HirExpr::Name {
            name: "y".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            value: RustExpr::Clone(ref rhs),
            ..
        } if let_name == "x" && matches!(rhs.as_ref(), RustExpr::Ident(name) if name == "y")
    ));
}

#[test]
fn lowers_simple_let_alias_int_literal_rhs() {
    let alias_int = Type::alias("Meters", Type::Int);
    let let_stmt = HirStmt::Let {
        name: "distance".to_string(),
        ty: alias_int,
        value: HirExpr::IntLiteral(7),
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let alias-int literal rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            value: RustExpr::FnCall { .. },
            ..
        } if let_name == "distance"
    ));
}

#[test]
fn lowers_simple_let_alias_enum_name_rhs() {
    let alias_enum = Type::alias(
        "ColorAlias",
        Type::Enum {
            identity: None,
            name: "Color".to_string(),
            variants: vec![("RED".to_string(), Some(1)), ("BLUE".to_string(), Some(2))],
        },
    );
    let let_stmt = HirStmt::Let {
        name: "shade".to_string(),
        ty: alias_enum.clone(),
        value: HirExpr::Name {
            name: "selected".to_string(),
            binding_id: None,
            ty: alias_enum,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let alias-enum name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            value: RustExpr::Ident(ref rhs),
            ..
        } if let_name == "shade" && rhs == "selected"
    ));
}

#[test]
fn lowers_simple_let_none_literal_to_unit() {
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::None,
        value: HirExpr::NoneLiteral,
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let none lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            ty: Some(RustType::Unit),
            value: RustExpr::Literal(RustLiteral::Unit),
        } if let_name == "x"
    ));
}

#[test]
fn lowers_simple_let_alias_none_literal_to_unit() {
    let alias_none = Type::alias("Nothing", Type::None);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: alias_none,
        value: HirExpr::NoneLiteral,
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let alias-none lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            value: RustExpr::Literal(RustLiteral::Unit),
            ..
        } if let_name == "x"
    ));
}

#[test]
fn lowers_simple_let_none_name_rhs() {
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: Type::None,
        value: HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: Type::None,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let none-name lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            value: RustExpr::Ident(ref rhs),
            ..
        } if let_name == "x" && rhs == "n"
    ));
}

#[test]
fn lowers_simple_let_alias_none_name_rhs() {
    let alias_none = Type::alias("Nothing", Type::None);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: alias_none.clone(),
        value: HirExpr::Name {
            name: "n".to_string(),
            binding_id: None,
            ty: alias_none,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("let alias-none-name lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            value: RustExpr::Ident(ref rhs),
            ..
        } if let_name == "x" && rhs == "n"
    ));
}

#[test]
fn lowers_simple_option_let_none_literal_to_none() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty.clone(),
        value: HirExpr::NoneLiteral,
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("option let none lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            ty: Some(RustType::Option(_)),
            value: RustExpr::Literal(RustLiteral::None),
        } if let_name == "x"
    ));
}

#[test]
fn lowers_simple_option_let_none_literal_to_none_with_alias_option_ty() {
    let option_ty = Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None]));
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty,
        value: HirExpr::NoneLiteral,
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias-option let none lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            value: RustExpr::Literal(RustLiteral::None),
            ..
        } if let_name == "x"
    ));
}

#[test]
fn lowers_simple_option_let_name_rhs_to_some() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty,
        value: HirExpr::Name {
            name: "y".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("option let name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            ty: Some(RustType::Option(_)),
            value: RustExpr::FnCall { ref func, ref args },
        } if let_name == "x"
            && matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
            && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "y")
    ));
}

#[test]
fn lowers_simple_option_let_leaf_rhs_to_some() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty,
        value: HirExpr::IntLiteral(7),
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("option let leaf rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            ty: Some(RustType::Option(_)),
            value: RustExpr::FnCall { ref func, ref args },
        } if let_name == "x"
            && matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
            && matches!(args.first(), Some(RustExpr::FnCall { .. }))
    ));
}

#[test]
fn lowers_simple_option_let_option_name_rhs_passthrough() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty.clone(),
        value: HirExpr::Name {
            name: "maybe_y".to_string(),
            binding_id: None,
            ty: option_ty,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("option let option name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            ty: Some(RustType::Option(_)),
            value: RustExpr::Ident(ref rhs),
        } if let_name == "x" && rhs == "maybe_y"
    ));
}

#[test]
fn does_not_lower_option_let_option_non_leaf_rhs_passthrough() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty.clone(),
        value: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "maybe_value".to_string(),
            args: vec![],
            ty: option_ty,
        },
        is_mutable: false,
    };

    assert!(try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn does_not_lower_option_let_non_leaf_rhs_to_some() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty,
        value: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "value".to_string(),
            args: vec![],
            ty: Type::Int,
        },
        is_mutable: false,
    };

    assert!(try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_option_let_none_name_rhs_to_none() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty,
        value: HirExpr::Name {
            name: "none_value".to_string(),
            binding_id: None,
            ty: Type::None,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("option let none-name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            ty: Some(RustType::Option(_)),
            value: RustExpr::Literal(RustLiteral::None),
        } if let_name == "x"
    ));
}

#[test]
fn lowers_simple_option_let_alias_none_name_rhs_to_none() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let alias_none = Type::alias("Nothing", Type::None);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty,
        value: HirExpr::Name {
            name: "none_value".to_string(),
            binding_id: None,
            ty: alias_none,
        },
        is_mutable: false,
    };
    let lowered = try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("option let alias-none-name rhs lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Let {
            mutable: false,
            name: ref let_name,
            ty: Some(RustType::Option(_)),
            value: RustExpr::Literal(RustLiteral::None),
        } if let_name == "x"
    ));
}

#[test]
fn does_not_lower_option_let_non_leaf_none_typed_rhs_to_none() {
    let option_ty = Type::Union(vec![Type::Int, Type::None]);
    let let_stmt = HirStmt::Let {
        name: "x".to_string(),
        ty: option_ty,
        value: HirExpr::Call {
            mutable_arg_places: Vec::new(),
            func: "none_value".to_string(),
            args: vec![],
            ty: Type::None,
        },
        is_mutable: false,
    };

    assert!(try_lower_simple_stmt(&let_stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_assign_name_rhs() {
    let assign_stmt = HirStmt::Assign {
        name: "x".to_string(),
        value: HirExpr::Name {
            name: "y".to_string(),
            binding_id: None,
            ty: Type::Int,
        },
    };
    let lowered = try_lower_simple_stmt(&assign_stmt, false, &HashSet::new(), &HashSet::new())
        .expect("name assign lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::Assign {
            target: RustExpr::Ident(ref lhs),
            value: RustExpr::Ident(ref rhs),
        } if lhs == "x" && rhs == "y"
    ));
}

#[test]
fn does_not_lower_assign_borrowed_typevar_name() {
    let assign_stmt = HirStmt::Assign {
        name: "dst".to_string(),
        value: HirExpr::Name {
            name: "param".to_string(),
            binding_id: None,
            ty: Type::TypeVar("T".to_string()),
        },
    };

    assert!(
        try_lower_simple_stmt(
            &assign_stmt,
            false,
            &HashSet::new(),
            &HashSet::from(["param".to_string()]),
        )
        .is_none()
    );
}
