use super::{collect_current_function_local_bindings, collect_nonlocal_names, FunctionEnv};
use sifr_python_ast::{Expr, Stmt, StmtFunctionDef};
use sifr_type_system::Type;
use std::collections::{HashMap, HashSet};

use super::state_collection::LocalFunctionState;

pub(super) fn collect_nested_function_captures(
    stmts: &[Stmt],
    env: &FunctionEnv,
    states: &HashMap<String, LocalFunctionState<'_>>,
) -> HashMap<String, Vec<(String, Type)>> {
    let mut captures = HashMap::new();
    for stmt in stmts {
        let Stmt::FunctionDef(func) = stmt else {
            continue;
        };
        let Some(state) = states.get(func.name.as_str()) else {
            continue;
        };
        let function_captures = collect_function_captures(func, state, env);
        // Keep capture-free nested functions in the map as positive evidence
        // that the callable was inspected. Consumers must distinguish them
        // from callable values whose closure environment is unknown.
        captures.insert(func.name.to_string(), function_captures);
    }
    captures
}

fn collect_function_captures(
    func: &StmtFunctionDef,
    state: &LocalFunctionState<'_>,
    env: &FunctionEnv,
) -> Vec<(String, Type)> {
    let mut references = HashSet::new();
    collect_referenced_names_in_stmts(&func.body, &mut references);

    let mut local_bindings = HashSet::new();
    collect_current_function_local_bindings(&func.body, &mut local_bindings);

    let mut nonlocal_names = HashSet::new();
    collect_nonlocal_names(&func.body, &mut nonlocal_names);

    let param_names = state
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect::<HashSet<_>>();

    let mut captures = references
        .into_iter()
        .filter(|name| !param_names.contains(name.as_str()))
        .filter(|name| !local_bindings.contains(name) || nonlocal_names.contains(name))
        .filter_map(|name| env.vars.get(&name).cloned().map(|ty| (name, ty)))
        .collect::<Vec<_>>();
    captures.sort_by(|left, right| left.0.cmp(&right.0));
    captures
}

fn collect_referenced_names_in_stmts(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Expr(expr_stmt) => collect_referenced_names_in_expr(&expr_stmt.value, names),
            Stmt::Assert(assert_stmt) => {
                collect_referenced_names_in_expr(&assert_stmt.test, names);
                if let Some(msg) = &assert_stmt.msg {
                    collect_referenced_names_in_expr(msg, names);
                }
            }
            Stmt::Return(ret) => {
                if let Some(value) = &ret.value {
                    collect_referenced_names_in_expr(value, names);
                }
            }
            Stmt::Assign(assign) => collect_referenced_names_in_expr(&assign.value, names),
            Stmt::AnnAssign(assign) => {
                if let Some(value) = &assign.value {
                    collect_referenced_names_in_expr(value, names);
                }
            }
            Stmt::AugAssign(aug) => {
                collect_referenced_names_in_expr(&aug.target, names);
                collect_referenced_names_in_expr(&aug.value, names);
            }
            Stmt::If(if_stmt) => {
                collect_referenced_names_in_expr(&if_stmt.test, names);
                collect_referenced_names_in_stmts(&if_stmt.body, names);
                for clause in &if_stmt.elif_else_clauses {
                    if let Some(test) = &clause.test {
                        collect_referenced_names_in_expr(test, names);
                    }
                    collect_referenced_names_in_stmts(&clause.body, names);
                }
            }
            Stmt::For(for_stmt) => {
                collect_referenced_names_in_expr(&for_stmt.iter, names);
                collect_referenced_names_in_stmts(&for_stmt.body, names);
                collect_referenced_names_in_stmts(&for_stmt.orelse, names);
            }
            Stmt::While(while_stmt) => {
                collect_referenced_names_in_expr(&while_stmt.test, names);
                collect_referenced_names_in_stmts(&while_stmt.body, names);
                collect_referenced_names_in_stmts(&while_stmt.orelse, names);
            }
            Stmt::With(with_stmt) => {
                for item in &with_stmt.items {
                    collect_referenced_names_in_expr(&item.context_expr, names);
                }
                collect_referenced_names_in_stmts(&with_stmt.body, names);
            }
            Stmt::Try(try_stmt) => {
                collect_referenced_names_in_stmts(&try_stmt.body, names);
                collect_referenced_names_in_stmts(&try_stmt.orelse, names);
                collect_referenced_names_in_stmts(&try_stmt.finalbody, names);
                for handler in &try_stmt.handlers {
                    let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                    if let Some(type_expr) = &handler.type_ {
                        collect_referenced_names_in_expr(type_expr, names);
                    }
                    collect_referenced_names_in_stmts(&handler.body, names);
                }
            }
            Stmt::FunctionDef(_) | Stmt::Nonlocal(_) => {}
            _ => {}
        }
    }
}

