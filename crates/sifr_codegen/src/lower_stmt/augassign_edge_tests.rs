    #[test]
    fn lowers_simple_attribute_augassign_floor_div_equal_alias_numeric_name() {
        let stmt = HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "count".to_string(),
            op: "//=".to_string(),
            value: HirExpr::Name {
                name: "step".to_string(),
                ty: Type::alias("Step", Type::Int),
            },
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("attribute floor-div augassign lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::AugAssign {
                target: RustExpr::Field { ref expr, ref field },
                op: ref lowered_op,
                value: RustExpr::Ident(ref rhs),
            } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                && field == "count"
                && lowered_op == "/"
                && rhs == "step"
        ));
    }

    #[test]
    fn does_not_lower_attribute_augassign_plus_equal_string_name() {
        let stmt = HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "label".to_string(),
            op: "+=".to_string(),
            value: HirExpr::Name {
                name: "suffix".to_string(),
                ty: Type::Str,
            },
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn does_not_lower_attribute_augassign_power_equal() {
        let stmt = HirStmt::AttributeAugAssign {
            object: "self".to_string(),
            field: "count".to_string(),
            op: "**=".to_string(),
            value: HirExpr::IntLiteral(3),
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new()).is_none());
    }

    #[test]
    fn lowers_simple_attribute_augassign_bitwise_and_shift_ops() {
        for (op, expected) in [
            ("&=", "&"),
            ("|=", "|"),
            ("^=", "^"),
            ("<<=", "<<"),
            (">>=", ">>"),
        ] {
            let stmt = HirStmt::AttributeAugAssign {
                object: "self".to_string(),
                field: "flags".to_string(),
                op: op.to_string(),
                value: HirExpr::Name {
                    name: "delta".to_string(),
                    ty: Type::Int,
                },
            };

            let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
                .expect("attribute bitwise/shift augassign lowered");
            assert_eq!(lowered.len(), 1);
            assert!(matches!(
                lowered[0],
                RustStmt::AugAssign {
                    target: RustExpr::Field { ref expr, ref field },
                    op: ref lowered_op,
                    value: RustExpr::Ident(ref rhs),
                } if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "self")
                    && field == "flags"
                    && lowered_op == expected
                    && rhs == "delta"
            ));
        }
    }

