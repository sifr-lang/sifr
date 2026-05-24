use super::typing_and_functions::resolve_annotation_expr;
use super::LowerCtx;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt, TypeParam};
use sifr_type_system::Type;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

#[derive(Clone)]
pub(in crate::lower) struct TypeAliasDecl {
    pub(in crate::lower) name: String,
    pub(in crate::lower) type_params: Vec<String>,
    pub(in crate::lower) value: Box<Expr>,
    pub(in crate::lower) value_range: TextRange,
    pub(in crate::lower) order: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencyEdge {
    target: String,
    crosses_boundary: bool,
}

pub(in crate::lower) fn collect_type_alias_decls(
    stmts: &[Stmt],
    ctx: &mut LowerCtx,
) -> Vec<TypeAliasDecl> {
    let mut decls = Vec::new();

    for (order, stmt) in stmts.iter().enumerate() {
        let Stmt::TypeAlias(type_alias) = stmt else {
            continue;
        };
        let Expr::Name(name_expr) = type_alias.name.as_ref() else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_INVALID_ANNOTATION,
                "type alias name must be a simple name".to_string(),
                type_alias.name.range(),
            );
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
            name: name_expr.id.to_string(),
            type_params,
            value_range: type_alias.value.range(),
            value: type_alias.value.clone(),
            order,
        });
    }

    decls
}

pub(in crate::lower) fn predeclare_type_aliases(alias_decls: &[TypeAliasDecl], ctx: &mut LowerCtx) {
    for decl in alias_decls {
        let alias_ty = alias_type_template(decl);
        if decl.type_params.is_empty() {
            ctx.scope.define_type_alias(decl.name.clone(), alias_ty);
        } else {
            ctx.scope.define_generic_type_alias(
                decl.name.clone(),
                decl.type_params.clone(),
                alias_ty,
            );
        }
    }
}

pub(in crate::lower) fn resolve_type_aliases(alias_decls: &[TypeAliasDecl], ctx: &mut LowerCtx) {
    if alias_decls.is_empty() {
        return;
    }

    let alias_names: HashSet<String> = alias_decls.iter().map(|decl| decl.name.clone()).collect();
    let graph = build_dependency_graph(alias_decls, &alias_names);
    let ordered_names = ordered_alias_names(alias_decls);
    let sccs = tarjan_scc(&ordered_names, &dependency_targets(&graph));
    let order_map: HashMap<String, usize> = alias_decls
        .iter()
        .map(|decl| (decl.name.clone(), decl.order))
        .collect();
    let decl_map: HashMap<String, TypeAliasDecl> = alias_decls
        .iter()
        .cloned()
        .map(|decl| (decl.name.clone(), decl))
        .collect();
    let invalid_aliases = validate_recursive_alias_sccs(&sccs, &graph, &decl_map, &order_map, ctx);

    for component in sccs {
        let mut members: Vec<TypeAliasDecl> = component
            .iter()
            .filter_map(|name| decl_map.get(name).cloned())
            .collect();
        members.sort_by_key(|decl| order_map.get(&decl.name).copied().unwrap_or(usize::MAX));

        for decl in members {
            let resolved = if invalid_aliases.contains(&decl.name) {
                Type::Unknown
            } else {
                resolve_alias_decl(&decl, ctx)
            };
            let alias_ty = Type::Alias {
                name: decl.name.clone(),
                type_args: alias_template_type_args(&decl.type_params),
                body: Box::new(resolved),
            };
            if decl.type_params.is_empty() {
                ctx.scope.define_type_alias(decl.name.clone(), alias_ty);
            } else {
                ctx.scope.define_generic_type_alias(
                    decl.name.clone(),
                    decl.type_params.clone(),
                    alias_ty,
                );
            }
        }
    }
}

fn alias_type_template(decl: &TypeAliasDecl) -> Type {
    Type::Alias {
        name: decl.name.clone(),
        type_args: alias_template_type_args(&decl.type_params),
        body: Box::new(Type::Unknown),
    }
}

fn alias_template_type_args(type_params: &[String]) -> Vec<Type> {
    type_params.iter().cloned().map(Type::TypeVar).collect()
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
) -> HashMap<String, Vec<DependencyEdge>> {
    let mut graph = HashMap::new();

    for decl in alias_decls {
        let local_type_params: HashSet<&str> =
            decl.type_params.iter().map(String::as_str).collect();
        let mut deps = BTreeMap::new();
        collect_alias_dependencies(
            &decl.value,
            alias_names,
            &local_type_params,
            false,
            &mut deps,
        );
        let edges = deps
            .into_iter()
            .map(|(target, crosses_boundary)| DependencyEdge {
                target,
                crosses_boundary,
            })
            .collect();
        graph.insert(decl.name.clone(), edges);
    }

    graph
}

fn dependency_targets(
    graph: &HashMap<String, Vec<DependencyEdge>>,
) -> HashMap<String, Vec<String>> {
    graph
        .iter()
        .map(|(name, edges)| {
            (
                name.clone(),
                edges.iter().map(|edge| edge.target.clone()).collect(),
            )
        })
        .collect()
}

