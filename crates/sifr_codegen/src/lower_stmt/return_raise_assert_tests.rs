    #[test]
    fn lowers_simple_bare_return_without_option_context() {
        let stmt = HirStmt::Return { value: None };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("bare return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::Return(None)));
    }

    #[test]
    fn lowers_simple_bare_return_to_none_in_option_context() {
        let stmt = HirStmt::Return { value: None };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("bare return lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn lowers_simple_bare_return_to_none_in_alias_option_context() {
        let stmt = HirStmt::Return { value: None };
        let option_ret = Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None]));
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("bare return lowered for alias option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn does_not_lower_bare_return_in_display_impl_context() {
        let stmt = HirStmt::Return { value: None };
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: None,
                in_display_impl: true,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_return_with_leaf_expr() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("return with value lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::Return(Some(_))));
    }

    #[test]
    fn lowers_simple_return_name_in_plain_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("plain return name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Ident(ref name))) if name == "x"
        ));
    }

    #[test]
    fn lowers_return_leaf_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("option return leaf lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(
                    matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
                );
            }
            _ => panic!("expected return Some(...)"),
        }
    }

    #[test]
    fn lowers_return_name_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("option return name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, ref args }))
                if matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Some".to_string()])
                    && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "x")
        ));
    }

    #[test]
    fn lowers_return_option_name_in_plain_context_without_unwrap() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
        };
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Int),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("plain return option name lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            &lowered[0],
            RustStmt::Return(Some(RustExpr::Ident(name))) if name == "maybe_x"
        ));
    }

    #[test]
    fn lowers_option_name_return_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("option passthrough name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Ident(ref name))) if name == "maybe_x"
        ));
    }

    #[test]
    fn lowers_option_name_return_with_alias_option_return_context() {
        let alias_option = Type::alias("MaybeInt", Type::Union(vec![Type::Int, Type::None]));
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "maybe_x".to_string(),
                ty: alias_option.clone(),
            }),
        };
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&alias_option),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("alias option passthrough name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Ident(ref name))) if name == "maybe_x"
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_option_return_passthrough_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "maybe_x".to_string(),
                args: vec![],
                ty: Type::Union(vec![Type::Int, Type::None]),
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_return_none_literal_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::NoneLiteral),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("return None lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn lowers_return_none_name_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "none_value".to_string(),
                ty: Type::None,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("return none-typed name lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn lowers_return_alias_none_name_with_option_return_context() {
        let alias_none = Type::alias("Nothing", Type::None);
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "none_value".to_string(),
                ty: alias_none,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("return alias-none name lowered for option context");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::Literal(RustLiteral::None)))
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_none_typed_return_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "produce_none".to_string(),
                args: vec![],
                ty: Type::None,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn does_not_lower_non_leaf_alias_none_typed_return_with_option_return_context() {
        let alias_none = Type::alias("Nothing", Type::None);
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "produce_none".to_string(),
                args: vec![],
                ty: alias_none,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_return_leaf_with_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let union_ret = Type::Union(vec![Type::Int, Type::Str]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("non-option union leaf return lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(matches!(func.as_ref(), RustExpr::Path(parts) if parts.len() == 2));
            }
            _ => panic!("expected union-variant wrapped return"),
        }
    }

    #[test]
    fn lowers_return_name_with_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let union_ret = Type::Union(vec![Type::Int, Type::Str]);
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("non-option union name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, ref args }))
                if matches!(func.as_ref(), RustExpr::Path(parts) if parts.len() == 2)
                    && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "x")
        ));
    }

    #[test]
    fn lowers_return_leaf_with_alias_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        let union_ret = Type::alias("ValueUnion", Type::Union(vec![Type::Int, Type::Str]));
        let expected_enum = union_ret.union_enum_name();
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("alias non-option union leaf return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, .. }))
                if matches!(
                    func.as_ref(),
                    RustExpr::Path(parts) if parts.first().is_some_and(|n| n == &expected_enum)
                )
        ));
    }

    #[test]
    fn lowers_return_name_with_alias_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "x".to_string(),
                ty: Type::Int,
            }),
        };
        let union_ret = Type::alias("ValueUnion", Type::Union(vec![Type::Int, Type::Str]));
        let expected_enum = union_ret.union_enum_name();
        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .expect("alias non-option union name return lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::FnCall { ref func, ref args }))
                if matches!(
                    func.as_ref(),
                    RustExpr::Path(parts) if parts.first().is_some_and(|n| n == &expected_enum)
                ) && matches!(args.first(), Some(RustExpr::Ident(name)) if name == "x")
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_return_with_non_option_union_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "value".to_string(),
                args: vec![],
                ty: Type::Int,
            }),
        };
        let union_ret = Type::Union(vec![Type::Int, Type::Str]);
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&union_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_return_in_class_scope() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::IntLiteral(5)),
        };
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Int),
                in_display_impl: false,
                in_class_scope: true,
                in_generator_closure: false,
            },
        )
        .is_some());
    }

    #[test]
    fn lowers_self_return_in_class_scope_with_clone() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Name {
                name: "self".to_string(),
                ty: Type::Class {
                    name: "Point".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                },
            }),
        };

        let lowered = try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&Type::Class {
                    name: "Point".to_string(),
                    fields: vec![],
                    methods: vec![],
                    parent_class: None,
                }),
                in_display_impl: false,
                in_class_scope: true,
                in_generator_closure: false,
            },
        )
        .expect("self return in class scope lowered");

        assert!(matches!(
            lowered[0],
            RustStmt::Return(Some(RustExpr::MethodCall { ref receiver, ref method, ref args }))
                if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "self")
                    && method == "clone"
                    && args.is_empty()
        ));
    }

    #[test]
    fn does_not_lower_non_leaf_return_with_option_return_context() {
        let stmt = HirStmt::Return {
            value: Some(HirExpr::Call {
                func: "value".to_string(),
                args: vec![],
                ty: Type::Int,
            }),
        };
        let option_ret = Type::Union(vec![Type::Int, Type::None]);
        assert!(try_lower_simple_stmt_with_ctx(
            &stmt,
            false,
            &HashSet::new(),
            &HashSet::new(),
            SimpleStmtLoweringCtx {
                return_type: Some(&option_ret),
                in_display_impl: false,
                in_class_scope: false,
                in_generator_closure: false,
            },
        )
        .is_none());
    }

    #[test]
    fn lowers_simple_raise_with_leaf_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::IntLiteral(7),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("raise lowered");

        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, .. })) => {
                assert!(
                    matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()])
                );
            }
            _ => panic!("expected return Err(...)"),
        }
    }

    #[test]
    fn does_not_lower_raise_with_non_leaf_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::Call {
                func: "err".to_string(),
                args: vec![],
                ty: Type::Int,
            },
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
    }

    #[test]
    fn lowers_simple_raise_with_name_expr() {
        let stmt = HirStmt::Raise {
            value: HirExpr::Name {
                name: "e".to_string(),
                ty: Type::Int,
            },
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("raise name lowered");
        assert_eq!(lowered.len(), 1);
        match &lowered[0] {
            RustStmt::Return(Some(RustExpr::FnCall { func, args })) => {
                assert!(
                    matches!(func.as_ref(), RustExpr::Path(parts) if parts == &vec!["Err".to_string()])
                );
                assert!(matches!(args.first(), Some(RustExpr::Ident(name)) if name == "e"));
            }
            _ => panic!("expected return Err(e)"),
        }
    }

    #[test]
    fn lowers_simple_assert_without_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: None,
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(lowered[0], RustStmt::Assert { msg: None, .. }));
    }

    #[test]
    fn lowers_simple_assert_with_leaf_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::StringLiteral("boom".to_string())),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert with msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::Literal(RustLiteral::Str(_))),
                ..
            }
        ));
    }

    #[test]
    fn lowers_simple_assert_with_name_msg() {
        let stmt = HirStmt::Assert {
            test: HirExpr::BoolLiteral(true),
            msg: Some(HirExpr::Name {
                name: "msg".to_string(),
                ty: Type::Str,
            }),
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert with name msg lowered");
        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                msg: Some(RustExpr::Ident(ref name)),
                ..
            } if name == "msg"
        ));
    }

    #[test]
    fn does_not_lower_assert_with_non_leaf_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Call {
                func: "is_ok".to_string(),
                args: vec![],
                ty: Type::Bool,
            },
            msg: None,
        };

        assert!(try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new(),).is_none());
    }

    #[test]
    fn lowers_simple_assert_with_bool_name_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::Name {
                name: "ok".to_string(),
                ty: Type::Bool,
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert bool name test lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                cond: RustExpr::Ident(ref name),
                msg: None,
            } if name == "ok"
        ));
    }

    #[test]
    fn lowers_simple_assert_with_not_bool_name_test() {
        let stmt = HirStmt::Assert {
            test: HirExpr::UnaryOp {
                op: "not".to_string(),
                operand: Box::new(HirExpr::Name {
                    name: "ok".to_string(),
                    ty: Type::Bool,
                }),
                ty: Type::Bool,
            },
            msg: None,
        };

        let lowered = try_lower_simple_stmt(&stmt, false, &HashSet::new(), &HashSet::new())
            .expect("assert not-bool name test lowered");

        assert_eq!(lowered.len(), 1);
        assert!(matches!(
            lowered[0],
            RustStmt::Assert {
                cond: RustExpr::UnaryOp {
                    ref op,
                    ref operand,
                },
                msg: None,
            } if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "ok")
        ));
    }

