use super::{
    CacheFamily, CacheKeyContext, CacheStatus, DiagnosticsCacheKey, FileId, FrontendContext,
    FrontendDiagnosticStyle, HirLoweringCacheKey, LoweredModuleView, ModuleAnalysisView,
    ModuleDiagnostics, ModuleGraphNode, ModuleGraphView, ModuleId, ModuleSignature, ParseCacheKey,
    ParsedModuleView, ProjectAnalysisView, ProjectDiagnostics, QueryKind, QueryResult,
    SourceFileView, SourceMapCacheKey, SourceMapView, SourceOrigin, SourceText, SymbolBucketScope,
    SymbolBucketsCacheKey, module_signature,
};
use crate::cache_keys::stable_cache_fingerprint;
use crate::{CacheKeyFingerprint, FrontendCacheEntryIdentity, FrontendReuseStats};
use crate::{QueryPolicyFingerprint, empty_hir_module};
#[cfg(test)]
use sifr_diagnostics::RenderedDiagnostic;
#[cfg(test)]
use sifr_lowering::LoweringResult;
use sifr_syntax::ParsedModule;
use std::sync::Arc;

impl FrontendContext {
    pub fn parse_module(&mut self, module: ModuleId) -> QueryResult<ParsedModuleView> {
        let cache_status = match self.ensure_parsed(module) {
            Ok(status) => status,
            Err(errors) => {
                let index = self.index_for_module(module);
                self.modules[index].diagnostics = Some(Arc::new(errors));
                CacheStatus::Miss
            }
        };
        let index = self.index_for_module(module);
        QueryResult::new(
            ParsedModuleView {
                module,
                parsed: self.modules[index]
                    .parsed
                    .as_ref()
                    .map(|parsed| parsed.as_ref().clone())
                    .unwrap_or_else(ParsedModule::empty),
            },
            self.metadata(QueryKind::Parse, cache_status),
        )
    }

