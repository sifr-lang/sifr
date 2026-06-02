use crate::document_store::DocumentState;
use crate::errors::{LspError, LspResult};
use sifr_analysis::{
    AnalysisHost, AnalysisSnapshot, DocumentVersion, FileId, FrontendMode, ProjectRoot, SourcePath,
    SourceText, SymbolQuery, WorkspaceSymbol,
};
use sifr_diagnostics::RenderedDiagnostic;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;
use url::Url;

#[derive(Default)]
pub(crate) struct LspAnalysisWorkspace {
    documents: BTreeMap<String, LspDocumentAnalysis>,
    projects: BTreeMap<PathBuf, LspProjectAnalysis>,
}

struct LspDocumentAnalysis {
    host: Option<AnalysisHost>,
    file: Option<FileId>,
    load_diagnostics: Vec<RenderedDiagnostic>,
}

struct LspProjectAnalysis {
    host: Option<AnalysisHost>,
    files_by_uri: BTreeMap<String, FileId>,
    load_diagnostics: BTreeMap<String, Vec<RenderedDiagnostic>>,
    open_uris: BTreeSet<String>,
}

pub(crate) struct LspFileMaps {
    uri_by_file: BTreeMap<u32, String>,
    source_by_file: BTreeMap<u32, String>,
}

pub(crate) struct LspWorkspaceSymbol {
    pub(crate) symbol: WorkspaceSymbol,
    pub(crate) uri: String,
}

impl LspAnalysisWorkspace {
    pub(crate) const WATCHER_STORM_THRESHOLD: usize = 64;

    pub(crate) fn open_document(&mut self, document: &DocumentState) -> bool {
        if let Some(root) = workspace_root_for(document.path()) {
            self.documents.remove(document.uri());
            match self.projects.get_mut(&root) {
                Some(project) => {
                    if project.open_document(document).is_ok() {
                        false
                    } else {
                        let analysis = LspDocumentAnalysis::open(document);
                        self.documents.insert(document.uri().to_string(), analysis);
                        false
                    }
                }
                None => true,
            }
        } else {
            let analysis = LspDocumentAnalysis::open(document);
            self.documents.insert(document.uri().to_string(), analysis);
            false
        }
    }

    pub(crate) fn update_document(&mut self, document: &DocumentState) -> bool {
        let uri = document.uri().to_string();
        if let Some(root) = workspace_root_for(document.path()) {
            if let Some(analysis) = self.documents.get_mut(&uri) {
                analysis.update(document);
                return false;
            }
            match self.projects.get_mut(&root) {
                Some(project) => {
                    if project.update_document(document).is_ok() {
                        false
                    } else {
                        let analysis = LspDocumentAnalysis::open(document);
                        self.documents.insert(uri, analysis);
                        false
                    }
                }
                None => true,
            }
        } else {
            if let Some(analysis) = self.documents.get_mut(&uri) {
                analysis.update(document);
            } else {
                let analysis = LspDocumentAnalysis::open(document);
                self.documents.insert(uri, analysis);
            }
            false
        }
    }

    pub(crate) fn close_document(&mut self, uri: &str) {
        self.documents.remove(uri);
    }

    pub(crate) fn refresh_projects(&mut self, documents: &crate::document_store::DocumentStore) {
        let mut grouped: BTreeMap<PathBuf, Vec<&DocumentState>> = BTreeMap::new();
        for document in documents.documents() {
            if let Some(root) = workspace_root_for(document.path()) {
                grouped.entry(root).or_default().push(document);
            }
        }
        self.projects.retain(|root, _| grouped.contains_key(root));
        for (root, documents) in grouped {
            let open_uris = open_uris(&documents);
            if self
                .projects
                .get(&root)
                .is_some_and(|project| project.open_uris == open_uris)
            {
                continue;
            }
            let analysis = LspProjectAnalysis::open(root.clone(), &documents);
            for document in &documents {
                if analysis.files_by_uri.contains_key(document.uri()) {
                    self.documents.remove(document.uri());
                } else {
                    let fallback = LspDocumentAnalysis::open(document);
                    self.documents.insert(document.uri().to_string(), fallback);
                }
            }
            self.projects.insert(root, analysis);
        }
    }

    pub(crate) fn record_watcher_events(&mut self, event_count: usize) {
        for analysis in self.projects.values_mut() {
            if let Some(host) = analysis.host.as_mut() {
                host.record_watcher_events(event_count, Self::WATCHER_STORM_THRESHOLD);
            }
        }
        for analysis in self.documents.values_mut() {
            if let Some(host) = analysis.host.as_mut() {
                host.record_watcher_events(event_count, Self::WATCHER_STORM_THRESHOLD);
            }
        }
    }

