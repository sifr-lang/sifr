use super::{
    FunctionEnv, collect_assignment_target_names, collect_current_function_local_bindings,
    collect_nonlocal_names,
};
use sifr_python_ast::{Expr, InterpolatedStringElement, Parameters, Stmt, StmtFunctionDef};
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
        if !states.contains_key(func.name.as_str()) {
            continue;
        }
        let function_captures = collect_function_captures(func, env, states);
        // Keep capture-free nested functions in the map as positive evidence
        // that the callable was inspected. Consumers must distinguish them
        // from callable values whose closure environment is unknown.
        captures.insert(func.name.to_string(), function_captures);
    }
    captures
}

fn collect_function_captures(
    func: &StmtFunctionDef,
    env: &FunctionEnv,
    states: &HashMap<String, LocalFunctionState<'_>>,
) -> Vec<(String, Type)> {
    let mut references = HashSet::new();
    collect_referenced_names_in_stmts(&func.body, &mut references);

    let mut local_bindings = HashSet::new();
    collect_current_function_local_bindings(&func.body, &mut local_bindings);

    let mut nonlocal_names = HashSet::new();
    collect_nonlocal_names(&func.body, &mut nonlocal_names);

    let param_names = parameter_names(&func.parameters);

    let mut captures = references
        .into_iter()
        .filter(|name| !param_names.contains(name))
        .filter(|name| !local_bindings.contains(name) || nonlocal_names.contains(name))
        .filter_map(|name| {
            env.vars
                .get(&name)
                .cloned()
                .or_else(|| {
                    states
                        .get(&name)
                        .map(|state| Type::Function(state.function_type()))
                })
                .map(|ty| (name, ty))
        })
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
            Stmt::Assign(assign) => {
                for target in &assign.targets {
                    collect_referenced_names_in_expr(target, names);
                }
                collect_referenced_names_in_expr(&assign.value, names);
            }
            Stmt::AnnAssign(assign) => {
                collect_referenced_names_in_expr(assign.target.as_ref(), names);
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
                    if let Some(target) = &item.optional_vars {
                        collect_referenced_names_in_expr(target.as_ref(), names);
                    }
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
            Stmt::Match(match_stmt) => {
                collect_referenced_names_in_expr(&match_stmt.subject, names);
                for case in &match_stmt.cases {
                    if let Some(guard) = &case.guard {
                        collect_referenced_names_in_expr(guard, names);
                    }
                    collect_referenced_names_in_stmts(&case.body, names);
                }
            }
            Stmt::Raise(raise_stmt) => {
                if let Some(exc) = &raise_stmt.exc {
                    collect_referenced_names_in_expr(exc, names);
                }
                if let Some(cause) = &raise_stmt.cause {
                    collect_referenced_names_in_expr(cause, names);
                }
            }
            Stmt::Delete(delete_stmt) => {
                for target in &delete_stmt.targets {
                    collect_referenced_names_in_expr(target, names);
                }
            }
            Stmt::FunctionDef(func) => collect_nested_function_free_names(func, names),
            Stmt::ClassDef(_) | Stmt::Nonlocal(_) => {}
            _ => {}
        }
    }
}

pub(in crate::lower) fn collect_referenced_names_in_expr(expr: &Expr, names: &mut HashSet<String>) {
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
            collect_comprehension_names(&comp.generators, names, &[comp.elt.as_ref()]);
        }
        Expr::SetComp(comp) => {
            collect_comprehension_names(&comp.generators, names, &[comp.elt.as_ref()]);
        }
        Expr::DictComp(comp) => {
            if let Some(key) = comp.key.as_deref() {
                collect_comprehension_names(&comp.generators, names, &[key, comp.value.as_ref()]);
            } else {
                collect_comprehension_names(&comp.generators, names, &[comp.value.as_ref()]);
            }
        }
        Expr::Generator(generator) => {
            collect_comprehension_names(&generator.generators, names, &[generator.elt.as_ref()]);
        }
        Expr::Lambda(lambda) => {
            if let Some(parameters) = lambda.parameters.as_deref() {
                collect_parameter_default_names(parameters, names);
            }
            let mut body_names = HashSet::new();
            collect_referenced_names_in_expr(lambda.body.as_ref(), &mut body_names);
            if let Some(parameters) = lambda.parameters.as_deref() {
                let shadowed = parameter_names(parameters);
                body_names.retain(|name| !shadowed.contains(name));
            }
            names.extend(body_names);
        }
        Expr::FString(fstring) => {
            for element in fstring.value.elements() {
                collect_interpolated_element_names(element, names);
            }
        }
        Expr::TString(tstring) => {
            for part in &tstring.value {
                for element in &part.elements {
                    collect_interpolated_element_names(element, names);
                }
            }
        }
        Expr::Starred(starred) => {
            collect_referenced_names_in_expr(starred.value.as_ref(), names);
        }
        Expr::Slice(slice) => {
            for bound in [
                slice.lower.as_deref(),
                slice.upper.as_deref(),
                slice.step.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                collect_referenced_names_in_expr(bound, names);
            }
        }
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
        Expr::Named(named) => {
            collect_referenced_names_in_expr(named.value.as_ref(), names);
        }
        Expr::StringLiteral(_)
        | Expr::BytesLiteral(_)
        | Expr::NumberLiteral(_)
        | Expr::BooleanLiteral(_)
        | Expr::NoneLiteral(_)
        | Expr::EllipsisLiteral(_)
        | Expr::IpyEscapeCommand(_) => {}
    }
}

