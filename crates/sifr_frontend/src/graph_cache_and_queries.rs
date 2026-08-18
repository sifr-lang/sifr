use super::{
    collect_module_exports, diagnostic_with_code, editor_semantics_from_module,
    hir_diagnostic_to_rendered, local_import_dependencies, module_state, reveal_type_diagnostics,
    source_hash, symbols_from_hir, warning_diagnostics, CacheFamily, CacheKeyContext,
    DiagnosticsCacheKey, DiskSourceProvider, DocumentVersion, FileId, HirLoweringCacheKey,
    ModuleAnalysisView, ParseCacheKey, ProjectAnalysisView, SourceDependency, SourceFileView,
    SourceHash, SourceMapCacheKey, SourceMapView, SourceOrigin, SourcePath, SourceProvider,
    SourceRevision, SourceText, SymbolBucketScope, SymbolBucketsCacheKey, TrackingSourceProvider,
    WorkspaceAuxiliarySource, WorkspaceCompilerOptions, WorkspaceDirtyReason, WorkspaceDirtyScope,
    WorkspaceDirtyScopeReport, WorkspacePackageConfigIdentity, WorkspaceSessionTarget,
    WorkspaceSingleFileTarget,
};
use crate::frontend_reuse::FrontendReuseCaches;
use crate::module_signatures::{module_signature, ModuleSignature};
use crate::source_maps::AuxiliarySourceState;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_lowering::{
    lower_module_with_externals_name_and_options, ExternalDefs, HirModule, LoweringOptions,
    LoweringResult, LoweringWarningDiagnostic, RevealTypeDiagnostic,
};
use sifr_python_ast::Stmt;
use sifr_syntax::ParsedModule;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;
use std::path::Path;
use std::sync::Arc;

mod loaders;
mod reuse;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(pub(crate) u32);