    pub fn lower_module(&mut self, module: ModuleId) -> QueryResult<LoweredModuleView> {
        let cache_status = self.ensure_lowered(module);
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
            self.metadata(QueryKind::Lower, cache_status),
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
                diagnostics: self.modules[index]
                    .diagnostics
                    .as_ref()
                    .map(|diagnostics| diagnostics.as_ref().clone())
                    .unwrap_or_default(),
            },
            self.metadata(QueryKind::ModuleDiagnostics, cache_status),
        )
    }

    pub fn diagnostics_for_project(&mut self) -> QueryResult<ProjectDiagnostics> {
        let module_ids = self
            .project_compile_order
            .clone()
            .unwrap_or_else(|| self.modules.iter().map(|module| module.id).collect());
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
                .as_ref()
                .map(|analysis| analysis.as_ref().clone())
                .unwrap_or(ModuleAnalysisView {
                    module,
                    symbols: Vec::new(),
                    editor_semantics: Default::default(),
                }),
            self.metadata(QueryKind::ModuleAnalysis, cache_status),
        )
    }

    pub fn analysis_for_project(&mut self) -> QueryResult<ProjectAnalysisView> {
        let module_ids = self
            .project_compile_order
            .clone()
            .unwrap_or_else(|| self.modules.iter().map(|module| module.id).collect());
        let modules = module_ids
            .into_iter()
            .map(|module| self.analysis_for_module(module).into_value())
            .collect();
        QueryResult::new(
            ProjectAnalysisView { modules },
            self.metadata(QueryKind::ProjectAnalysis, CacheStatus::Miss),
        )
    }

    pub fn can_replace_module_in_project(
        &self,
        module: ModuleId,
        replacement_source: &SourceText,
    ) -> bool {
        let index = self.index_for_module(module);
        let parsed = sifr_syntax::parse_module(
            replacement_source.as_str(),
            Some(&self.modules[index].module_name),
        );
        let parse_failed = parsed.is_err();
        let new_signature = parsed.as_ref().map_or_else(
            |_| ModuleSignature::default(),
            |parsed| module_signature(parsed.suite()),
        );
        Self::signatures_can_replace_module_in_project(
            &self.modules[index].signature,
            &new_signature,
            parse_failed,
        )
    }

    pub(super) fn signatures_can_replace_module_in_project(
        old_signature: &ModuleSignature,
        new_signature: &ModuleSignature,
        parse_failed: bool,
    ) -> bool {
        !parse_failed
            && old_signature.imports == new_signature.imports
            && old_signature.exports == new_signature.exports
    }

    pub fn module_graph_arc_for_reuse(&mut self) -> Arc<ModuleGraphView> {
        if let Some(graph) = &self.module_graph_cache {
            return Arc::clone(graph);
        }
        let graph = Arc::new(self.module_graph_view());
        self.module_graph_cache = Some(Arc::clone(&graph));
        graph
    }

    pub fn source_map_arc_for_reuse(&mut self) -> Arc<SourceMapView> {
        if let Some(source_map) = &self.source_map_cache {
            return Arc::clone(source_map);
        }
        let mut files = (0..self.modules.len())
            .map(|index| self.cached_source_file_view(index).as_ref().clone())
            .collect::<Vec<_>>();
        files.extend(
            self.auxiliary_sources
                .iter()
                .map(|source| source.source_file.clone()),
        );
        let source_map = Arc::new(SourceMapView {
            files,
            revision: self.source_revision,
        });
        self.source_map_cache = Some(Arc::clone(&source_map));
        source_map
    }

    #[must_use]
    pub fn cache_reuse_stats(&self) -> FrontendReuseStats {
        self.reuse_caches.stats()
    }

    #[must_use]
    pub fn parse_cache_identity(&self, module: ModuleId) -> Option<FrontendCacheEntryIdentity> {
        let index = self.index_for_module(module);
        self.modules[index].parsed.as_ref().map(|_| {
            FrontendCacheEntryIdentity::from_fingerprint(&self.parse_key_fingerprint(index))
        })
    }

    pub fn source_file_cache_identity(
        &mut self,
        file: FileId,
    ) -> Option<FrontendCacheEntryIdentity> {
        let module = self.module_for_file(file)?;
        let index = self.index_for_module(module);
        let _ = self.cached_source_file_view(index);
        Some(FrontendCacheEntryIdentity::from_fingerprint(
            &self.source_file_key_fingerprint(index),
        ))
    }

    pub(super) fn parse_key_fingerprint(&self, index: usize) -> CacheKeyFingerprint {
        self.parse_cache_key(index).fingerprint()
    }

    pub(super) fn hir_key_fingerprint(&self, index: usize) -> CacheKeyFingerprint {
        let base = self.hir_cache_key(index).fingerprint();
        self.module_scoped_fingerprint("frontend-hir-entry", index, &base)
    }

    pub(super) fn diagnostics_key_fingerprint(&self, index: usize) -> CacheKeyFingerprint {
        let base = self.diagnostics_cache_key(index).fingerprint();
        self.module_scoped_fingerprint("frontend-diagnostics-entry", index, &base)
    }

    pub(super) fn index_key_fingerprint(&self, index: usize) -> CacheKeyFingerprint {
        let base = self.index_cache_key(index).fingerprint();
        self.module_scoped_fingerprint("frontend-symbol-index-entry", index, &base)
    }

    pub(super) fn cached_source_file_view(&mut self, index: usize) -> Arc<SourceFileView> {
        if let Some(source_file) = &self.modules[index].source_file_view {
            return Arc::clone(source_file);
        }
        let key = self.source_file_key_fingerprint(index);
        if let Some(source_file) = self.reuse_caches.source_file(&key) {
            self.modules[index].source_file_view = Some(Arc::clone(&source_file));
            return source_file;
        }
        let module = &self.modules[index];
        let source_file = self.reuse_caches.insert_source_file(
            key,
            SourceFileView {
                id: module.file,
                canonical_path: module.path.clone(),
                module_name: Some(module.module_name.clone()),
                origin: SourceOrigin::UserSource,
                uri: None,
                source_hash: module.source_hash.clone(),
                source: module.source.clone(),
            },
        );
        self.modules[index].source_file_view = Some(Arc::clone(&source_file));
        source_file
    }

    pub(super) fn module_graph_view(&self) -> ModuleGraphView {
        ModuleGraphView {
            modules: self
                .modules
                .iter()
                .map(|module| ModuleGraphNode {
                    id: module.id,
                    file: module.file,
                    canonical_path: module.path.clone(),
                    source_hash: module.source_hash.clone(),
                    origin: SourceOrigin::UserSource,
                })
                .collect(),
            edges: self.edges.clone(),
            entrypoint: self.entrypoint,
            revision: self.graph_revision,
        }
    }

    pub(super) fn source_map_view(&self) -> SourceMapView {
        SourceMapView {
            files: self
                .modules
                .iter()
                .map(|module| SourceFileView {
                    id: module.file,
                    canonical_path: module.path.clone(),
                    module_name: Some(module.module_name.clone()),
                    origin: SourceOrigin::UserSource,
                    uri: None,
                    source_hash: module.source_hash.clone(),
                    source: module.source.clone(),
                })
                .chain(
                    self.auxiliary_sources
                        .iter()
                        .map(|source| source.source_file.clone()),
                )
                .collect(),
            revision: self.source_revision,
        }
    }

    fn parse_cache_key(&self, index: usize) -> ParseCacheKey {
        ParseCacheKey::new(
            self.modules[index].source_hash.clone(),
            self.cache_context(CacheFamily::Parse),
        )
    }

    fn source_map_cache_key(&self, index: usize) -> SourceMapCacheKey {
        SourceMapCacheKey::new(
            self.modules[index].source_hash.clone(),
            self.cache_context(CacheFamily::SourceMap),
        )
    }

    fn source_file_key_fingerprint(&self, index: usize) -> CacheKeyFingerprint {
        let module = &self.modules[index];
        stable_cache_fingerprint(
            "frontend-source-file-view",
            [
                (
                    "source_map",
                    self.source_map_cache_key(index)
                        .fingerprint()
                        .as_str()
                        .to_string(),
                ),
                ("file", module.file.as_u32().to_string()),
                ("path", module.path.as_path().to_string_lossy().into_owned()),
            ],
        )
    }

    fn hir_cache_key(&self, index: usize) -> HirLoweringCacheKey {
        HirLoweringCacheKey {
            source_hash: self.modules[index].source_hash.clone(),
            parse_fingerprint: self.parse_key_fingerprint(index),
            compiler_options: self.compiler_options.clone(),
            context: self.semantic_cache_context(CacheFamily::HirLowering),
        }
    }

    fn diagnostics_cache_key(&self, index: usize) -> DiagnosticsCacheKey {
        DiagnosticsCacheKey {
            source_hash: self.modules[index].source_hash.clone(),
            hir_fingerprint: self.hir_key_fingerprint(index),
            diagnostic_style: FrontendDiagnosticStyle::Bare,
            context: self.semantic_cache_context(CacheFamily::Diagnostics),
        }
    }

    fn index_cache_key(&self, index: usize) -> SymbolBucketsCacheKey {
        SymbolBucketsCacheKey {
            source_hash: self.modules[index].source_hash.clone(),
            module_graph_fingerprint: self.semantic_graph_fingerprint(),
            bucket_scope: SymbolBucketScope::Module,
            context: self.semantic_cache_context(CacheFamily::SymbolBuckets),
        }
    }

    fn cache_context(&self, family: CacheFamily) -> CacheKeyContext {
        CacheKeyContext::from_workspace(family, &self.cache_target, &self.package_config_identity)
    }

    fn semantic_cache_context(&self, family: CacheFamily) -> CacheKeyContext {
        let context = self.cache_context(family);
        let policy = format!(
            "{}:{}",
            context.query_policy.as_str(),
            self.semantic_graph_fingerprint().as_str()
        );
        context.with_query_policy(QueryPolicyFingerprint::new(policy))
    }

    fn semantic_graph_fingerprint(&self) -> CacheKeyFingerprint {
        let mut fields = vec![
            ("entrypoint", self.entrypoint.as_u32().to_string()),
            ("module_count", self.modules.len().to_string()),
        ];
        for module in &self.modules {
            fields.push((
                "module",
                format!(
                    "{}|{}|{}",
                    module.id.as_u32(),
                    module.path.as_path().display(),
                    module.signature.cache_key_input()
                ),
            ));
        }
        for edge in &self.edges {
            fields.push((
                "edge",
                format!("{}>{}", edge.importer.as_u32(), edge.imported.as_u32()),
            ));
        }
        stable_cache_fingerprint("semantic-module-graph", fields)
    }

    fn module_scoped_fingerprint(
        &self,
        domain: &'static str,
        index: usize,
        base: &CacheKeyFingerprint,
    ) -> CacheKeyFingerprint {
        let module = &self.modules[index];
        stable_cache_fingerprint(
            domain,
            [
                ("base", base.as_str().to_string()),
                ("module_id", module.id.as_u32().to_string()),
                ("module_name", module.module_name.clone()),
                ("path", module.path.as_path().to_string_lossy().into_owned()),
            ],
        )
    }

    #[cfg(test)]
    pub(crate) fn module_cache_reuse_identity(&self, module: ModuleId) -> ModuleCacheReuseIdentity {
        let index = self.index_for_module(module);
        let module = &self.modules[index];
        ModuleCacheReuseIdentity {
            parsed: module.parsed.as_ref().map(Arc::as_ptr),
            lowered: module.lowered.as_ref().map(Arc::as_ptr),
            diagnostics: module.diagnostics.as_ref().map(Arc::as_ptr),
            analysis: module.analysis.as_ref().map(Arc::as_ptr),
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModuleCacheReuseIdentity {
    pub(crate) parsed: Option<*const ParsedModule>,
    pub(crate) lowered: Option<*const LoweringResult>,
    pub(crate) diagnostics: Option<*const Vec<RenderedDiagnostic>>,
    pub(crate) analysis: Option<*const ModuleAnalysisView>,
}

#[cfg(test)]
mod tests {
    use crate::{
        CacheStatus, DiskSourceProvider, DocumentVersion, FileId, FrontendContext, FrontendInput,
        FrontendMode, FrontendReuseStats, ModuleId, ProjectRoot, SourcePath, SourceText,
        WorkspaceDirtyScope,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn input(source: &str) -> FrontendInput {
        FrontendInput {
            path: SourcePath::new("main.sifr"),
            source: SourceText::new(source),
            mode: FrontendMode::SingleFile,
        }
    }

    #[test]
    fn ref_counted_module_caches_reuse_identity_on_hits() {
        let mut context = FrontendContext::load_single_file(input(
            "def main():\n    value: int = 1\n    reveal_type(value)\n",
        ))
        .expect("context should load");
        let main_module = ModuleId(0);

        assert_eq!(context.cache_reuse_stats(), FrontendReuseStats::default());
        let _ = context.parse_module(main_module);
        let _ = context.lower_module(main_module);
        let _ = context.diagnostics_for_module(main_module);
        let _ = context.analysis_for_module(main_module);
        let first_identity = context.module_cache_reuse_identity(main_module);
        let stats = context.cache_reuse_stats();
        assert_eq!(stats.parse_entries, 1);
        assert_eq!(stats.hir_entries, 1);
        assert_eq!(stats.diagnostics_entries, 1);
        assert_eq!(stats.index_entries, 1);

        assert_eq!(
            context.parse_module(main_module).metadata().cache_status,
            CacheStatus::Hit
        );
        assert_eq!(
            context.lower_module(main_module).metadata().cache_status,
            CacheStatus::Hit
        );
        assert_eq!(
            context
                .diagnostics_for_module(main_module)
                .metadata()
                .cache_status,
            CacheStatus::Hit
        );
        assert_eq!(
            context
                .analysis_for_module(main_module)
                .metadata()
                .cache_status,
            CacheStatus::Hit
        );
        assert_eq!(
            first_identity,
            context.module_cache_reuse_identity(main_module)
        );
        assert!(context.parse_cache_identity(main_module).is_some());
    }

    #[test]
    fn structural_one_module_replacement_reuses_unchanged_cache_entries() {
        let project = TempProject::new("structural_replacement_reuse");
        project.write(
            "main.sifr",
            "from helper import value\n\ndef main() -> int:\n    return value()\n",
        );
        project.write("helper.sifr", "def value() -> int:\n    return 1\n");
        let mut context = load_project(&project.root);
        let main = ModuleId(0);
        let helper = ModuleId(1);

        let _ = context.parse_module(main);
        let _ = context.lower_module(main);
        let _ = context.diagnostics_for_module(main);
        let _ = context.analysis_for_module(main);
        let main_identity = context.module_cache_reuse_identity(main);
        let first_graph = context.module_graph_arc_for_reuse();
        let first_source_map = context.source_map_arc_for_reuse();
        let main_source_identity = context
            .source_file_cache_identity(FileId::new(0))
            .expect("main source file should be cached");
        let helper_source_identity = context
            .source_file_cache_identity(FileId::new(1))
            .expect("helper source file should be cached");
        let replacement = SourceText::new("def value() -> int:\n    return 2\n");

        assert!(context.can_replace_module_in_project(helper, &replacement));
        let report = context
            .update_module_source(helper, replacement, Some(DocumentVersion::new(2)))
            .expect("helper update should succeed");
        assert_eq!(report.previous_revision, report.next_revision);
        assert_eq!(
            report.dirty_scope_report.scope,
            WorkspaceDirtyScope::OneModule {
                path: SourcePath::new(project.root.join("helper.sifr"))
            }
        );

        let second_graph = context.module_graph_arc_for_reuse();
        let second_source_map = context.source_map_arc_for_reuse();
        assert!(!Arc::ptr_eq(&first_graph, &second_graph));
        assert!(!Arc::ptr_eq(&first_source_map, &second_source_map));
        assert_eq!(first_graph.revision, second_graph.revision);
        assert_ne!(
            first_graph.modules[1].source_hash,
            second_graph.modules[1].source_hash
        );
        assert_eq!(
            main_source_identity,
            context
                .source_file_cache_identity(FileId::new(0))
                .expect("main source file should still be cached")
        );
        assert_ne!(
            helper_source_identity,
            context
                .source_file_cache_identity(FileId::new(1))
                .expect("helper source file should be recached")
        );
        assert_eq!(main_identity, context.module_cache_reuse_identity(main));
        assert!(!context.can_replace_module_in_project(
            helper,
            &SourceText::new("def renamed() -> int:\n    return 3\n"),
        ));
    }

    #[test]
    fn document_version_only_update_reuses_source_file_view() {
        let mut context =
            FrontendContext::load_single_file(input("def main() -> int:\n    return 1\n"))
                .expect("context should load");
        let main = ModuleId(0);
        let file = FileId::new(0);
        let first_graph = context.module_graph_arc_for_reuse();
        let first_source_map = context.source_map_arc_for_reuse();
        let first_source_identity = context
            .source_file_cache_identity(file)
            .expect("source file should be cached");

        let report = context
            .update_module_source(
                main,
                SourceText::new("def main() -> int:\n    return 1\n"),
                Some(DocumentVersion::new(2)),
            )
            .expect("version-only update should succeed");

        assert!(!report.updated_documents[0].text_changed);
        assert_eq!(report.previous_revision, report.next_revision);
        let second_graph = context.module_graph_arc_for_reuse();
        let second_source_map = context.source_map_arc_for_reuse();
        assert!(Arc::ptr_eq(&first_graph, &second_graph));
        assert!(Arc::ptr_eq(&first_source_map, &second_source_map));
        assert_eq!(
            context.document_version_for_file(file),
            Some(DocumentVersion::new(2))
        );
        assert_eq!(
            first_source_identity,
            context
                .source_file_cache_identity(file)
                .expect("source file should be reused for the same source text")
        );
        assert_eq!(context.cache_reuse_stats().source_map_entries, 1);
    }

    #[test]
    fn reverse_dependent_invalidation_reuses_unchanged_parse_entry() {
        let project = TempProject::new("reverse_dependent_parse_reuse");
        project.write(
            "main.sifr",
            "from helper import value\n\ndef main() -> int:\n    return value()\n",
        );
        project.write("helper.sifr", "def value() -> int:\n    return 1\n");
        let mut context = load_project(&project.root);
        let main = ModuleId(0);
        let helper = ModuleId(1);

        assert_eq!(
            context.parse_module(main).metadata().cache_status,
            CacheStatus::Miss
        );
        let main_identity = context
            .parse_cache_identity(main)
            .expect("main parse should be cached");
        let report = context
            .update_module_source(
                helper,
                SourceText::new("def value() -> str:\n    return \"changed\"\n"),
                Some(DocumentVersion::new(2)),
            )
            .expect("helper update should succeed");

        assert_eq!(report.invalidated_modules, vec![main, helper]);
        assert_eq!(
            context.parse_module(main).metadata().cache_status,
            CacheStatus::Hit
        );
        assert_eq!(
            main_identity,
            context
                .parse_cache_identity(main)
                .expect("main parse cache should be retained")
        );
    }

    fn load_project(root: &Path) -> FrontendContext {
        let mut provider = DiskSourceProvider::new();
        FrontendContext::load_project(
            &ProjectRoot {
                root: SourcePath::new(root.to_path_buf()),
                entrypoint: SourcePath::new(root.join("main.sifr")),
            },
            &mut provider,
        )
        .expect("project should load")
    }

    struct TempProject {
        root: PathBuf,
    }

    impl TempProject {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock should be after epoch")
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "sifr_frontend_{name}_{}_{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&root).expect("create temp project");
            Self { root }
        }

        fn write(&self, relative: &str, source: &str) {
            let path = self.root.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, source).expect("write source");
        }
    }

    impl Drop for TempProject {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(Path::new(&self.root));
        }
    }
}
