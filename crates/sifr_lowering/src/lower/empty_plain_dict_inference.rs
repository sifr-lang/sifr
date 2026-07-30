use super::nested_function_inference::collect_current_function_local_bindings;
use super::statements::empty_collection_literal_kind;
use sifr_python_ast::{ExceptHandler, Expr, Stmt};
use std::collections::{HashMap, HashSet};

pub(in crate::lower) fn safe_hint_names_for_block(stmts: &[Stmt]) -> HashSet<String> {
    let mut direct_empty_dict_names = HashSet::new();
    let mut direct_binding_counts = HashMap::<String, usize>::new();

    for stmt in stmts {
        collect_direct_binding_counts(stmt, &mut direct_binding_counts);
        if let Stmt::Assign(assign) = stmt {
            if assign.targets.len() == 1
                && empty_collection_literal_kind(&assign.value) == Some("dict")
            {
                if let Expr::Name(name) = &assign.targets[0] {
                    direct_empty_dict_names.insert(name.id.to_string());
                }
            }
        }
    }

    direct_empty_dict_names.retain(|name| {
        direct_binding_counts.get(name) == Some(&1)
            && !nested_block_binds_name(stmts, name.as_str())
    });
    direct_empty_dict_names
}

pub(in crate::lower) fn safe_defaultdict_hint_names_for_block(stmts: &[Stmt]) -> HashSet<String> {
    let mut direct_defaultdict_names = HashSet::new();
    let mut direct_binding_counts = HashMap::<String, usize>::new();

    for stmt in stmts {
        collect_direct_binding_counts(stmt, &mut direct_binding_counts);
        if let Stmt::Assign(assign) = stmt {
            if assign.targets.len() == 1 && is_unseeded_defaultdict_call(&assign.value) {
                if let Expr::Name(name) = &assign.targets[0] {
                    direct_defaultdict_names.insert(name.id.to_string());
                }
            }
        }
    }

    direct_defaultdict_names.retain(|name| {
        direct_binding_counts.get(name) == Some(&1)
            && !nested_block_binds_name(stmts, name.as_str())
    });
    direct_defaultdict_names
}

fn is_unseeded_defaultdict_call(expr: &Expr) -> bool {
    let Expr::Call(call) = expr else {
        return false;
    };
    if call.arguments.args.len() != 1 || !call.arguments.keywords.is_empty() {
        return false;
    }
    let Expr::Name(function) = call.func.as_ref() else {
        return false;
    };
    let Some(Expr::Name(factory)) = call.arguments.args.first() else {
        return false;
    };
    function.id == "defaultdict" && matches!(factory.id.as_str(), "int" | "list" | "set")
}

fn collect_direct_binding_counts(stmt: &Stmt, counts: &mut HashMap<String, usize>) {
    // This census only needs targets that can lower as a new local `Let`. Imports,
    // classes, exception aliases, and match captures predefine the name, so a later
    // empty-dict assignment is a rebinding and cannot reach declaration adoption.
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
