use sifr_python_ast::{ExceptHandler, Expr, Stmt};
use std::collections::HashSet;

pub(super) fn collect_mutated_binding_names(
    stmts: &[Stmt],
    candidate_names: &HashSet<String>,
) -> HashSet<String> {
    let mut mutated = HashSet::new();
    for stmt in stmts {
        collect_mutated_binding_names_in_stmt(stmt, candidate_names, &mut mutated);
    }
    mutated
}

fn collect_mutated_binding_names_in_stmt(
    stmt: &Stmt,
    candidate_names: &HashSet<String>,
    mutated: &mut HashSet<String>,
) {
    match stmt {
        Stmt::Assign(assign) => {
            for target in &assign.targets {
                collect_mutated_binding_names_in_target(target, candidate_names, mutated);
            }
            collect_mutated_binding_names_in_expr(&assign.value, candidate_names, mutated);
        }
        Stmt::AnnAssign(assign) => {
            collect_mutated_binding_names_in_target(
                assign.target.as_ref(),
                candidate_names,
                mutated,
            );
            if let Some(value) = &assign.value {
                collect_mutated_binding_names_in_expr(value, candidate_names, mutated);
            }
        }
        Stmt::AugAssign(assign) => {
            collect_mutated_binding_names_in_target(
                assign.target.as_ref(),
                candidate_names,
                mutated,
            );
            collect_mutated_binding_names_in_expr(&assign.value, candidate_names, mutated);
        }
        Stmt::Expr(expr_stmt) => {
            collect_mutated_binding_names_in_expr(&expr_stmt.value, candidate_names, mutated);
        }
        Stmt::Assert(assert_stmt) => {
            collect_mutated_binding_names_in_expr(&assert_stmt.test, candidate_names, mutated);
            if let Some(message) = &assert_stmt.msg {
                collect_mutated_binding_names_in_expr(message, candidate_names, mutated);
            }
        }
        Stmt::Return(ret) => {
            if let Some(value) = &ret.value {
                collect_mutated_binding_names_in_expr(value, candidate_names, mutated);
            }
        }
        Stmt::Raise(raise_stmt) => {
            if let Some(exc) = &raise_stmt.exc {
                collect_mutated_binding_names_in_expr(exc, candidate_names, mutated);
            }
            if let Some(cause) = &raise_stmt.cause {
                collect_mutated_binding_names_in_expr(cause, candidate_names, mutated);
            }
        }
        Stmt::Delete(delete_stmt) => {
            for target in &delete_stmt.targets {
                collect_mutated_binding_names_in_target(target, candidate_names, mutated);
            }
        }
        Stmt::If(if_stmt) => {
            collect_mutated_binding_names_in_expr(&if_stmt.test, candidate_names, mutated);
            collect_mutated_binding_names_into(&if_stmt.body, candidate_names, mutated);
            for clause in &if_stmt.elif_else_clauses {
                if let Some(test) = &clause.test {
                    collect_mutated_binding_names_in_expr(test, candidate_names, mutated);
                }
                collect_mutated_binding_names_into(&clause.body, candidate_names, mutated);
            }
        }
        Stmt::While(while_stmt) => {
            collect_mutated_binding_names_in_expr(&while_stmt.test, candidate_names, mutated);
            collect_mutated_binding_names_into(&while_stmt.body, candidate_names, mutated);
            collect_mutated_binding_names_into(&while_stmt.orelse, candidate_names, mutated);
        }
        Stmt::For(for_stmt) => {
            collect_mutated_binding_names_in_target(
                for_stmt.target.as_ref(),
                candidate_names,
                mutated,
            );
            collect_mutated_binding_names_in_expr(&for_stmt.iter, candidate_names, mutated);
            collect_mutated_binding_names_into(&for_stmt.body, candidate_names, mutated);
            collect_mutated_binding_names_into(&for_stmt.orelse, candidate_names, mutated);
        }
        Stmt::With(with_stmt) => {
            for item in &with_stmt.items {
                collect_mutated_binding_names_in_expr(&item.context_expr, candidate_names, mutated);
                if let Some(target) = &item.optional_vars {
                    collect_mutated_binding_names_in_target(
                        target.as_ref(),
                        candidate_names,
                        mutated,
                    );
                }
            }
            collect_mutated_binding_names_into(&with_stmt.body, candidate_names, mutated);
        }
        Stmt::Try(try_stmt) => {
            collect_mutated_binding_names_into(&try_stmt.body, candidate_names, mutated);
            collect_mutated_binding_names_into(&try_stmt.orelse, candidate_names, mutated);
            collect_mutated_binding_names_into(&try_stmt.finalbody, candidate_names, mutated);
            for handler in &try_stmt.handlers {
                let ExceptHandler::ExceptHandler(handler) = handler;
                if let Some(type_expr) = &handler.type_ {
                    collect_mutated_binding_names_in_expr(type_expr, candidate_names, mutated);
                }
                collect_mutated_binding_names_into(&handler.body, candidate_names, mutated);
            }
        }
        Stmt::Match(match_stmt) => {
            collect_mutated_binding_names_in_expr(&match_stmt.subject, candidate_names, mutated);
            for case in &match_stmt.cases {
                if let Some(guard) = &case.guard {
                    collect_mutated_binding_names_in_expr(guard, candidate_names, mutated);
                }
                collect_mutated_binding_names_into(&case.body, candidate_names, mutated);
            }
        }
        Stmt::FunctionDef(_) | Stmt::ClassDef(_) => {}
        _ => {}
    }
}

fn collect_mutated_binding_names_into(
    stmts: &[Stmt],
    candidate_names: &HashSet<String>,
    mutated: &mut HashSet<String>,
) {
    for stmt in stmts {
        collect_mutated_binding_names_in_stmt(stmt, candidate_names, mutated);
    }
}

