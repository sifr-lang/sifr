use crate::{RustExpr, RustItem, RustStmt};
use std::collections::HashSet;

pub(crate) fn remove_unread_pure_bindings_in_items(items: &mut [RustItem]) -> usize {
    items.iter_mut().map(remove_from_item).sum()
}

fn remove_from_item(item: &mut RustItem) -> usize {
    match item {
        RustItem::Fn { body, .. } => remove_from_block(body),
        RustItem::Trait { methods, .. } | RustItem::Impl { items: methods, .. } => {
            methods.iter_mut().map(remove_from_item).sum()
        }
        RustItem::Use(_)
        | RustItem::UseAlias { .. }
        | RustItem::Struct { .. }
        | RustItem::TupleStruct { .. }
        | RustItem::Enum { .. }
        | RustItem::TraitMethodSig { .. }
        | RustItem::TypeAlias { .. }
        | RustItem::Const { .. }
        | RustItem::Static { .. }
        | RustItem::Attr(_) => 0,
    }
}

fn remove_from_block(body: &mut Vec<RustStmt>) -> usize {
    let mut removed = 0;
    for stmt in body.iter_mut() {
        removed += remove_from_stmt(stmt);
    }
    let mut referenced_after = HashSet::new();
    collect_block_identifiers(body, &mut referenced_after);
    let mut all_names = referenced_after.clone();
    collect_bound_names(body, &mut all_names);
    referenced_after.clear();
    let mut kept = Vec::with_capacity(body.len());
    for mut stmt in body.drain(..).rev() {
        let removable = matches!(
            &stmt,
            RustStmt::Let { name, value, .. }
                if name.starts_with("__sifr_")
                    && !referenced_after.contains(name)
                    && expr_is_pure(value)
                    && !kept.is_empty()
        );
        if removable {
            removed += 1;
            continue;
        }
        if let RustStmt::Let { name, .. } = &mut stmt
            && !name.starts_with('_')
            && !referenced_after.contains(name)
        {
            let mut replacement = format!("_{name}");
            while all_names.contains(&replacement) {
                replacement.insert(0, '_');
            }
            all_names.insert(replacement.clone());
            *name = replacement;
            removed += 1;
        }
        collect_stmt_identifiers(&stmt, &mut referenced_after);
        kept.push(stmt);
    }
    kept.reverse();
    *body = kept;
    removed
}

fn collect_bound_names(body: &[RustStmt], names: &mut HashSet<String>) {
    for stmt in body {
        match stmt {
            RustStmt::Let { name, .. } | RustStmt::LetDecl { name, .. } => {
                names.insert(name.clone());
            }
            RustStmt::LetPattern { pattern, .. } | RustStmt::LetElse { pattern, .. } => {
                collect_text_identifiers(pattern, names);
            }
            RustStmt::If {
                then_body,
                else_body,
                ..
            }
            | RustStmt::IfLet {
                then_body,
                else_body,
                ..
            } => {
                collect_bound_names(then_body, names);
                if let Some(else_body) = else_body {
                    collect_bound_names(else_body, names);
                }
            }
            RustStmt::Match { arms, .. } => {
                for arm in arms {
                    names.extend(arm.bindings.iter().cloned());
                    collect_bound_names(&arm.body, names);
                }
            }
            RustStmt::For { var, body, .. } => {
                names.insert(var.clone());
                collect_bound_names(body, names);
            }
            RustStmt::With { items, body } => {
                names.extend(items.iter().map(|item| item.binding.clone()));
                collect_bound_names(body, names);
            }
            RustStmt::While { body, .. }
            | RustStmt::Loop { body }
            | RustStmt::Block(body)
            | RustStmt::LocalFn { body, .. } => collect_bound_names(body, names),
            RustStmt::Verbatim(_)
            | RustStmt::Assign { .. }
            | RustStmt::AugAssign { .. }
            | RustStmt::Expr(_)
            | RustStmt::TailExpr(_)
            | RustStmt::Assert { .. }
            | RustStmt::Return(_)
            | RustStmt::Break
            | RustStmt::Continue => {}
        }
    }
}

