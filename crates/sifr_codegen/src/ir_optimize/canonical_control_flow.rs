use crate::{RustExpr, RustItem, RustLiteral, RustStmt};

pub(crate) fn simplify_control_flow_in_items(items: &mut [RustItem]) -> usize {
    items.iter_mut().map(simplify_item).sum()
}

fn simplify_item(item: &mut RustItem) -> usize {
    match item {
        RustItem::Fn { body, .. } => simplify_block(body) + normalize_tail_returns(body),
        RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
            methods.iter_mut().map(simplify_item).sum()
        }
        RustItem::Enum { variants, .. } => variants
            .iter_mut()
            .filter_map(|variant| variant.value.as_mut())
            .map(simplify_expr)
            .sum(),
        RustItem::Const { value, .. } | RustItem::Static { value, .. } => simplify_expr(value),
        RustItem::Use(_)
        | RustItem::UseAlias { .. }
        | RustItem::Struct { .. }
        | RustItem::TupleStruct { .. }
        | RustItem::TraitMethodSig { .. }
        | RustItem::TypeAlias { .. }
        | RustItem::Attr(_) => 0,
    }
}

fn simplify_block(body: &mut Vec<RustStmt>) -> usize {
    let mut changed = body.iter_mut().map(simplify_stmt).sum();
    let before_empty = body.len();
    body.retain(|stmt| !matches!(stmt, RustStmt::Block(block) if block.is_empty()));
    changed += before_empty - body.len();

    if let Some(last_reachable) = body.iter().position(stmt_always_exits) {
        let reachable_len = last_reachable + 1;
        changed += body.len().saturating_sub(reachable_len);
        body.truncate(reachable_len);
    }
    changed
}

fn simplify_stmt(stmt: &mut RustStmt) -> usize {
    let mut changed = match stmt {
        RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => simplify_expr(value),
        RustStmt::LetElse {
            value, else_body, ..
        } => simplify_expr(value) + simplify_block(else_body),
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            simplify_expr(target) + simplify_expr(value)
        }
        RustStmt::Expr(expr) | RustStmt::TailExpr(expr) | RustStmt::Return(Some(expr)) => {
            simplify_expr(expr)
        }
        RustStmt::Assert { cond, msg } => {
            simplify_expr(cond) + msg.as_mut().map(simplify_expr).unwrap_or_default()
        }
        RustStmt::If {
            cond,
            then_body,
            else_body,
        }
        | RustStmt::IfLet {
            expr: cond,
            then_body,
            else_body,
            ..
        } => {
            let mut count = simplify_expr(cond) + simplify_block(then_body);
            count += else_body.as_mut().map(simplify_block).unwrap_or_default();
            count
        }
        RustStmt::Match { expr, arms } => {
            let mut count = simplify_expr(expr);
            for arm in arms {
                count += arm.guard.as_mut().map(simplify_expr).unwrap_or_default();
                count += simplify_block(&mut arm.body);
            }
            count
        }
        RustStmt::For { iter, body, .. } => simplify_expr(iter) + simplify_block(body),
        RustStmt::With { items, body } => {
            items
                .iter_mut()
                .map(|item| simplify_expr(&mut item.value))
                .sum::<usize>()
                + simplify_block(body)
        }
        RustStmt::While { cond, body } => simplify_expr(cond) + simplify_block(body),
        RustStmt::Loop { body } | RustStmt::Block(body) => simplify_block(body),
        RustStmt::LocalFn { body, .. } => simplify_block(body) + normalize_tail_returns(body),
        RustStmt::Verbatim(_)
        | RustStmt::LetDecl { .. }
        | RustStmt::Return(None)
        | RustStmt::Break
        | RustStmt::Continue => 0,
    };

    if let RustStmt::If {
        cond: RustExpr::Literal(RustLiteral::Bool(value)),
        then_body,
        else_body,
    } = stmt
    {
        let selected = if *value {
            std::mem::take(then_body)
        } else {
            else_body.take().unwrap_or_default()
        };
        *stmt = RustStmt::Block(selected);
        changed += 1;
    }
    if let RustStmt::IfLet {
        pattern,
        expr,
        then_body,
        else_body: None,
    } = stmt
        && pattern == "Some(_)"
        && then_body.is_empty()
    {
        *stmt = RustStmt::Let {
            mutable: false,
            name: "_".to_string(),
            ty: None,
            value: expr.clone(),
        };
        changed += 1;
    }
    if let RustStmt::Assert { cond, msg } = stmt {
        let comparison = unparenthesized_expr(cond);
        if let RustExpr::BinOp { left, op, right } = comparison
            && matches!(op.as_str(), "==" | "!=")
        {
            let mut args = vec![left.as_ref().clone(), right.as_ref().clone()];
            if let Some(msg) = msg {
                args.push(RustExpr::Literal(RustLiteral::Str("{}".to_string())));
                args.push(msg.clone());
            }
            *stmt = RustStmt::Expr(RustExpr::MacroCall {
                name: if op == "==" { "assert_eq" } else { "assert_ne" }.to_string(),
                args,
            });
            changed += 1;
        }
    }
    changed
}

