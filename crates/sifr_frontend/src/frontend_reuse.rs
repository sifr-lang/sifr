use crate::{CacheKeyFingerprint, ModuleAnalysisView, SourceFileView};
use sifr_diagnostics::RenderedDiagnostic;
use sifr_lowering::LoweringResult;
use sifr_syntax::ParsedModule;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendCacheEntryIdentity(String);

impl FrontendCacheEntryIdentity {
    pub(crate) fn from_fingerprint(fingerprint: &CacheKeyFingerprint) -> Self {
        Self(fingerprint.as_str().to_string())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrontendReuseStats {
    pub parse_entries: usize,
    pub source_map_entries: usize,
    pub hir_entries: usize,
    pub diagnostics_entries: usize,
    pub index_entries: usize,
}

pub(crate) struct FrontendReuseCaches {
    parse: RefCountedCache<ParsedModule>,
    source_maps: RefCountedCache<SourceFileView>,
    hir: RefCountedCache<LoweringResult>,
    diagnostics: RefCountedCache<Vec<RenderedDiagnostic>>,
    indexes: RefCountedCache<ModuleAnalysisView>,
}

impl FrontendReuseCaches {
    pub(crate) fn new() -> Self {
        Self {
            parse: RefCountedCache::new(),
            source_maps: RefCountedCache::new(),
            hir: RefCountedCache::new(),
            diagnostics: RefCountedCache::new(),
            indexes: RefCountedCache::new(),
        }
    }

    pub(crate) fn stats(&self) -> FrontendReuseStats {
        FrontendReuseStats {
            parse_entries: self.parse.len(),
            source_map_entries: self.source_maps.len(),
            hir_entries: self.hir.len(),
            diagnostics_entries: self.diagnostics.len(),
            index_entries: self.indexes.len(),
        }
    }

    pub(crate) fn prune_unshared(&mut self) {
        self.parse.prune_unshared();
        self.source_maps.prune_unshared();
        self.hir.prune_unshared();
        self.diagnostics.prune_unshared();
        self.indexes.prune_unshared();
    }

    pub(crate) fn parse(&self, key: &CacheKeyFingerprint) -> Option<Arc<ParsedModule>> {
        self.parse.get(key)
    }

    pub(crate) fn insert_parse(
        &mut self,
        key: CacheKeyFingerprint,
        parsed: ParsedModule,
    ) -> Arc<ParsedModule> {
        self.parse.insert(key, parsed)
    }

    pub(crate) fn source_file(&self, key: &CacheKeyFingerprint) -> Option<Arc<SourceFileView>> {
        self.source_maps.get(key)
    }

    pub(crate) fn insert_source_file(
        &mut self,
        key: CacheKeyFingerprint,
        source_file: SourceFileView,
    ) -> Arc<SourceFileView> {
        self.source_maps.insert(key, source_file)
    }

    pub(crate) fn hir(&self, key: &CacheKeyFingerprint) -> Option<Arc<LoweringResult>> {
        self.hir.get(key)
    }

    pub(crate) fn insert_hir(
        &mut self,
        key: CacheKeyFingerprint,
        lowered: LoweringResult,
    ) -> Arc<LoweringResult> {
        self.hir.insert(key, lowered)
    }

    pub(crate) fn diagnostics(
        &self,
        key: &CacheKeyFingerprint,
    ) -> Option<Arc<Vec<RenderedDiagnostic>>> {
        self.diagnostics.get(key)
    }

    pub(crate) fn insert_diagnostics(
        &mut self,
        key: CacheKeyFingerprint,
        diagnostics: Vec<RenderedDiagnostic>,
    ) -> Arc<Vec<RenderedDiagnostic>> {
        self.diagnostics.insert(key, diagnostics)
    }

    pub(crate) fn index(&self, key: &CacheKeyFingerprint) -> Option<Arc<ModuleAnalysisView>> {
        self.indexes.get(key)
    }

    pub(crate) fn insert_index(
        &mut self,
        key: CacheKeyFingerprint,
        analysis: ModuleAnalysisView,
    ) -> Arc<ModuleAnalysisView> {
        self.indexes.insert(key, analysis)
    }
}

impl Default for FrontendReuseCaches {
    fn default() -> Self {
        Self::new()
    }
}

struct RefCountedCache<T> {
    entries: BTreeMap<CacheKeyFingerprint, Arc<T>>,
}

impl<T> RefCountedCache<T> {
    fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn get(&self, key: &CacheKeyFingerprint) -> Option<Arc<T>> {
        self.entries.get(key).cloned()
    }

    fn insert(&mut self, key: CacheKeyFingerprint, value: T) -> Arc<T> {
        let entry = Arc::new(value);
        self.entries.insert(key, Arc::clone(&entry));
        entry
    }

    fn prune_unshared(&mut self) {
        self.entries.retain(|_, entry| Arc::strong_count(entry) > 1);
    }
}

impl<T> Default for RefCountedCache<T> {
    fn default() -> Self {
        Self::new()
    }
}
