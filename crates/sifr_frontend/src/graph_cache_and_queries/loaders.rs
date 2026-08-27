use super::{
    AuxiliarySourceState, BTreeMap, BTreeSet, DiagnosticCode, ExternalDefs, FileId,
    FrontendContext, FrontendInput, FrontendMode, FrontendProjectModule, FrontendReuseCaches,
    GraphRevision, ModuleId, ProjectRoot, RenderedDiagnostic, SourcePath, SourceProvider,
    SourceRevision, SourceText, WorkspaceAuxiliarySource, WorkspaceCompilerOptions,
    WorkspacePackageConfigIdentity, WorkspaceSessionTarget, WorkspaceSingleFileTarget,
    diagnostic_with_code, module_state,
};

fn auxiliary_source_states(
    module_count: usize,
    sources: Vec<WorkspaceAuxiliarySource>,
) -> Result<Vec<AuxiliarySourceState>, Vec<RenderedDiagnostic>> {
    sources
        .into_iter()
        .enumerate()
        .map(|(offset, source)| {
            let file_id = u32::try_from(
                module_count
                    .checked_add(offset)
                    .ok_or_else(too_many_source_files_diagnostic)?,
            )
            .map_err(|_| too_many_source_files_diagnostic())?;
            Ok(AuxiliarySourceState::new(FileId(file_id), source))
        })
        .collect()
}

fn too_many_source_files_diagnostic() -> Vec<RenderedDiagnostic> {
    vec![diagnostic_with_code(
        "workspace has too many source files for frontend file identity space",
        DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
    )]
}