fn unparenthesized_expr(mut expr: &RustExpr) -> &RustExpr {
    while let RustExpr::Paren(inner) = expr {
        expr = inner;
    }
    expr
}

fn simplify_expr(expr: &mut RustExpr) -> usize {
    let mut changed = match expr {
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) | RustExpr::Verbatim(_) => 0,
        RustExpr::MethodCall { receiver, args, .. }
        | RustExpr::FnCall {
            func: receiver,
            args,
        } => simplify_expr(receiver) + args.iter_mut().map(simplify_expr).sum::<usize>(),
        RustExpr::MacroCall { args, .. }
        | RustExpr::FormatMacro { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => args.iter_mut().map(simplify_expr).sum(),
        RustExpr::BinOp { left, right, .. }
        | RustExpr::Range {
            start: left,
            end: right,
        } => simplify_expr(left) + simplify_expr(right),
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Ref { expr: operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Cast { expr: operand, .. }
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Paren(operand) => simplify_expr(operand),
        RustExpr::Field { expr, .. } => simplify_expr(expr),
        RustExpr::Index { expr, index } => simplify_expr(expr) + simplify_expr(index),
        RustExpr::Slice { expr, start, stop } => {
            simplify_expr(expr)
                + start
                    .as_mut()
                    .map(|value| simplify_expr(value))
                    .unwrap_or_default()
                + stop
                    .as_mut()
                    .map(|value| simplify_expr(value))
                    .unwrap_or_default()
        }
        RustExpr::Block { stmts, expr } => {
            simplify_block(stmts)
                + expr
                    .as_mut()
                    .map(|value| simplify_expr(value))
                    .unwrap_or_default()
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            simplify_expr(cond)
                + simplify_expr(then_expr)
                + else_expr
                    .as_mut()
                    .map(|value| simplify_expr(value))
                    .unwrap_or_default()
        }
        RustExpr::Match { expr, arms } => {
            let mut count = simplify_expr(expr);
            for arm in arms {
                count += arm.guard.as_mut().map(simplify_expr).unwrap_or_default();
                count += simplify_block(&mut arm.body);
            }
            count
        }
        RustExpr::Closure { body, .. } => simplify_expr(body),
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            simplify_block(body) + normalize_tail_returns(body)
        }
        RustExpr::StructInit { fields, .. } => fields
            .iter_mut()
            .map(|(_, value)| simplify_expr(value))
            .sum(),
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => simplify_expr(duration) + simplify_expr(future) + simplify_expr(error),
    };

    if let RustExpr::If {
        cond,
        then_expr,
        else_expr,
    } = expr
        && let RustExpr::Literal(RustLiteral::Bool(value)) = cond.as_ref()
    {
        let selected = if *value {
            std::mem::replace(then_expr, Box::new(RustExpr::Literal(RustLiteral::Unit)))
        } else {
            else_expr
                .take()
                .unwrap_or_else(|| Box::new(RustExpr::Literal(RustLiteral::Unit)))
        };
        *expr = *selected;
        changed += 1;
    }
    if let RustExpr::FormatMacro {
        name,
        format_str,
        args,
    } = expr
        && name == "format"
        && format_str == "{}"
        && let [value] = args.as_slice()
    {
        *expr = RustExpr::MethodCall {
            receiver: Box::new(value.clone()),
            method: "to_string".to_string(),
            args: Vec::new(),
        };
        changed += 1;
    }
    if let RustExpr::MethodCall {
        receiver,
        method,
        args,
    } = expr
        && method == "to_string"
        && args.is_empty()
        && matches!(receiver.as_ref(), RustExpr::Literal(RustLiteral::Str(value)) if value.is_empty())
    {
        *expr = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "String".to_string(),
                "new".to_string(),
            ])),
            args: Vec::new(),
        };
        changed += 1;
    }
    if let RustExpr::FnCall { func, args } = expr
        && matches!(
            func.as_ref(),
            RustExpr::Path(path) if path.as_slice() == ["String", "with_capacity"]
        )
        && matches!(args.as_slice(), [capacity] if is_zero_literal(capacity))
    {
        *expr = RustExpr::FnCall {
            func: Box::new(RustExpr::Path(vec![
                "String".to_string(),
                "new".to_string(),
            ])),
            args: Vec::new(),
        };
        changed += 1;
    }
    if let RustExpr::BinOp { left, op, right } = expr
        && op == "+"
    {
        if is_zero_literal(right) {
            *expr = *left.clone();
            changed += 1;
        } else if is_zero_literal(left) {
            *expr = *right.clone();
            changed += 1;
        }
    }
    if let RustExpr::MethodCall {
        receiver,
        method,
        args,
    } = expr
        && matches!(method.as_str(), "map" | "map_err")
        && let [RustExpr::Closure { params, body, .. }] = args.as_slice()
        && let [crate::RustParam::Named { name, .. }] = params.as_slice()
    {
        if matches!(body.as_ref(), RustExpr::Ident(body_name) if body_name == name) {
            *expr = *receiver.clone();
            changed += 1;
        } else if let RustExpr::FnCall {
            func,
            args: call_args,
        } = body.as_ref()
            && matches!(call_args.as_slice(), [RustExpr::Ident(arg)] if arg == name)
        {
            *args = vec![func.as_ref().clone()];
            changed += 1;
        }
    }
    changed
}

