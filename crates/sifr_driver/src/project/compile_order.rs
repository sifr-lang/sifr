use crate::diagnostics::RenderedDiagnostic;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::Stmt;
use std::collections::{BTreeMap, BTreeSet, HashMap};

struct ModuleDependencyGraph {
    dependencies: BTreeMap<String, BTreeSet<String>>,
    reverse_dependencies: BTreeMap<String, BTreeSet<String>>,
}

fn collect_local_module_dependencies(
    current_module: &str,
    stmts: &[Stmt],
    local_modules: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut deps = BTreeSet::new();
    for stmt in stmts {
        let Stmt::ImportFrom(import_from) = stmt else {
            continue;
        };
        if import_from.level > 1 {
            continue;
        }
        let Some(module) = &import_from.module else {
            continue;
        };
        let module_name = module.to_string();
        if module_name == "typing"
            || module_name == "enum"
            || module_name.starts_with("sifr.")
            || module_name.starts_with("_sifr.")
        {
            continue;
        }
        for dependency in
            dependency_candidates(current_module, module_name.as_str(), import_from.level)
        {
            if local_modules.contains(&dependency) {
                deps.insert(dependency);
                break;
            }
        }
    }
    deps
}

fn dependency_candidates(current_module: &str, module_name: &str, level: u32) -> Vec<String> {
    if level != 1 {
        return vec![module_name.to_string()];
    }
    let mut candidates = Vec::new();
    if current_module != "main" && !current_module.is_empty() {
        candidates.push(format!("{current_module}.{module_name}"));
    }
    if let Some((parent, _)) = current_module.rsplit_once('.') {
        if !parent.is_empty() {
            candidates.push(format!("{parent}.{module_name}"));
        }
    }
    candidates.push(module_name.to_string());
    candidates.dedup();
    candidates
}

fn build_module_dependency_graph(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
) -> ModuleDependencyGraph {
    let local_modules: BTreeSet<String> = parsed_modules.keys().cloned().collect();
    let mut dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for module_name in &local_modules {
        let module_deps = parsed_modules
            .get(module_name)
            .map(|stmts| collect_local_module_dependencies(module_name, stmts, &local_modules))
            .unwrap_or_default();
        dependencies.insert(module_name.clone(), module_deps);
    }

    let mut reverse_dependencies: BTreeMap<String, BTreeSet<String>> = local_modules
        .iter()
        .cloned()
        .map(|name| (name, BTreeSet::new()))
        .collect();
    for (module_name, deps) in &dependencies {
        for dep in deps {
            if let Some(reverse_deps) = reverse_dependencies.get_mut(dep) {
                reverse_deps.insert(module_name.clone());
            }
        }
    }

    ModuleDependencyGraph {
        dependencies,
        reverse_dependencies,
    }
}

fn find_dependency_cycle_path(
    dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Unvisited,
        Visiting,
        Done,
    }

    fn dfs(
        node: &str,
        dependencies: &BTreeMap<String, BTreeSet<String>>,
        states: &mut BTreeMap<String, VisitState>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        states.insert(node.to_string(), VisitState::Visiting);
        stack.push(node.to_string());

        if let Some(neighbors) = dependencies.get(node) {
            for neighbor in neighbors {
                match states
                    .get(neighbor.as_str())
                    .copied()
                    .unwrap_or(VisitState::Unvisited)
                {
                    VisitState::Unvisited => {
                        if let Some(cycle) = dfs(neighbor, dependencies, states, stack) {
                            return Some(cycle);
                        }
                    }
                    VisitState::Visiting => {
                        if let Some(start_idx) = stack.iter().position(|entry| entry == neighbor) {
                            let mut cycle = stack[start_idx..].to_vec();
                            cycle.push(neighbor.clone());
                            return Some(cycle);
                        }
                    }
                    VisitState::Done => {}
                }
            }
        }

        let _ = stack.pop();
        states.insert(node.to_string(), VisitState::Done);
        None
    }

    let mut states: BTreeMap<String, VisitState> = dependencies
        .keys()
        .cloned()
        .map(|node| (node, VisitState::Unvisited))
        .collect();
    let mut stack = Vec::new();

    for node in dependencies.keys() {
        if states
            .get(node.as_str())
            .copied()
            .unwrap_or(VisitState::Unvisited)
            == VisitState::Unvisited
        {
            if let Some(cycle) = dfs(node, dependencies, &mut states, &mut stack) {
                return Some(cycle);
            }
        }
    }

    None
}

pub(crate) fn compute_module_compile_order(
    parsed_modules: &HashMap<String, Vec<Stmt>>,
) -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let graph = build_module_dependency_graph(parsed_modules);
    let mut indegree: BTreeMap<String, usize> = graph
        .dependencies
        .iter()
        .map(|(module_name, deps)| (module_name.clone(), deps.len()))
        .collect();
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(module_name, _)| module_name.clone())
        .collect();
    let mut compile_order = Vec::with_capacity(indegree.len());

    while let Some(module_name) = ready.iter().next().cloned() {
        ready.remove(&module_name);
        compile_order.push(module_name.clone());
        if let Some(dependents) = graph.reverse_dependencies.get(&module_name) {
            for dependent in dependents {
                if let Some(degree) = indegree.get_mut(dependent) {
                    *degree = degree.saturating_sub(1);
                    if *degree == 0 {
                        ready.insert(dependent.clone());
                    }
                }
            }
        }
    }

    if compile_order.len() == indegree.len() {
        return Ok(compile_order);
    }

    let cycle_path = canonicalize_cycle_path(
        find_dependency_cycle_path(&graph.dependencies)
            .unwrap_or_else(|| vec!["<cycle>".to_string()]),
    );
    let cycle_render = cycle_path.join(" -> ");
    let edge_render = cycle_path
        .windows(2)
        .map(|edge| format!("{} imports {}", edge[0], edge[1]))
        .collect::<Vec<_>>()
        .join(", ");
    let message = format!(
        "module dependency cycle detected: {cycle_render}; import chain: {edge_render}. Break the cycle by moving shared declarations into a separate module."
    );
    Err(vec![crate::diagnostics::diagnostic_with_code(
        message,
        DiagnosticCode::WORKSPACE_IMPORT_CYCLE,
    )])
}

fn canonicalize_cycle_path(cycle_path: Vec<String>) -> Vec<String> {
    if cycle_path.len() <= 2 {
        return cycle_path;
    }

    let mut nodes = cycle_path;
    if nodes.first() == nodes.last() {
        let _ = nodes.pop();
    }
    if nodes.is_empty() {
        return Vec::new();
    }

    let mut best_rotation = nodes.clone();
    for start in 1..nodes.len() {
        let candidate: Vec<String> = nodes[start..]
            .iter()
            .chain(nodes[..start].iter())
            .cloned()
            .collect();
        if candidate < best_rotation {
            best_rotation = candidate;
        }
    }

    best_rotation.push(best_rotation[0].clone());
    best_rotation
}