impl FrontendContext {
    pub fn load_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::load_single_file_with_external_defs(input, ExternalDefs::default())
    }

    pub fn load_single_file_with_external_defs(
        input: FrontendInput,
        external_defs: ExternalDefs,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::load_single_file_with_external_defs_and_auxiliary_sources(
            input,
            external_defs,
            Vec::new(),
        )
    }

    pub fn load_single_file_with_external_defs_and_auxiliary_sources(
        input: FrontendInput,
        external_defs: ExternalDefs,
        auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let cache_target = WorkspaceSessionTarget::SingleFile(WorkspaceSingleFileTarget {
            path: input.path.clone(),
            mode: input.mode,
        });
        let compiler_options = WorkspaceCompilerOptions { mode: input.mode };
        let module = module_state(
            ModuleId(0),
            FileId(0),
            "main",
            input.path,
            input.source,
            None,
        );
        let mut context = Self {
            modules: vec![module],
            module_by_id: BTreeMap::from([(ModuleId(0), 0)]),
            entrypoint: ModuleId(0),
            edges: Vec::new(),
            reverse_edges: BTreeMap::new(),
            graph_revision: GraphRevision(0),
            source_revision: SourceRevision(0),
            auxiliary_sources: auxiliary_source_states(1, auxiliary_sources)?,
            module_graph_cache: None,
            source_map_cache: None,
            reuse_caches: FrontendReuseCaches::new(),
            cache_target,
            compiler_options,
            package_config_identity: WorkspacePackageConfigIdentity::default(),
            base_external_defs: external_defs.clone(),
            external_defs,
            lowering_modules: BTreeSet::new(),
            compilation_suites: BTreeMap::new(),
            project_compile_order: None,
        };
        context.rebuild_edges();
        Ok(context)
    }

    pub fn load_project(
        root: &ProjectRoot,
        provider: &mut impl SourceProvider,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::load_project_with_external_defs(root, provider, ExternalDefs::default())
    }

    pub fn load_project_with_external_defs(
        root: &ProjectRoot,
        provider: &mut impl SourceProvider,
        external_defs: ExternalDefs,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::load_project_with_external_defs_and_auxiliary_sources(
            root,
            provider,
            external_defs,
            Vec::new(),
        )
    }

    pub fn load_project_with_external_defs_and_auxiliary_sources(
        root: &ProjectRoot,
        provider: &mut impl SourceProvider,
        external_defs: ExternalDefs,
        auxiliary_sources: Vec<WorkspaceAuxiliarySource>,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        let entrypoint = root.entrypoint.as_path();
        let project_dir = root.root.as_path();
        let entry_source = provider.read_file(entrypoint).map_err(|error| {
            vec![diagnostic_with_code(
                format!(
                    "failed to read project entrypoint '{}': {error}",
                    entrypoint.display()
                ),
                DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
            )]
        })?;
        let mut files = vec![entrypoint.to_path_buf()];
        for entry in provider.read_dir(project_dir).map_err(|error| {
            vec![diagnostic_with_code(
                format!(
                    "failed to read project root '{}': {error}",
                    project_dir.display()
                ),
                DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
            )]
        })? {
            let path = entry.path;
            if path.extension().is_some_and(|ext| ext == "sifr") && path != entrypoint {
                files.push(path);
            }
        }
        files.sort();
        files.dedup();
        files.retain(|path| path != entrypoint);
        files.insert(0, entrypoint.to_path_buf());

        let mut modules = Vec::with_capacity(files.len());
        for (idx, path) in files.into_iter().enumerate() {
            let numeric_id = u32::try_from(idx).map_err(|_| {
                vec![diagnostic_with_code(
                    "project has too many modules for frontend module identity space",
                    DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
                )]
            })?;
            let source = if path == entrypoint {
                entry_source.clone()
            } else {
                provider.read_file(&path).map_err(|error| {
                    vec![diagnostic_with_code(
                        format!(
                            "failed to read project module '{}': {error}",
                            path.display()
                        ),
                        DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
                    )]
                })?
            };
            let module_name = if path == entrypoint {
                "main".to_string()
            } else {
                path.file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("module_{idx}"))
            };
            modules.push(module_state(
                ModuleId(numeric_id),
                FileId(numeric_id),
                module_name,
                SourcePath::new(path),
                source,
                None,
            ));
        }

        let module_by_id = modules
            .iter()
            .enumerate()
            .map(|(index, module)| (module.id, index))
            .collect();
        let cache_target = WorkspaceSessionTarget::Project(root.clone());
        let compiler_options = WorkspaceCompilerOptions {
            mode: FrontendMode::ProjectEntrypoint,
        };
        let package_config_identity = WorkspacePackageConfigIdentity {
            workspace_root: Some(root.root.clone()),
            entrypoint: Some(root.entrypoint.clone()),
        };
        let mut context = Self {
            auxiliary_sources: auxiliary_source_states(modules.len(), auxiliary_sources)?,
            modules,
            module_by_id,
            entrypoint: ModuleId(0),
            edges: Vec::new(),
            reverse_edges: BTreeMap::new(),
            graph_revision: GraphRevision(0),
            source_revision: SourceRevision(0),
            module_graph_cache: None,
            source_map_cache: None,
            reuse_caches: FrontendReuseCaches::new(),
            cache_target,
            compiler_options,
            package_config_identity,
            base_external_defs: external_defs.clone(),
            external_defs,
            lowering_modules: BTreeSet::new(),
            compilation_suites: BTreeMap::new(),
            project_compile_order: None,
        };
        context.rebuild_edges();
        Ok(context)
    }

    pub fn load_project_modules(
        project_modules: &BTreeMap<String, FrontendProjectModule>,
        external_defs: ExternalDefs,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
        if project_modules.is_empty() {
            return Err(vec![diagnostic_with_code(
                "project compilation requires at least one source module",
                DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
            )]);
        }

        let mut ordered_names = project_modules.keys().cloned().collect::<Vec<_>>();
        ordered_names.sort();
        if let Some(main_index) = ordered_names.iter().position(|name| name == "main") {
            let main = ordered_names.remove(main_index);
            ordered_names.insert(0, main);
        }

        let mut modules = Vec::with_capacity(ordered_names.len());
        let mut compilation_suites = BTreeMap::new();
        for (index, module_name) in ordered_names.into_iter().enumerate() {
            let numeric_id = u32::try_from(index).map_err(|_| {
                vec![diagnostic_with_code(
                    "project has too many modules for frontend module identity space",
                    DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
                )]
            })?;
            let Some(module) = project_modules.get(&module_name) else {
                return Err(vec![diagnostic_with_code(
                    format!("project module '{module_name}' disappeared during frontend loading"),
                    DiagnosticCode::INTERNAL_COMPILER_PANIC,
                )]);
            };
            let module_id = ModuleId(numeric_id);
            compilation_suites.insert(module_id, std::sync::Arc::new(module.suite.clone()));
            modules.push(module_state(
                module_id,
                FileId(numeric_id),
                module_name,
                SourcePath::new(module.display_path.clone()),
                SourceText::new(module.source.clone()),
                None,
            ));
        }

        let module_by_id = modules
            .iter()
            .enumerate()
            .map(|(index, module)| (module.id, index))
            .collect();
        let entrypoint = modules[0].id;
        let entrypoint_path = modules[0].path.clone();
        let cache_target = WorkspaceSessionTarget::SingleFile(WorkspaceSingleFileTarget {
            path: entrypoint_path,
            mode: FrontendMode::ProjectEntrypoint,
        });
        let compiler_options = WorkspaceCompilerOptions {
            mode: FrontendMode::ProjectEntrypoint,
        };
        let mut context = Self {
            modules,
            module_by_id,
            entrypoint,
            edges: Vec::new(),
            reverse_edges: BTreeMap::new(),
            graph_revision: GraphRevision(0),
            source_revision: SourceRevision(0),
            auxiliary_sources: Vec::new(),
            module_graph_cache: None,
            source_map_cache: None,
            reuse_caches: FrontendReuseCaches::new(),
            cache_target,
            compiler_options,
            package_config_identity: WorkspacePackageConfigIdentity::default(),
            base_external_defs: external_defs.clone(),
            external_defs,
            lowering_modules: BTreeSet::new(),
            compilation_suites,
            project_compile_order: None,
        };
        context.rebuild_edges();
        Ok(context)
    }
}