fn is_zero_literal(expr: &RustExpr) -> bool {
    matches!(expr, RustExpr::Literal(RustLiteral::Int(0)))
        || matches!(expr, RustExpr::Verbatim(value) if value == "0usize")
}

fn normalize_tail_returns(body: &mut [RustStmt]) -> usize {
    let Some(tail) = body.last_mut() else {
        return 0;
    };
    match tail {
        RustStmt::Return(Some(_)) => {
            let RustStmt::Return(Some(value)) = std::mem::replace(tail, RustStmt::Return(None))
            else {
                unreachable!("tail return shape changed during canonicalization")
            };
            *tail = RustStmt::TailExpr(value);
            1
        }
        RustStmt::Return(None) => {
            *tail = RustStmt::TailExpr(RustExpr::Literal(RustLiteral::Unit));
            1
        }
        RustStmt::If {
            then_body,
            else_body: Some(else_body),
            ..
        }
        | RustStmt::IfLet {
            then_body,
            else_body: Some(else_body),
            ..
        } => normalize_tail_returns(then_body) + normalize_tail_returns(else_body),
        RustStmt::Match { arms, .. } => arms
            .iter_mut()
            .map(|arm| normalize_tail_returns(&mut arm.body))
            .sum(),
        RustStmt::Block(block) => normalize_tail_returns(block),
        _ => 0,
    }
}

fn stmt_always_exits(stmt: &RustStmt) -> bool {
    match stmt {
        RustStmt::Return(_) | RustStmt::Break | RustStmt::Continue => true,
        RustStmt::If {
            then_body,
            else_body: Some(else_body),
            ..
        }
        | RustStmt::IfLet {
            then_body,
            else_body: Some(else_body),
            ..
        } => block_always_exits(then_body) && block_always_exits(else_body),
        RustStmt::Match { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| block_always_exits(&arm.body))
        }
        RustStmt::Block(body) => block_always_exits(body),
        RustStmt::Expr(RustExpr::Match { arms, .. }) => arms.is_empty(),
        _ => false,
    }
}

