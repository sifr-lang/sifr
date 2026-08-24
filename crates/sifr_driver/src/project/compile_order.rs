use crate::diagnostics::RenderedDiagnostic;
use ruff_text_size::{Ranged as _, TextRange};
use sifr_diagnostics::{
    ChildSeverity, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, RelatedKind, Severity,
    SourceMap, SourceSpan,
};
use sifr_python_ast::Stmt;
#[cfg(test)]
use sifr_python_ast::Suite;
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

fn collect_local_module_dependency_ranges(
    current_module: &str,
    stmts: &[Stmt],
    local_modules: &BTreeSet<String>,
) -> BTreeMap<String, TextRange> {
    let mut deps = BTreeMap::new();
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
                deps.entry(dependency).or_insert_with(|| module.range());
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

fn build_module_dependency_graph<'a>(
    parsed_modules: impl IntoIterator<Item = (&'a str, &'a [Stmt])>,
) -> ModuleDependencyGraph {
    let parsed_modules: BTreeMap<&str, &[Stmt]> = parsed_modules.into_iter().collect();
    let local_modules: BTreeSet<String> = parsed_modules
        .keys()
        .map(|module_name| (*module_name).to_string())
        .collect();
    let mut dependencies: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for module_name in &local_modules {
        let module_deps = parsed_modules
            .get(&module_name.as_str())
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

#[cfg(test)]
pub(crate) fn compute_module_compile_order(
    parsed_modules: &HashMap<String, Suite>,
) -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let graph = build_module_dependency_graph(
        parsed_modules
            .iter()
            .map(|(module_name, suite)| (module_name.as_str(), suite.as_slice())),
    );
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
    let (cycle_render, edge_render, notes) = cycle_diagnostic_parts(&cycle_path);
    let args = [
        (
            "cycle",
            sifr_diagnostics::DiagnosticArg::String(cycle_render),
        ),
        (
            "cycle_edges",
            sifr_diagnostics::DiagnosticArg::String(edge_render),
        ),
    ];
    Err(vec![crate::diagnostics::diagnostic_without_source(
        DiagnosticCode::IMPORT_CYCLE,
        "circular import detected: {cycle}",
        &args,
        &notes,
        Some("break the cycle by moving shared declarations into a separate module".to_string()),
    )])
}

pub(crate) struct CompileOrderSourceModule<'a> {
    pub(crate) suite: &'a [Stmt],
    pub(crate) source: &'a str,
    pub(crate) display_path: &'a str,
}

pub(crate) fn compute_module_compile_order_with_sources(
    parsed_modules: &HashMap<String, CompileOrderSourceModule<'_>>,
) -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let graph = build_module_dependency_graph(
        parsed_modules
            .iter()
            .map(|(module_name, input)| (module_name.as_str(), input.suite)),
    );
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
    Err(vec![cycle_source_diagnostic(parsed_modules, &cycle_path)])
}

fn cycle_source_diagnostic(
    parsed_modules: &HashMap<String, CompileOrderSourceModule<'_>>,
    cycle_path: &[String],
) -> RenderedDiagnostic {
    let (cycle, cycle_edges, _) = cycle_diagnostic_parts(cycle_path);
    let local_modules = parsed_modules.keys().cloned().collect::<BTreeSet<_>>();
    let mut edge_spans = Vec::new();
    for edge in cycle_path.windows(2) {
        let [from, to] = edge else {
            continue;
        };
        let Some(module) = parsed_modules.get(from) else {
            continue;
        };
        let ranges = collect_local_module_dependency_ranges(from, module.suite, &local_modules);
        if let Some(range) = ranges.get(to) {
            edge_spans.push((from.clone(), to.clone(), module, *range));
        }
    }
    let Some((from, to, first_module, first_range)) = edge_spans.first() else {
        return cycle_without_source(cycle_path);
    };
    let mut source_map = SourceMap::new();
    let first_source_id =
        source_map.register_source(first_module.display_path, first_module.source);
    let primary = match SourceSpan::new_validated(&source_map, first_source_id, *first_range) {
        Ok(span) => span,
        Err(error) => {
            return crate::diagnostics::diagnostic_with_code(
                format!("internal compiler error: invalid import cycle span: {error:?}"),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            );
        }
    };
    let mut builder =
        DiagnosticBuilder::source(DiagnosticCode::IMPORT_CYCLE, Severity::Error, primary)
            .message_template("circular import detected: {cycle}")
            .arg("cycle", cycle)
            .arg("cycle_edges", cycle_edges)
            .child(
                ChildSeverity::Help,
                "break the cycle by moving shared declarations into a separate module",
            )
            .related(
                SourceSpan::new(first_source_id, *first_range),
                RelatedKind::Note,
                Some(format!("{from} imports {to}")),
            );
    for (from, to, module, range) in edge_spans.iter().skip(1) {
        let source_id = source_map.register_source(module.display_path, module.source);
        if let Ok(span) = SourceSpan::new_validated(&source_map, source_id, *range) {
            builder = builder.related(
                span,
                RelatedKind::Note,
                Some(format!("{from} imports {to}")),
            );
        }
    }
    let diagnostic = builder.build();
    let mut sink = DiagnosticSink::new();
    let _ = sink.emit_error(diagnostic);
    match sifr_diagnostics::render::render_sink(&sink, &source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        Ok(_) => crate::diagnostics::diagnostic_with_code(
            "internal compiler error: import cycle renderer emitted an unexpected diagnostic count",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
        Err(error) => crate::diagnostics::diagnostic_with_code(
            format!("internal compiler error: invalid import cycle span: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

fn cycle_diagnostic_parts(cycle_path: &[String]) -> (String, String, Vec<String>) {
    let notes = cycle_path
        .windows(2)
        .map(|edge| format!("{} imports {}", edge[0], edge[1]))
        .collect::<Vec<_>>();
    (cycle_path.join(" -> "), notes.join("; "), notes)
}

fn cycle_without_source(cycle_path: &[String]) -> RenderedDiagnostic {
    let (cycle, cycle_edges, notes) = cycle_diagnostic_parts(cycle_path);
    let args = [
        ("cycle", sifr_diagnostics::DiagnosticArg::String(cycle)),
        (
            "cycle_edges",
            sifr_diagnostics::DiagnosticArg::String(cycle_edges),
        ),
    ];
    crate::diagnostics::diagnostic_without_source(
        DiagnosticCode::IMPORT_CYCLE,
        "circular import detected: {cycle}",
        &args,
        &notes,
        Some("break the cycle by moving shared declarations into a separate module".to_string()),
    )
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