fn remove_from_stmt(stmt: &mut RustStmt) -> usize {
    match stmt {
        RustStmt::If {
            then_body,
            else_body,
            ..
        }
        | RustStmt::IfLet {
            then_body,
            else_body,
            ..
        } => {
            remove_from_block(then_body)
                + else_body
                    .as_mut()
                    .map(remove_from_block)
                    .unwrap_or_default()
        }
        RustStmt::LetElse { else_body, .. }
        | RustStmt::For {
            body: else_body, ..
        }
        | RustStmt::With {
            body: else_body, ..
        }
        | RustStmt::While {
            body: else_body, ..
        }
        | RustStmt::Loop { body: else_body }
        | RustStmt::Block(else_body)
        | RustStmt::LocalFn {
            body: else_body, ..
        } => remove_from_block(else_body),
        RustStmt::Match { arms, .. } => arms
            .iter_mut()
            .map(|arm| remove_from_block(&mut arm.body))
            .sum(),
        RustStmt::Verbatim(_)
        | RustStmt::Let { .. }
        | RustStmt::LetDecl { .. }
        | RustStmt::LetPattern { .. }
        | RustStmt::Assign { .. }
        | RustStmt::AugAssign { .. }
        | RustStmt::Expr(_)
        | RustStmt::TailExpr(_)
        | RustStmt::Assert { .. }
        | RustStmt::Return(_)
        | RustStmt::Break
        | RustStmt::Continue => 0,
    }
}