    pub(crate) fn load_diagnostics(&self, uri: &str) -> &[RenderedDiagnostic] {
        if let Some(diagnostics) = self
            .projects
            .values()
            .find_map(|project| project.load_diagnostics.get(uri))
        {
            return diagnostics;
        }
        self.documents
            .get(uri)
            .map_or(&[][..], |analysis| analysis.load_diagnostics.as_slice())
    }

    pub(crate) fn file_maps_for_document(
        &self,
        document: &DocumentState,
        documents: &crate::document_store::DocumentStore,
    ) -> LspResult<LspFileMaps> {
        if let Some(root) = workspace_root_for(document.path()) {
            if let Some(project) = self.projects.get(&root) {
                if project.files_by_uri.contains_key(document.uri()) {
                    return project.file_maps();
                }
            }
        }
        let analysis = self.documents.get(document.uri()).ok_or_else(|| {
            LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            ))
        })?;
        analysis.file_maps(document.uri(), documents)
    }

    pub(crate) fn workspace_symbols(
        &mut self,
        query: &SymbolQuery,
    ) -> LspResult<Vec<LspWorkspaceSymbol>> {
        let mut symbols = Vec::new();
        for project in self.projects.values_mut() {
            if let Some(project_symbols) = project.workspace_symbols(query)? {
                symbols.extend(project_symbols);
            }
        }
        let project_uris = self.project_owned_uris();
        for (uri, analysis) in self
            .documents
            .iter_mut()
            .filter(|(uri, _)| !project_uris.contains(uri.as_str()))
        {
            symbols.extend(analysis.workspace_symbols(query, uri)?);
        }
        Ok(symbols)
    }

    pub(crate) fn with_document<T>(
        &mut self,
        document: &DocumentState,
        operation: impl FnOnce(&AnalysisSnapshot, &mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        if let Some(root) = workspace_root_for(document.path()) {
            if let Some(project) = self.projects.get_mut(&root) {
                if project.files_by_uri.contains_key(document.uri()) {
                    return project.with_host(document, operation);
                }
            }
        }
        let analysis = self.documents.get_mut(document.uri()).ok_or_else(|| {
            LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            ))
        })?;
        analysis.with_host(document, operation)
    }

    fn project_owned_uris(&self) -> BTreeSet<String> {
        self.projects
            .values()
            .flat_map(|project| project.files_by_uri.keys().cloned())
            .collect()
    }
}

impl LspFileMaps {
    pub(crate) fn uri_for(&self, file: FileId) -> LspResult<String> {
        self.uri_by_file
            .get(&file.as_u32())
            .cloned()
            .ok_or_else(|| LspError::internal(format!("unknown file {}", file.as_u32())))
    }

    pub(crate) fn source_for(&self, file: FileId) -> LspResult<String> {
        self.source_by_file
            .get(&file.as_u32())
            .cloned()
            .ok_or_else(|| LspError::internal(format!("unknown source {}", file.as_u32())))
    }
}

impl LspProjectAnalysis {
    fn open(root: PathBuf, documents: &[&DocumentState]) -> Self {
        let overlays = documents
            .iter()
            .map(|document| {
                (
                    SourcePath::new(document.path().to_path_buf()),
                    Some(document.uri().to_string()),
                    document_version(document),
                    SourceText::new(document.text().to_string()),
                )
            })
            .collect::<Vec<_>>();
        let Some(entrypoint) = project_entrypoint(&root, documents) else {
            return Self {
                host: None,
                files_by_uri: BTreeMap::new(),
                load_diagnostics: BTreeMap::new(),
                open_uris: open_uris(documents),
            };
        };
        let started = Instant::now();
        let project_root = ProjectRoot {
            root: SourcePath::new(root),
            entrypoint: SourcePath::new(entrypoint),
        };
        match AnalysisHost::open_project_with_overlays(&project_root, overlays) {
            Ok(mut host) => {
                host.record_update_latency_ms(elapsed_ms(started));
                let files_by_uri = documents
                    .iter()
                    .filter_map(|document| {
                        host.document_file_for_path(document.path())
                            .ok()
                            .map(|file| (document.uri().to_string(), file))
                    })
                    .collect();
                Self {
                    host: Some(host),
                    files_by_uri,
                    load_diagnostics: BTreeMap::new(),
                    open_uris: open_uris(documents),
                }
            }
            Err(diagnostics) => Self {
                host: None,
                files_by_uri: BTreeMap::new(),
                load_diagnostics: documents
                    .iter()
                    .map(|document| (document.uri().to_string(), diagnostics.clone()))
                    .collect(),
                open_uris: open_uris(documents),
            },
        }
    }

