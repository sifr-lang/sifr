use crate::diagnostics::RenderedDiagnostic;
use crate::workspace::WorkspaceRoot;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::Stmt;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct ParsedProjectModule {
    pub(crate) suite: Vec<Stmt>,
    pub(crate) source: String,
    pub(crate) display_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ModuleOrigin {
    EntryParent,
    WorkspaceSource { source_root: PathBuf },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedModule {
    pub(crate) module_name: String,
    pub(crate) path: PathBuf,
    pub(crate) origin: ModuleOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResolutionError {
    pub(crate) module_name: String,
    pub(crate) tried_paths: Vec<PathBuf>,
    pub(crate) matches: Vec<PathBuf>,
    kind: ResolutionFailureKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ModuleResolver {
    entry_parent: PathBuf,
    workspace: Option<WorkspaceModuleSources>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkspaceModuleSources {
    workspace_root: PathBuf,
    source_roots: Vec<PathBuf>,
}

impl ModuleResolver {
    pub(crate) fn entry_parent(entry_parent: impl Into<PathBuf>) -> Self {
        Self {
            entry_parent: entry_parent.into(),
            workspace: None,
        }
    }

    pub(crate) fn with_workspace(
        entry_parent: impl Into<PathBuf>,
        workspace: WorkspaceRoot,
    ) -> Self {
        Self {
            entry_parent: entry_parent.into(),
            workspace: Some(WorkspaceModuleSources {
                workspace_root: workspace.dir,
                source_roots: workspace.config.source_roots,
            }),
        }
    }

    pub(crate) fn module_source_path(&self, module_name: &str) -> PathBuf {
        self.entry_parent
            .join(module_name_to_relative_path(module_name))
    }

    pub(crate) fn has_workspace(&self) -> bool {
        self.workspace.is_some()
    }

    pub(crate) fn resolve(&self, module_name: &str) -> Result<ResolvedModule, ResolutionError> {
        let entry_path = self.module_source_path(module_name);
        if entry_path.is_file() {
            let resolved = ResolvedModule {
                module_name: module_name.to_string(),
                path: entry_path,
                origin: ModuleOrigin::EntryParent,
            };
            self.reject_namespace_file_collision(&resolved)?;
            return Ok(resolved);
        }

        let Some(workspace) = &self.workspace else {
            return Err(ResolutionError {
                module_name: module_name.to_string(),
                tried_paths: vec![entry_path],
                matches: Vec::new(),
                kind: ResolutionFailureKind::Unresolved,
            });
        };

        let relative_path = module_name_to_relative_path(module_name);
        let mut tried_paths = vec![entry_path];
        let mut matches = Vec::new();
        for source_root in &workspace.source_roots {
            let candidate = workspace
                .workspace_root
                .join(source_root)
                .join(&relative_path);
            tried_paths.push(candidate.clone());
            if candidate.is_file() && !matches.contains(&candidate) {
                matches.push(candidate);
            }
        }

        if matches.len() > 1 {
            return Err(ResolutionError {
                module_name: module_name.to_string(),
                tried_paths,
                matches,
                kind: ResolutionFailureKind::Ambiguous,
            });
        }

        if let Some(path) = matches.into_iter().next() {
            let source_root = workspace
                .source_roots
                .iter()
                .find(|source_root| path.starts_with(workspace.workspace_root.join(source_root)))
                .cloned()
                .unwrap_or_else(|| PathBuf::from("."));
            let resolved = ResolvedModule {
                module_name: module_name.to_string(),
                path,
                origin: ModuleOrigin::WorkspaceSource { source_root },
            };
            self.reject_namespace_file_collision(&resolved)?;
            return Ok(resolved);
        }

        Err(ResolutionError {
            module_name: module_name.to_string(),
            tried_paths,
            matches: Vec::new(),
            kind: ResolutionFailureKind::Unresolved,
        })
    }

    fn reject_namespace_file_collision(
        &self,
        resolved: &ResolvedModule,
    ) -> Result<(), ResolutionError> {
        let Some((parent_name, parent_path)) = self.parent_module_file(&resolved.module_name)
        else {
            return Ok(());
        };
        Err(ResolutionError {
            module_name: resolved.module_name.clone(),
            tried_paths: vec![resolved.path.clone(), parent_path],
            matches: Vec::new(),
            kind: ResolutionFailureKind::NamespaceFileCollision { parent_name },
        })
    }

    fn parent_module_file(&self, module_name: &str) -> Option<(String, PathBuf)> {
        let parts: Vec<&str> = module_name.split('.').collect();
        if parts.len() < 2 {
            return None;
        }
        for end in 1..parts.len() {
            let parent_name = parts[..end].join(".");
            for candidate in self.candidate_paths(&parent_name) {
                if candidate.is_file() {
                    return Some((parent_name, candidate));
                }
            }
        }
        None
    }

    fn candidate_paths(&self, module_name: &str) -> Vec<PathBuf> {
        let relative_path = module_name_to_relative_path(module_name);
        let mut candidates = vec![self.entry_parent.join(&relative_path)];
        if let Some(workspace) = &self.workspace {
            candidates.extend(workspace.source_roots.iter().map(|source_root| {
                workspace
                    .workspace_root
                    .join(source_root)
                    .join(&relative_path)
            }));
        }
        candidates
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolutionFailureKind {
    Unresolved,
    Ambiguous,
    NamespaceFileCollision { parent_name: String },
}

impl ResolutionError {
    fn to_diagnostic(&self, resolver: &ModuleResolver) -> RenderedDiagnostic {
        match &self.kind {
            ResolutionFailureKind::Unresolved => crate::diagnostics::diagnostic_with_code(
                unresolved_import_message(&self.module_name, &self.tried_paths),
                DiagnosticCode::WORKSPACE_UNRESOLVED_IMPORT,
            ),
            ResolutionFailureKind::Ambiguous => crate::diagnostics::diagnostic_with_code(
                ambiguous_import_message(&self.module_name, resolver, &self.matches),
                DiagnosticCode::WORKSPACE_AMBIGUOUS_IMPORT,
            ),
            ResolutionFailureKind::NamespaceFileCollision { parent_name } => {
                let resolved_path = self
                    .tried_paths
                    .first()
                    .cloned()
                    .unwrap_or_else(|| resolver.module_source_path(&self.module_name));
                let parent_path = self
                    .tried_paths
                    .get(1)
                    .cloned()
                    .unwrap_or_else(|| resolver.module_source_path(parent_name));
                crate::diagnostics::diagnostic_with_code(
                    namespace_collision_message(
                        &self.module_name,
                        &resolved_path,
                        parent_name,
                        &parent_path,
                    ),
                    DiagnosticCode::WORKSPACE_NAMESPACE_COLLISION,
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DiscoveryDiagnosticStyle {
    ModuleName,
    FilePath,
}

fn discover_project_sifr_files(project_dir: &Path) -> Vec<PathBuf> {
    let mut sifr_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sifr") {
                sifr_files.push(path);
            }
        }
    }
    sifr_files.sort();
    sifr_files
}

fn is_test_module_name(module_name: &str) -> bool {
    module_name.starts_with("test_") || module_name.ends_with("_test")
}

pub(crate) fn discover_test_root_modules(test_dir: &Path) -> BTreeMap<String, PathBuf> {
    let mut test_files_by_module = BTreeMap::new();
    for path in discover_project_sifr_files(test_dir) {
        let Some(file_stem) = path.file_stem() else {
            continue;
        };
        let module_name = file_stem.to_string_lossy().to_string();
        if is_test_module_name(&module_name) {
            test_files_by_module.insert(module_name, path);
        }
    }
    test_files_by_module
}

fn module_name_to_relative_path(module_name: &str) -> PathBuf {
    let mut path = PathBuf::new();
    for component in module_name.split('.') {
        path.push(component);
    }
    path.set_extension("sifr");
    path
}

fn unresolved_import_message(module_name: &str, tried_paths: &[PathBuf]) -> String {
    let entry_path = tried_paths
        .first()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let workspace_paths: Vec<String> = tried_paths
        .iter()
        .skip(1)
        .map(|path| format!("'{}'", path.display()))
        .collect();
    if workspace_paths.is_empty() {
        return format!(
            "could not resolve import '{module_name}'; tried entry-relative '{entry_path}'"
        );
    }
    format!(
        "could not resolve import '{module_name}'; tried entry-relative '{entry_path}' and workspace-relative {}",
        workspace_paths.join(", ")
    )
}

fn ambiguous_import_message(
    module_name: &str,
    resolver: &ModuleResolver,
    matches: &[PathBuf],
) -> String {
    let workspace_root = resolver
        .workspace
        .as_ref()
        .map(|workspace| workspace.workspace_root.display().to_string())
        .unwrap_or_default();
    let match_paths: Vec<String> = matches
        .iter()
        .map(|path| format!("'{}'", path.display()))
        .collect();
    format!(
        "module '{module_name}' is ambiguous in workspace '{workspace_root}': matches {}; reorder [source].roots or rename one module to disambiguate",
        match_paths.join(" and ")
    )
}

fn namespace_collision_message(
    module_name: &str,
    path: &Path,
    parent_name: &str,
    parent_path: &Path,
) -> String {
    format!(
        "module '{module_name}' resolves to file '{}' but parent name '{parent_name}' is also a module file '{}'; package directories are not supported",
        path.display(),
        parent_path.display()
    )
}

fn discovery_label(
    module_name: &str,
    path: &Path,
    diagnostic_style: DiscoveryDiagnosticStyle,
) -> String {
    match diagnostic_style {
        DiscoveryDiagnosticStyle::ModuleName => module_name.to_string(),
        DiscoveryDiagnosticStyle::FilePath => path.display().to_string(),
    }
}

fn collect_import_closure_module_dependencies(stmts: &[Stmt]) -> BTreeSet<String> {
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
        deps.insert(module_name);
    }
    deps
}

#[cfg(test)]
pub(crate) fn parse_import_closure_modules(
    resolver: &ModuleResolver,
    root_modules: &BTreeSet<String>,
    diagnostic_style: DiscoveryDiagnosticStyle,
) -> Result<HashMap<String, Vec<Stmt>>, Vec<RenderedDiagnostic>> {
    parse_import_closure_source_modules(resolver, root_modules, diagnostic_style).map(|modules| {
        modules
            .into_iter()
            .map(|(name, module)| (name, module.suite))
            .collect()
    })
}

pub(crate) fn parse_import_closure_source_modules(
    resolver: &ModuleResolver,
    root_modules: &BTreeSet<String>,
    diagnostic_style: DiscoveryDiagnosticStyle,
) -> Result<HashMap<String, ParsedProjectModule>, Vec<RenderedDiagnostic>> {
    let mut parsed_modules: HashMap<String, ParsedProjectModule> = HashMap::new();
    let mut parsed_names: BTreeSet<String> = BTreeSet::new();
    let mut pending = root_modules.clone();

    while let Some(module_name) = pending.pop_first() {
        if !parsed_names.insert(module_name.clone()) {
            continue;
        }

        let path = match resolver.resolve(&module_name) {
            Ok(resolved) => resolved.path,
            Err(error) if resolver.has_workspace() => {
                return Err(vec![error.to_diagnostic(resolver)]);
            }
            Err(error) => error
                .tried_paths
                .into_iter()
                .next()
                .unwrap_or_else(|| resolver.module_source_path(&module_name)),
        };
        let source = std::fs::read_to_string(&path).map_err(|e| {
            vec![crate::diagnostics::diagnostic_with_code(
                format!("failed to read '{}': {}", path.display(), e),
                DiagnosticCode::BUILD_MATERIALIZATION_FAILURE,
            )]
        })?;
        let label = discovery_label(&module_name, &path, diagnostic_style);
        let suite = sifr_frontend::parse_source(&source, Some(&label))?;
        for dependency in collect_import_closure_module_dependencies(&suite) {
            if parsed_names.contains(dependency.as_str()) {
                continue;
            }
            match resolver.resolve(&dependency) {
                Ok(_) => {
                    pending.insert(dependency);
                }
                Err(error) if resolver.has_workspace() => {
                    return Err(vec![error.to_diagnostic(resolver)]);
                }
                Err(_) => {}
            }
        }
        parsed_modules.insert(
            module_name,
            ParsedProjectModule {
                suite,
                source,
                display_path: path.display().to_string(),
            },
        );
    }

    Ok(parsed_modules)
}
