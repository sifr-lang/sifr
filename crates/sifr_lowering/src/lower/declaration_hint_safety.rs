use super::nested_function_inference::collect_current_function_local_bindings;
use sifr_python_ast::{ExceptHandler, Expr, Stmt};
use std::collections::{HashMap, HashSet};

pub(in crate::lower) fn safe_direct_assignment_names(
    stmts: &[Stmt],
    accepts_value: impl Fn(&Expr) -> bool,
) -> HashSet<String> {
    let mut candidate_names = HashSet::new();
    let mut direct_binding_counts = HashMap::<String, usize>::new();

    for stmt in stmts {
        collect_direct_binding_counts(stmt, &mut direct_binding_counts);
        if let Stmt::Assign(assign) = stmt {
            if assign.targets.len() == 1 && accepts_value(&assign.value) {
                if let Expr::Name(name) = &assign.targets[0] {
                    candidate_names.insert(name.id.to_string());
                }
            }
        }
    }

    candidate_names.retain(|name| {
        direct_binding_counts.get(name) == Some(&1)
            && !nested_block_binds_name(stmts, name.as_str())
    });
    candidate_names
}

fn collect_direct_binding_counts(stmt: &Stmt, counts: &mut HashMap<String, usize>) {
    // This census only needs targets that can lower as a new local `Let`. Imports,
    // classes, exception aliases, and match captures predefine the name, so a later
    // candidate assignment is a rebinding and cannot reach declaration adoption.
    match stmt {
        Stmt::Assign(assign) => {
            for target in &assign.targets {
                count_target_names(target, counts);
            }
        }
        Stmt::AnnAssign(assign) => count_target_names(&assign.target, counts),
        Stmt::AugAssign(assign) => count_target_names(&assign.target, counts),
        Stmt::For(for_stmt) => count_target_names(&for_stmt.target, counts),
        Stmt::With(with_stmt) => {
            for item in &with_stmt.items {
                if let Some(optional_vars) = &item.optional_vars {
                    count_target_names(optional_vars, counts);
                }
            }
        }
        Stmt::FunctionDef(function) => {
            *counts.entry(function.name.to_string()).or_default() += 1;
        }
        _ => {}
    }
}

fn count_target_names(target: &Expr, counts: &mut HashMap<String, usize>) {
    match target {
        Expr::Name(name) => {
            *counts.entry(name.id.to_string()).or_default() += 1;
        }
        Expr::Tuple(tuple) => {
            for element in &tuple.elts {
                count_target_names(element, counts);
            }
        }
        _ => {}
    }
}

fn nested_block_binds_name(stmts: &[Stmt], name: &str) -> bool {
    stmts.iter().any(|stmt| match stmt {
        Stmt::If(if_stmt) => {
            block_binds_name(&if_stmt.body, name)
                || if_stmt
                    .elif_else_clauses
                    .iter()
                    .any(|clause| block_binds_name(&clause.body, name))
        }
        Stmt::While(while_stmt) => {
            block_binds_name(&while_stmt.body, name) || block_binds_name(&while_stmt.orelse, name)
        }
        Stmt::For(for_stmt) => {
            block_binds_name(&for_stmt.body, name) || block_binds_name(&for_stmt.orelse, name)
        }
        Stmt::With(with_stmt) => block_binds_name(&with_stmt.body, name),
        Stmt::Try(try_stmt) => {
            block_binds_name(&try_stmt.body, name)
                || block_binds_name(&try_stmt.orelse, name)
                || block_binds_name(&try_stmt.finalbody, name)
                || try_stmt.handlers.iter().any(|handler| {
                    let ExceptHandler::ExceptHandler(handler) = handler;
                    block_binds_name(&handler.body, name)
                })
        }
        Stmt::Match(match_stmt) => match_stmt
            .cases
            .iter()
            .any(|case| block_binds_name(&case.body, name)),
        Stmt::FunctionDef(_) | Stmt::ClassDef(_) => false,
        _ => false,
    })
}

fn block_binds_name(stmts: &[Stmt], name: &str) -> bool {
    let mut bindings = HashSet::new();
    collect_current_function_local_bindings(stmts, &mut bindings);
    bindings.contains(name)
}