fn collect_referenced_names_in_expr(expr: &Expr, names: &mut HashSet<String>) {
    match expr {
        Expr::Name(name) => {
            names.insert(name.id.to_string());
        }
        Expr::Call(call) => {
            collect_referenced_names_in_expr(call.func.as_ref(), names);
            for arg in &call.arguments.args {
                collect_referenced_names_in_expr(arg, names);
            }
            for keyword in &call.arguments.keywords {
                collect_referenced_names_in_expr(&keyword.value, names);
            }
        }
        Expr::Attribute(attr) => collect_referenced_names_in_expr(attr.value.as_ref(), names),
        Expr::Subscript(sub) => {
            collect_referenced_names_in_expr(sub.value.as_ref(), names);
            collect_referenced_names_in_expr(sub.slice.as_ref(), names);
        }
        Expr::BinOp(bin) => {
            collect_referenced_names_in_expr(bin.left.as_ref(), names);
            collect_referenced_names_in_expr(bin.right.as_ref(), names);
        }
        Expr::BoolOp(bool_op) => {
            for value in &bool_op.values {
                collect_referenced_names_in_expr(value, names);
            }
        }
        Expr::UnaryOp(unary) => collect_referenced_names_in_expr(unary.operand.as_ref(), names),
        Expr::Compare(compare) => {
            collect_referenced_names_in_expr(compare.left.as_ref(), names);
            for comparator in &compare.comparators {
                collect_referenced_names_in_expr(comparator, names);
            }
        }
        Expr::If(if_expr) => {
            collect_referenced_names_in_expr(if_expr.test.as_ref(), names);
            collect_referenced_names_in_expr(if_expr.body.as_ref(), names);
            collect_referenced_names_in_expr(if_expr.orelse.as_ref(), names);
        }
        Expr::List(list) => {
            for element in &list.elts {
                collect_referenced_names_in_expr(element, names);
            }
        }
        Expr::Tuple(tuple) => {
            for element in &tuple.elts {
                collect_referenced_names_in_expr(element, names);
            }
        }
        Expr::Set(set) => {
            for element in &set.elts {
                collect_referenced_names_in_expr(element, names);
            }
        }
        Expr::Dict(dict) => {
            for item in &dict.items {
                if let Some(key) = &item.key {
                    collect_referenced_names_in_expr(key, names);
                }
                collect_referenced_names_in_expr(&item.value, names);
            }
        }
        Expr::ListComp(comp) => {
            collect_comprehension_names(&comp.generators, names, Some(&comp.elt));
        }
        Expr::SetComp(comp) => {
            collect_comprehension_names(&comp.generators, names, Some(&comp.elt));
        }
        Expr::DictComp(comp) => {
            collect_comprehension_names(&comp.generators, names, Some(&comp.key));
            collect_referenced_names_in_expr(comp.value.as_ref(), names);
        }
        Expr::Generator(gen) => collect_comprehension_names(&gen.generators, names, Some(&gen.elt)),
        Expr::Await(await_expr) => {
            collect_referenced_names_in_expr(await_expr.value.as_ref(), names);
        }
        Expr::Yield(yield_expr) => {
            if let Some(value) = &yield_expr.value {
                collect_referenced_names_in_expr(value, names);
            }
        }
        Expr::YieldFrom(yield_from) => {
            collect_referenced_names_in_expr(yield_from.value.as_ref(), names);
        }
        _ => {}
    }
}

fn collect_comprehension_names(
    generators: &[sifr_python_ast::Comprehension],
    names: &mut HashSet<String>,
    element: Option<&Expr>,
) {
    if let Some(element) = element {
        collect_referenced_names_in_expr(element, names);
    }
    for generator in generators {
        collect_referenced_names_in_expr(&generator.iter, names);
        for condition in &generator.ifs {
            collect_referenced_names_in_expr(condition, names);
        }
    }
}
