
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
