use crate::queries::{DocumentSymbol, WorkspaceSymbol};
use crate::snapshot::AnalysisRevision;
use sifr_frontend::{FileId, ModuleGraphView, ModuleId, ProjectAnalysisView, SymbolKind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(String);

impl SymbolId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolIndexEntry {
    pub id: SymbolId,
    pub name: String,
    pub kind: String,
    pub file: FileId,
    pub module: ModuleId,
    pub ordinal: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolIndex {
    revision: AnalysisRevision,
    buckets: BTreeMap<SymbolBucketId, SymbolBucket>,
    entries: Vec<SymbolIndexEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolBucketId {
    pub kind: SymbolBucketKind,
    pub module: Option<ModuleId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SymbolBucketKind {
    Workspace,
    Package,
    Stdlib,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolBucketReadinessState {
    Exact,
    StaleButUsable,
    NeedsBackgroundRefresh,
    Unavailable,
}

impl SymbolBucketReadinessState {
    #[must_use]
    fn is_available(self) -> bool {
        matches!(self, Self::Exact | Self::StaleButUsable)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolBucketReadiness {
    pub id: SymbolBucketId,
    pub state: SymbolBucketReadinessState,
    pub entry_count: usize,
    pub import_entry_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SymbolBucket {
    id: SymbolBucketId,
    entries: Vec<SymbolIndexEntry>,
    readiness: SymbolBucketReadinessState,
}

impl SymbolIndex {
    #[must_use]
    pub fn build(
        revision: AnalysisRevision,
        graph: &ModuleGraphView,
        analysis: &ProjectAnalysisView,
    ) -> Self {
        let buckets = build_buckets(graph, analysis);
        let entries = flatten_buckets(&buckets);
        Self {
            revision,
            buckets,
            entries,
        }
    }

    pub fn refresh_modules(
        &mut self,
        revision: AnalysisRevision,
        graph: &ModuleGraphView,
        analysis: &ProjectAnalysisView,
        dirty_modules: &[ModuleId],
    ) {
        let dirty = dirty_modules.iter().copied().collect::<BTreeSet<_>>();
        if dirty.is_empty() {
            self.revision = revision;
            return;
        }
        let refreshed = build_buckets(graph, analysis);
        self.buckets
            .retain(|id, _| id.module.is_none_or(|module| !dirty.contains(&module)));
        for (id, bucket) in refreshed {
            if id.module.is_some_and(|module| dirty.contains(&module))
                || (id.module.is_some() && !self.buckets.contains_key(&id))
            {
                self.buckets.insert(id, bucket);
            }
        }
        self.revision = revision;
        self.entries = flatten_buckets(&self.buckets);
    }

    #[must_use]
    pub fn revision(&self) -> AnalysisRevision {
        self.revision
    }

    #[must_use]
    pub fn entries(&self) -> &[SymbolIndexEntry] {
        &self.entries
    }

    #[must_use]
    pub fn bucket_readiness(&self) -> Vec<SymbolBucketReadiness> {
        self.buckets
            .values()
            .map(|bucket| SymbolBucketReadiness {
                id: bucket.id.clone(),
                state: bucket.readiness,
                entry_count: bucket.entries.len(),
                import_entry_count: bucket
                    .entries
                    .iter()
                    .filter(|entry| entry.kind == "import")
                    .count(),
            })
            .collect()
    }

    #[must_use]
    pub fn document_symbols(&self, file: FileId) -> Vec<DocumentSymbol> {
        self.entries
            .iter()
            .filter(|entry| entry.file == file)
            .map(|entry| DocumentSymbol {
                name: entry.name.clone(),
                kind: entry.kind.clone(),
                file: entry.file,
                range: None,
            })
            .collect()
    }

    #[must_use]
    pub fn workspace_symbols(&self, query: &str) -> Vec<WorkspaceSymbol> {
        self.symbols_from_available_buckets(query, |_| true)
    }

    #[must_use]
    pub fn completion_symbols(&self, query: &str) -> Vec<WorkspaceSymbol> {
        self.symbols_from_available_buckets(query, |_| true)
    }

    #[must_use]
    pub fn workspace_import_symbols(&self, query: &str) -> Vec<WorkspaceSymbol> {
        self.symbols_from_available_buckets(query, |entry| entry.kind == "import")
    }

    fn symbols_from_available_buckets(
        &self,
        query: &str,
        include: impl Fn(&SymbolIndexEntry) -> bool,
    ) -> Vec<WorkspaceSymbol> {
        let mut entries = self
            .buckets
            .values()
            .filter(|bucket| bucket.readiness.is_available())
            .flat_map(|bucket| bucket.entries.iter())
            .filter(|entry| include(entry))
            .filter(|entry| query.is_empty() || entry.name.contains(query))
            .cloned()
            .collect::<Vec<_>>();
        entries.sort_by(symbol_entry_order);
        entries
            .into_iter()
            .map(|entry| WorkspaceSymbol {
                name: entry.name,
                kind: entry.kind,
                file: entry.file,
                container_name: Some(format!("module:{}", entry.module.as_u32())),
            })
            .collect()
    }

    #[must_use]
    pub fn unique_symbol_named(&self, name: &str) -> Option<WorkspaceSymbol> {
        let mut matches = self.workspace_symbols(name);
        matches.retain(|symbol| symbol.name == name);
        (matches.len() == 1).then(|| matches.remove(0))
    }
}

fn build_buckets(
    graph: &ModuleGraphView,
    analysis: &ProjectAnalysisView,
) -> BTreeMap<SymbolBucketId, SymbolBucket> {
    let file_by_module = graph
        .modules
        .iter()
        .map(|module| (module.id, module.file))
        .collect::<BTreeMap<_, _>>();
    let mut buckets = BTreeMap::new();
    for module in &analysis.modules {
        let Some(file) = file_by_module.get(&module.module).copied() else {
            continue;
        };
        let id = SymbolBucketId {
            kind: SymbolBucketKind::Workspace,
            module: Some(module.module),
        };
        let mut entries = Vec::new();
        for (ordinal, symbol) in module.symbols.iter().enumerate() {
            let kind = symbol_kind_label(&symbol.kind);
            entries.push(SymbolIndexEntry {
                id: SymbolId(format!(
                    "m{}:f{}:{kind}:{}:{ordinal}",
                    module.module.as_u32(),
                    file.as_u32(),
                    symbol.name
                )),
                name: symbol.name.clone(),
                kind,
                file,
                module: module.module,
                ordinal,
            });
        }
        entries.sort_by(symbol_entry_order);
        buckets.insert(
            id.clone(),
            SymbolBucket {
                id,
                entries,
                readiness: SymbolBucketReadinessState::Exact,
            },
        );
    }
    for kind in [
        SymbolBucketKind::Workspace,
        SymbolBucketKind::Package,
        SymbolBucketKind::Stdlib,
    ] {
        let id = SymbolBucketId { kind, module: None };
        buckets.entry(id.clone()).or_insert(SymbolBucket {
            id,
            entries: Vec::new(),
            readiness: aggregate_readiness(kind),
        });
    }
    buckets
}

fn flatten_buckets(buckets: &BTreeMap<SymbolBucketId, SymbolBucket>) -> Vec<SymbolIndexEntry> {
    let mut entries = buckets
        .values()
        .flat_map(|bucket| bucket.entries.iter().cloned())
        .collect::<Vec<_>>();
    entries.sort_by(symbol_entry_order);
    entries
}

fn symbol_entry_order(left: &SymbolIndexEntry, right: &SymbolIndexEntry) -> std::cmp::Ordering {
    left.file
        .cmp(&right.file)
        .then_with(|| left.name.cmp(&right.name))
        .then_with(|| left.kind.cmp(&right.kind))
        .then_with(|| left.ordinal.cmp(&right.ordinal))
}

fn aggregate_readiness(kind: SymbolBucketKind) -> SymbolBucketReadinessState {
    match kind {
        SymbolBucketKind::Workspace => SymbolBucketReadinessState::Exact,
        SymbolBucketKind::Package | SymbolBucketKind::Stdlib => {
            SymbolBucketReadinessState::Unavailable
        }
    }
}

fn symbol_kind_label(kind: &SymbolKind) -> String {
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Class => "class",
        SymbolKind::Constant => "constant",
        SymbolKind::Import => "import",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{SymbolBucketKind, SymbolBucketReadinessState, SymbolIndex};
    use crate::snapshot::AnalysisRevision;
    use sifr_frontend::{
        FileId, GraphRevision, ModuleAnalysisView, ModuleGraphNode, ModuleGraphView, ModuleId,
        ProjectAnalysisView, SourceHash, SourcePath, SourceRevision, SymbolKind, SymbolView,
    };

    fn revision(graph: u64, source: u64) -> AnalysisRevision {
        AnalysisRevision {
            graph: GraphRevision::new(graph),
            source: SourceRevision::new(source),
        }
    }

    fn graph() -> ModuleGraphView {
        ModuleGraphView {
            modules: vec![
                ModuleGraphNode {
                    id: ModuleId::new(1),
                    file: FileId::new(0),
                    canonical_path: SourcePath::new("main.sifr"),
                    source_hash: SourceHash::new("main"),
                },
                ModuleGraphNode {
                    id: ModuleId::new(2),
                    file: FileId::new(2),
                    canonical_path: SourcePath::new("helper.sifr"),
                    source_hash: SourceHash::new("helper"),
                },
            ],
            edges: Vec::new(),
            entrypoint: ModuleId::new(1),
            revision: sifr_frontend::GraphRevision::new(1),
        }
    }

    fn analysis(main_name: &str, helper_name: &str) -> ProjectAnalysisView {
        ProjectAnalysisView {
            modules: vec![
                ModuleAnalysisView {
                    module: ModuleId::new(1),
                    symbols: vec![
                        SymbolView {
                            name: main_name.to_string(),
                            kind: SymbolKind::Function,
                        },
                        SymbolView {
                            name: helper_name.to_string(),
                            kind: SymbolKind::Import,
                        },
                    ],
                },
                ModuleAnalysisView {
                    module: ModuleId::new(2),
                    symbols: vec![SymbolView {
                        name: helper_name.to_string(),
                        kind: SymbolKind::Constant,
                    }],
                },
            ],
        }
    }

    #[test]
    fn symbol_index_records_workspace_package_and_stdlib_readiness() {
        let index = SymbolIndex::build(revision(1, 1), &graph(), &analysis("main", "helper"));
        let readiness = index.bucket_readiness();

        assert!(readiness
            .iter()
            .any(|bucket| bucket.id.kind == SymbolBucketKind::Workspace
                && bucket.state == SymbolBucketReadinessState::Exact));
        assert!(readiness
            .iter()
            .any(|bucket| bucket.id.kind == SymbolBucketKind::Package
                && bucket.state == SymbolBucketReadinessState::Unavailable));
        assert!(readiness
            .iter()
            .any(|bucket| bucket.id.kind == SymbolBucketKind::Stdlib
                && bucket.state == SymbolBucketReadinessState::Unavailable));
        assert!(readiness.iter().any(|bucket| {
            bucket.id.kind == SymbolBucketKind::Workspace
                && bucket.id.module == Some(ModuleId::new(1))
                && bucket.state == SymbolBucketReadinessState::Exact
        }));
        assert!(readiness
            .iter()
            .any(|bucket| bucket.id.kind == SymbolBucketKind::Workspace
                && bucket.import_entry_count > 0));
        assert_eq!(index.workspace_import_symbols("helper").len(), 1);
    }

    #[test]
    fn readiness_state_availability_matches_bucket_query_rules() {
        assert!(SymbolBucketReadinessState::Exact.is_available());
        assert!(SymbolBucketReadinessState::StaleButUsable.is_available());
        assert!(!SymbolBucketReadinessState::NeedsBackgroundRefresh.is_available());
        assert!(!SymbolBucketReadinessState::Unavailable.is_available());
    }

    #[test]
    fn dirty_module_refresh_preserves_unchanged_bucket_identity() {
        let graph = graph();
        let mut index = SymbolIndex::build(revision(1, 1), &graph, &analysis("main", "helper"));
        let helper_id_before = index
            .entries()
            .iter()
            .find(|entry| entry.name == "helper" && entry.kind == "constant")
            .expect("helper symbol should exist")
            .id
            .clone();

        index.refresh_modules(
            revision(1, 2),
            &graph,
            &analysis("renamed", "helper"),
            &[ModuleId::new(1)],
        );

        assert!(index.entries().iter().any(|entry| entry.name == "renamed"));
        let helper_id_after = index
            .entries()
            .iter()
            .find(|entry| entry.name == "helper" && entry.kind == "constant")
            .expect("unchanged helper symbol should remain")
            .id
            .clone();
        assert_eq!(helper_id_before, helper_id_after);
    }

    #[test]
    fn dirty_refresh_matches_cold_rebuild_symbol_entries() {
        let graph = graph();
        let mut refreshed = SymbolIndex::build(revision(1, 1), &graph, &analysis("main", "helper"));
        refreshed.refresh_modules(
            revision(1, 2),
            &graph,
            &analysis("renamed", "helper"),
            &[ModuleId::new(1)],
        );
        let rebuilt = SymbolIndex::build(revision(1, 2), &graph, &analysis("renamed", "helper"));

        assert_eq!(refreshed.entries(), rebuilt.entries());
    }

    #[test]
    fn empty_dirty_refresh_advances_revision_without_changing_entries() {
        let graph = graph();
        let mut index = SymbolIndex::build(revision(1, 1), &graph, &analysis("main", "helper"));
        let entries = index.entries().to_vec();

        index.refresh_modules(revision(1, 2), &graph, &analysis("main", "helper"), &[]);

        assert_eq!(index.revision(), revision(1, 2));
        assert_eq!(index.entries(), entries.as_slice());
    }

    #[test]
    fn refresh_inserts_new_module_bucket_even_when_not_marked_dirty() {
        let graph = ModuleGraphView {
            modules: vec![
                ModuleGraphNode {
                    id: ModuleId::new(1),
                    file: FileId::new(0),
                    canonical_path: SourcePath::new("main.sifr"),
                    source_hash: SourceHash::new("main"),
                },
                ModuleGraphNode {
                    id: ModuleId::new(2),
                    file: FileId::new(2),
                    canonical_path: SourcePath::new("helper.sifr"),
                    source_hash: SourceHash::new("helper"),
                },
                ModuleGraphNode {
                    id: ModuleId::new(3),
                    file: FileId::new(3),
                    canonical_path: SourcePath::new("extra.sifr"),
                    source_hash: SourceHash::new("extra"),
                },
            ],
            edges: Vec::new(),
            entrypoint: ModuleId::new(1),
            revision: sifr_frontend::GraphRevision::new(2),
        };
        let mut analysis = analysis("main", "helper");
        let mut index = SymbolIndex::build(revision(1, 1), &graph, &analysis);
        analysis.modules.push(ModuleAnalysisView {
            module: ModuleId::new(3),
            symbols: vec![SymbolView {
                name: "extra".to_string(),
                kind: SymbolKind::Function,
            }],
        });

        index.refresh_modules(revision(2, 2), &graph, &analysis, &[ModuleId::new(1)]);

        assert!(index.entries().iter().any(|entry| entry.name == "extra"));
    }
}
