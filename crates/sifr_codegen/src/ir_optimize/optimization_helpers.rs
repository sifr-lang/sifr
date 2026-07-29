use super::{RustExpr, RustLiteral, RustType};

pub(crate) fn not_expr(expr: RustExpr) -> RustExpr {
    match expr {
        RustExpr::Literal(RustLiteral::Bool(value)) => RustExpr::Literal(RustLiteral::Bool(!value)),
        RustExpr::UnaryOp { op, operand } if op == "!" => *operand,
        other => RustExpr::UnaryOp {
            op: "!".to_string(),
            operand: Box::new(other),
        },
    }
}

pub(crate) fn is_copy_type(ty: &RustType) -> bool {
    match ty {
        RustType::I64 | RustType::F64 | RustType::Bool | RustType::Unit => true,
        RustType::Ref { .. } => true,
        RustType::Tuple(items) => items.iter().all(is_copy_type),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir_optimize::{remove_trivial_clones_in_items, remove_unneeded_mutability_in_items};
    use crate::{RustItem, RustParam, RustStmt, Visibility};

    #[test]
    fn removes_clone_on_literals_and_copy_casts() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![RustParam::Named {
                name: "n".to_string(),
                ty: RustType::I64,
            }],
            ret: None,
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "a".to_string(),
                    ty: None,
                    value: RustExpr::Clone(Box::new(RustExpr::Literal(RustLiteral::Int(1)))),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "b".to_string(),
                    ty: None,
                    value: RustExpr::Clone(Box::new(RustExpr::Cast {
                        expr: Box::new(RustExpr::Ident("n".to_string())),
                        ty: RustType::I64,
                    })),
                },
            ],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 2);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        let RustStmt::Let { value: first, .. } = &body[0] else {
            panic!("expected let statement");
        };
        assert!(matches!(first, RustExpr::Literal(RustLiteral::Int(1))));

        let RustStmt::Let { value: second, .. } = &body[1] else {
            panic!("expected let statement");
        };
        assert!(matches!(second, RustExpr::Cast { .. }));
    }

    #[test]
    fn keeps_clone_on_non_trivial_identifier() {
        let mut items = vec![RustItem::Const {
            name: "X".to_string(),
            visibility: Visibility::Private,
            ty: RustType::String_,
            value: RustExpr::Clone(Box::new(RustExpr::Ident("value".to_string()))),
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 0);
        let RustItem::Const { value, .. } = &items[0] else {
            panic!("expected const item");
        };
        assert!(matches!(value, RustExpr::Clone(_)));
    }

    #[test]
    fn optimizes_nested_clone_sites() {
        let mut items = vec![RustItem::Fn {
            name: "nested".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::Expr(RustExpr::FnCall {
                func: Box::new(RustExpr::Ident("consume".to_string())),
                args: vec![RustExpr::Clone(Box::new(RustExpr::Literal(
                    RustLiteral::Str("x".to_string()),
                )))],
            })],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 1);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        let RustStmt::Expr(RustExpr::FnCall { args, .. }) = &body[0] else {
            panic!("expected fn call expression");
        };
        assert!(matches!(
            args.first(),
            Some(RustExpr::Literal(RustLiteral::Str(s))) if s == "x"
        ));
    }

    #[test]
    fn preserves_mutable_callable_bindings() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![
                RustStmt::Let {
                    mutable: true,
                    name: "apply".to_string(),
                    ty: None,
                    value: RustExpr::ClosureBlock {
                        params: vec![],
                        body: vec![],
                        is_move: false,
                        is_async: false,
                    },
                },
                RustStmt::Expr(RustExpr::FnCall {
                    func: Box::new(RustExpr::Ident("apply".to_string())),
                    args: vec![],
                }),
            ],
            is_async: false,
        }];

        let removed =
            remove_unneeded_mutability_in_items(&mut items, &std::collections::HashSet::new());
        assert_eq!(removed, 0);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        assert!(matches!(
            body.first(),
            Some(RustStmt::Let {
                mutable: true,
                name,
                ..
            }) if name == "apply"
        ));
    }

    #[test]
    fn source_only_method_names_require_checked_place_protection() {
        let make_items = || {
            vec![RustItem::Fn {
                name: "demo".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![],
                ret: None,
                body: vec![
                    RustStmt::Let {
                        mutable: true,
                        name: "items".to_string(),
                        ty: None,
                        value: RustExpr::Vec(vec![]),
                    },
                    RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("items".to_string())),
                        method: "source_mutation".to_string(),
                        args: vec![],
                    }),
                ],
                is_async: false,
            }]
        };

        let mut unprotected = make_items();
        assert_eq!(
            remove_unneeded_mutability_in_items(
                &mut unprotected,
                &std::collections::HashSet::new()
            ),
            1
        );

        let mut protected = make_items();
        let protected_names = std::collections::HashSet::from(["items".to_string()]);
        assert_eq!(
            remove_unneeded_mutability_in_items(&mut protected, &protected_names),
            0
        );
    }

    #[test]
    fn compiler_generated_method_names_preserve_unproven_ir_locals() {
        let mut items = vec![
            compiler_generated_mutating_method_item("__writer", "write"),
            compiler_generated_mutating_method_item("__items", "append"),
        ];

        assert_eq!(
            remove_unneeded_mutability_in_items(&mut items, &std::collections::HashSet::new()),
            0
        );
    }

    fn compiler_generated_mutating_method_item(name: &str, method: &str) -> RustItem {
        RustItem::Fn {
            name: format!("exercise_{method}"),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![
                RustStmt::Let {
                    mutable: true,
                    name: name.to_string(),
                    ty: None,
                    value: RustExpr::Verbatim(format!("make_{name}()")),
                },
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident(name.to_string())),
                    method: method.to_string(),
                    args: vec![],
                }),
            ],
            is_async: false,
        }
    }

    #[test]
    fn rewrites_true_while_to_loop() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::While {
                cond: RustExpr::Literal(RustLiteral::Bool(true)),
                body: vec![RustStmt::Break],
            }],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 1);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        assert!(matches!(body.first(), Some(RustStmt::Loop { .. })));
    }

    #[test]
    fn removes_zero_skip_method_call() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("items".to_string())),
                    method: "skip".to_string(),
                    args: vec![RustExpr::Cast {
                        expr: Box::new(RustExpr::Literal(RustLiteral::Int(0))),
                        ty: RustType::Named("usize".to_string()),
                    }],
                }),
                method: "take".to_string(),
                args: vec![RustExpr::Literal(RustLiteral::Int(3))],
            })],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 1);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        let Some(RustStmt::Expr(RustExpr::MethodCall {
            receiver, method, ..
        })) = body.first()
        else {
            panic!("expected outer method call");
        };
        assert_eq!(method, "take");
        assert!(matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "items"));
    }

    #[test]
    fn rewrites_identity_map_or_else_to_unwrap_or_else() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "Decimal".to_string(),
                        "checked_div".to_string(),
                    ])),
                    args: vec![
                        RustExpr::Ident("left".to_string()),
                        RustExpr::Ident("right".to_string()),
                    ],
                }),
                method: "map_or_else".to_string(),
                args: vec![
                    RustExpr::Closure {
                        params: vec![],
                        body: Box::new(RustExpr::Ident("fallback".to_string())),
                        is_move: false,
                    },
                    RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "value".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Ident("value".to_string())),
                        is_move: false,
                    },
                ],
            })],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 1);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        let Some(RustStmt::Expr(RustExpr::MethodCall { method, args, .. })) = body.first() else {
            panic!("expected method call");
        };
        assert_eq!(method, "unwrap_or_else");
        assert_eq!(args.len(), 1);
    }

    #[test]
    fn keeps_identity_map_or_else_on_unknown_receivers() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![RustStmt::Expr(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("maybe_value".to_string())),
                method: "map_or_else".to_string(),
                args: vec![
                    RustExpr::Closure {
                        params: vec![],
                        body: Box::new(RustExpr::Ident("fallback".to_string())),
                        is_move: false,
                    },
                    RustExpr::Closure {
                        params: vec![RustParam::Named {
                            name: "value".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Ident("value".to_string())),
                        is_move: false,
                    },
                ],
            })],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 0);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        let Some(RustStmt::Expr(RustExpr::MethodCall { method, args, .. })) = body.first() else {
            panic!("expected method call");
        };
        assert_eq!(method, "map_or_else");
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn simplifies_bool_literal_comparisons() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: vec![],
            params: vec![],
            ret: None,
            body: vec![
                RustStmt::Expr(RustExpr::BinOp {
                    left: Box::new(RustExpr::Ident("flag".to_string())),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Bool(false))),
                }),
                RustStmt::Expr(RustExpr::BinOp {
                    left: Box::new(RustExpr::Literal(RustLiteral::Bool(false))),
                    op: "!=".to_string(),
                    right: Box::new(RustExpr::Ident("other".to_string())),
                }),
            ],
            is_async: false,
        }];

        let removed = remove_trivial_clones_in_items(&mut items);
        assert_eq!(removed, 2);

        let RustItem::Fn { body, .. } = &items[0] else {
            panic!("expected fn item");
        };
        assert!(matches!(
            body.first(),
            Some(RustStmt::Expr(RustExpr::UnaryOp { op, operand }))
                if op == "!" && matches!(operand.as_ref(), RustExpr::Ident(name) if name == "flag")
        ));
        assert!(matches!(
            body.get(1),
            Some(RustStmt::Expr(RustExpr::Ident(name))) if name == "other"
        ));
    }
}