fn collect_alias_dependencies(
    expr: &Expr,
    alias_names: &HashSet<String>,
    local_type_params: &HashSet<&str>,
    crosses_boundary: bool,
    deps: &mut BTreeMap<String, bool>,
) {
    match expr {
        Expr::Name(name) => {
            if alias_names.contains(name.id.as_str())
                && !local_type_params.contains(name.id.as_str())
            {
                deps.entry(name.id.to_string())
                    .and_modify(|existing| *existing &= crosses_boundary)
                    .or_insert(crosses_boundary);
            }
        }
        Expr::BinOp(binop) => {
            collect_alias_dependencies(
                &binop.left,
                alias_names,
                local_type_params,
                crosses_boundary,
                deps,
            );
            collect_alias_dependencies(
                &binop.right,
                alias_names,
                local_type_params,
                crosses_boundary,
                deps,
            );
        }
        Expr::Subscript(subscript) => {
            collect_alias_dependencies(
                &subscript.value,
                alias_names,
                local_type_params,
                crosses_boundary,
                deps,
            );
            let crosses_container_boundary = matches!(
                subscript.value.as_ref(),
                Expr::Name(name) if matches!(name.id.as_str(), "list" | "dict" | "set")
            );
            collect_alias_dependencies(
                &subscript.slice,
                alias_names,
                local_type_params,
                crosses_boundary || crosses_container_boundary,
                deps,
            );
        }
        Expr::Tuple(tuple) => {
            for elt in &tuple.elts {
                collect_alias_dependencies(
                    elt,
                    alias_names,
                    local_type_params,
                    crosses_boundary,
                    deps,
                );
            }
        }
        Expr::List(list) => {
            for elt in &list.elts {
                collect_alias_dependencies(
                    elt,
                    alias_names,
                    local_type_params,
                    crosses_boundary,
                    deps,
                );
            }
        }
        Expr::BooleanLiteral(_)
        | Expr::NumberLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::NoneLiteral(_) => {}
        _ => {}
    }
}

fn validate_recursive_alias_sccs(
    sccs: &[Vec<String>],
    graph: &HashMap<String, Vec<DependencyEdge>>,
    decl_map: &HashMap<String, TypeAliasDecl>,
    order_map: &HashMap<String, usize>,
    ctx: &mut LowerCtx,
) -> HashSet<String> {
    let mut invalid_aliases = HashSet::new();

    for component in sccs {
        let component_set: HashSet<&str> = component.iter().map(String::as_str).collect();
        let has_recursive_cycle = component.len() > 1
            || component.iter().any(|name| {
                graph
                    .get(name)
                    .is_some_and(|edges| edges.iter().any(|edge| edge.target == *name))
            });
        if !has_recursive_cycle {
            continue;
        }

        let mut unbounded_graph: HashMap<String, Vec<String>> = HashMap::new();
        for name in component {
            let mut targets = BTreeSet::new();
            for edge in graph.get(name).into_iter().flatten() {
                if !edge.crosses_boundary && component_set.contains(edge.target.as_str()) {
                    targets.insert(edge.target.clone());
                }
            }
            unbounded_graph.insert(name.clone(), targets.into_iter().collect());
        }

        let mut ordered_component = component.clone();
        ordered_component.sort_by_key(|name| order_map.get(name).copied().unwrap_or(usize::MAX));

        for bad_component in tarjan_scc(&ordered_component, &unbounded_graph) {
            let has_unbounded_cycle = bad_component.len() > 1
                || bad_component.iter().any(|name| {
                    unbounded_graph
                        .get(name)
                        .is_some_and(|targets| targets.contains(name))
                });
            if !has_unbounded_cycle {
                continue;
            }

            for name in bad_component {
                if let Some(decl) = decl_map.get(&name) {
                    ctx.error_with_code_at(
                        DiagnosticCode::TYPE_INVALID_ANNOTATION,
                        recursive_alias_error_message(decl),
                        decl.value_range,
                    );
                    invalid_aliases.insert(name);
                }
            }
        }
    }

    invalid_aliases
}

fn recursive_alias_error_message(decl: &TypeAliasDecl) -> String {
    if decl.type_params.is_empty() {
        format!(
            "ill-formed recursive type alias '{}': recursion must cross an indirection boundary",
            decl.name
        )
    } else {
        format!(
            "ill-formed recursive generic alias '{}[{}]': recursion must cross an indirection boundary",
            decl.name,
            decl.type_params.join(", "),
        )
    }
}

fn ordered_alias_names(alias_decls: &[TypeAliasDecl]) -> Vec<String> {
    let mut ordered_names: Vec<String> = alias_decls.iter().map(|decl| decl.name.clone()).collect();
    ordered_names.sort_by_key(|name| {
        alias_decls
            .iter()
            .find(|decl| decl.name == *name)
            .map(|decl| decl.order)
            .unwrap_or(usize::MAX)
    });
    ordered_names
}

fn tarjan_scc(ordered_names: &[String], graph: &HashMap<String, Vec<String>>) -> Vec<Vec<String>> {
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

    for name in ordered_names {
        if !state.indices.contains_key(name) {
            strong_connect(name, &mut state);
        }
    }

    state.components
}
