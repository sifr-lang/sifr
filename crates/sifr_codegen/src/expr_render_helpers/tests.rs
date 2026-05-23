#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustExpr, RustLiteral, RustStmt, RustType};

    fn emitter_with_large_int_const() -> RustEmitter {
        let mut emitter = RustEmitter::new();
        emitter.module_constants.insert(
            "BIG_LIMIT".to_string(),
            (Type::Int, "__const_BIG_LIMIT()".to_string()),
        );
        emitter
    }

    #[test]
    fn rewrites_large_int_module_const_arithmetic_to_sifr_int_operands() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(1))),
                ty: RustType::I64,
            }),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt binary expression");
        };
        assert_eq!(op, "+");
        assert!(matches!(
            left.as_ref(),
            RustExpr::FnCall { func, args }
                if args.is_empty()
                    && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::FnCall { func, args }
                if args.len() == 1
                    && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
        ));
    }

    #[test]
    fn rewrites_large_int_floor_division_by_nonzero_literal_to_checked_runtime_call() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "quotient".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
                op: "/".to_string(),
                right: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                }),
            },
        });

        let RustStmt::Let {
            ty: Some(RustType::Named(ty)),
            value:
                RustExpr::MethodCall {
                    receiver,
                    method,
                    args,
                },
            ..
        } = rewritten
        else {
            panic!("expected SifrInt floor division let");
        };
        assert_eq!(ty, "SifrInt");
        assert_eq!(method, "floor_div_known_nonzero");
        assert!(matches!(
            receiver.as_ref(),
            RustExpr::FnCall { func, args }
                if args.is_empty()
                    && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
        ));
        assert!(matches!(
            args.as_slice(),
            [RustExpr::Ref {
                mutable: false,
                expr,
            }] if matches!(
                expr.as_ref(),
                RustExpr::FnCall { func, args }
                    if args.len() == 1
                        && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
            )
        ));
    }

    #[test]
    fn rewrites_large_int_modulo_by_nonzero_literal_to_checked_runtime_call() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "remainder".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::BinOp {
                left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
                op: "%".to_string(),
                right: Box::new(RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                }),
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref ty)),
                value: RustExpr::MethodCall { ref method, .. },
                ..
            } if ty == "SifrInt" && method == "floor_mod_known_nonzero"
        ));
    }

    #[test]
    fn rewrites_large_int_module_const_let_type_to_sifr_int() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "x".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("BIG_LIMIT".to_string()),
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                value: RustExpr::FnCall { .. },
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn local_binding_shadows_large_int_module_const_rewrite() {
        let mut emitter = emitter_with_large_int_const();
        emitter
            .local_binding_types
            .insert("BIG_LIMIT".to_string(), Type::Int);

        let rewritten = emitter
            .rewrite_stdlib_constant_idents_in_expr(RustExpr::Ident("BIG_LIMIT".to_string()));

        assert!(matches!(rewritten, RustExpr::Ident(name) if name == "BIG_LIMIT"));
    }

    #[test]
    fn rewrites_registered_sifr_int_local_arithmetic_to_sifr_int_operands() {
        let emitter = emitter_with_large_int_const();
        let _ = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "oversized_local".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("BIG_LIMIT".to_string()),
        });

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("oversized_local".to_string())),
            op: "+".to_string(),
            right: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::I64,
            }),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt local binary expression");
        };
        assert_eq!(op, "+");
        assert!(matches!(
            left.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "oversized_local")
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::FnCall { func, args }
                if args.len() == 1
                    && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
        ));
    }

    #[test]
    fn rewrites_large_int_module_const_comparison_to_sifr_int_operands() {
        let emitter = emitter_with_large_int_const();
        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
            op: ">".to_string(),
            right: Box::new(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(100))),
                ty: RustType::I64,
            }),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt comparison expression");
        };
        assert_eq!(op, ">");
        assert!(matches!(
            left.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(
                    expr.as_ref(),
                    RustExpr::FnCall { func, args }
                        if args.is_empty()
                            && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
                )
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(
                    expr.as_ref(),
                    RustExpr::FnCall { func, args }
                        if args.len() == 1
                            && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
                )
        ));
    }

    #[test]
    fn rewrites_registered_sifr_int_local_comparison_to_borrowed_operands() {
        let emitter = emitter_with_large_int_const();
        let _ = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "oversized_local".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("BIG_LIMIT".to_string()),
        });

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::BinOp {
            left: Box::new(RustExpr::Ident("oversized_local".to_string())),
            op: "<".to_string(),
            right: Box::new(RustExpr::Ident("BIG_LIMIT".to_string())),
        });

        let RustExpr::BinOp { left, op, right } = rewritten else {
            panic!("expected SifrInt local comparison expression");
        };
        assert_eq!(op, "<");
        assert!(matches!(
            left.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "oversized_local")
        ));
        assert!(matches!(
            right.as_ref(),
            RustExpr::Ref { mutable: false, expr }
                if matches!(
                    expr.as_ref(),
                    RustExpr::FnCall { func, args }
                        if args.is_empty()
                            && matches!(func.as_ref(), RustExpr::Ident(name) if name == "__const_BIG_LIMIT")
            )
        ));
    }

    #[test]
    fn rewrites_forced_sifr_int_assignment_target_storage() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("total".to_string());

        let rewritten_let = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: true,
            name: "total".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                ty: RustType::I64,
            },
        });

        assert!(matches!(
            rewritten_let,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                value: RustExpr::FnCall { .. },
                ..
            } if name == "SifrInt"
        ));

        let rewritten_assign = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Assign {
            target: RustExpr::Ident("total".to_string()),
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::I64,
            },
        });

        assert!(matches!(
            rewritten_assign,
            RustStmt::Assign {
                target: RustExpr::Ident(ref name),
                value: RustExpr::FnCall { func, args },
            } if name == "total"
                && args.len() == 1
                && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
        ));
    }

    #[test]
    fn rewrites_sifr_int_value_position_aliases_to_clone() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_local_bindings
            .borrow_mut()
            .insert("source".to_string());
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("target".to_string());

        let rewritten_let = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "target".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::Ident("source".to_string()),
        });
        assert!(matches!(
            rewritten_let,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                value: RustExpr::Clone(inner),
                ..
            } if name == "SifrInt"
                && matches!(inner.as_ref(), RustExpr::Ident(source) if source == "source")
        ));

        let rewritten_assign = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Assign {
            target: RustExpr::Ident("target".to_string()),
            value: RustExpr::Ident("source".to_string()),
        });
        assert!(matches!(
            rewritten_assign,
            RustStmt::Assign {
                target: RustExpr::Ident(ref target),
                value: RustExpr::Clone(inner),
            } if target == "target"
                && matches!(inner.as_ref(), RustExpr::Ident(source) if source == "source")
        ));
    }

    #[test]
    fn rewrites_forced_sifr_int_augassign_to_assignment() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("total".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
            target: RustExpr::Ident("total".to_string()),
            op: "+".to_string(),
            value: RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                ty: RustType::I64,
            },
        });

        let RustStmt::Assign { target, value } = rewritten else {
            panic!("expected SifrInt augassign rewrite to plain assignment");
        };
        assert!(matches!(target, RustExpr::Ident(ref name) if name == "total"));
        assert!(matches!(
            value,
            RustExpr::BinOp { left, op, right }
                if op == "+"
                    && matches!(
                        left.as_ref(),
                        RustExpr::Ref { mutable: false, expr }
                            if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "total")
                    )
                    && matches!(
                        right.as_ref(),
                        RustExpr::FnCall { func, args }
                            if args.len() == 1
                                && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
                    )
        ));
    }

    #[test]
    fn rewrites_sifr_int_augassign_registered_source_to_borrowed_operand() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_forced_local_bindings
            .borrow_mut()
            .insert("total".to_string());
        emitter
            .sifr_int_local_bindings
            .borrow_mut()
            .insert("source".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
            target: RustExpr::Ident("total".to_string()),
            op: "+".to_string(),
            value: RustExpr::Ident("source".to_string()),
        });

        let RustStmt::Assign { value, .. } = rewritten else {
            panic!("expected SifrInt augassign rewrite to plain assignment");
        };
        assert!(matches!(
            value,
            RustExpr::BinOp { right, .. }
                if matches!(
                    right.as_ref(),
                    RustExpr::Ref { mutable: false, expr }
                        if matches!(expr.as_ref(), RustExpr::Ident(name) if name == "source")
                )
        ));
    }

    #[test]
    fn rewrites_sifr_int_augassign_for_supported_ops() {
        for op in ["+", "-", "*"] {
            let emitter = RustEmitter::new();
            emitter
                .sifr_int_forced_local_bindings
                .borrow_mut()
                .insert("total".to_string());

            let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
                target: RustExpr::Ident("total".to_string()),
                op: op.to_string(),
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(2))),
                    ty: RustType::I64,
                },
            });

            assert!(matches!(
                rewritten,
                RustStmt::Assign {
                    value: RustExpr::BinOp { op: ref rewritten_op, .. },
                    ..
                } if rewritten_op == op
            ));
        }
    }

    #[test]
    fn rewrites_sifr_int_floor_mod_augassign_by_nonzero_literal_to_assignment() {
        for (op, expected_method) in [
            ("/", "floor_div_known_nonzero"),
            ("%", "floor_mod_known_nonzero"),
        ] {
            let emitter = RustEmitter::new();
            emitter
                .sifr_int_forced_local_bindings
                .borrow_mut()
                .insert("total".to_string());

            let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::AugAssign {
                target: RustExpr::Ident("total".to_string()),
                op: op.to_string(),
                value: RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                },
            });

            assert!(matches!(
                rewritten,
                RustStmt::Assign {
                    target: RustExpr::Ident(ref target),
                    value:
                        RustExpr::MethodCall {
                            receiver,
                            ref method,
                            ref args,
                        },
                } if target == "total"
                    && method == expected_method
                    && matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "total")
                    && matches!(
                        args.as_slice(),
                        [RustExpr::Ref {
                            mutable: false,
                            expr,
                        }] if matches!(
                            expr.as_ref(),
                            RustExpr::FnCall { func, args }
                                if args.len() == 1
                                    && matches!(func.as_ref(), RustExpr::Path(path) if path.as_slice() == ["SifrInt", "from_i64"])
                        )
                    )
            ));
        }
    }

    #[test]
    fn rewrites_sifr_int_returning_function_call_let_type() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_function_returns
            .borrow_mut()
            .insert("make_big".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "value".to_string(),
            ty: Some(RustType::I64),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("make_big".to_string())),
                args: vec![],
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn rewrites_sifr_int_returning_function_call_named_i64_let_type() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_function_returns
            .borrow_mut()
            .insert("make_big".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "value".to_string(),
            ty: Some(RustType::Named("i64".to_string())),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("make_big".to_string())),
                args: vec![],
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn rewrites_sifr_int_returning_function_call_with_args_let_type() {
        let emitter = RustEmitter::new();
        emitter
            .sifr_int_function_returns
            .borrow_mut()
            .insert("make_big_with_arg".to_string());

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_stmt(RustStmt::Let {
            mutable: false,
            name: "value".to_string(),
            ty: Some(RustType::Named("i64".to_string())),
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("make_big_with_arg".to_string())),
                args: vec![RustExpr::Cast {
                    expr: Box::new(RustExpr::Literal(RustLiteral::Int(3))),
                    ty: RustType::I64,
                }],
            },
        });

        assert!(matches!(
            rewritten,
            RustStmt::Let {
                ty: Some(RustType::Named(ref name)),
                ..
            } if name == "SifrInt"
        ));
    }

    #[test]
    fn closure_block_returns_do_not_inherit_sifr_int_return_state() {
        let emitter = RustEmitter::new();
        emitter.current_sifr_int_return.set(true);

        let rewritten = emitter.rewrite_stdlib_constant_idents_in_expr(RustExpr::ClosureBlock {
            params: vec![],
            body: vec![RustStmt::Return(Some(RustExpr::Cast {
                expr: Box::new(RustExpr::Literal(RustLiteral::Int(42))),
                ty: RustType::I64,
            }))],
            is_move: false,
            is_async: false,
        });

        let RustExpr::ClosureBlock { body, .. } = rewritten else {
            panic!("expected closure block");
        };
        assert!(matches!(
            body.as_slice(),
            [RustStmt::Return(Some(RustExpr::Cast {
                ty: RustType::I64,
                ..
            }))]
        ));
        assert!(emitter.current_sifr_int_return.get());
    }
}
