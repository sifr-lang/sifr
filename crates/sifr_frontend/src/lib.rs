//! Canonical Sifr frontend query API.
//!
//! This crate owns project/session loading, source maps, module graph identity,
//! parse/lower/type-check diagnostics, process-local query caching, and
//! deterministic invalidation reports.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use ruff_text_size::TextRange;
use sifr_diagnostics::{
    DiagnosticArg, DiagnosticBuilder, DiagnosticCode, DiagnosticSink, RenderedDiagnostic, Severity,
    SourceMap, SourceSpan,
};
use sifr_hir::{
    lower_module_with_externals_and_name, ExternalDefs, HirDiagnostic, HirModule, LoweringResult,
    LoweringWarningDiagnostic, RevealTypeDiagnostic,
};
use sifr_python_ast::Stmt;
use sifr_syntax::{ParsedModule, TextPosition, TextRangeUtf};
use sifr_type_system::{FunctionType, ParamConvention, Type};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(u32);

impl FileId {
    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(u32);

impl ModuleId {
    #[must_use]
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphRevision(u64);

impl GraphRevision {
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceRevision(u64);

impl SourceRevision {
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceHash(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourcePath(PathBuf);

impl SourcePath {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceText(String);

impl SourceText {
    #[must_use]
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceUri(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentVersion(i64);

impl DocumentVersion {
    #[must_use]
    pub fn new(version: i64) -> Self {
        Self(version)
    }

    #[must_use]
    pub fn as_i64(self) -> i64 {
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
pub enum PositionEncoding {
    UTF8,
    UTF16,
    UTF32,
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
pub struct SourceFileView {
    pub id: FileId,
    pub canonical_path: SourcePath,
    pub uri: Option<SourceUri>,
    pub source_hash: SourceHash,
    pub document_version: Option<DocumentVersion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceMapView {
    pub files: Vec<SourceFileView>,
    pub revision: SourceRevision,
}

impl SourceMapView {
    #[must_use]
    pub fn text_position_to_span(
        &self,
        _file: FileId,
        _position: TextPosition,
        _encoding: PositionEncoding,
    ) -> Option<TextRange> {
        None
    }

    #[must_use]
    pub fn span_to_text_range(
        &self,
        _span: TextRange,
        _encoding: PositionEncoding,
    ) -> Option<TextRangeUtf> {
        None
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolView {
    pub name: String,
    pub kind: SymbolKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Class,
    Constant,
    Import,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleAnalysisView {
    pub module: ModuleId,
    pub symbols: Vec<SymbolView>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectAnalysisView {
    pub modules: Vec<ModuleAnalysisView>,
}

struct ModuleState {
    id: ModuleId,
    file: FileId,
    module_name: String,
    path: SourcePath,
    source: SourceText,
    source_hash: SourceHash,
    document_version: Option<DocumentVersion>,
    parsed: Option<ParsedModule>,
    lowered: Option<LoweringResult>,
    diagnostics: Option<Vec<RenderedDiagnostic>>,
    analysis: Option<ModuleAnalysisView>,
}

pub struct FrontendContext {
    modules: Vec<ModuleState>,
    module_by_id: BTreeMap<ModuleId, usize>,
    entrypoint: ModuleId,
    edges: Vec<ModuleGraphEdge>,
    graph_revision: GraphRevision,
    source_revision: SourceRevision,
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
    match lower_module_with_externals_and_name(module_name, stmts, external_defs) {
        Ok(result) => Ok(result),
        Err(errors) => Err(errors
            .into_iter()
            .map(|error| {
                hir_diagnostic_to_rendered(module_name, diagnostic_style, source_context, error)
            })
            .collect()),
    }
}

impl FrontendContext {
    pub fn load_single_file(input: FrontendInput) -> Result<Self, Vec<RenderedDiagnostic>> {
        Self::load_single_file_with_external_defs(input, ExternalDefs::default())
    }

    pub fn load_single_file_with_external_defs(
        input: FrontendInput,
        external_defs: ExternalDefs,
    ) -> Result<Self, Vec<RenderedDiagnostic>> {
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
            graph_revision: GraphRevision(0),
            source_revision: SourceRevision(0),
            base_external_defs: external_defs.clone(),
            external_defs,
            lowering_modules: BTreeSet::new(),
        };
        context.rebuild_edges();
        Ok(context)
    }

    pub fn load_project(root: &ProjectRoot) -> Result<Self, Vec<RenderedDiagnostic>> {
        let entrypoint = root.entrypoint.as_path();
        let project_dir = root.root.as_path();
        let entry_source = std::fs::read_to_string(entrypoint).map_err(|error| {
            vec![diagnostic_with_code(
                format!(
                    "failed to read project entrypoint '{}': {error}",
                    entrypoint.display()
                ),
                DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
            )]
        })?;
        let mut files = vec![entrypoint.to_path_buf()];
        for entry in std::fs::read_dir(project_dir).map_err(|error| {
            vec![diagnostic_with_code(
                format!(
                    "failed to read project root '{}': {error}",
                    project_dir.display()
                ),
                DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
            )]
        })? {
            let path = entry
                .map_err(|error| {
                    vec![diagnostic_with_code(
                        format!("failed to read project root entry: {error}"),
                        DiagnosticCode::WORKSPACE_MALFORMED_MANIFEST,
                    )]
                })?
                .path();
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
                std::fs::read_to_string(&path).map_err(|error| {
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
                SourceText::new(source),
                None,
            ));
        }

        let module_by_id = modules
            .iter()
            .enumerate()
            .map(|(index, module)| (module.id, index))
            .collect();
        let mut context = Self {
            modules,
            module_by_id,
            entrypoint: ModuleId(0),
            edges: Vec::new(),
            graph_revision: GraphRevision(0),
            source_revision: SourceRevision(0),
            base_external_defs: ExternalDefs::default(),
            external_defs: ExternalDefs::default(),
            lowering_modules: BTreeSet::new(),
        };
        context.rebuild_edges();
        Ok(context)
    }

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
        let file = self.modules[index].file;
        let text_changed = old_hash != new_hash;
        self.modules[index].source = source;
        self.modules[index].source_hash = new_hash;
        self.modules[index].document_version = document_version;
        self.source_revision.0 += 1;

        let mut invalidated_modules = Vec::new();
        let mut invalidated_queries = Vec::new();
        if text_changed {
            self.external_defs = self.base_external_defs.clone();
            self.lowering_modules.clear();
            for module_state in &mut self.modules {
                if module_state.id == module {
                    module_state.parsed = None;
                }
                module_state.lowered = None;
                module_state.diagnostics = None;
                module_state.analysis = None;
                invalidated_modules.push(module_state.id);
            }
            invalidated_modules.sort();
            invalidated_modules.dedup();
            invalidated_queries.extend([
                QueryKind::Parse,
                QueryKind::Lower,
                QueryKind::TypeCheck,
                QueryKind::ModuleDiagnostics,
                QueryKind::ProjectDiagnostics,
                QueryKind::ModuleAnalysis,
                QueryKind::ProjectAnalysis,
            ]);
            self.graph_revision.0 += 1;
            self.rebuild_edges();
        }

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
        })
    }

    #[must_use]
    pub fn module_graph(&self) -> ModuleGraphView {
        ModuleGraphView {
            modules: self
                .modules
                .iter()
                .map(|module| ModuleGraphNode {
                    id: module.id,
                    file: module.file,
                    canonical_path: module.path.clone(),
                    source_hash: module.source_hash.clone(),
                })
                .collect(),
            edges: self.edges.clone(),
            entrypoint: self.entrypoint,
            revision: self.graph_revision,
        }
    }

    #[must_use]
    pub fn source_map(&self) -> SourceMapView {
        SourceMapView {
            files: self
                .modules
                .iter()
                .map(|module| SourceFileView {
                    id: module.file,
                    canonical_path: module.path.clone(),
                    uri: None,
                    source_hash: module.source_hash.clone(),
                    document_version: module.document_version,
                })
                .collect(),
            revision: self.source_revision,
        }
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
        let module = self.module_for_file(file)?;
        let index = self.module_by_id.get(&module).copied()?;
        Some(self.modules[index].source.as_str())
    }

    #[must_use]
    pub fn path_for_file(&self, file: FileId) -> Option<&Path> {
        let module = self.module_for_file(file)?;
        let index = self.module_by_id.get(&module).copied()?;
        Some(self.modules[index].path.as_path())
    }

    #[must_use]
    pub fn document_version_for_file(&self, file: FileId) -> Option<DocumentVersion> {
        let module = self.module_for_file(file)?;
        let index = self.module_by_id.get(&module).copied()?;
        self.modules[index].document_version
    }

    pub fn parse_module(&mut self, module: ModuleId) -> QueryResult<ParsedModuleView> {
        let index = self.index_for_module(module);
        let cache_status = if self.modules[index].parsed.is_some() {
            CacheStatus::Hit
        } else {
            let parsed = sifr_syntax::parse_module(
                self.modules[index].source.as_str(),
                Some(&self.modules[index].module_name),
            )
            .unwrap_or_else(|_| ParsedModule::empty());
            self.modules[index].parsed = Some(parsed);
            CacheStatus::Miss
        };
        QueryResult::new(
            ParsedModuleView {
                module,
                parsed: self.modules[index]
                    .parsed
                    .clone()
                    .unwrap_or_else(ParsedModule::empty),
            },
            self.metadata(QueryKind::Parse, cache_status),
        )
    }

    pub fn lower_module(&mut self, module: ModuleId) -> QueryResult<LoweredModuleView> {
        let _ = self.ensure_lowered(module);
        let index = self.index_for_module(module);
        QueryResult::new(
            LoweredModuleView {
                module,
                hir: self.modules[index]
                    .lowered
                    .as_ref()
                    .map(|lowered| lowered.module.clone())
                    .unwrap_or_else(empty_hir_module),
            },
            self.metadata(QueryKind::Lower, CacheStatus::Hit),
        )
    }

    pub fn type_check_module(&mut self, module: ModuleId) -> QueryResult<ModuleDiagnostics> {
        self.diagnostics_for_module(module)
    }

    pub fn diagnostics_for_module(&mut self, module: ModuleId) -> QueryResult<ModuleDiagnostics> {
        let cache_status = self.ensure_diagnostics(module);
        let index = self.index_for_module(module);
        QueryResult::new(
            ModuleDiagnostics {
                module,
                diagnostics: self.modules[index].diagnostics.clone().unwrap_or_default(),
            },
            self.metadata(QueryKind::ModuleDiagnostics, cache_status),
        )
    }

    pub fn diagnostics_for_project(&mut self) -> QueryResult<ProjectDiagnostics> {
        let module_ids: Vec<ModuleId> = self.modules.iter().map(|module| module.id).collect();
        let mut diagnostics = Vec::new();
        for module in module_ids {
            diagnostics.extend(self.diagnostics_for_module(module).into_value().diagnostics);
        }
        QueryResult::new(
            ProjectDiagnostics { diagnostics },
            self.metadata(QueryKind::ProjectDiagnostics, CacheStatus::Miss),
        )
    }

    pub fn analysis_for_module(&mut self, module: ModuleId) -> QueryResult<ModuleAnalysisView> {
        let cache_status = self.ensure_analysis(module);
        let index = self.index_for_module(module);
        QueryResult::new(
            self.modules[index]
                .analysis
                .clone()
                .unwrap_or(ModuleAnalysisView {
                    module,
                    symbols: Vec::new(),
                }),
            self.metadata(QueryKind::ModuleAnalysis, cache_status),
        )
    }

    pub fn analysis_for_project(&mut self) -> QueryResult<ProjectAnalysisView> {
        let module_ids: Vec<ModuleId> = self.modules.iter().map(|module| module.id).collect();
        let modules = module_ids
            .into_iter()
            .map(|module| self.analysis_for_module(module).into_value())
            .collect();
        QueryResult::new(
            ProjectAnalysisView { modules },
            self.metadata(QueryKind::ProjectAnalysis, CacheStatus::Miss),
        )
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
        let parsed = sifr_syntax::parse_module(
            self.modules[index].source.as_str(),
            Some(&self.modules[index].module_name),
        )?;
        self.modules[index].parsed = Some(parsed);
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
                self.modules[index].diagnostics = Some(errors);
                self.lowering_modules.remove(&module);
                return CacheStatus::Miss;
            }
        };
        let module_names: BTreeMap<String, ModuleId> = self
            .modules
            .iter()
            .map(|module| (module.module_name.clone(), module.id))
            .collect();
        for dependency in local_import_dependencies(&parsed, &module_names) {
            let _ = self.ensure_lowered(dependency);
        }
        let index = self.index_for_module(module);
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
                self.modules[index].lowered = Some(lowered);
            }
            Err(errors) => {
                self.modules[index].diagnostics = Some(errors);
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
        let status = self.ensure_lowered(module);
        let index = self.index_for_module(module);
        if self.modules[index].diagnostics.is_none() {
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
            self.modules[index].diagnostics = Some(diagnostics);
        }
        status
    }

    fn ensure_analysis(&mut self, module: ModuleId) -> CacheStatus {
        let index = self.index_for_module(module);
        if self.modules[index].analysis.is_some() {
            return CacheStatus::Hit;
        }
        let _ = self.ensure_lowered(module);
        let index = self.index_for_module(module);
        let symbols = self.modules[index]
            .lowered
            .as_ref()
            .map(|lowered| symbols_from_hir(&lowered.module))
            .unwrap_or_default();
        self.modules[index].analysis = Some(ModuleAnalysisView { module, symbols });
        CacheStatus::Miss
    }

    fn rebuild_edges(&mut self) {
        let module_names: BTreeMap<String, ModuleId> = self
            .modules
            .iter()
            .map(|module| (module.module_name.clone(), module.id))
            .collect();
        let mut edges = BTreeSet::new();
        for module in &self.modules {
            if let Ok(parsed) =
                sifr_syntax::parse_module(module.source.as_str(), Some(&module.module_name))
            {
                for import in local_import_dependencies(parsed.suite(), &module_names) {
                    edges.insert((module.id, import));
                }
            }
        }
        self.edges = edges
            .into_iter()
            .map(|(importer, imported)| ModuleGraphEdge { importer, imported })
            .collect();
    }
}

fn module_state(
    id: ModuleId,
    file: FileId,
    module_name: impl Into<String>,
    path: SourcePath,
    source: SourceText,
    document_version: Option<DocumentVersion>,
) -> ModuleState {
    let source_hash = source_hash(source.as_str());
    ModuleState {
        id,
        file,
        module_name: module_name.into(),
        path,
        source,
        source_hash,
        document_version,
        parsed: None,
        lowered: None,
        diagnostics: None,
        analysis: None,
    }
}

fn source_hash(source: &str) -> SourceHash {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source.hash(&mut hasher);
    SourceHash(format!("{:016x}", hasher.finish()))
}

fn local_import_dependencies(
    stmts: &[Stmt],
    module_names: &BTreeMap<String, ModuleId>,
) -> Vec<ModuleId> {
    let mut deps = Vec::new();
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
        if let Some(module_id) = module_names.get(&module_name) {
            deps.push(*module_id);
        }
    }
    deps.sort();
    deps.dedup();
    deps
}

fn symbols_from_hir(module: &HirModule) -> Vec<SymbolView> {
    let mut symbols = Vec::new();
    symbols.extend(module.functions.iter().map(|function| SymbolView {
        name: function.name.clone(),
        kind: SymbolKind::Function,
    }));
    symbols.extend(module.classes.iter().map(|class| SymbolView {
        name: class.name.clone(),
        kind: SymbolKind::Class,
    }));
    symbols.extend(module.constants.iter().map(|(name, _, _)| SymbolView {
        name: name.clone(),
        kind: SymbolKind::Constant,
    }));
    symbols.extend(module.imports.iter().map(|import| SymbolView {
        name: import.module.clone(),
        kind: SymbolKind::Import,
    }));
    symbols.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| format!("{:?}", left.kind).cmp(&format!("{:?}", right.kind)))
    });
    symbols
}

fn empty_hir_module() -> HirModule {
    HirModule {
        functions: Vec::new(),
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}

fn hir_diagnostic_to_rendered(
    module_name: &str,
    diagnostic_style: FrontendDiagnosticStyle,
    source_context: Option<FrontendSourceContext<'_>>,
    error: HirDiagnostic,
) -> RenderedDiagnostic {
    let code = error
        .code
        .unwrap_or(DiagnosticCode::INTERNAL_COMPILER_PANIC);
    let uncoded = error.code.is_none();
    let primary_range = error.primary_range;
    let message = match diagnostic_style {
        FrontendDiagnosticStyle::Bare => error.message,
        FrontendDiagnosticStyle::ModulePrefixed => {
            format!("[{}] {}", module_name, error.message)
        }
    };
    let message = if uncoded {
        format!(
            "internal compiler error: HIR lowering emitted a diagnostic without canonical code: {message}"
        )
    } else {
        message
    };
    if let (Some(context), Some(range)) = (source_context, primary_range) {
        return diagnostic_with_source_range(
            code,
            context,
            range,
            "{message}",
            &[("message", DiagnosticArg::String(message.clone()))],
        );
    }
    diagnostic_with_code(message, code)
}

fn diagnostic_with_source_range(
    code: DiagnosticCode,
    source_context: FrontendSourceContext<'_>,
    range: TextRange,
    message_template: &'static str,
    args: &[(&'static str, DiagnosticArg)],
) -> RenderedDiagnostic {
    let mut source_map = SourceMap::new();
    let source_id = source_map.register_source(source_context.display_path, source_context.source);
    let span = SourceSpan::new(source_id, range);
    let mut builder = DiagnosticBuilder::source(code, code.declared_severity(), span)
        .message_template(message_template);
    for (name, value) in args {
        builder = builder.arg(name, value.clone());
    }
    let diagnostic = builder.build();
    let mut sink = DiagnosticSink::new();
    if code.declared_severity() == Severity::Error {
        let _ = sink.emit_error(diagnostic);
    } else {
        sink.emit(diagnostic);
    }
    match sifr_diagnostics::render::render_sink(&sink, &source_map) {
        Ok(mut envelope) if envelope.diagnostics.len() == 1 => envelope.diagnostics.remove(0),
        Ok(_) => diagnostic_with_code(
            "internal compiler error: frontend diagnostic renderer emitted an unexpected diagnostic count",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
        Err(error) => diagnostic_with_code(
            format!("internal compiler error: invalid frontend diagnostic span: {error:?}"),
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        ),
    }
}

pub fn reveal_type_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    reveal_types: &[RevealTypeDiagnostic],
) -> Vec<RenderedDiagnostic> {
    reveal_types
        .iter()
        .map(|diagnostic| reveal_type_diagnostic(source_context, diagnostic))
        .collect()
}

pub fn warning_diagnostics(
    source_context: Option<FrontendSourceContext<'_>>,
    warnings: &[LoweringWarningDiagnostic],
) -> Vec<RenderedDiagnostic> {
    warnings
        .iter()
        .map(|diagnostic| warning_diagnostic(source_context, diagnostic))
        .collect()
}

fn warning_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &LoweringWarningDiagnostic,
) -> RenderedDiagnostic {
    let (code, message, message_template, args, primary_range) = match diagnostic {
        LoweringWarningDiagnostic::ArithmeticOverflowRisk {
            operation,
            primary_range,
        } => (
            DiagnosticCode::TYPE_ARITHMETIC_OVERFLOW_RISK,
            format!("integer {operation} may overflow at runtime"),
            "integer {operation} may overflow at runtime",
            vec![("operation", DiagnosticArg::String(operation.clone()))],
            *primary_range,
        ),
        LoweringWarningDiagnostic::UnreachableStatement { primary_range } => (
            DiagnosticCode::FLOW_UNREACHABLE_STATEMENT,
            "unreachable statement ignored".to_string(),
            "unreachable statement ignored",
            Vec::new(),
            *primary_range,
        ),
        LoweringWarningDiagnostic::BigIntTransitionAlias { primary_range } => (
            DiagnosticCode::INT_BIGINT_TRANSITION_ALIAS,
            "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values".to_string(),
            "bigint is a temporary transition alias; use int for exact integers or an explicit fixed-width type for representation-sensitive values",
            Vec::new(),
            *primary_range,
        ),
    };
    if let (Some(context), Some(range)) = (source_context, primary_range) {
        return diagnostic_with_source_range(code, context, range, message_template, &args);
    }
    rendered_spanless_diagnostic(code, message, message_template, &args)
}

fn reveal_type_diagnostic(
    source_context: Option<FrontendSourceContext<'_>>,
    diagnostic: &RevealTypeDiagnostic,
) -> RenderedDiagnostic {
    let code = DiagnosticCode::TYPE_REVEAL_TYPE;
    let message = format!("revealed type is {}", diagnostic.revealed_type);
    let args = [(
        "revealed_type",
        DiagnosticArg::String(diagnostic.revealed_type.clone()),
    )];
    if let (Some(context), Some(range)) = (source_context, diagnostic.primary_range) {
        return diagnostic_with_source_range(
            code,
            context,
            range,
            "revealed type is {revealed_type}",
            &args,
        );
    }
    rendered_spanless_diagnostic(code, message, "revealed type is {revealed_type}", &args)
}

fn rendered_spanless_diagnostic(
    code: DiagnosticCode,
    message: String,
    message_template: &'static str,
    args: &[(&'static str, DiagnosticArg)],
) -> RenderedDiagnostic {
    let mut rendered_args = BTreeMap::new();
    for (name, value) in args {
        rendered_args.insert((*name).to_string(), value.clone());
    }
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message,
        message_template: message_template.to_string(),
        args: rendered_args,
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

fn diagnostic_with_code(message: impl Into<String>, code: DiagnosticCode) -> RenderedDiagnostic {
    let message = message.into();
    RenderedDiagnostic {
        code: code.code().to_string(),
        severity: code.declared_severity(),
        message: message.clone(),
        message_template: "{message}".to_string(),
        args: BTreeMap::from([("message".to_string(), DiagnosticArg::String(message))]),
        url: code.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: None,
        suggestions: Vec::new(),
    }
}

fn should_export_callable(module_name: &str, callable_name: &str) -> bool {
    !callable_name.starts_with('_')
        || matches!(
            (module_name, callable_name),
            (
                "sifr.heapq",
                "_heapify_max" | "_heappop_max" | "_heapreplace_max"
            )
        )
}

pub fn collect_module_exports(
    module_name: &str,
    lowering_result: &LoweringResult,
    external_defs: &mut ExternalDefs,
) {
    let module = &lowering_result.module;
    let mut fn_exports = HashMap::new();
    let mut class_exports = HashMap::new();
    let mut class_type_param_exports = HashMap::new();
    let mut const_exports = HashMap::new();
    let mut const_integer_value_exports = HashMap::new();
    let mut default_exports = HashMap::new();
    let mut vararg_exports = HashMap::new();

    for func in &module.functions {
        if should_export_callable(module_name, &func.name) {
            let params: Vec<(String, Type, ParamConvention)> = func
                .params
                .iter()
                .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                .collect();
            fn_exports.insert(
                func.name.clone(),
                FunctionType {
                    params,
                    return_type: Box::new(func.return_type.clone()),
                },
            );
            if let Some(vararg_index) = lowering_result.function_varargs.get(&func.name) {
                vararg_exports.insert(func.name.clone(), *vararg_index);
            }
        }
    }

    for (callable_name, defaults) in &lowering_result.function_defaults {
        if should_export_callable(module_name, callable_name) {
            default_exports.insert(callable_name.clone(), defaults.clone());
        }
    }

    for class in &module.classes {
        if !class.name.starts_with('_') {
            let mut methods: Vec<(String, FunctionType)> = class
                .methods
                .iter()
                .map(|m| {
                    let params: Vec<(String, Type, ParamConvention)> = m
                        .params
                        .iter()
                        .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                        .collect();
                    (
                        m.name.clone(),
                        FunctionType {
                            params,
                            return_type: Box::new(m.return_type.clone()),
                        },
                    )
                })
                .collect();
            for (dunder_name, op_func) in &class.operator_impls {
                let params: Vec<(String, Type, ParamConvention)> = op_func
                    .params
                    .iter()
                    .map(|p| (p.name.clone(), p.ty.clone(), p.convention))
                    .collect();
                methods.push((
                    dunder_name.clone(),
                    FunctionType {
                        params,
                        return_type: Box::new(op_func.return_type.clone()),
                    },
                ));
            }
            let class_ty = Type::Class {
                name: class.name.clone(),
                fields: class.fields.clone(),
                methods,
                parent_class: None,
            };
            class_exports.insert(class.name.clone(), class_ty);
            if !class.type_params.is_empty() {
                class_type_param_exports.insert(class.name.clone(), class.type_params.clone());
            }
        }
    }

    for (name, ty, _) in &module.constants {
        if !name.starts_with('_') {
            const_exports.insert(name.clone(), ty.clone());
            if let Some(value) = lowering_result.constant_integer_values.get(name) {
                const_integer_value_exports.insert(name.clone(), value.clone());
            }
        }
    }

    for import in &module.imports {
        for name in &import.names {
            let local_name = import
                .aliases
                .iter()
                .find(|(original, _)| original == name)
                .map_or_else(|| name.clone(), |(_, alias)| alias.clone());
            if local_name.starts_with('_') {
                continue;
            }
            if let Some(module_fns) = external_defs.functions.get(&import.module) {
                if let Some(function_type) = module_fns.get(name) {
                    fn_exports.insert(local_name.clone(), function_type.clone());
                    if let Some(defaults) = external_defs
                        .function_defaults
                        .get(&import.module)
                        .and_then(|module_defaults| module_defaults.get(name))
                    {
                        default_exports.insert(local_name.clone(), defaults.clone());
                    }
                    if let Some(vararg_index) = external_defs
                        .function_varargs
                        .get(&import.module)
                        .and_then(|module_varargs| module_varargs.get(name))
                    {
                        vararg_exports.insert(local_name.clone(), *vararg_index);
                    }
                    if let Some(type_vars) = external_defs
                        .generic_functions
                        .get(&import.module)
                        .and_then(|module_generics| module_generics.get(name))
                        .cloned()
                    {
                        // Generic function metadata is keyed by the local export name.
                        // It is consumed by later modules through ExternalDefs.
                        external_defs
                            .generic_functions
                            .entry(module_name.to_string())
                            .or_default()
                            .insert(local_name.clone(), type_vars);
                    }
                    continue;
                }
            }
            if let Some(module_classes) = external_defs.classes.get(&import.module) {
                if let Some(class_type) = module_classes.get(name) {
                    class_exports.insert(local_name.clone(), class_type.clone());
                    if let Some(type_params) = external_defs
                        .class_type_params
                        .get(&import.module)
                        .and_then(|module_params| module_params.get(name))
                    {
                        class_type_param_exports.insert(local_name.clone(), type_params.clone());
                    }
                    continue;
                }
            }
            if let Some(module_consts) = external_defs.constants.get(&import.module) {
                if let Some(const_type) = module_consts.get(name) {
                    const_exports.insert(local_name.clone(), const_type.clone());
                    if let Some(value) = external_defs
                        .constant_integer_values
                        .get(&import.module)
                        .and_then(|module_values| module_values.get(name))
                    {
                        const_integer_value_exports.insert(local_name, value.clone());
                    }
                }
            }
        }
    }

    external_defs
        .functions
        .insert(module_name.to_string(), fn_exports);
    external_defs
        .classes
        .insert(module_name.to_string(), class_exports);
    if !class_type_param_exports.is_empty() {
        external_defs
            .class_type_params
            .insert(module_name.to_string(), class_type_param_exports);
    }
    if !default_exports.is_empty() {
        external_defs
            .function_defaults
            .insert(module_name.to_string(), default_exports);
    }
    if !vararg_exports.is_empty() {
        external_defs
            .function_varargs
            .insert(module_name.to_string(), vararg_exports);
    }
    external_defs
        .constants
        .insert(module_name.to_string(), const_exports);
    if !const_integer_value_exports.is_empty() {
        external_defs
            .constant_integer_values
            .insert(module_name.to_string(), const_integer_value_exports);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CacheStatus, DocumentVersion, FrontendContext, FrontendInput, FrontendMode, ModuleId,
        SourcePath, SourceText,
    };

    fn input(source: &str) -> FrontendInput {
        FrontendInput {
            path: SourcePath::new("main.sifr"),
            source: SourceText::new(source),
            mode: FrontendMode::SingleFile,
        }
    }

    #[test]
    fn single_file_queries_are_cached_and_deterministic() {
        let mut context = FrontendContext::load_single_file(input(
            "def main():\n    value: int = 1\n    reveal_type(value)\n",
        ))
        .expect("context should load");

        let first = context.diagnostics_for_module(ModuleId(0));
        let second = context.diagnostics_for_module(ModuleId(0));

        assert_eq!(first.value().module, ModuleId(0));
        assert_eq!(second.metadata().cache_status, CacheStatus::Hit);
        assert_eq!(first.value().diagnostics, second.value().diagnostics);
    }

    #[test]
    fn source_update_invalidates_cached_queries() {
        let mut context = FrontendContext::load_single_file(input("def main():\n    return 1\n"))
            .expect("context should load");
        let _ = context.diagnostics_for_module(ModuleId(0));

        let report = context
            .update_module_source(
                ModuleId(0),
                SourceText::new("def main():\n    return 2\n"),
                Some(DocumentVersion::new(2)),
            )
            .expect("update should succeed");

        assert!(report.invalidated_modules.contains(&ModuleId(0)));
        assert_eq!(
            context
                .diagnostics_for_module(ModuleId(0))
                .metadata()
                .cache_status,
            CacheStatus::Miss
        );
    }

    #[test]
    fn project_graph_records_local_import_edges() {
        let dir = std::env::temp_dir().join(format!(
            "sifr_frontend_project_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).expect("temp project should be created");
        std::fs::write(
            dir.join("main.sifr"),
            "from helper import value\n\ndef main():\n    print(value)\n",
        )
        .expect("main should be written");
        std::fs::write(dir.join("helper.sifr"), "value: int = 1\n")
            .expect("helper should be written");

        let mut context = FrontendContext::load_project(&super::ProjectRoot {
            root: SourcePath::new(&dir),
            entrypoint: SourcePath::new(dir.join("main.sifr")),
        })
        .expect("project should load");

        let graph = context.module_graph();
        assert_eq!(graph.entrypoint, ModuleId(0));
        assert_eq!(graph.edges.len(), 1);

        let diagnostics = context.diagnostics_for_project().into_value().diagnostics;
        assert!(
            diagnostics.is_empty(),
            "project diagnostics should consume dependency exports from the canonical frontend: {diagnostics:?}"
        );
    }
}
