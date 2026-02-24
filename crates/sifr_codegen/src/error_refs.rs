use sifr_hir::{HirExpr, HirFStringPart, HirModule, HirStmt};
use std::collections::HashSet;

pub(crate) fn collect_referenced_builtin_error_classes(
    module: &HirModule,
    stdlib_preamble: &str,
    intrinsic_functions: &HashSet<String>,
    needs_file_handles: bool,
    builtin_error_classes: &[&str],
) -> HashSet<String> {
    let mut referenced = HashSet::new();

    for func in &module.functions {
        collect_stmt_error_refs(&func.body, &mut referenced, builtin_error_classes);
    }
    for class in &module.classes {
        for method in &class.methods {
            collect_stmt_error_refs(&method.body, &mut referenced, builtin_error_classes);
        }
    }
    for (_, _, value) in &module.constants {
        collect_expr_error_refs(value, &mut referenced, builtin_error_classes);
    }

    collect_text_error_refs(stdlib_preamble, &mut referenced, builtin_error_classes);

    if needs_file_handles {
        referenced.insert("IOError".to_string());
    }

    // Intrinsics can produce these builtin errors through generated helper code.
    if !intrinsic_functions.is_empty() {
        for error_name in [
            "IOError",
            "ParseError",
            "ValueError",
            "JSONDecodeError",
            "TOMLDecodeError",
            "RegexError",
        ] {
            referenced.insert(error_name.to_string());
        }
    }

    referenced
}

