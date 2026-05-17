use crate::queries::{DocumentSymbol, WorkspaceSymbol};
use crate::snapshot::AnalysisRevision;
use sifr_frontend::{FileId, ModuleGraphView, ModuleId, ProjectAnalysisView, SymbolKind};
use std::collections::BTreeMap;

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
    entries: Vec<SymbolIndexEntry>,
}

impl SymbolIndex {
    #[must_use]
    pub fn build(
        revision: AnalysisRevision,
        graph: &ModuleGraphView,
        analysis: &ProjectAnalysisView,
    ) -> Self {
        let file_by_module = graph
            .modules
            .iter()
            .map(|module| (module.id, module.file))
            .collect::<BTreeMap<_, _>>();
        let mut entries = Vec::new();
        for module in &analysis.modules {
            let Some(file) = file_by_module.get(&module.module).copied() else {
                continue;
            };
            for (ordinal, symbol) in module.symbols.iter().enumerate() {
                let kind = symbol_kind_label(&symbol.kind);
                entries.push(SymbolIndexEntry {
                    id: SymbolId(format!(
                        "g{}:s{}:m{}:f{}:{kind}:{}:{ordinal}",
                        revision.graph.as_u64(),
                        revision.source.as_u64(),
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
        }
        entries.sort_by(|left, right| {
            left.file
                .cmp(&right.file)
                .then_with(|| left.name.cmp(&right.name))
                .then_with(|| left.kind.cmp(&right.kind))
                .then_with(|| left.ordinal.cmp(&right.ordinal))
        });
        Self { revision, entries }
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
        self.entries
            .iter()
            .filter(|entry| query.is_empty() || entry.name.contains(query))
            .map(|entry| WorkspaceSymbol {
                name: entry.name.clone(),
                kind: entry.kind.clone(),
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

fn symbol_kind_label(kind: &SymbolKind) -> String {
    match kind {
        SymbolKind::Function => "function",
        SymbolKind::Class => "class",
        SymbolKind::Constant => "constant",
        SymbolKind::Import => "import",
    }
    .to_string()
}