    fn open_document(&mut self, document: &DocumentState) -> Result<(), ()> {
        let Some(host) = self.host.as_mut() else {
            return Err(());
        };
        let started = Instant::now();
        host.upsert_overlay_document(
            SourcePath::new(document.path().to_path_buf()),
            Some(document.uri().to_string()),
            document_version(document),
            SourceText::new(document.text().to_string()),
        )
        .map_err(|_| ())?;
        let file = host
            .document_file_for_path(document.path())
            .map_err(|_| ())?;
        host.record_update_latency_ms(elapsed_ms(started));
        self.files_by_uri.insert(document.uri().to_string(), file);
        self.load_diagnostics.remove(document.uri());
        self.open_uris.insert(document.uri().to_string());
        Ok(())
    }

    fn update_document(&mut self, document: &DocumentState) -> Result<(), ()> {
        let Some(file) = self.files_by_uri.get(document.uri()).copied() else {
            return Err(());
        };
        let Some(host) = self.host.as_mut() else {
            return Err(());
        };
        let started = Instant::now();
        host.update_document(
            file,
            document_version(document),
            SourceText::new(document.text().to_string()),
        )
        .map_err(|_| ())?;
        host.record_update_latency_ms(elapsed_ms(started));
        self.load_diagnostics.remove(document.uri());
        Ok(())
    }

    fn with_host<T>(
        &mut self,
        document: &DocumentState,
        operation: impl FnOnce(&AnalysisSnapshot, &mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        let Some(file) = self.files_by_uri.get(document.uri()).copied() else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            )));
        };
        let Some(host) = self.host.as_mut() else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            )));
        };
        let snapshot = host.snapshot();
        let result = operation(&snapshot, host, file, document.text())?;
        if !host.is_snapshot_current(&snapshot) {
            return Err(LspError::request_cancelled(
                "query result was superseded by a newer analysis snapshot",
            ));
        }
        Ok(result)
    }

    fn file_maps(&self) -> LspResult<LspFileMaps> {
        let Some(host) = self.host.as_ref() else {
            return Err(LspError::internal(
                "analysis is unavailable for project file maps",
            ));
        };
        let uri_by_file = Self::uri_by_file(&self.files_by_uri, host);
        let source_by_file = host
            .files()
            .into_iter()
            .filter_map(|file| {
                host.source_text_for_file(file)
                    .ok()
                    .map(|source| (file.as_u32(), source.to_string()))
            })
            .collect();
        Ok(LspFileMaps {
            uri_by_file,
            source_by_file,
        })
    }

    fn uri_by_file(
        files_by_uri: &BTreeMap<String, FileId>,
        host: &AnalysisHost,
    ) -> BTreeMap<u32, String> {
        let mut uri_by_file = BTreeMap::new();
        for (uri, file) in files_by_uri {
            uri_by_file.insert(file.as_u32(), uri.clone());
        }
        for file in host.files() {
            if uri_by_file.contains_key(&file.as_u32()) {
                continue;
            }
            let Some(uri) = host.path_for_file(file).ok().and_then(file_uri_for_path) else {
                continue;
            };
            uri_by_file.insert(file.as_u32(), uri);
        }
        uri_by_file
    }

    fn workspace_symbols(
        &mut self,
        query: &SymbolQuery,
    ) -> LspResult<Option<Vec<LspWorkspaceSymbol>>> {
        let Some(host) = self.host.as_mut() else {
            return Ok(None);
        };
        let uri_by_file = Self::uri_by_file(&self.files_by_uri, host);
        let snapshot = host.snapshot();
        let result = snapshot
            .workspace_symbols(host, query)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        if !host.is_snapshot_current(&snapshot) {
            return Err(LspError::request_cancelled(
                "query result was superseded by a newer analysis snapshot",
            ));
        }
        Ok(Some(
            result
                .into_iter()
                .filter_map(|symbol| {
                    let uri = uri_by_file.get(&symbol.file.as_u32())?.clone();
                    Some(LspWorkspaceSymbol { symbol, uri })
                })
                .collect(),
        ))
    }
}