fn expr_is_pure(expr: &RustExpr) -> bool {
    match expr {
        RustExpr::Literal(_) | RustExpr::Ident(_) | RustExpr::Path(_) => true,
        RustExpr::Tuple(values) | RustExpr::Array(values) | RustExpr::Vec(values) => {
            values.iter().all(expr_is_pure)
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Ref { expr: operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Cast { expr: operand, .. }
        | RustExpr::Paren(operand) => expr_is_pure(operand),
        RustExpr::BinOp { left, right, .. }
        | RustExpr::Range {
            start: left,
            end: right,
        } => expr_is_pure(left) && expr_is_pure(right),
        RustExpr::Field { expr, .. } => expr_is_pure(expr),
        RustExpr::StructInit { fields, .. } => fields.iter().all(|(_, value)| expr_is_pure(value)),
        RustExpr::FnCall { func, args }
            if matches!(func.as_ref(), RustExpr::Path(path)
                if path.first().is_some_and(|segment| segment == "SifrInt")) =>
        {
            args.iter().all(expr_is_pure)
        }
        RustExpr::Verbatim(_)
        | RustExpr::MethodCall { .. }
        | RustExpr::FnCall { .. }
        | RustExpr::MacroCall { .. }
        | RustExpr::FormatMacro { .. }
        | RustExpr::Index { .. }
        | RustExpr::Slice { .. }
        | RustExpr::Clone(_)
        | RustExpr::Block { .. }
        | RustExpr::If { .. }
        | RustExpr::Match { .. }
        | RustExpr::Closure { .. }
        | RustExpr::ClosureBlock { .. }
        | RustExpr::AsyncBlock { .. }
        | RustExpr::TimeoutAwait { .. }
        | RustExpr::Try(_)
        | RustExpr::Await(_) => false,
    }
}

fn collect_stmt_identifiers(stmt: &RustStmt, names: &mut HashSet<String>) {
    match stmt {
        RustStmt::Verbatim(source) => collect_text_identifiers(source, names),
        RustStmt::Let { value, .. } | RustStmt::LetPattern { value, .. } => {
            collect_expr_identifiers(value, names);
        }
        RustStmt::LetDecl { .. }
        | RustStmt::Return(None)
        | RustStmt::Break
        | RustStmt::Continue => {}
        RustStmt::LetElse {
            value, else_body, ..
        } => {
            collect_expr_identifiers(value, names);
            collect_block_identifiers(else_body, names);
        }
        RustStmt::Assign { target, value } | RustStmt::AugAssign { target, value, .. } => {
            collect_expr_identifiers(target, names);
            collect_expr_identifiers(value, names);
        }
        RustStmt::Expr(expr) | RustStmt::TailExpr(expr) | RustStmt::Return(Some(expr)) => {
            collect_expr_identifiers(expr, names);
        }
        RustStmt::Assert { cond, msg } => {
            collect_expr_identifiers(cond, names);
            if let Some(msg) = msg {
                collect_expr_identifiers(msg, names);
            }
        }
        RustStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            collect_expr_identifiers(cond, names);
            collect_block_identifiers(then_body, names);
            if let Some(else_body) = else_body {
                collect_block_identifiers(else_body, names);
            }
        }
        RustStmt::IfLet {
            pattern,
            expr,
            then_body,
            else_body,
        } => {
            collect_text_identifiers(pattern, names);
            collect_expr_identifiers(expr, names);
            collect_block_identifiers(then_body, names);
            if let Some(else_body) = else_body {
                collect_block_identifiers(else_body, names);
            }
        }
        RustStmt::Match { expr, arms } => {
            collect_expr_identifiers(expr, names);
            for arm in arms {
                collect_text_identifiers(&arm.pattern, names);
                if let Some(guard) = &arm.guard {
                    collect_expr_identifiers(guard, names);
                }
                collect_block_identifiers(&arm.body, names);
            }
        }
        RustStmt::For { iter, body, .. } => {
            collect_expr_identifiers(iter, names);
            collect_block_identifiers(body, names);
        }
        RustStmt::With { items, body } => {
            for item in items {
                collect_expr_identifiers(&item.value, names);
            }
            collect_block_identifiers(body, names);
        }
        RustStmt::While { cond, body } => {
            collect_expr_identifiers(cond, names);
            collect_block_identifiers(body, names);
        }
        RustStmt::Loop { body } | RustStmt::Block(body) | RustStmt::LocalFn { body, .. } => {
            collect_block_identifiers(body, names);
        }
    }
}

fn collect_block_identifiers(body: &[RustStmt], names: &mut HashSet<String>) {
    for stmt in body {
        collect_stmt_identifiers(stmt, names);
    }
}

fn collect_expr_identifiers(expr: &RustExpr, names: &mut HashSet<String>) {
    match expr {
        RustExpr::Ident(name) => {
            names.insert(name.clone());
        }
        RustExpr::Path(path) => {
            if let [name] = path.as_slice() {
                names.insert(name.clone());
            }
        }
        RustExpr::Verbatim(source) => collect_text_identifiers(source, names),
        RustExpr::Literal(_) => {}
        RustExpr::MethodCall { receiver, args, .. } => {
            collect_expr_identifiers(receiver, names);
            collect_exprs(args, names);
        }
        RustExpr::FnCall { func, args } => {
            collect_expr_identifiers(func, names);
            collect_exprs(args, names);
        }
        RustExpr::MacroCall { args, .. }
        | RustExpr::FormatMacro { args, .. }
        | RustExpr::Tuple(args)
        | RustExpr::Array(args)
        | RustExpr::Vec(args) => collect_exprs(args, names),
        RustExpr::BinOp { left, right, .. }
        | RustExpr::Range {
            start: left,
            end: right,
        } => {
            collect_expr_identifiers(left, names);
            collect_expr_identifiers(right, names);
        }
        RustExpr::UnaryOp { operand, .. }
        | RustExpr::Ref { expr: operand, .. }
        | RustExpr::Deref(operand)
        | RustExpr::Clone(operand)
        | RustExpr::Cast { expr: operand, .. }
        | RustExpr::Try(operand)
        | RustExpr::Await(operand)
        | RustExpr::Paren(operand) => collect_expr_identifiers(operand, names),
        RustExpr::Field { expr, .. } => collect_expr_identifiers(expr, names),
        RustExpr::Index { expr, index } => {
            collect_expr_identifiers(expr, names);
            collect_expr_identifiers(index, names);
        }
        RustExpr::Slice { expr, start, stop } => {
            collect_expr_identifiers(expr, names);
            if let Some(start) = start {
                collect_expr_identifiers(start, names);
            }
            if let Some(stop) = stop {
                collect_expr_identifiers(stop, names);
            }
        }
        RustExpr::Block { stmts, expr } => {
            collect_block_identifiers(stmts, names);
            if let Some(expr) = expr {
                collect_expr_identifiers(expr, names);
            }
        }
        RustExpr::If {
            cond,
            then_expr,
            else_expr,
        } => {
            collect_expr_identifiers(cond, names);
            collect_expr_identifiers(then_expr, names);
            if let Some(else_expr) = else_expr {
                collect_expr_identifiers(else_expr, names);
            }
        }
        RustExpr::Match { expr, arms } => {
            collect_expr_identifiers(expr, names);
            for arm in arms {
                collect_text_identifiers(&arm.pattern, names);
                if let Some(guard) = &arm.guard {
                    collect_expr_identifiers(guard, names);
                }
                collect_block_identifiers(&arm.body, names);
            }
        }
        RustExpr::Closure { body, .. } => collect_expr_identifiers(body, names),
        RustExpr::ClosureBlock { body, .. } | RustExpr::AsyncBlock { body, .. } => {
            collect_block_identifiers(body, names);
        }
        RustExpr::StructInit { fields, .. } => {
            for (_, value) in fields {
                collect_expr_identifiers(value, names);
            }
        }
        RustExpr::TimeoutAwait {
            duration,
            future,
            error,
        } => {
            collect_expr_identifiers(duration, names);
            collect_expr_identifiers(future, names);
            collect_expr_identifiers(error, names);
        }
    }
}

fn collect_exprs(exprs: &[RustExpr], names: &mut HashSet<String>) {
    for expr in exprs {
        collect_expr_identifiers(expr, names);
    }
}

fn collect_text_identifiers(source: &str, names: &mut HashSet<String>) {
    names.extend(
        source
            .split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
            .filter(|token| !token.is_empty())
            .map(str::to_string),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RustLiteral, RustType, Visibility};

    #[test]
    fn removes_generated_pure_bindings_and_silences_source_bindings() {
        let mut items = vec![RustItem::Fn {
            name: "main".to_string(),
            visibility: Visibility::Private,
            type_params: Vec::new(),
            params: Vec::new(),
            ret: None,
            body: vec![
                RustStmt::Let {
                    mutable: false,
                    name: "__sifr_dead".to_string(),
                    ty: Some(RustType::I64),
                    value: RustExpr::Literal(RustLiteral::Int(1)),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "source_dead".to_string(),
                    ty: Some(RustType::I64),
                    value: RustExpr::Literal(RustLiteral::Int(1)),
                },
                RustStmt::Let {
                    mutable: false,
                    name: "live".to_string(),
                    ty: Some(RustType::I64),
                    value: RustExpr::Literal(RustLiteral::Int(2)),
                },
                RustStmt::Expr(RustExpr::Ident("live".to_string())),
            ],
            is_async: false,
        }];
        assert_eq!(remove_unread_pure_bindings_in_items(&mut items), 2);
        let RustItem::Fn { body, .. } = &items[0] else {
            unreachable!();
        };
        assert_eq!(body.len(), 3);
        assert!(matches!(
            &body[0],
            RustStmt::Let { name, .. } if name == "_source_dead"
        ));
    }
}
