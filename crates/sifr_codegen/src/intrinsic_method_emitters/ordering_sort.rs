use super::{
    HirExpr, RustEmitter, Type, registry_call_callable_with_retained_owned_args,
    registry_callable_signature, registry_iterable_to_owned_iter_expr,
    registry_iterable_to_vec_expr,
};
use crate::{RustExpr, RustParam, RustStmt, RustType};

pub(super) fn lower_sorted(emitter: &mut RustEmitter, args: &[HirExpr]) -> Option<RustExpr> {
    let element_ty =
        crate::resolve_alias_type_for_plain_call(args.first()?.ty()).iterable_element_type()?;
    let keyed = args.len() == 3 && !matches!(args[1], HirExpr::NoneLiteral);
    let mut stmts = Vec::new();
    if !keyed {
        stmts.push(RustStmt::Let {
            mutable: true,
            name: "__sifr_sorted_values".to_string(),
            ty: None,
            value: registry_iterable_to_vec_expr(emitter, &args[0])?,
        });
    }
    let reverse_name = (args.len() == 3).then(|| "__sifr_sorted_reverse".to_string());
    if let Some(reverse_name) = &reverse_name {
        stmts.push(RustStmt::Let {
            mutable: false,
            name: reverse_name.clone(),
            ty: None,
            value: emitter.try_lower_registry_expr_strict(&args[2])?,
        });
    }

    if keyed {
        let values = registry_iterable_to_owned_iter_expr(emitter, &args[0])?;
        lower_keyed_sort(
            emitter,
            &args[1],
            &element_ty,
            values,
            reverse_name.as_deref(),
            &mut stmts,
        )?;
        return Some(RustExpr::Block {
            stmts,
            expr: Some(Box::new(undecorate_sorted_values())),
        });
    }

    stmts.push(RustStmt::Expr(sort_values_expr(
        "__sifr_sorted_values",
        &element_ty,
        reverse_name.as_deref(),
        false,
    )));
    Some(RustExpr::Block {
        stmts,
        expr: Some(Box::new(RustExpr::Ident(
            "__sifr_sorted_values".to_string(),
        ))),
    })
}

fn lower_keyed_sort(
    emitter: &mut RustEmitter,
    key: &HirExpr,
    element_ty: &Type,
    values: RustExpr,
    reverse_name: Option<&str>,
    stmts: &mut Vec<RustStmt>,
) -> Option<()> {
    let (_, _, key_ty) = registry_callable_signature(key)?;
    let key_call = registry_call_callable_with_retained_owned_args(
        emitter,
        key,
        &[("__sifr_sorted_value".to_string(), element_ty.clone())],
    )?;
    stmts.push(RustStmt::Let {
        mutable: true,
        name: "__sifr_sorted_pairs".to_string(),
        ty: None,
        value: RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(values),
                method: "map".to_string(),
                args: vec![RustExpr::Closure {
                    params: vec![RustParam::Named {
                        name: "__sifr_sorted_value".to_string(),
                        ty: RustType::Named("_".to_string()),
                    }],
                    body: Box::new(RustExpr::Tuple(vec![
                        key_call,
                        RustExpr::Ident("__sifr_sorted_value".to_string()),
                    ])),
                    is_move: false,
                }],
            }),
            method: "collect::<Vec<_>>".to_string(),
            args: Vec::new(),
        },
    });
    stmts.push(RustStmt::Expr(sort_values_expr(
        "__sifr_sorted_pairs",
        &key_ty,
        reverse_name,
        true,
    )));
    Some(())
}

fn sort_values_expr(
    values_name: &str,
    ordering_ty: &Type,
    reverse_name: Option<&str>,
    decorated: bool,
) -> RustExpr {
    let ascending = compare_items(
        "__sifr_sorted_left",
        "__sifr_sorted_right",
        ordering_ty,
        decorated,
    );
    let comparison = if let Some(reverse_name) = reverse_name {
        RustExpr::If {
            cond: Box::new(RustExpr::Ident(reverse_name.to_string())),
            then_expr: Box::new(compare_items(
                "__sifr_sorted_right",
                "__sifr_sorted_left",
                ordering_ty,
                decorated,
            )),
            else_expr: Some(Box::new(ascending)),
        }
    } else {
        ascending
    };
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::Ident(values_name.to_string())),
        method: "sort_by".to_string(),
        args: vec![RustExpr::Closure {
            params: vec![
                RustParam::Named {
                    name: "__sifr_sorted_left".to_string(),
                    ty: RustType::Named("_".to_string()),
                },
                RustParam::Named {
                    name: "__sifr_sorted_right".to_string(),
                    ty: RustType::Named("_".to_string()),
                },
            ],
            body: Box::new(comparison),
            is_move: false,
        }],
    }
}

fn compare_items(left: &str, right: &str, ty: &Type, decorated: bool) -> RustExpr {
    let is_float = matches!(ty.resolve_alias(), Type::Float);
    let right = ordering_value(right, decorated);
    let right = if is_float && !decorated {
        right
    } else {
        RustExpr::Ref {
            mutable: false,
            expr: Box::new(right),
        }
    };
    let comparison = RustExpr::MethodCall {
        receiver: Box::new(ordering_value(left, decorated)),
        method: if is_float {
            "partial_cmp".to_string()
        } else {
            "cmp".to_string()
        },
        args: vec![right],
    };
    if !is_float {
        return comparison;
    }
    RustExpr::MethodCall {
        receiver: Box::new(comparison),
        method: "unwrap_or".to_string(),
        args: vec![RustExpr::Path(vec![
            "std".to_string(),
            "cmp".to_string(),
            "Ordering".to_string(),
            "Equal".to_string(),
        ])],
    }
}

fn ordering_value(name: &str, decorated: bool) -> RustExpr {
    let value = RustExpr::Ident(name.to_string());
    if decorated {
        RustExpr::Field {
            expr: Box::new(value),
            field: "0".to_string(),
        }
    } else {
        value
    }
}

fn undecorate_sorted_values() -> RustExpr {
    RustExpr::MethodCall {
        receiver: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::MethodCall {
                receiver: Box::new(RustExpr::Ident("__sifr_sorted_pairs".to_string())),
                method: "into_iter".to_string(),
                args: Vec::new(),
            }),
            method: "map".to_string(),
            args: vec![RustExpr::Closure {
                params: vec![RustParam::Named {
                    name: "__sifr_sorted_pair".to_string(),
                    ty: RustType::Named("_".to_string()),
                }],
                body: Box::new(RustExpr::Field {
                    expr: Box::new(RustExpr::Ident("__sifr_sorted_pair".to_string())),
                    field: "1".to_string(),
                }),
                is_move: false,
            }],
        }),
        method: "collect::<Vec<_>>".to_string(),
        args: Vec::new(),
    }
}