fn block_always_exits(body: &[RustStmt]) -> bool {
    body.last().is_some_and(stmt_always_exits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustType, Visibility};

    #[test]
    fn removes_constant_branches_and_unreachable_tails() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: Some(RustType::I64),
            body: vec![
                RustStmt::If {
                    cond: RustExpr::Literal(RustLiteral::Bool(true)),
                    then_body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(
                        1,
                    ))))],
                    else_body: Some(vec![RustStmt::Return(Some(RustExpr::Literal(
                        RustLiteral::Int(2),
                    )))]),
                },
                RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(3)))),
            ],
            is_async: false,
        }];

        assert_eq!(simplify_control_flow_in_items(&mut items), 3);
        let RustItem::Fn { body, .. } = &items[0] else {
            unreachable!();
        };
        assert!(matches!(body.as_slice(), [RustStmt::Block(branch)] if branch.len() == 1));
    }

    #[test]
    fn canonicalizes_equality_assertions_and_identifier_format_captures() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: None,
            body: vec![RustStmt::Assert {
                cond: RustExpr::Paren(Box::new(RustExpr::BinOp {
                    left: Box::new(RustExpr::FormatMacro {
                        name: "format".to_string(),
                        format_str: "{}".to_string(),
                        args: vec![RustExpr::Ident("value".to_string())],
                    }),
                    op: "==".to_string(),
                    right: Box::new(RustExpr::Literal(RustLiteral::Str("1".to_string()))),
                })),
                msg: None,
            }],
            is_async: false,
        }];

        assert_eq!(simplify_control_flow_in_items(&mut items), 2);
        let RustItem::Fn { body, .. } = &items[0] else {
            unreachable!();
        };
        let [RustStmt::Expr(RustExpr::MacroCall { name, args })] = body.as_slice() else {
            panic!("expected canonical assertion macro, got {body:?}");
        };
        assert_eq!(name, "assert_eq");
        assert!(matches!(
            args.first(),
            Some(RustExpr::MethodCall { receiver, method, args })
                if matches!(receiver.as_ref(), RustExpr::Ident(name) if name == "value")
                    && method == "to_string"
                    && args.is_empty()
        ));
    }

    #[test]
    fn canonicalizes_empty_optional_wildcard_scaffolds_as_discarded_values() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: None,
            body: vec![RustStmt::IfLet {
                pattern: "Some(_)".to_string(),
                expr: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("values".to_string())),
                    method: "pop".to_string(),
                    args: Vec::new(),
                },
                then_body: Vec::new(),
                else_body: None,
            }],
            is_async: false,
        }];

        assert_eq!(simplify_control_flow_in_items(&mut items), 1);
        let RustItem::Fn { body, .. } = &items[0] else {
            unreachable!();
        };
        assert!(matches!(
            body.as_slice(),
            [RustStmt::Let {
                name,
                value: RustExpr::MethodCall { method, .. },
                ..
            }] if name == "_" && method == "pop"
        ));
    }

    #[test]
    fn canonicalizes_returns_only_along_tail_position_paths() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: Some(RustType::I64),
            body: vec![RustStmt::If {
                cond: RustExpr::Ident("condition".to_string()),
                then_body: vec![RustStmt::Return(Some(RustExpr::Literal(RustLiteral::Int(
                    1,
                ))))],
                else_body: Some(vec![RustStmt::Match {
                    expr: RustExpr::Ident("choice".to_string()),
                    arms: vec![
                        crate::RustMatchArm {
                            pattern: "0".to_string(),
                            bindings: Vec::new(),
                            guard: None,
                            body: vec![RustStmt::Return(Some(RustExpr::Literal(
                                RustLiteral::Int(2),
                            )))],
                        },
                        crate::RustMatchArm {
                            pattern: "_".to_string(),
                            bindings: Vec::new(),
                            guard: None,
                            body: vec![RustStmt::Return(Some(RustExpr::Literal(
                                RustLiteral::Int(3),
                            )))],
                        },
                    ],
                }]),
            }],
            is_async: false,
        }];

        assert_eq!(simplify_control_flow_in_items(&mut items), 3);
        let RustItem::Fn { body, .. } = &items[0] else {
            unreachable!();
        };
        let [
            RustStmt::If {
                then_body,
                else_body: Some(else_body),
                ..
            },
        ] = body.as_slice()
        else {
            panic!("expected tail if, got {body:?}");
        };
        assert!(matches!(then_body.as_slice(), [RustStmt::TailExpr(_)]));
        let [RustStmt::Match { arms, .. }] = else_body.as_slice() else {
            panic!("expected tail match, got {else_body:?}");
        };
        assert!(
            arms.iter()
                .all(|arm| matches!(arm.body.as_slice(), [RustStmt::TailExpr(_)]))
        );
    }

    #[test]
    fn removes_identity_maps_and_forwarding_closures() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: None,
            body: vec![
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("result".to_string())),
                    method: "map".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "value".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::Ident("value".to_string())),
                        is_move: false,
                    }],
                }),
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("result".to_string())),
                    method: "map_err".to_string(),
                    args: vec![RustExpr::Closure {
                        params: vec![crate::RustParam::Named {
                            name: "error".to_string(),
                            ty: RustType::Named("_".to_string()),
                        }],
                        body: Box::new(RustExpr::FnCall {
                            func: Box::new(RustExpr::Path(vec![
                                "Error".to_string(),
                                "from".to_string(),
                            ])),
                            args: vec![RustExpr::Ident("error".to_string())],
                        }),
                        is_move: false,
                    }],
                }),
            ],
            is_async: false,
        }];

        assert_eq!(simplify_control_flow_in_items(&mut items), 2);
        let RustItem::Fn { body, .. } = &items[0] else {
            unreachable!();
        };
        assert!(matches!(
            body.first(),
            Some(RustStmt::Expr(RustExpr::Ident(name))) if name == "result"
        ));
        assert!(matches!(
            body.get(1),
            Some(RustStmt::Expr(RustExpr::MethodCall { args, .. }))
                if matches!(args.as_slice(), [RustExpr::Path(path)] if path == &["Error", "from"])
        ));
    }

    #[test]
    fn removes_scalar_and_formatting_identities() {
        let mut items = vec![RustItem::Fn {
            name: "demo".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: None,
            body: vec![
                RustStmt::Expr(RustExpr::BinOp {
                    left: Box::new(RustExpr::Paren(Box::new(RustExpr::Ident(
                        "length".to_string(),
                    )))),
                    op: "+".to_string(),
                    right: Box::new(RustExpr::Verbatim("0usize".to_string())),
                }),
                RustStmt::Expr(RustExpr::FormatMacro {
                    name: "format".to_string(),
                    format_str: "{}".to_string(),
                    args: vec![RustExpr::Ident("value".to_string())],
                }),
                RustStmt::Expr(RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Literal(RustLiteral::Str(String::new()))),
                    method: "to_string".to_string(),
                    args: Vec::new(),
                }),
            ],
            is_async: false,
        }];

        assert_eq!(simplify_control_flow_in_items(&mut items), 3);
        let RustItem::Fn { body, .. } = &items[0] else {
            unreachable!();
        };
        assert!(matches!(
            body.first(),
            Some(RustStmt::Expr(RustExpr::Paren(inner)))
                if matches!(inner.as_ref(), RustExpr::Ident(name) if name == "length")
        ));
        assert!(matches!(
            body.get(1),
            Some(RustStmt::Expr(RustExpr::MethodCall { method, .. })) if method == "to_string"
        ));
        assert!(matches!(
            body.get(2),
            Some(RustStmt::Expr(RustExpr::FnCall { func, args }))
                if args.is_empty()
                    && matches!(func.as_ref(), RustExpr::Path(path) if path == &["String", "new"])
        ));
    }

    #[test]
    fn canonicalizes_zero_capacity_strings() {
        let mut items = vec![RustItem::Const {
            name: "VALUE".to_string(),
            ty: RustType::String_,
            value: RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "String".to_string(),
                    "with_capacity".to_string(),
                ])),
                args: vec![RustExpr::Verbatim("0usize".to_string())],
            },
            visibility: Visibility::Private,
        }];

        assert_eq!(simplify_control_flow_in_items(&mut items), 1);
        let RustItem::Const { value, .. } = &items[0] else {
            unreachable!();
        };
        assert!(matches!(
            value,
            RustExpr::FnCall { func, args }
                if args.is_empty()
                    && matches!(func.as_ref(), RustExpr::Path(path) if path == &["String", "new"])
        ));
    }
}
