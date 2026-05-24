use super::*;

#[test]
fn lowers_simple_augassign_for_supported_ops() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "-=".to_string(),
        value: HirExpr::IntLiteral(2),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("augassign lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::AugAssign {
            target: RustExpr::Ident(ref name),
            op: ref lowered_op,
            ..
        } if name == "x" && lowered_op == "-"
    ));
}

#[test]
fn does_not_lower_augassign_plus_equal() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "+=".to_string(),
        value: HirExpr::StringLiteral("a".to_string()),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_augassign_plus_equal_numeric() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "+=".to_string(),
        value: HirExpr::IntLiteral(1),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("numeric += lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::AugAssign {
            target: RustExpr::Ident(ref name),
            op: ref lowered_op,
            ..
        } if name == "x" && lowered_op == "+"
    ));
}

#[test]
fn lowers_simple_augassign_plus_equal_numeric_name() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "+=".to_string(),
        value: HirExpr::Name {
            name: "delta".to_string(),
            ty: Type::Int,
        },
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("numeric name += lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::AugAssign {
            target: RustExpr::Ident(ref name),
            op: ref lowered_op,
            value: RustExpr::Ident(ref rhs),
        } if name == "x" && lowered_op == "+" && rhs == "delta"
    ));
}

#[test]
fn lowers_simple_augassign_plus_equal_alias_numeric_name() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "+=".to_string(),
        value: HirExpr::Name {
            name: "delta".to_string(),
            ty: Type::alias("Meters", Type::Int),
        },
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias numeric name += lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::AugAssign {
            target: RustExpr::Ident(ref name),
            op: ref lowered_op,
            value: RustExpr::Ident(ref rhs),
        } if name == "x" && lowered_op == "+" && rhs == "delta"
    ));
}

#[test]
fn does_not_lower_augassign_plus_equal_string_name() {
    let stmt = HirStmt::AugAssign {
        name: "s".to_string(),
        op: "+=".to_string(),
        value: HirExpr::Name {
            name: "suffix".to_string(),
            ty: Type::Str,
        },
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_augassign_floor_div_equal() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "//=".to_string(),
        value: HirExpr::IntLiteral(2),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("floor-div augassign lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::AugAssign {
            target: RustExpr::Ident(ref name),
            op: ref lowered_op,
            ..
        } if name == "x" && lowered_op == "/"
    ));
}

#[test]
fn lowers_simple_augassign_floor_div_equal_alias_numeric_name() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "//=".to_string(),
        value: HirExpr::Name {
            name: "step".to_string(),
            ty: Type::alias("Step", Type::Int),
        },
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("alias numeric //= lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::AugAssign {
            target: RustExpr::Ident(ref name),
            op: ref lowered_op,
            value: RustExpr::Ident(ref rhs),
        } if name == "x" && lowered_op == "/" && rhs == "step"
    ));
}

#[test]
fn does_not_lower_augassign_power_equal() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "**=".to_string(),
        value: HirExpr::IntLiteral(3),
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_augassign_bitwise_and_shift_ops() {
    for (op, expected) in [
        ("&=", "&"),
        ("|=", "|"),
        ("^=", "^"),
        ("<<=", "<<"),
        (">>=", ">>"),
    ] {
        let stmt = HirStmt::AugAssign {
            name: "x".to_string(),
            op: op.to_string(),
            value: HirExpr::Name {
                name: "delta".to_string(),
                ty: Type::alias("Bits", Type::Int),
            },
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("bitwise/shift augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Ident(ref name),
                op: ref lowered_op,
                value: RustExpr::Ident(ref rhs),
            } if name == "x" && lowered_op == expected && rhs == "delta"
        ));
    }
}

#[test]
fn does_not_lower_augassign_bitwise_for_float() {
    let stmt = HirStmt::AugAssign {
        name: "x".to_string(),
        op: "&=".to_string(),
        value: HirExpr::Name {
            name: "mask".to_string(),
            ty: Type::Float,
        },
    };

    assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
}

#[test]
fn lowers_simple_attribute_augassign_for_supported_ops() {
    let stmt = HirStmt::AttributeAugAssign {
        object: "self".to_string(),
        field: "count".to_string(),
        op: "-=".to_string(),
        value: HirExpr::IntLiteral(2),
    };

    let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
        .expect("attribute augassign lowered");
    assert_eq!(lowered.len(), 1);
    assert!(matches!(
        lowered[0],
        RustStmt::AugAssign {
            target: RustExpr::Field { ref expr, ref field },
            op: ref lowered_op,
            ..
        } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
            && field == "count"
            && lowered_op == "-"
    ));
}
