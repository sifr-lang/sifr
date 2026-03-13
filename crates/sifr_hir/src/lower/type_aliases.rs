use super::typing_and_functions::resolve_annotation_expr;
use super::LowerCtx;
use sifr_python_ast::{Expr, Stmt, TypeParam};
use sifr_type_system::Type;
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Clone)]
pub(super) struct TypeAliasDecl {
    pub(super) name: String,
    pub(super) type_params: Vec<String>,
    pub(super) value: Box<Expr>,
    pub(super) order: usize,
}

pub(super) fn collect_type_alias_decls(stmts: &[Stmt], ctx: &mut LowerCtx) -> Vec<TypeAliasDecl> {
    let mut decls = Vec::new();

    for (order, stmt) in stmts.iter().enumerate() {
        let Stmt::TypeAlias(type_alias) = stmt else {
            continue;
        };
        let Expr::Name(name_expr) = type_alias.name.as_ref() else {
            ctx.error("type alias name must be a simple name".to_string());
            continue;
        };

        let mut type_params = Vec::new();
        if let Some(ref params) = type_alias.type_params {
            for param in params.iter() {
                if let TypeParam::TypeVar(type_var) = param {
                    type_params.push(type_var.name.to_string());
                }
            }
        }

        decls.push(TypeAliasDecl {
            name: name_expr.id.clone(),
            type_params,
            value: type_alias.value.clone(),
            order,
        });
    }

    decls
}

pub(super) fn predeclare_type_aliases(alias_decls: &[TypeAliasDecl], ctx: &mut LowerCtx) {
    for decl in alias_decls {
        if decl.type_params.is_empty() {
            ctx.scope
                .define_type_alias(decl.name.clone(), Type::Unknown);
        } else {
            ctx.scope.define_generic_type_alias(
                decl.name.clone(),
                decl.type_params.clone(),
                Type::Unknown,
            );
        }
    }
}

pub(super) fn resolve_type_aliases(alias_decls: &[TypeAliasDecl], ctx: &mut LowerCtx) {
    if alias_decls.is_empty() {
        return;
    }

    let alias_names: HashSet<String> = alias_decls.iter().map(|decl| decl.name.clone()).collect();
    let graph = build_dependency_graph(alias_decls, &alias_names);
    let sccs = tarjan_scc(alias_decls, &graph);
    let order_map: HashMap<String, usize> = alias_decls
        .iter()
        .map(|decl| (decl.name.clone(), decl.order))
        .collect();
    let decl_map: HashMap<String, TypeAliasDecl> = alias_decls
        .iter()
        .cloned()
        .map(|decl| (decl.name.clone(), decl))
        .collect();

    for component in sccs {
        let mut members: Vec<TypeAliasDecl> = component
            .iter()
            .filter_map(|name| decl_map.get(name).cloned())
            .collect();
        members.sort_by_key(|decl| order_map.get(&decl.name).copied().unwrap_or(usize::MAX));

        for decl in members {
            let resolved = resolve_alias_decl(&decl, ctx);
            if decl.type_params.is_empty() {
                ctx.scope.define_type_alias(decl.name.clone(), resolved);
            } else {
                ctx.scope.define_generic_type_alias(
                    decl.name.clone(),
                    decl.type_params.clone(),
                    resolved,
                );
            }
        }
    }
}

fn resolve_alias_decl(decl: &TypeAliasDecl, ctx: &mut LowerCtx) -> Type {
    for type_param in &decl.type_params {
        ctx.type_vars.insert(type_param.clone());
    }

    let resolved = resolve_annotation_expr(&decl.value, ctx);

    for type_param in &decl.type_params {
        ctx.type_vars.remove(type_param.as_str());
    }

    resolved
}

fn build_dependency_graph(
    alias_decls: &[TypeAliasDecl],
    alias_names: &HashSet<String>,
) -> HashMap<String, Vec<String>> {
    let mut graph = HashMap::new();

    for decl in alias_decls {
        let local_type_params: HashSet<&str> =
            decl.type_params.iter().map(String::as_str).collect();
        let mut deps = BTreeSet::new();
        collect_alias_dependencies(&decl.value, alias_names, &local_type_params, &mut deps);
        graph.insert(decl.name.clone(), deps.into_iter().collect());
    }

    graph
}