impl LspDocumentAnalysis {
    fn open(document: &DocumentState) -> Self {
        let started = Instant::now();
        match AnalysisHost::open_single_file_overlay(
            SourcePath::new(document.path().to_path_buf()),
            Some(document.uri().to_string()),
            document_version(document),
            SourceText::new(document.text().to_string()),
            FrontendMode::SingleFile,
        ) {
            Ok(mut host) => {
                host.record_update_latency_ms(elapsed_ms(started));
                Self::from_host(host, document)
            }
            Err(diagnostics) => Self {
                host: None,
                file: None,
                load_diagnostics: diagnostics,
            },
        }
    }

    fn update(&mut self, document: &DocumentState) {
        let started = Instant::now();
        let result = if let Some(host) = self.host.as_mut() {
            host.upsert_overlay_document(
                SourcePath::new(document.path().to_path_buf()),
                Some(document.uri().to_string()),
                document_version(document),
                SourceText::new(document.text().to_string()),
            )
        } else {
            return *self = Self::open(document);
        };
        match result {
            Ok(()) => {
                if let Some(mut host) = self.host.take() {
                    host.record_update_latency_ms(elapsed_ms(started));
                    *self = Self::from_host(host, document);
                }
            }
            Err(diagnostics) => {
                self.host = None;
                self.file = None;
                self.load_diagnostics = diagnostics;
            }
        }
    }

    fn from_host(host: AnalysisHost, document: &DocumentState) -> Self {
        let file = host.document_file_for_path(document.path()).ok();
        Self {
            host: Some(host),
            file,
            load_diagnostics: Vec::new(),
        }
    }

    fn with_host<T>(
        &mut self,
        document: &DocumentState,
        operation: impl FnOnce(&AnalysisSnapshot, &mut AnalysisHost, FileId, &str) -> LspResult<T>,
    ) -> LspResult<T> {
        let Some(file) = self.file else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            )));
        };
        let Some(host) = self.host.as_mut() else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {}",
                document.path().display()
            )));
        };
        let snapshot = host.snapshot();
        let result = operation(&snapshot, host, file, document.text())?;
        if !host.is_snapshot_current(&snapshot) {
            return Err(LspError::request_cancelled(
                "query result was superseded by a newer analysis snapshot",
            ));
        }
        Ok(result)
    }

    fn file_maps(
        &self,
        uri: &str,
        documents: &crate::document_store::DocumentStore,
    ) -> LspResult<LspFileMaps> {
        let file = self.file.ok_or_else(|| {
            LspError::internal(format!(
                "analysis is unavailable for standalone document {uri}"
            ))
        })?;
        let source = documents.document(uri)?.text().to_string();
        Ok(LspFileMaps {
            uri_by_file: BTreeMap::from([(file.as_u32(), uri.to_string())]),
            source_by_file: BTreeMap::from([(file.as_u32(), source)]),
        })
    }

    fn workspace_symbols(
        &mut self,
        query: &SymbolQuery,
        uri: &str,
    ) -> LspResult<Vec<LspWorkspaceSymbol>> {
        let Some(host) = self.host.as_mut() else {
            return Err(LspError::internal(format!(
                "analysis is unavailable for {uri}"
            )));
        };
        let snapshot = host.snapshot();
        let result = snapshot
            .workspace_symbols(host, query)
            .map_err(|error| LspError::internal(error.message))?
            .into_value();
        if !host.is_snapshot_current(&snapshot) {
            return Err(LspError::request_cancelled(
                "query result was superseded by a newer analysis snapshot",
            ));
        }
        Ok(result
            .into_iter()
            .map(|symbol| LspWorkspaceSymbol {
                symbol,
                uri: uri.to_string(),
            })
            .collect())
    }
}

fn document_version(document: &DocumentState) -> DocumentVersion {
    DocumentVersion::new(i64::from(document.version().unwrap_or_default()))
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn workspace_root_for(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent()?.to_path_buf();
    loop {
        if current.join("sifr.toml").is_file() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn file_uri_for_path(path: &Path) -> Option<String> {
    Url::from_file_path(path).ok().map(Into::into)
}

fn project_entrypoint(root: &Path, documents: &[&DocumentState]) -> Option<PathBuf> {
    for candidate in [root.join("src/main.sifr"), root.join("main.sifr")] {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    documents
        .iter()
        .copied()
        .find(|document| {
            document
                .path()
                .file_stem()
                .is_some_and(|stem| stem == "main")
        })
        .or_else(|| documents.first().copied())
        .map(|document| document.path().to_path_buf())
}

fn open_uris(documents: &[&DocumentState]) -> BTreeSet<String> {
    documents
        .iter()
        .map(|document| document.uri().to_string())
        .collect()
}