fn collect_nested_function_free_names(func: &StmtFunctionDef, names: &mut HashSet<String>) {
    collect_parameter_default_names(&func.parameters, names);
    for decorator in &func.decorator_list {
        collect_referenced_names_in_expr(&decorator.expression, names);
    }

    let mut references = HashSet::new();
    collect_referenced_names_in_stmts(&func.body, &mut references);

    let mut local_bindings = HashSet::new();
    collect_current_function_local_bindings(&func.body, &mut local_bindings);
    let mut nonlocal_names = HashSet::new();
    collect_nonlocal_names(&func.body, &mut nonlocal_names);
    let parameter_names = parameter_names(&func.parameters);

    names.extend(
        references
            .into_iter()
            .filter(|name| !parameter_names.contains(name))
            .filter(|name| !local_bindings.contains(name) || nonlocal_names.contains(name)),
    );
}

fn collect_comprehension_names(
    generators: &[sifr_python_ast::Comprehension],
    names: &mut HashSet<String>,
    elements: &[&Expr],
) {
    let mut shadowed = HashSet::new();
    for generator in generators {
        collect_names_excluding(&generator.iter, &shadowed, names);
        collect_assignment_target_names(std::slice::from_ref(&generator.target), &mut shadowed);
        for condition in &generator.ifs {
            collect_names_excluding(condition, &shadowed, names);
        }
    }
    for element in elements {
        collect_names_excluding(element, &shadowed, names);
    }
}

fn collect_names_excluding(expr: &Expr, shadowed: &HashSet<String>, names: &mut HashSet<String>) {
    let mut referenced = HashSet::new();
    collect_referenced_names_in_expr(expr, &mut referenced);
    names.extend(
        referenced
            .into_iter()
            .filter(|name| !shadowed.contains(name)),
    );
}

fn collect_parameter_default_names(parameters: &Parameters, names: &mut HashSet<String>) {
    for parameter in parameters {
        if let Some(default) = parameter.default() {
            collect_referenced_names_in_expr(default, names);
        }
    }
}

fn parameter_names(parameters: &Parameters) -> HashSet<String> {
    parameters
        .iter()
        .map(|parameter| parameter.name().to_string())
        .collect()
}

fn collect_interpolated_element_names(
    element: &InterpolatedStringElement,
    names: &mut HashSet<String>,
) {
    let InterpolatedStringElement::Interpolation(interpolation) = element else {
        return;
    };
    collect_referenced_names_in_expr(&interpolation.expression, names);
    if let Some(format_spec) = &interpolation.format_spec {
        for nested in &format_spec.elements {
            collect_interpolated_element_names(nested, names);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_python_parser::parse_module;

    #[test]
    fn nested_function_all_parameter_kinds_shadow_outer_names() {
        let parsed = parse_module(
            "def outer() -> None:\n    def inner(pos, /, regular, *items, keyword, **extras):\n        return (pos, regular, items, keyword, extras, captured)\n    return None\n",
        )
        .expect("source should parse");
        let Stmt::FunctionDef(outer) = &parsed.suite()[0] else {
            panic!("outer function missing");
        };
        let mut names = HashSet::new();
        collect_referenced_names_in_stmts(&outer.body, &mut names);

        assert_eq!(names, HashSet::from([String::from("captured")]));
    }
}