fn collect_stmt_error_refs(
    stmts: &[HirStmt],
    referenced: &mut HashSet<String>,
    builtin_error_classes: &[&str],
) {
    for stmt in stmts {
        match stmt {
            HirStmt::Let { value, .. }
            | HirStmt::Assign { value, .. }
            | HirStmt::AugAssign { value, .. }
            | HirStmt::AttributeAugAssign { value, .. }
            | HirStmt::FieldAssign { value, .. }
            | HirStmt::Raise { value }
            | HirStmt::Yield { value } => {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
            HirStmt::Return { value } => {
                if let Some(value) = value {
                    collect_expr_error_refs(value, referenced, builtin_error_classes);
                }
            }
            HirStmt::Expr { expr } => {
                collect_expr_error_refs(expr, referenced, builtin_error_classes);
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                collect_expr_error_refs(condition, referenced, builtin_error_classes);
                collect_stmt_error_refs(then_body, referenced, builtin_error_classes);
                for (elif_cond, elif_body) in elif_clauses {
                    collect_expr_error_refs(elif_cond, referenced, builtin_error_classes);
                    collect_stmt_error_refs(elif_body, referenced, builtin_error_classes);
                }
                if let Some(else_body) = else_body {
                    collect_stmt_error_refs(else_body, referenced, builtin_error_classes);
                }
            }
            HirStmt::While {
                condition,
                body,
                else_body,
            } => {
                collect_expr_error_refs(condition, referenced, builtin_error_classes);
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                if let Some(else_body) = else_body {
                    collect_stmt_error_refs(else_body, referenced, builtin_error_classes);
                }
            }
            HirStmt::For {
                iter,
                body,
                else_body,
                ..
            } => {
                collect_expr_error_refs(iter, referenced, builtin_error_classes);
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                if let Some(else_body) = else_body {
                    collect_stmt_error_refs(else_body, referenced, builtin_error_classes);
                }
            }
            HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
            HirStmt::Assert { test, msg } => {
                collect_expr_error_refs(test, referenced, builtin_error_classes);
                if let Some(msg) = msg {
                    collect_expr_error_refs(msg, referenced, builtin_error_classes);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
                for handler in handlers {
                    if let Some(error_type) = &handler.error_type {
                        if builtin_error_classes.contains(&error_type.as_str()) {
                            referenced.insert(error_type.clone());
                        }
                    }
                    collect_stmt_error_refs(&handler.body, referenced, builtin_error_classes);
                }
            }
            HirStmt::SubscriptAssign { index, value, .. }
            | HirStmt::SubscriptAugAssign { index, value, .. }
            | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
                collect_expr_error_refs(index, referenced, builtin_error_classes);
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
            HirStmt::NestedSubscriptAssign {
                outer_index,
                inner_index,
                value,
                ..
            } => {
                collect_expr_error_refs(outer_index, referenced, builtin_error_classes);
                collect_expr_error_refs(inner_index, referenced, builtin_error_classes);
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
            HirStmt::Delete { object, index } => {
                collect_expr_error_refs(object, referenced, builtin_error_classes);
                collect_expr_error_refs(index, referenced, builtin_error_classes);
            }
            HirStmt::With { items, body } => {
                for (_, expr, _) in items {
                    collect_expr_error_refs(expr, referenced, builtin_error_classes);
                }
                collect_stmt_error_refs(body, referenced, builtin_error_classes);
            }
            HirStmt::NestedFunction { func } => {
                collect_stmt_error_refs(&func.body, referenced, builtin_error_classes);
            }
            HirStmt::Match { subject, arms, .. } => {
                collect_expr_error_refs(subject, referenced, builtin_error_classes);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        collect_expr_error_refs(guard, referenced, builtin_error_classes);
                    }
                    collect_stmt_error_refs(&arm.body, referenced, builtin_error_classes);
                }
            }
            HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
        }
    }
}

fn collect_expr_error_refs(
    expr: &HirExpr,
    referenced: &mut HashSet<String>,
    builtin_error_classes: &[&str],
) {
    match expr {
        HirExpr::Call { func, args, .. } => {
            if builtin_error_classes.contains(&func.as_str()) {
                referenced.insert(func.clone());
            }
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::ConstructorCall {
            class_name, args, ..
        } => {
            if builtin_error_classes.contains(&class_name.as_str()) {
                referenced.insert(class_name.clone());
            }
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_expr_error_refs(left, referenced, builtin_error_classes);
            collect_expr_error_refs(right, referenced, builtin_error_classes);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::WalrusExpr {
            value: operand, ..
        }
        | HirExpr::FieldAccess { object: operand, .. } => {
            collect_expr_error_refs(operand, referenced, builtin_error_classes);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_expr_error_refs(left, referenced, builtin_error_classes);
            for comparator in comparators {
                collect_expr_error_refs(comparator, referenced, builtin_error_classes);
            }
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::ListLiteral { elements: values, .. }
        | HirExpr::SetLiteral { elements: values, .. }
        | HirExpr::TupleLiteral { elements: values, .. } => {
            for value in values {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_expr_error_refs(condition, referenced, builtin_error_classes);
            collect_expr_error_refs(then_expr, referenced, builtin_error_classes);
            collect_expr_error_refs(else_expr, referenced, builtin_error_classes);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            collect_expr_error_refs(start, referenced, builtin_error_classes);
            collect_expr_error_refs(end, referenced, builtin_error_classes);
            if let Some(step) = step {
                collect_expr_error_refs(step, referenced, builtin_error_classes);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for key in keys {
                collect_expr_error_refs(key, referenced, builtin_error_classes);
            }
            for value in values {
                collect_expr_error_refs(value, referenced, builtin_error_classes);
            }
        }
        HirExpr::Index { object, index, .. } | HirExpr::ContainsOp {
            element: index,
            collection: object,
            ..
        } => {
            collect_expr_error_refs(object, referenced, builtin_error_classes);
            collect_expr_error_refs(index, referenced, builtin_error_classes);
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_expr_error_refs(object, referenced, builtin_error_classes);
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    collect_expr_error_refs(expr, referenced, builtin_error_classes);
                }
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            collect_expr_error_refs(object, referenced, builtin_error_classes);
            if let Some(start) = start {
                collect_expr_error_refs(start, referenced, builtin_error_classes);
            }
            if let Some(stop) = stop {
                collect_expr_error_refs(stop, referenced, builtin_error_classes);
            }
            if let Some(step) = step {
                collect_expr_error_refs(step, referenced, builtin_error_classes);
            }
        }
        HirExpr::SuperCall { args, .. } => {
            for arg in args {
                collect_expr_error_refs(arg, referenced, builtin_error_classes);
            }
        }
        HirExpr::Lambda { body, .. } => {
            collect_expr_error_refs(body, referenced, builtin_error_classes);
        }
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            collect_expr_error_refs(expr, referenced, builtin_error_classes);
            for (_, iter_expr, filter) in generators {
                collect_expr_error_refs(iter_expr, referenced, builtin_error_classes);
                if let Some(filter) = filter {
                    collect_expr_error_refs(filter, referenced, builtin_error_classes);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            collect_expr_error_refs(key_expr, referenced, builtin_error_classes);
            collect_expr_error_refs(val_expr, referenced, builtin_error_classes);
            for (_, iter_expr, filter) in generators {
                collect_expr_error_refs(iter_expr, referenced, builtin_error_classes);
                if let Some(filter) = filter {
                    collect_expr_error_refs(filter, referenced, builtin_error_classes);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            collect_expr_error_refs(expr, referenced, builtin_error_classes);
            collect_expr_error_refs(iter, referenced, builtin_error_classes);
            if let Some(filter) = filter {
                collect_expr_error_refs(filter, referenced, builtin_error_classes);
            }
        }
        HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::Name { .. }
        | HirExpr::EnumVariant { .. } => {}
    }
}

fn collect_text_error_refs(
    text: &str,
    referenced: &mut HashSet<String>,
    builtin_error_classes: &[&str],
) {
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            token.push(ch);
            continue;
        }
        if !token.is_empty() {
            if builtin_error_classes.contains(&token.as_str()) {
                referenced.insert(token.clone());
            }
            token.clear();
        }
    }
    if !token.is_empty() && builtin_error_classes.contains(&token.as_str()) {
        referenced.insert(token);
    }
}
