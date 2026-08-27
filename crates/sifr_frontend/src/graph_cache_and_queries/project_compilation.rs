use super::{FrontendContext, FrontendDiagnosticStyle, FrontendSourceContext, ModuleId};
use crate::{
    collect_module_exports, diagnostic_with_code, erase_marker_imports, reveal_type_diagnostics,
    warning_diagnostics,
};
use ruff_text_size::{Ranged as _, TextRange};
use sifr_diagnostics::{
    ChildSeverity, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, RelatedKind,
    RenderedDiagnostic, Severity, SourceMap, SourceSpan,
};
use sifr_lowering::{
    ExternalDefs, FlowGraph, HirModule, LoweringOptions, LoweringResult, LoweringWarningDiagnostic,
    RevealTypeDiagnostic,
};
use sifr_python_ast::{Stmt, Suite};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

struct ModuleDependencyGraph {
    dependencies: BTreeMap<String, BTreeSet<String>>,
    reverse_dependencies: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrontendModuleDiagnostics {
    pub reveal_types: Vec<RevealTypeDiagnostic>,
    pub rendered_reveal_types: Vec<RenderedDiagnostic>,
    pub warnings: Vec<LoweringWarningDiagnostic>,
    pub rendered_warnings: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleDiagnostics {
    pub module: ModuleId,
    pub diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectDiagnostics {
    pub diagnostics: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Debug)]
pub struct FrontendProjectModule {
    pub suite: Suite,
    pub source: String,
    pub display_path: String,
}

pub struct FrontendProjectCompilation {
    pub lowering_results: BTreeMap<String, LoweringResult>,
    pub hir_modules: BTreeMap<String, HirModule>,
    pub flow_graphs: BTreeMap<String, FlowGraph>,
    pub external_defs: ExternalDefs,
    pub compile_order: Vec<String>,
    pub module_diagnostics: BTreeMap<String, FrontendModuleDiagnostics>,
}

impl FrontendProjectCompilation {
    #[must_use]
    pub fn empty(external_defs: ExternalDefs) -> Self {
        Self {
            lowering_results: BTreeMap::new(),
            hir_modules: BTreeMap::new(),
            flow_graphs: BTreeMap::new(),
            external_defs,
            compile_order: Vec::new(),
            module_diagnostics: BTreeMap::new(),
        }
    }
}

impl FrontendContext {
    pub fn compile_project_modules(
        modules: &BTreeMap<String, FrontendProjectModule>,
        external_defs: ExternalDefs,
        diagnostic_style: FrontendDiagnosticStyle,
        lowering_options: &LoweringOptions,
    ) -> Result<FrontendProjectCompilation, Vec<RenderedDiagnostic>> {
        if modules.is_empty() {
            return Ok(FrontendProjectCompilation::empty(external_defs));
        }
        let mut context = Self::load_project_modules(modules, external_defs)?;
        context.compile_project(diagnostic_style, lowering_options)
    }

    pub fn compile_project(
        &mut self,
        diagnostic_style: FrontendDiagnosticStyle,
        lowering_options: &LoweringOptions,
    ) -> Result<FrontendProjectCompilation, Vec<RenderedDiagnostic>> {
        let modules = self.compilation_modules()?;
        let compile_order = compute_project_compile_order(&modules)?;
        self.project_compile_order = Some(
            compile_order
                .iter()
                .filter_map(|module_name| {
                    self.modules
                        .iter()
                        .find(|module| module.module_name == *module_name)
                        .map(|module| module.id)
                })
                .collect(),
        );
        let mut external_defs = self.base_external_defs.clone();
        let mut lowering_results = BTreeMap::new();
        let mut hir_modules = BTreeMap::new();
        let mut flow_graphs = BTreeMap::new();
        let mut module_diagnostics = BTreeMap::new();

        self.lowering_modules.clear();
        for state in &mut self.modules {
            state.lowered = None;
            state.diagnostics = None;
            state.analysis = None;
        }

        for module_name in &compile_order {
            let Some(module) = modules.get(module_name) else {
                return Err(vec![diagnostic_with_code(
                    format!("project compile order references missing module '{module_name}'"),
                    DiagnosticCode::INTERNAL_COMPILER_PANIC,
                )]);
            };
            let source_context =
                (!module.display_path.is_empty()).then_some(FrontendSourceContext {
                    display_path: &module.display_path,
                    source: &module.source,
                });
            let mut lowering = match super::compile_module_hir_with_source_and_options(
                module_name,
                &module.suite,
                &external_defs,
                diagnostic_style,
                source_context,
                lowering_options.clone(),
            ) {
                Ok(lowering) => lowering,
                Err(errors) => {
                    self.store_module_errors(module_name, &errors);
                    return Err(errors);
                }
            };
            collect_module_exports(module_name, &lowering, &mut external_defs);
            erase_marker_imports(&mut lowering.module, &external_defs);

            let diagnostics = FrontendModuleDiagnostics {
                rendered_reveal_types: reveal_type_diagnostics(
                    source_context,
                    &lowering.reveal_types,
                ),
                reveal_types: lowering.reveal_types.clone(),
                rendered_warnings: warning_diagnostics(source_context, &lowering.warnings),
                warnings: lowering.warnings.clone(),
            };
            let rendered = diagnostics
                .rendered_warnings
                .iter()
                .chain(&diagnostics.rendered_reveal_types)
                .cloned()
                .collect::<Vec<_>>();
            lowering_results.insert(module_name.clone(), lowering.clone());
            let lowering = Arc::new(lowering);
            self.store_compiled_module(module_name, Arc::clone(&lowering), rendered);
            hir_modules.insert(module_name.clone(), lowering.module.clone());
            flow_graphs.insert(module_name.clone(), lowering.flow_graph.clone());
            module_diagnostics.insert(module_name.clone(), diagnostics);
        }

        self.external_defs = external_defs.clone();
        Ok(FrontendProjectCompilation {
            lowering_results,
            hir_modules,
            flow_graphs,
            external_defs,
            compile_order,
            module_diagnostics,
        })
    }

    fn compilation_modules(
        &mut self,
    ) -> Result<BTreeMap<String, FrontendProjectModule>, Vec<RenderedDiagnostic>> {
        let module_ids = self
            .modules
            .iter()
            .map(|module| module.id)
            .collect::<Vec<_>>();
        let mut modules = BTreeMap::new();
        for module_id in module_ids {
            let suite = if let Some(suite) = self.compilation_suites.get(&module_id) {
                suite.as_ref().clone()
            } else {
                self.ensure_parsed(module_id)?;
                let index = self.index_for_module(module_id);
                self.modules[index]
                    .parsed
                    .as_ref()
                    .map(|parsed| parsed.suite().iter().cloned().collect())
                    .unwrap_or_default()
            };
            let index = self.index_for_module(module_id);
            modules.insert(
                self.modules[index].module_name.clone(),
                FrontendProjectModule {
                    suite,
                    source: self.modules[index].source.as_str().to_string(),
                    display_path: self.modules[index].path.as_path().display().to_string(),
                },
            );
        }
        Ok(modules)
    }

    fn store_module_errors(&mut self, module_name: &str, errors: &[RenderedDiagnostic]) {
        if let Some(state) = self
            .modules
            .iter_mut()
            .find(|state| state.module_name == module_name)
        {
            state.diagnostics = Some(Arc::new(errors.to_vec()));
        }
    }

    fn store_compiled_module(
        &mut self,
        module_name: &str,
        lowering: Arc<sifr_lowering::LoweringResult>,
        diagnostics: Vec<RenderedDiagnostic>,
    ) {
        if let Some(state) = self
            .modules
            .iter_mut()
            .find(|state| state.module_name == module_name)
        {
            state.lowered = Some(lowering);
            state.diagnostics = Some(Arc::new(diagnostics));
        }
    }
}

pub fn compute_project_compile_order(
    modules: &BTreeMap<String, FrontendProjectModule>,
) -> Result<Vec<String>, Vec<RenderedDiagnostic>> {
    let graph = build_module_dependency_graph(
        modules
            .iter()
            .map(|(module_name, module)| (module_name.as_str(), module.suite.as_slice())),
    );
    let mut indegree = graph
        .dependencies
        .iter()
        .map(|(module_name, dependencies)| (module_name.clone(), dependencies.len()))
        .collect::<BTreeMap<_, _>>();
    let mut ready = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(module_name, _)| module_name.clone())
        .collect::<BTreeSet<_>>();
    let mut compile_order = Vec::with_capacity(indegree.len());

    while let Some(module_name) = ready.pop_first() {
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
    Err(vec![cycle_source_diagnostic(modules, &cycle_path)])
}

fn build_module_dependency_graph<'a>(
    modules: impl IntoIterator<Item = (&'a str, &'a [Stmt])>,
) -> ModuleDependencyGraph {
    let modules = modules.into_iter().collect::<BTreeMap<_, _>>();
    let local_modules = modules
        .keys()
        .map(|module_name| (*module_name).to_string())
        .collect::<BTreeSet<_>>();
    let dependencies = local_modules
        .iter()
        .map(|module_name| {
            let dependencies = modules
                .get(module_name.as_str())
                .map_or_else(BTreeSet::new, |suite| {
                    collect_local_module_dependencies(module_name, suite, &local_modules)
                });
            (module_name.clone(), dependencies)
        })
        .collect::<BTreeMap<_, _>>();
    let mut reverse_dependencies = local_modules
        .into_iter()
        .map(|module_name| (module_name, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for (module_name, dependencies) in &dependencies {
        for dependency in dependencies {
            if let Some(dependents) = reverse_dependencies.get_mut(dependency) {
                dependents.insert(module_name.clone());
            }
        }
    }
    ModuleDependencyGraph {
        dependencies,
        reverse_dependencies,
    }
}

fn collect_local_module_dependencies(
    current_module: &str,
    suite: &[Stmt],
    local_modules: &BTreeSet<String>,
) -> BTreeSet<String> {
    local_module_dependency_names(current_module, suite, local_modules)
}

pub(crate) fn local_module_dependency_names(
    current_module: &str,
    suite: &[Stmt],
    local_modules: &BTreeSet<String>,
) -> BTreeSet<String> {
    collect_local_module_dependency_ranges(current_module, suite, local_modules)
        .into_keys()
        .collect()
}

fn collect_local_module_dependency_ranges(
    current_module: &str,
    suite: &[Stmt],
    local_modules: &BTreeSet<String>,
) -> BTreeMap<String, TextRange> {
    let mut dependencies = BTreeMap::new();
    for statement in suite {
        let Stmt::ImportFrom(import_from) = statement else {
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
        for dependency in dependency_candidates(current_module, &module_name, import_from.level) {
            if local_modules.contains(&dependency) {
                dependencies
                    .entry(dependency)
                    .or_insert_with(|| module.range());
                break;
            }
        }
    }
    dependencies
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

fn find_dependency_cycle_path(
    dependencies: &BTreeMap<String, BTreeSet<String>>,
) -> Option<Vec<String>> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Unvisited,
        Visiting,
        Done,
    }

    fn visit(
        module: &str,
        dependencies: &BTreeMap<String, BTreeSet<String>>,
        states: &mut BTreeMap<String, VisitState>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        states.insert(module.to_string(), VisitState::Visiting);
        stack.push(module.to_string());
        if let Some(neighbors) = dependencies.get(module) {
            for neighbor in neighbors {
                match states
                    .get(neighbor)
                    .copied()
                    .unwrap_or(VisitState::Unvisited)
                {
                    VisitState::Unvisited => {
                        if let Some(cycle) = visit(neighbor, dependencies, states, stack) {
                            return Some(cycle);
                        }
                    }
                    VisitState::Visiting => {
                        if let Some(start) = stack.iter().position(|entry| entry == neighbor) {
                            let mut cycle = stack[start..].to_vec();
                            cycle.push(neighbor.clone());
                            return Some(cycle);
                        }
                    }
                    VisitState::Done => {}
                }
            }
        }
        let _ = stack.pop();
        states.insert(module.to_string(), VisitState::Done);
        None
    }

    let mut states = dependencies
        .keys()
        .cloned()
        .map(|module| (module, VisitState::Unvisited))
        .collect::<BTreeMap<_, _>>();
    let mut stack = Vec::new();
    for module in dependencies.keys() {
        if states.get(module) == Some(&VisitState::Unvisited) {
            if let Some(cycle) = visit(module, dependencies, &mut states, &mut stack) {
                return Some(cycle);
            }
        }
    }
    None
}

fn cycle_source_diagnostic(
    modules: &BTreeMap<String, FrontendProjectModule>,
    cycle_path: &[String],
) -> RenderedDiagnostic {
    let local_modules = modules.keys().cloned().collect::<BTreeSet<_>>();
    let mut edges = Vec::new();
    for edge in cycle_path.windows(2) {
        let [from, to] = edge else {
            continue;
        };
        let Some(module) = modules.get(from) else {
            continue;
        };
        let ranges = collect_local_module_dependency_ranges(from, &module.suite, &local_modules);
        if let Some(range) = ranges.get(to) {
            edges.push((from, to, module, *range));
        }
    }
    let Some((first_from, first_to, first_module, first_range)) = edges.first() else {
        return cycle_without_source(cycle_path);
    };
    if first_module.display_path.is_empty() || first_module.source.is_empty() {
        return cycle_without_source(cycle_path);
    }
    let (cycle, cycle_edges, _) = cycle_diagnostic_parts(cycle_path);
    let mut source_map = SourceMap::new();
    let first_source = source_map.register_source(&first_module.display_path, &first_module.source);
    let primary = SourceSpan::new(first_source, *first_range);
    let mut builder =
        DiagnosticBuilder::source(DiagnosticCode::IMPORT_CYCLE, Severity::Error, primary)
            .message_template("circular import detected: {cycle}")
            .arg("cycle", cycle)
            .arg("cycle_edges", cycle_edges)
            .related(
                SourceSpan::new(first_source, *first_range),
                RelatedKind::Note,
                Some(format!("{first_from} imports {first_to}")),
            )
            .help("break the cycle by moving shared declarations into a separate module");
    for (from, to, module, range) in edges.iter().skip(1) {
        let source_id = source_map.register_source(&module.display_path, &module.source);
        builder = builder.related(
            SourceSpan::new(source_id, *range),
            RelatedKind::Note,
            Some(format!("{from} imports {to}")),
        );
    }
    render_cycle_diagnostic(builder, &source_map)
}

fn cycle_without_source(cycle_path: &[String]) -> RenderedDiagnostic {
    let (cycle, cycle_edges, notes) = cycle_diagnostic_parts(cycle_path);
    let mut builder = DiagnosticBuilder::internal(DiagnosticCode::IMPORT_CYCLE, Severity::Error)
        .message_template("circular import detected: {cycle}")
        .arg("cycle", cycle)
        .arg("cycle_edges", cycle_edges)
        .help("break the cycle by moving shared declarations into a separate module");
    for note in notes {
        builder = builder.child(ChildSeverity::Note, note);
    }
    render_cycle_diagnostic(builder, &SourceMap::new())
}

fn render_cycle_diagnostic(
    builder: DiagnosticBuilder,
    source_map: &SourceMap,
) -> RenderedDiagnostic {
    let mut sink = DiagnosticSink::new();
    let _ = sink.emit_error(builder.build());
    match sifr_diagnostics::render::render_sink(&sink, source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        Ok(_) => diagnostic_with_code(
            "internal compiler error: import cycle renderer emitted an unexpected diagnostic count",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
        Err(error) => diagnostic_with_code(
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

fn canonicalize_cycle_path(mut cycle_path: Vec<String>) -> Vec<String> {
    if cycle_path.len() <= 2 {
        return cycle_path;
    }
    if cycle_path.first() == cycle_path.last() {
        let _ = cycle_path.pop();
    }
    if cycle_path.is_empty() {
        return Vec::new();
    }
    let mut best = cycle_path.clone();
    for start in 1..cycle_path.len() {
        let candidate = cycle_path[start..]
            .iter()
            .chain(&cycle_path[..start])
            .cloned()
            .collect::<Vec<_>>();
        if candidate < best {
            best = candidate;
        }
    }
    best.push(best[0].clone());
    best
}