impl ModuleId {
    #[doc(hidden)]
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphRevision(u64);

impl GraphRevision {
    #[doc(hidden)]
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendMode {
    SingleFile,
    ProjectEntrypoint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendInput {
    pub path: SourcePath,
    pub source: SourceText,
    pub mode: FrontendMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectRoot {
    pub root: SourcePath,
    pub entrypoint: SourcePath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryKind {
    Parse,
    Lower,
    TypeCheck,
    ModuleDiagnostics,
    ProjectDiagnostics,
    ModuleAnalysis,
    ProjectAnalysis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheStatus {
    Hit,
    Miss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryMetadata {
    pub query: QueryKind,
    pub cache_status: CacheStatus,
    pub graph_revision: GraphRevision,
    pub source_revision: SourceRevision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryResult<T> {
    value: T,
    metadata: QueryMetadata,
}

impl<T> QueryResult<T> {
    #[must_use]
    pub fn new(value: T, metadata: QueryMetadata) -> Self {
        Self { value, metadata }
    }

    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    #[must_use]
    pub fn metadata(&self) -> &QueryMetadata {
        &self.metadata
    }

    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleGraphNode {
    pub id: ModuleId,
    pub file: FileId,
    pub canonical_path: SourcePath,
    pub source_hash: SourceHash,
    pub origin: SourceOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleGraphEdge {
    pub importer: ModuleId,
    pub imported: ModuleId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleGraphView {
    pub modules: Vec<ModuleGraphNode>,
    pub edges: Vec<ModuleGraphEdge>,
    pub entrypoint: ModuleId,
    pub revision: GraphRevision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdatedDocumentInfo {
    pub file: FileId,
    pub old_version: Option<DocumentVersion>,
    pub new_version: Option<DocumentVersion>,
    pub text_changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidationReport {
    pub previous_revision: GraphRevision,
    pub next_revision: GraphRevision,
    pub invalidated_modules: Vec<ModuleId>,
    pub invalidated_queries: Vec<QueryKind>,
    pub updated_documents: Vec<UpdatedDocumentInfo>,
    pub dirty_scope_report: WorkspaceDirtyScopeReport,
}

#[derive(Clone, Debug)]
pub struct ParsedModuleView {
    pub module: ModuleId,
    pub parsed: ParsedModule,
}

#[derive(Clone, Debug)]
pub struct LoweredModuleView {
    pub module: ModuleId,
    pub hir: HirModule,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrontendModuleDiagnostics {
    pub reveal_types: Vec<RevealTypeDiagnostic>,
    pub rendered_reveal_types: Vec<RenderedDiagnostic>,
    pub warnings: Vec<LoweringWarningDiagnostic>,
    pub rendered_warnings: Vec<RenderedDiagnostic>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendDiagnosticStyle {
    Bare,
    ModulePrefixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendSourceContext<'a> {
    pub display_path: &'a str,
    pub source: &'a str,
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

pub(super) struct ModuleState {
    pub(super) id: ModuleId,
    pub(super) file: FileId,
    pub(super) module_name: String,
    pub(super) path: SourcePath,
    pub(super) source: SourceText,
    pub(super) source_hash: SourceHash,
    pub(super) document_version: Option<DocumentVersion>,
    pub(super) signature: ModuleSignature,
    pub(super) source_file_view: Option<Arc<SourceFileView>>,
    pub(super) parsed: Option<Arc<ParsedModule>>,
    pub(super) lowered: Option<Arc<LoweringResult>>,
    pub(super) diagnostics: Option<Arc<Vec<RenderedDiagnostic>>>,
    pub(super) analysis: Option<Arc<ModuleAnalysisView>>,
}

pub struct FrontendContext {
    modules: Vec<ModuleState>,
    module_by_id: BTreeMap<ModuleId, usize>,
    entrypoint: ModuleId,
    edges: Vec<ModuleGraphEdge>,
    reverse_edges: BTreeMap<ModuleId, Vec<ModuleId>>,
    graph_revision: GraphRevision,
    source_revision: SourceRevision,
    auxiliary_sources: Vec<AuxiliarySourceState>,
    module_graph_cache: Option<Arc<ModuleGraphView>>,
    source_map_cache: Option<Arc<SourceMapView>>,
    reuse_caches: FrontendReuseCaches,
    cache_target: WorkspaceSessionTarget,
    compiler_options: WorkspaceCompilerOptions,
    package_config_identity: WorkspacePackageConfigIdentity,
    base_external_defs: ExternalDefs,
    external_defs: ExternalDefs,
    lowering_modules: BTreeSet<ModuleId>,
}

pub fn parse_source(
    source: &str,
    context: Option<&str>,
) -> Result<Vec<Stmt>, Vec<RenderedDiagnostic>> {
    sifr_syntax::parse_module_suite(source, context)
}

pub fn parse_source_module(
    source: &str,
    context: Option<&str>,
) -> Result<sifr_syntax::ParsedModule, Vec<RenderedDiagnostic>> {
    sifr_syntax::parse_module(source, context)
}

pub fn compile_module_hir(
    module_name: &str,
    stmts: &[Stmt],
    external_defs: &ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    compile_module_hir_with_source(module_name, stmts, external_defs, diagnostic_style, None)
}

pub fn compile_module_hir_with_source(
    module_name: &str,
    stmts: &[Stmt],
    external_defs: &ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    compile_module_hir_with_source_and_options(
        module_name,
        stmts,
        external_defs,
        diagnostic_style,
        source_context,
        LoweringOptions::default(),
    )
}

pub fn compile_module_hir_with_source_and_options(
    module_name: &str,
    stmts: &[Stmt],
    external_defs: &ExternalDefs,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
    lowering_options: LoweringOptions,
) -> Result<LoweringResult, Vec<RenderedDiagnostic>> {
    // Collect source declarations before lowering finalizes class storage and
    // constructor shape. Later adapter stages consume this same representation.
    let class_declarations =
        crate::class_declarations::ClassDeclarationSet::collect(module_name, stmts);
    match lower_module_with_externals_name_and_options(
        module_name,
        stmts,
        external_defs,
        lowering_options.clone(),
    ) {
        Ok(mut result) => {
            if let Err(errors) =
                crate::typed_descriptors::collect(module_name, stmts, &mut result, external_defs)
            {
                return Err(errors
                    .into_iter()
                    .map(|error| {
                        hir_diagnostic_to_rendered(
                            module_name,
                            diagnostic_style,
                            source_context,
                            error,
                        )
                    })
                    .collect());
            }
            let lowering_warning_count = result.warnings.len();
            if let Err(errors) = crate::early_adapters::run(
                module_name,
                &mut result,
                external_defs,
                &class_declarations,
            ) {
                return Err(errors
                    .into_iter()
                    .map(|error| match error {
                        crate::package_issues::SpecializationDiagnostic::Hir(error) => {
                            hir_diagnostic_to_rendered(
                                module_name,
                                diagnostic_style,
                                source_context,
                                error,
                            )
                        }
                        crate::package_issues::SpecializationDiagnostic::Package(issue) => {
                            crate::package_issues::render_package_issue(
                                module_name,
                                diagnostic_style,
                                source_context,
                                &issue,
                            )
                        }
                    })
                    .collect());
            }
            if !result.class_adapter_selections.is_empty() {
                // The first lowering pass provides typed, provisional declarations to
                // adapters. Rebuild adapted modules afterward so the HIR consumed by
                // specialization/codegen is the finalized class representation. M4 can
                // apply validated field/default plans at this explicit boundary.
                let descriptors = std::mem::take(&mut result.declaration_descriptors);
                let applied_metadata = std::mem::take(&mut result.applied_adapter_metadata);
                let specialization_requests = std::mem::take(&mut result.specialization_requests);
                let adapter_warnings = result.warnings[lowering_warning_count..].to_vec();
                let mut final_options = lowering_options;
                final_options.adapter_field_plans = result
                    .class_adapter_selections
                    .iter()
                    .map(|selection| (selection.owner.clone(), selection.field_plans.clone()))
                    .collect();
                let applied_selections = result.class_adapter_selections.clone();
                result = match lower_module_with_externals_name_and_options(
                    module_name,
                    stmts,
                    external_defs,
                    final_options,
                ) {
                    Ok(mut finalized) => {
                        finalized.declaration_descriptors = descriptors;
                        finalized.applied_adapter_metadata = applied_metadata;
                        finalized.specialization_requests = specialization_requests;
                        finalized.class_adapter_selections = applied_selections;
                        finalized.warnings.extend(adapter_warnings);
                        finalized
                    }
                    Err(errors) => {
                        return Err(errors
                            .into_iter()
                            .map(|error| {
                                hir_diagnostic_to_rendered(
                                    module_name,
                                    diagnostic_style,
                                    source_context,
                                    error,
                                )
                            })
                            .collect());
                    }
                };
            }
            match crate::specialization_runner::run_specializations(
                module_name,
                &mut result,
                external_defs,
                &class_declarations,
            ) {
                Ok(()) => Ok(result),
                Err(errors) => Err(errors
                    .into_iter()
                    .map(|error| match error {
                        crate::package_issues::SpecializationDiagnostic::Hir(error) => {
                            hir_diagnostic_to_rendered(
                                module_name,
                                diagnostic_style,
                                source_context,
                                error,
                            )
                        }
                        crate::package_issues::SpecializationDiagnostic::Package(issue) => {
                            crate::package_issues::render_package_issue(
                                module_name,
                                diagnostic_style,
                                source_context,
                                &issue,
                            )
                        }
                    })
                    .collect()),
            }
        }
        Err(errors) => Err(errors
            .into_iter()
            .map(|error| {
                hir_diagnostic_to_rendered(module_name, diagnostic_style, source_context, error)
            })
            .collect()),
    }
}

impl FrontendContext {
    pub fn update_module_source(
        &mut self,
        module: ModuleId,
        source: SourceText,
        document_version: Option<DocumentVersion>,
    ) -> Result<InvalidationReport, Vec<RenderedDiagnostic>> {
        let Some(index) = self.module_by_id.get(&module).copied() else {
            return Err(vec![diagnostic_with_code(
                format!("unknown module id {}", module.as_u32()),
                DiagnosticCode::INTERNAL_COMPILER_PANIC,
            )]);
        };
        let previous_revision = self.graph_revision;
        let old_hash = self.modules[index].source_hash.clone();
        let new_hash = source_hash(source.as_str());
        let old_version = self.modules[index].document_version;
        let old_signature = self.modules[index].signature.clone();
        let file = self.modules[index].file;
        let path = self.modules[index].path.clone();
        let text_changed = old_hash != new_hash;
        let parsed =
            sifr_syntax::parse_module(source.as_str(), Some(&self.modules[index].module_name));
        let new_signature = parsed.as_ref().map_or_else(
            |_| ModuleSignature::default(),
            |parsed| module_signature(parsed.suite()),
        );
        self.modules[index].source = source;
        self.modules[index].source_hash = new_hash;
        self.modules[index].document_version = document_version;
        self.modules[index].signature = new_signature.clone();
        if text_changed {
            self.modules[index].source_file_view = None;
            self.source_revision.0 += 1;
            self.source_map_cache = None;
            self.module_graph_cache = None;
        }

        let mut invalidated_modules = Vec::new();
        let mut invalidated_queries = Vec::new();
        let dirty_scope_report = if text_changed {
            let imports_changed = old_signature.imports != new_signature.imports;
            let exports_changed = old_signature.exports != new_signature.exports;
            let parse_failed = parsed.is_err();
            let can_replace_module = Self::signatures_can_replace_module_in_project(
                &old_signature,
                &new_signature,
                parse_failed,
            );
            invalidated_modules = if can_replace_module {
                vec![module]
            } else {
                self.reverse_dependency_closure(module)
            };
            self.clear_module_caches(&invalidated_modules, &[module]);
            if !can_replace_module {
                self.external_defs = self.base_external_defs.clone();
                self.rebuild_external_defs_from_lowered();
                self.lowering_modules.clear();
                self.graph_revision.0 += 1;
                self.rebuild_edges();
            }
            invalidated_queries.extend([
                QueryKind::Parse,
                QueryKind::Lower,
                QueryKind::TypeCheck,
                QueryKind::ModuleDiagnostics,
                QueryKind::ProjectDiagnostics,
                QueryKind::ModuleAnalysis,
                QueryKind::ProjectAnalysis,
            ]);
            let mut reasons = vec![WorkspaceDirtyReason::SourceTextChanged];
            if imports_changed {
                reasons.push(WorkspaceDirtyReason::ImportSignatureChanged);
            }
            if exports_changed {
                reasons.push(WorkspaceDirtyReason::ExportSignatureChanged);
            }
            if parse_failed {
                reasons.push(WorkspaceDirtyReason::Unknown);
            }
            let scope = if parse_failed || imports_changed {
                WorkspaceDirtyScope::GraphStructure
            } else if exports_changed {
                WorkspaceDirtyScope::ReverseDependencies { path }
            } else {
                WorkspaceDirtyScope::OneModule { path }
            };
            WorkspaceDirtyScopeReport::new(scope, reasons)
        } else {
            WorkspaceDirtyScopeReport::new(
                WorkspaceDirtyScope::None,
                vec![WorkspaceDirtyReason::DocumentVersionOnly],
            )
        };
        self.reuse_caches.prune_unshared();

        Ok(InvalidationReport {
            previous_revision,
            next_revision: self.graph_revision,
            invalidated_modules,
            invalidated_queries,
            updated_documents: vec![UpdatedDocumentInfo {
                file,
                old_version,
                new_version: document_version,
                text_changed,
            }],
            dirty_scope_report,
        })
    }

    #[must_use]
    pub fn module_graph(&self) -> ModuleGraphView {
        self.module_graph_view()
    }

    #[must_use]
    pub fn source_map(&self) -> SourceMapView {
        self.source_map_view()
    }

    #[must_use]
    pub fn module_for_file(&self, file: FileId) -> Option<ModuleId> {
        self.modules
            .iter()
            .find(|module| module.file == file)
            .map(|module| module.id)
    }

    #[must_use]
    pub fn source_text_for_file(&self, file: FileId) -> Option<&str> {
        if let Some(module) = self.module_for_file(file) {
            let index = self.module_by_id.get(&module).copied()?;
            return Some(self.modules[index].source.as_str());
        }
        self.auxiliary_sources
            .iter()
            .find(|source| source.source_file.id == file)
            .map(|source| source.source_file.source.as_str())
    }

    #[must_use]
    pub fn path_for_file(&self, file: FileId) -> Option<&Path> {
        if let Some(module) = self.module_for_file(file) {
            let index = self.module_by_id.get(&module).copied()?;
            return Some(self.modules[index].path.as_path());
        }
        self.auxiliary_sources
            .iter()
            .find(|source| source.source_file.id == file)
            .map(|source| source.source_file.canonical_path.as_path())
    }

    #[must_use]
    pub fn source_file_for_file(&self, file: FileId) -> Option<&SourceFileView> {
        if let Some(module) = self.module_for_file(file) {
            let index = self.module_by_id.get(&module).copied()?;
            return self.modules[index].source_file_view.as_deref();
        }
        self.auxiliary_sources
            .iter()
            .find(|source| source.source_file.id == file)
            .map(|source| &source.source_file)
    }

    #[must_use]
    pub fn document_version_for_file(&self, file: FileId) -> Option<DocumentVersion> {
        let module = self.module_for_file(file)?;
        let index = self.module_by_id.get(&module).copied()?;
        self.modules[index].document_version
    }

    fn index_for_module(&self, module: ModuleId) -> usize {
        self.module_by_id
            .get(&module)
            .copied()
            .unwrap_or_else(|| panic!("unknown module id {}", module.as_u32()))
    }

    fn metadata(&self, query: QueryKind, cache_status: CacheStatus) -> QueryMetadata {
        QueryMetadata {
            query,
            cache_status,
            graph_revision: self.graph_revision,
            source_revision: self.source_revision,
        }
    }

    fn ensure_parsed(&mut self, module: ModuleId) -> Result<CacheStatus, Vec<RenderedDiagnostic>> {
        let index = self.index_for_module(module);
        if self.modules[index].parsed.is_some() {
            return Ok(CacheStatus::Hit);
        }
        let key = self.parse_key_fingerprint(index);
        if let Some(parsed) = self.reuse_caches.parse(&key) {
            self.modules[index].signature = module_signature(parsed.suite());
            self.modules[index].parsed = Some(parsed);
            return Ok(CacheStatus::Hit);
        }
        let parsed = sifr_syntax::parse_module(
            self.modules[index].source.as_str(),
            Some(&self.modules[index].module_name),
        )?;
        self.modules[index].signature = module_signature(parsed.suite());
        self.modules[index].parsed = Some(self.reuse_caches.insert_parse(key, parsed));
        Ok(CacheStatus::Miss)
    }

    fn ensure_lowered(&mut self, module: ModuleId) -> CacheStatus {
        let index = self.index_for_module(module);
        if self.modules[index].lowered.is_some() {
            return CacheStatus::Hit;
        }
        if !self.lowering_modules.insert(module) {
            return CacheStatus::Miss;
        }
        let parsed_status = self.ensure_parsed(module);
        let index = self.index_for_module(module);
        let parsed = match parsed_status {
            Ok(_) => self.modules[index]
                .parsed
                .as_ref()
                .map(|parsed| parsed.suite().to_vec())
                .unwrap_or_default(),
            Err(errors) => {
                self.modules[index].diagnostics = Some(Arc::new(errors));
                self.lowering_modules.remove(&module);
                return CacheStatus::Miss;
            }
        };
        self.modules[index].signature = module_signature(&parsed);
        let module_names: BTreeMap<String, ModuleId> = self
            .modules
            .iter()
            .map(|module| (module.module_name.clone(), module.id))
            .collect();
        for dependency in local_import_dependencies(&parsed, &module_names) {
            let _ = self.ensure_lowered(dependency);
        }
        let index = self.index_for_module(module);
        let hir_key = self.hir_key_fingerprint(index);
        if let Some(lowered) = self.reuse_caches.hir(&hir_key) {
            let module_name = self.modules[index].module_name.clone();
            collect_module_exports(&module_name, &lowered, &mut self.external_defs);
            self.modules[index].lowered = Some(lowered);
            self.lowering_modules.remove(&module);
            return CacheStatus::Hit;
        }
        match compile_module_hir_with_source(
            &self.modules[index].module_name,
            &parsed,
            &self.external_defs,
            FrontendDiagnosticStyle::Bare,
            Some(FrontendSourceContext {
                display_path: &self.modules[index].module_name,
                source: self.modules[index].source.as_str(),
            }),
        ) {
            Ok(lowered) => {
                let module_name = self.modules[index].module_name.clone();
                collect_module_exports(&module_name, &lowered, &mut self.external_defs);
                self.modules[index].lowered = Some(self.reuse_caches.insert_hir(hir_key, lowered));
            }
            Err(errors) => {
                self.modules[index].diagnostics = Some(Arc::new(errors));
            }
        }
        self.lowering_modules.remove(&module);
        CacheStatus::Miss
    }

    fn ensure_diagnostics(&mut self, module: ModuleId) -> CacheStatus {
        let index = self.index_for_module(module);
        if self.modules[index].diagnostics.is_some() {
            return CacheStatus::Hit;
        }
        let _ = self.ensure_lowered(module);
        let index = self.index_for_module(module);
        if self.modules[index].diagnostics.is_none() {
            let diagnostics_key = self.diagnostics_key_fingerprint(index);
            if let Some(diagnostics) = self.reuse_caches.diagnostics(&diagnostics_key) {
                self.modules[index].diagnostics = Some(diagnostics);
                return CacheStatus::Hit;
            }
            let diagnostics = self.modules[index]
                .lowered
                .as_ref()
                .map(|lowered| {
                    let source_context = FrontendSourceContext {
                        display_path: &self.modules[index].module_name,
                        source: self.modules[index].source.as_str(),
                    };
                    let mut diagnostics =
                        warning_diagnostics(Some(source_context), &lowered.warnings);
                    diagnostics.extend(reveal_type_diagnostics(
                        Some(source_context),
                        &lowered.reveal_types,
                    ));
                    diagnostics
                })
                .unwrap_or_default();
            self.modules[index].diagnostics = Some(
                self.reuse_caches
                    .insert_diagnostics(diagnostics_key, diagnostics),
            );
            return CacheStatus::Miss;
        }
        CacheStatus::Miss
    }

    fn ensure_analysis(&mut self, module: ModuleId) -> CacheStatus {
        let index = self.index_for_module(module);
        if self.modules[index].analysis.is_some() {
            return CacheStatus::Hit;
        }
        let _ = self.ensure_lowered(module);
        let index = self.index_for_module(module);
        let index_key = self.index_key_fingerprint(index);
        if let Some(analysis) = self.reuse_caches.index(&index_key) {
            self.modules[index].analysis = Some(analysis);
            return CacheStatus::Hit;
        }
        let symbols = self.modules[index]
            .lowered
            .as_ref()
            .map(|lowered| symbols_from_hir(&lowered.module))
            .unwrap_or_default();
        let editor_semantics = self.modules[index]
            .lowered
            .as_ref()
            .and_then(|lowered| {
                self.modules[index].parsed.as_ref().map(|parsed| {
                    editor_semantics_from_module(
                        module,
                        &self.modules[index].module_name,
                        self.modules[index].source.as_str(),
                        parsed.suite(),
                        &lowered.module,
                        &self.external_defs,
                    )
                })
            })
            .unwrap_or_default();
        self.modules[index].analysis = Some(self.reuse_caches.insert_index(
            index_key,
            ModuleAnalysisView {
                module,
                symbols,
                editor_semantics,
            },
        ));
        CacheStatus::Miss
    }

    fn rebuild_edges(&mut self) {
        let module_names: BTreeMap<String, ModuleId> = self
            .modules
            .iter()
            .map(|module| (module.module_name.clone(), module.id))
            .collect();
        let mut edges = BTreeSet::new();
        for index in 0..self.modules.len() {
            let module = &self.modules[index];
            if let Ok(parsed) =
                sifr_syntax::parse_module(module.source.as_str(), Some(&module.module_name))
            {
                self.modules[index].signature = module_signature(parsed.suite());
                for import in local_import_dependencies(parsed.suite(), &module_names) {
                    edges.insert((self.modules[index].id, import));
                }
            }
        }
        self.edges = edges
            .into_iter()
            .map(|(importer, imported)| ModuleGraphEdge { importer, imported })
            .collect();
        self.reverse_edges = BTreeMap::new();
        for edge in &self.edges {
            self.reverse_edges
                .entry(edge.imported)
                .or_default()
                .push(edge.importer);
        }
        for dependents in self.reverse_edges.values_mut() {
            dependents.sort();
            dependents.dedup();
        }
    }

    fn reverse_dependency_closure(&self, module: ModuleId) -> Vec<ModuleId> {
        let mut seen = BTreeSet::from([module]);
        let mut queue = vec![module];
        while let Some(current) = queue.pop() {
            if let Some(dependents) = self.reverse_edges.get(&current) {
                for dependent in dependents {
                    if seen.insert(*dependent) {
                        queue.push(*dependent);
                    }
                }
            }
        }
        seen.into_iter().collect()
    }

    fn clear_module_caches(
        &mut self,
        modules: &[ModuleId],
        modules_with_source_changes: &[ModuleId],
    ) {
        let clear_parse_modules = modules_with_source_changes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        for module in modules {
            let index = self.index_for_module(*module);
            let module_state = &mut self.modules[index];
            if clear_parse_modules.contains(module) {
                module_state.parsed = None;
            }
            module_state.lowered = None;
            module_state.diagnostics = None;
            module_state.analysis = None;
        }
        self.reuse_caches.prune_unshared();
    }

    fn rebuild_external_defs_from_lowered(&mut self) {
        for module in &self.modules {
            if let Some(lowered) = &module.lowered {
                collect_module_exports(&module.module_name, lowered, &mut self.external_defs);
            }
        }
    }
}
