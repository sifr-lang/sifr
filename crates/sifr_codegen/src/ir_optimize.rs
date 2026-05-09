use crate::{RustExpr, RustItem, RustLiteral, RustStmt, RustType};

/// Remove conservatively-trivial `.clone()` expressions from IR items.
///
/// This pass is intentionally narrow: it only removes clones on expressions
/// that are always safe to move without changing semantics.
pub(crate) fn remove_trivial_clones_in_items(items: &mut [RustItem]) -> usize {
    let mut removed = 0usize;
    for item in items {
        removed += optimize_item(item);
    }
    removed
}

fn optimize_item(item: &mut RustItem) -> usize {
    match item {
        RustItem::Use(_) | RustItem::UseAlias { .. } | RustItem::Attr(_) => 0,
        RustItem::Struct { .. } | RustItem::TupleStruct { .. } => 0,
        RustItem::Enum { variants, .. } => {
            let mut removed = 0usize;
            for variant in variants {
                if let Some(value) = &mut variant.value {
                    removed += optimize_expr(value);
                }
            }
            removed
        }
        RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
            let mut removed = 0usize;
            for method in methods {
                removed += optimize_item(method);
            }
            removed
        }
        RustItem::Fn { body, .. } => {
            let mut removed = 0usize;
            for stmt in body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
        RustItem::TraitMethodSig { .. } => 0,
        RustItem::TypeAlias { .. } => 0,
        RustItem::Const { value, .. } | RustItem::Static { value, .. } => optimize_expr(value),
    }
}

fn optimize_stmt(stmt: &mut RustStmt) -> usize {
    match stmt {
        RustStmt::Let { value, .. } => optimize_expr(value),
        RustStmt::LetPattern { value, .. } => optimize_expr(value),
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            let mut removed = optimize_expr(value);
            for stmt in else_body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            optimize_expr(target) + optimize_expr(value)
        }
        RustStmt::Expr(expr) | RustStmt::Return(Some(expr)) => optimize_expr(expr),
        RustStmt::Assert { cond, msg } => {
            optimize_expr(cond) + msg.as_mut().map(optimize_expr).unwrap_or(0)
        }
        RustStmt::Return(None) | RustStmt::Break | RustStmt::Continue => 0,
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            let mut removed = optimize_expr(cond);
            for stmt in then_body {
                removed += optimize_stmt(stmt);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    removed += optimize_stmt(stmt);
                }
            }
            removed
        }
        RustStmt::IfLet {
            expr,
            then_body,
            else_body,
            ..
        } => {
            let mut removed = optimize_expr(expr);
            for stmt in then_body {
                removed += optimize_stmt(stmt);
            }
            if let Some(else_body) = else_body {
                for stmt in else_body {
                    removed += optimize_stmt(stmt);
                }
            }
            removed
        }
        RustStmt::Match { expr, arms } => {
            let mut removed = optimize_expr(expr);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += optimize_expr(guard);
                }
                for stmt in &mut arm.body {
                    removed += optimize_stmt(stmt);
                }
            }
            removed
        }
        RustStmt::For { iter, body, .. } => {
            let mut removed = optimize_expr(iter);
            for stmt in body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
        RustStmt::With { items, body } => {
            let mut removed = 0usize;
            for item in items {
                removed += optimize_expr(&mut item.value);
            }
            for stmt in body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
        RustStmt::While { cond, body } => {
            let mut removed = optimize_expr(cond);
            for stmt in body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
        RustStmt::Loop { body } | RustStmt::Block(body) => {
            let mut removed = 0usize;
            for stmt in body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
        RustStmt::LocalFn { body, .. } => {
            let mut removed = 0usize;
            for stmt in body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
    }
}

fn optimize_expr(expr: &mut RustExpr) -> usize {
    match expr {
        RustExpr::Clone(inner) => {
            let mut removed = optimize_expr(inner);
            if should_remove_clone(inner.as_ref()) {
                let replacement =
                    *std::mem::replace(inner, Box::new(RustExpr::Literal(RustLiteral::Unit)));
                *expr = replacement;
                removed += 1;
            }
            removed
        }
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) => 0,
        RustExpr::MethodCall { receiver, args, .. } => {
            let mut removed = optimize_expr(receiver);
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::FnCall { func, args } => {
            let mut removed = optimize_expr(func);
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => {
            let mut removed = 0usize;
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::TimeoutAwait { duration, future } => {
            optimize_expr(duration) + optimize_expr(future)
        }
        RustExpr::FormatMacro { args, .. } => {
            let mut removed = 0usize;
            for arg in args {
                removed += optimize_expr(arg);
            }
            removed
        }
        RustExpr::BinOp { left, right, .. } => optimize_expr(left) + optimize_expr(right),
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Try(operand)
        | RustExpr::Paren(operand)
        | RustExpr::Await(operand) => optimize_expr(operand),
        RustExpr::Field { expr, .. } => optimize_expr(expr),
        RustExpr::Index { expr, index } => optimize_expr(expr) + optimize_expr(index),
        RustExpr::Slice { expr, start, stop } => {
            optimize_expr(expr)
                + start.as_mut().map(|s| optimize_expr(s)).unwrap_or(0)
                + stop.as_mut().map(|s| optimize_expr(s)).unwrap_or(0)
        }
        RustExpr::Ref { expr, .. } => optimize_expr(expr),
        RustExpr::Cast { expr, .. } => optimize_expr(expr),
        RustExpr::Block { stmts, expr } => {
            let mut removed = 0usize;
            for stmt in stmts {
                removed += optimize_stmt(stmt);
            }
            if let Some(expr) = expr {
                removed += optimize_expr(expr);
            }
            removed
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            let mut removed = optimize_expr(cond) + optimize_expr(then_expr);
            if let Some(else_expr) = else_expr {
                removed += optimize_expr(else_expr);
            }
            removed
        }
        RustExpr::Match { expr, arms } => {
            let mut removed = optimize_expr(expr);
            for arm in arms {
                if let Some(guard) = &mut arm.guard {
                    removed += optimize_expr(guard);
                }
                for stmt in &mut arm.body {
                    removed += optimize_stmt(stmt);
                }
            }
            removed
        }
        RustExpr::Closure { body, .. } => optimize_expr(body),
        RustExpr::ClosureBlock { body, .. } => {
            let mut removed = 0usize;
            for stmt in body {
                removed += optimize_stmt(stmt);
            }
            removed
        }
        RustExpr::StructInit { fields, .. } => {
            let mut removed = 0usize;
            for (_, value) in fields {
                removed += optimize_expr(value);
            }
            removed
        }
        RustExpr::Range { start, end } => optimize_expr(start) + optimize_expr(end),
    }
}

fn should_remove_clone(inner: &RustExpr) -> bool {
    match inner {
        RustExpr::Literal(_) => true,
        RustExpr::Ref { .. } => true,
        RustExpr::Cast { ty, .. } => is_copy_type(ty),
        _ => false,
    }
}

fn is_copy_type(ty: &RustType) -> bool {
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
    use crate::{RustParam, Visibility};

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
}