fn collect_mutated_binding_names_in_target(
    expr: &Expr,
    candidate_names: &HashSet<String>,
    mutated: &mut HashSet<String>,
) {
    match expr {
        Expr::Name(name) => {
            if candidate_names.contains(name.id.as_str()) {
                mutated.insert(name.id.to_string());
            }
        }
        Expr::Attribute(_) | Expr::Subscript(_) => {
            if let Some(name) = mutation_root_name(expr) {
                if candidate_names.contains(name) {
                    mutated.insert(name.to_string());
                }
            }
            if let Expr::Subscript(subscript) = expr {
                collect_mutated_binding_names_in_expr(&subscript.slice, candidate_names, mutated);
            }
        }
        Expr::Tuple(tuple) => {
            for element in &tuple.elts {
                collect_mutated_binding_names_in_target(element, candidate_names, mutated);
            }
        }
        _ => {}
    }
}

fn mutation_root_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Name(name) => Some(name.id.as_str()),
        Expr::Attribute(attribute) => mutation_root_name(attribute.value.as_ref()),
        Expr::Subscript(subscript) => mutation_root_name(subscript.value.as_ref()),
        _ => None,
    }
}

fn collect_mutated_binding_names_in_expr(
    expr: &Expr,
    candidate_names: &HashSet<String>,
    mutated: &mut HashSet<String>,
) {
    match expr {
        Expr::Call(call) => {
            if let Expr::Attribute(attribute) = call.func.as_ref() {
                if matches!(
                    attribute.attr.as_str(),
                    "write"
                        | "append"
                        | "appendleft"
                        | "clear"
                        | "extend"
                        | "insert"
                        | "pop"
                        | "popleft"
                        | "remove"
                        | "reverse"
                        | "sort"
                        | "update"
                        | "setdefault"
                        | "add"
                        | "discard"
                        | "intersection_update"
                        | "difference_update"
                        | "symmetric_difference_update"
                ) {
                    if let Some(name) = mutation_root_name(attribute.value.as_ref()) {
                        if candidate_names.contains(name) {
                            mutated.insert(name.to_string());
                        }
                    }
                }
            }
            collect_mutated_binding_names_in_expr(call.func.as_ref(), candidate_names, mutated);
            for argument in &call.arguments.args {
                collect_mutated_binding_names_in_expr(argument, candidate_names, mutated);
            }
            for keyword in &call.arguments.keywords {
                collect_mutated_binding_names_in_expr(&keyword.value, candidate_names, mutated);
            }
        }
        Expr::Attribute(attribute) => {
            collect_mutated_binding_names_in_expr(
                attribute.value.as_ref(),
                candidate_names,
                mutated,
            );
        }
        Expr::Subscript(subscript) => {
            collect_mutated_binding_names_in_expr(
                subscript.value.as_ref(),
                candidate_names,
                mutated,
            );
            collect_mutated_binding_names_in_expr(
                subscript.slice.as_ref(),
                candidate_names,
                mutated,
            );
        }
        Expr::BinOp(binary) => {
            collect_mutated_binding_names_in_expr(binary.left.as_ref(), candidate_names, mutated);
            collect_mutated_binding_names_in_expr(binary.right.as_ref(), candidate_names, mutated);
        }
        Expr::BoolOp(boolean) => {
            for value in &boolean.values {
                collect_mutated_binding_names_in_expr(value, candidate_names, mutated);
            }
        }
        Expr::UnaryOp(unary) => {
            collect_mutated_binding_names_in_expr(unary.operand.as_ref(), candidate_names, mutated);
        }
        Expr::Compare(compare) => {
            collect_mutated_binding_names_in_expr(compare.left.as_ref(), candidate_names, mutated);
            for comparator in &compare.comparators {
                collect_mutated_binding_names_in_expr(comparator, candidate_names, mutated);
            }
        }
        Expr::If(if_expr) => {
            collect_mutated_binding_names_in_expr(if_expr.test.as_ref(), candidate_names, mutated);
            collect_mutated_binding_names_in_expr(if_expr.body.as_ref(), candidate_names, mutated);
            collect_mutated_binding_names_in_expr(
                if_expr.orelse.as_ref(),
                candidate_names,
                mutated,
            );
        }
        Expr::List(list) => {
            for element in &list.elts {
                collect_mutated_binding_names_in_expr(element, candidate_names, mutated);
            }
        }
        Expr::Tuple(tuple) => {
            for element in &tuple.elts {
                collect_mutated_binding_names_in_expr(element, candidate_names, mutated);
            }
        }
        Expr::Set(set) => {
            for element in &set.elts {
                collect_mutated_binding_names_in_expr(element, candidate_names, mutated);
            }
        }
        Expr::Dict(dict) => {
            for item in &dict.items {
                if let Some(key) = &item.key {
                    collect_mutated_binding_names_in_expr(key, candidate_names, mutated);
                }
                collect_mutated_binding_names_in_expr(&item.value, candidate_names, mutated);
            }
        }
        Expr::Await(await_expr) => {
            collect_mutated_binding_names_in_expr(
                await_expr.value.as_ref(),
                candidate_names,
                mutated,
            );
        }
        Expr::Yield(yield_expr) => {
            if let Some(value) = &yield_expr.value {
                collect_mutated_binding_names_in_expr(value, candidate_names, mutated);
            }
        }
        Expr::YieldFrom(yield_from) => {
            collect_mutated_binding_names_in_expr(
                yield_from.value.as_ref(),
                candidate_names,
                mutated,
            );
        }
        _ => {}
    }
}
