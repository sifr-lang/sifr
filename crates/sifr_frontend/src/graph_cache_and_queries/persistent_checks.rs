//! Restore only the completed diagnostics family. Rich queries still lower.
use super::{
    Arc, BTreeSet, ExternalDefs, FileId, FrontendContext, FrontendInput, FrontendMode, ModuleId,
    ProjectRoot, SourcePath, SourceProvider, SourceText, WorkspaceSessionTarget, module_state,
};
use crate::persistence::{CompletedCheck, Observation, SemanticInputs, SourceOutcome};

#[derive(Clone, Debug, serde::Serialize)]
pub struct ModuleCheckDecision {
    pub path: std::path::PathBuf,
    pub family: &'static str,
    pub action: &'static str,
    pub reason: &'static str,
}

impl FrontendContext {
    /// The caller supplies its pinned semantic context. All source/resolver
    /// observations are revalidated, including absent candidates. Saved results
    /// never replace a different editor overlay.
    pub fn restore_completed_checks(
        &mut self,
        record: &CompletedCheck,
        semantic: &SemanticInputs,
        provider: &mut dyn SourceProvider,
        allow_body_edits: bool,
    ) -> Option<Vec<ModuleCheckDecision>> {
        if record.result.outcome != SourceOutcome::Success
            || !record.diagnostics().ok()?.is_empty()
            || self.compiler_identity.as_str() != semantic.compiler
        {
            return None;
        }
        let saved = record.result.resolution.ready()?;
        if saved
            .sources
            .iter()
            .filter(|source| source.path.extension().is_some_and(|ext| ext == "sifr"))
            .count()
            != self.modules.len()
            || !self.auxiliary_sources.is_empty()
        {
            return None;
        }
        let mut updated = record.clone();
        let mut changed = Vec::new();
        let mut current_sources = saved.sources.clone();
        for module in &self.modules {
            let old = saved
                .sources
                .iter()
                .find(|source| source.path == module.path.as_path())?;
            let current = crate::persistence::CapturedSource {
                path: old.path.clone(),
                text: module.source.as_str().to_owned(),
            };
            if old.text != current.text {
                if !allow_body_edits {
                    return None;
                }
                let old_ast =
                    sifr_syntax::parse_module(&old.text, Some(&module.module_name)).ok()?;
                let new_ast =
                    sifr_syntax::parse_module(&current.text, Some(&module.module_name)).ok()?;
                if crate::module_signatures::interface_projection(old_ast.suite())
                    != crate::module_signatures::interface_projection(new_ast.suite())
                {
                    return None;
                }
                changed.push(module.id);
            }
            let slot = current_sources
                .iter_mut()
                .find(|source| source.path == current.path)?;
            *slot = current;
        }
        // Rewrite only the file observations whose semantic equivalence is
        // proved above. Search order, absence, config and context remain exact.
        for observation in &mut updated.result.inputs.observations {
            if let Observation::File { path, identity } = observation {
                if let Some(source) = current_sources.iter().find(|source| &source.path == path) {
                    *identity = source.identity();
                }
            }
        }
        updated.result.inputs.source = current_sources
            .iter()
            .find(|source| source.path == record.result.inputs.source.path)?
            .clone();
        if let crate::persistence::Family::Complete(resolution) = &mut updated.result.resolution {
            resolution.sources = current_sources;
            resolution
                .observations
                .clone_from(&updated.result.inputs.observations);
        }
        if !updated.validate(semantic, provider) {
            return None;
        }
        // Interface equivalence is not a substitute for checking changed code.
        // Failed checking cannot leave a successful importer result installed.
        for module in changed {
            if !self
                .diagnostics_for_module(module)
                .value()
                .diagnostics
                .is_empty()
            {
                return None;
            }
        }
        let mut decisions = Vec::new();
        for module in &mut self.modules {
            let computed = module.lowered.is_some() || module.diagnostics.is_some();
            if !computed {
                module.diagnostics = Some(Arc::new(Vec::new()));
            }
            decisions.push(ModuleCheckDecision {
                path: module.path.as_path().to_owned(),
                family: "diagnostics",
                action: if computed { "computed" } else { "restored" },
                reason: if computed {
                    "changed-source-or-required-dependency"
                } else {
                    "unchanged-source-and-proven-consumed-interface"
                },
            });
        }
        Some(decisions)
    }
}