fn collect_alias_dependencies(
    expr: &Expr,
    alias_names: &HashSet<String>,
    local_type_params: &HashSet<&str>,
    deps: &mut BTreeSet<String>,
) {
    match expr {
        Expr::Name(name) => {
            if alias_names.contains(name.id.as_str())
                && !local_type_params.contains(name.id.as_str())
            {
                deps.insert(name.id.clone());
            }
        }
        Expr::BinOp(binop) => {
            collect_alias_dependencies(&binop.left, alias_names, local_type_params, deps);
            collect_alias_dependencies(&binop.right, alias_names, local_type_params, deps);
        }
        Expr::Subscript(subscript) => {
            collect_alias_dependencies(&subscript.value, alias_names, local_type_params, deps);
            collect_alias_dependencies(&subscript.slice, alias_names, local_type_params, deps);
        }
        Expr::Tuple(tuple) => {
            for elt in &tuple.elts {
                collect_alias_dependencies(elt, alias_names, local_type_params, deps);
            }
        }
        Expr::List(list) => {
            for elt in &list.elts {
                collect_alias_dependencies(elt, alias_names, local_type_params, deps);
            }
        }
        Expr::BooleanLiteral(_)
        | Expr::NumberLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::NoneLiteral(_) => {}
        _ => {}
    }
}

fn tarjan_scc(
    alias_decls: &[TypeAliasDecl],
    graph: &HashMap<String, Vec<String>>,
) -> Vec<Vec<String>> {
    struct TarjanState<'a> {
        index: usize,
        indices: HashMap<String, usize>,
        lowlinks: HashMap<String, usize>,
        stack: Vec<String>,
        on_stack: HashSet<String>,
        graph: &'a HashMap<String, Vec<String>>,
        components: Vec<Vec<String>>,
    }

    fn strong_connect(name: &str, state: &mut TarjanState<'_>) {
        let node = name.to_string();
        state.indices.insert(node.clone(), state.index);
        state.lowlinks.insert(node.clone(), state.index);
        state.index += 1;
        state.stack.push(node.clone());
        state.on_stack.insert(node.clone());

        let neighbors = state.graph.get(name).cloned().unwrap_or_default();
        for neighbor in neighbors {
            if !state.indices.contains_key(&neighbor) {
                strong_connect(&neighbor, state);
                let neighbor_lowlink = state.lowlinks.get(&neighbor).copied().unwrap_or(usize::MAX);
                if let Some(lowlink) = state.lowlinks.get_mut(name) {
                    *lowlink = (*lowlink).min(neighbor_lowlink);
                }
            } else if state.on_stack.contains(&neighbor) {
                let neighbor_index = state.indices.get(&neighbor).copied().unwrap_or(usize::MAX);
                if let Some(lowlink) = state.lowlinks.get_mut(name) {
                    *lowlink = (*lowlink).min(neighbor_index);
                }
            }
        }

        let node_lowlink = state.lowlinks.get(name).copied();
        let node_index = state.indices.get(name).copied();
        if node_lowlink != node_index {
            return;
        }

        let mut component = Vec::new();
        while let Some(stack_name) = state.stack.pop() {
            state.on_stack.remove(&stack_name);
            component.push(stack_name.clone());
            if stack_name == name {
                break;
            }
        }
        component.sort();
        state.components.push(component);
    }

    let mut state = TarjanState {
        index: 0,
        indices: HashMap::new(),
        lowlinks: HashMap::new(),
        stack: Vec::new(),
        on_stack: HashSet::new(),
        graph,
        components: Vec::new(),
    };

    let mut ordered_names: Vec<String> = alias_decls.iter().map(|decl| decl.name.clone()).collect();
    ordered_names.sort_by_key(|name| {
        alias_decls
            .iter()
            .find(|decl| decl.name == *name)
            .map(|decl| decl.order)
            .unwrap_or(usize::MAX)
    });

    for name in ordered_names {
        if !state.indices.contains_key(&name) {
            strong_connect(&name, &mut state);
        }
    }

    state.components
}