impl FrontendContext {
    /// Construct the existing query context from an already resolved flat source
    /// inventory. The persistence adapter must validate the resolver observations
    /// before using any restored result; this function performs no disk discovery.
    pub fn from_resolved_sources(
        root: ProjectRoot,
        sources: Vec<crate::persistence::CapturedSource>,
        defs: ExternalDefs,
    ) -> Option<Self> {
        let entry = sources
            .iter()
            .find(|source| source.path == root.entrypoint.as_path())?;
        let mut context = Self::load_single_file_with_external_defs(
            FrontendInput {
                path: root.entrypoint.clone(),
                source: SourceText::new(entry.text.clone()),
                mode: FrontendMode::ProjectEntrypoint,
            },
            defs,
        )
        .ok()?;
        let mut rest: Vec<_> = sources
            .into_iter()
            .filter(|source| source.path != root.entrypoint.as_path())
            .collect();
        rest.sort_by(|a, b| a.path.cmp(&b.path));
        let mut names = BTreeSet::from(["main".to_owned()]);
        for source in rest {
            if source.path.parent() != root.entrypoint.as_path().parent() {
                return None;
            }
            let name = source.path.file_stem()?.to_str()?.to_owned();
            if !names.insert(name.clone()) {
                return None;
            }
            let id = u32::try_from(context.modules.len()).ok()?;
            context.modules.push(module_state(
                ModuleId(id),
                FileId::new(id),
                &name,
                SourcePath::new(source.path),
                SourceText::new(source.text),
                None,
            ));
            context.module_by_id.insert(ModuleId(id), id as usize);
        }
        context.cache_target = WorkspaceSessionTarget::Project(root);
        context.rebuild_edges();
        Some(context)
    }
}

/// The existing package resolver owns names and rewritten imports; original
/// source bytes remain the authority for persistence and diagnostics.
pub struct ResolvedCheckModule {
    pub name: String,
    pub source: crate::persistence::CapturedSource,
    pub suite: sifr_python_ast::Suite,
}

impl FrontendContext {
    /// Adapt an already resolved package closure to the existing checking
    /// queries. This performs neither package discovery nor an alternate check.
    pub fn from_resolved_check_modules(
        root: ProjectRoot,
        modules: Vec<ResolvedCheckModule>,
        defs: ExternalDefs,
    ) -> Option<Self> {
        let entry = modules
            .iter()
            .find(|module| module.source.path == root.entrypoint.as_path())?;
        let mut context = Self::load_single_file_with_external_defs(
            FrontendInput {
                path: root.entrypoint.clone(),
                source: SourceText::new(entry.source.text.clone()),
                mode: FrontendMode::ProjectEntrypoint,
            },
            defs,
        )
        .ok()?;
        context.modules.clear();
        context.module_by_id.clear();
        let mut names = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for module in modules {
            if !names.insert(module.name.clone()) || !paths.insert(module.source.path.clone()) {
                // Multiple logical aliases for one source are not represented
                // by the current completed-check source inventory.
                return None;
            }
            let id = ModuleId(u32::try_from(context.modules.len()).ok()?);
            let parsed = sifr_syntax::parse_module(&module.source.text, Some(&module.name))
                .ok()?
                .with_resolved_suite(module.suite);
            if module.source.path == root.entrypoint.as_path() {
                context.entrypoint = id;
            }
            let mut state = module_state(
                id,
                FileId(id.as_u32()),
                module.name,
                SourcePath::new(module.source.path),
                SourceText::new(module.source.text),
                None,
            );
            state.parsed = Some(Arc::new(parsed));
            context.module_by_id.insert(id, context.modules.len());
            context.modules.push(state);
        }
        context.cache_target = WorkspaceSessionTarget::Project(root);
        context.rebuild_edges();
        Some(context)
    }
}
