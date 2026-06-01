use super::{source_hash, DocumentVersion, SourceHash, SourcePath, SourceText};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceProviderErrorKind {
    Canonicalize,
    FileRead,
    DirectoryRead,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceProviderError {
    pub kind: SourceProviderErrorKind,
    pub path: PathBuf,
    pub message: String,
}

impl SourceProviderError {
    #[must_use]
    pub fn new(kind: SourceProviderErrorKind, path: impl Into<PathBuf>, message: String) -> Self {
        Self {
            kind,
            path: path.into(),
            message,
        }
    }
}

impl fmt::Display for SourceProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.path.display(), self.message)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDirEntry {
    pub path: PathBuf,
    pub file_name: std::ffi::OsString,
    pub is_file: bool,
    pub is_dir: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceDependencyKind {
    FileRead,
    DirectoryRead,
    FileProbe { exists: bool },
    DirectoryProbe { exists: bool },
    Canonicalize,
    FailedLookup,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceDependency {
    pub path: PathBuf,
    pub kind: SourceDependencyKind,
}

pub trait SourceProvider {
    fn read_file(&mut self, path: &Path) -> Result<SourceText, SourceProviderError>;
    fn read_dir(&mut self, path: &Path) -> Result<Vec<SourceDirEntry>, SourceProviderError>;
    fn is_file(&mut self, path: &Path) -> bool;
    fn is_dir(&mut self, path: &Path) -> bool;
    fn canonicalize(&mut self, path: &Path) -> Result<PathBuf, SourceProviderError>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiskSourceProvider;

impl DiskSourceProvider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl SourceProvider for DiskSourceProvider {
    fn read_file(&mut self, path: &Path) -> Result<SourceText, SourceProviderError> {
        std::fs::read_to_string(path)
            .map(SourceText::new)
            .map_err(|error| {
                SourceProviderError::new(SourceProviderErrorKind::FileRead, path, error.to_string())
            })
    }

    fn read_dir(&mut self, path: &Path) -> Result<Vec<SourceDirEntry>, SourceProviderError> {
        let entries = std::fs::read_dir(path).map_err(|error| {
            SourceProviderError::new(
                SourceProviderErrorKind::DirectoryRead,
                path,
                error.to_string(),
            )
        })?;
        let mut output = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| {
                SourceProviderError::new(
                    SourceProviderErrorKind::DirectoryRead,
                    path,
                    error.to_string(),
                )
            })?;
            let file_type = entry.file_type().map_err(|error| {
                SourceProviderError::new(
                    SourceProviderErrorKind::DirectoryRead,
                    entry.path(),
                    error.to_string(),
                )
            })?;
            output.push(SourceDirEntry {
                path: entry.path(),
                file_name: entry.file_name(),
                is_file: file_type.is_file(),
                is_dir: file_type.is_dir(),
            });
        }
        Ok(output)
    }

    fn is_file(&mut self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&mut self, path: &Path) -> bool {
        path.is_dir()
    }

    fn canonicalize(&mut self, path: &Path) -> Result<PathBuf, SourceProviderError> {
        path.canonicalize().map_err(|error| {
            SourceProviderError::new(
                SourceProviderErrorKind::Canonicalize,
                path,
                error.to_string(),
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlayDocument {
    pub path: SourcePath,
    pub uri: Option<String>,
    pub version: DocumentVersion,
    pub source: SourceText,
    pub source_hash: SourceHash,
    pub matches_disk: bool,
}

impl OverlayDocument {
    #[must_use]
    pub fn new(
        path: SourcePath,
        uri: Option<String>,
        version: DocumentVersion,
        source: SourceText,
        disk_source: Option<&str>,
    ) -> Self {
        let source_hash = source_hash(source.as_str());
        let matches_disk = disk_source.is_some_and(|disk_source| disk_source == source.as_str());
        Self {
            path,
            uri,
            version,
            source,
            source_hash,
            matches_disk,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlaySourceProvider<P> {
    inner: P,
    overlays: BTreeMap<PathBuf, OverlayDocument>,
}

impl<P> OverlaySourceProvider<P> {
    #[must_use]
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            overlays: BTreeMap::new(),
        }
    }

    pub fn insert_overlay(&mut self, overlay: OverlayDocument) {
        self.overlays
            .insert(overlay.path.as_path().to_path_buf(), overlay);
    }

    #[must_use]
    pub fn overlays(&self) -> &BTreeMap<PathBuf, OverlayDocument> {
        &self.overlays
    }
}

impl<P: SourceProvider> SourceProvider for OverlaySourceProvider<P> {
    fn read_file(&mut self, path: &Path) -> Result<SourceText, SourceProviderError> {
        if let Some(overlay) = self.overlays.get(path) {
            return Ok(overlay.source.clone());
        }
        self.inner.read_file(path)
    }

    fn read_dir(&mut self, path: &Path) -> Result<Vec<SourceDirEntry>, SourceProviderError> {
        let mut entries = match self.inner.read_dir(path) {
            Ok(entries) => entries,
            Err(error) => {
                if !self
                    .overlays
                    .keys()
                    .any(|overlay| overlay_descends_from(overlay, path))
                {
                    return Err(error);
                }
                // TODO(m6): surface the dirty directory when watcher invalidation
                // starts consuming overlay-backed directory dependencies.
                Vec::new()
            }
        };
        for overlay_path in self.overlays.keys() {
            if let Some(entry) = overlay_dir_entry(path, overlay_path) {
                if entries.iter().any(|existing| existing.path == entry.path) {
                    continue;
                }
                entries.push(SourceDirEntry {
                    path: entry.path,
                    file_name: entry
                        .file_name
                        .file_name()
                        .map_or_else(std::ffi::OsString::new, std::ffi::OsString::from),
                    is_file: entry.is_file,
                    is_dir: entry.is_dir,
                });
            }
        }
        entries.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(entries)
    }

    fn is_file(&mut self, path: &Path) -> bool {
        self.overlays.contains_key(path) || self.inner.is_file(path)
    }

    fn is_dir(&mut self, path: &Path) -> bool {
        self.inner.is_dir(path)
            || self
                .overlays
                .keys()
                .any(|overlay| overlay_descends_from(overlay, path))
    }

    fn canonicalize(&mut self, path: &Path) -> Result<PathBuf, SourceProviderError> {
        if self.overlays.contains_key(path) {
            Ok(path.to_path_buf())
        } else {
            self.inner.canonicalize(path)
        }
    }
}

struct OverlayDirEntry {
    path: PathBuf,
    file_name: PathBuf,
    is_file: bool,
    is_dir: bool,
}

fn overlay_descends_from(overlay: &Path, directory: &Path) -> bool {
    overlay_dir_entry(directory, overlay).is_some()
}

fn overlay_dir_entry(directory: &Path, overlay: &Path) -> Option<OverlayDirEntry> {
    let relative = overlay.strip_prefix(directory).ok()?;
    let mut components = relative.components();
    let first = components.next()?;
    let file_name = PathBuf::from(first.as_os_str());
    let entry_path = directory.join(&file_name);
    if components.next().is_some() {
        Some(OverlayDirEntry {
            path: entry_path,
            file_name,
            is_file: false,
            is_dir: true,
        })
    } else {
        Some(OverlayDirEntry {
            path: overlay.to_path_buf(),
            file_name,
            is_file: true,
            is_dir: false,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackingSourceProvider<P> {
    inner: P,
    dependencies: Vec<SourceDependency>,
}

impl<P> TrackingSourceProvider<P> {
    #[must_use]
    pub fn new(inner: P) -> Self {
        Self {
            inner,
            dependencies: Vec::new(),
        }
    }

    #[must_use]
    pub fn dependencies(&self) -> &[SourceDependency] {
        &self.dependencies
    }

    pub fn into_parts(self) -> (P, Vec<SourceDependency>) {
        (self.inner, self.dependencies)
    }

    fn record(&mut self, path: &Path, kind: SourceDependencyKind) {
        self.dependencies.push(SourceDependency {
            path: path.to_path_buf(),
            kind,
        });
    }
}

impl<P: SourceProvider> SourceProvider for TrackingSourceProvider<P> {
    fn read_file(&mut self, path: &Path) -> Result<SourceText, SourceProviderError> {
        match self.inner.read_file(path) {
            Ok(source) => {
                self.record(path, SourceDependencyKind::FileRead);
                Ok(source)
            }
            Err(error) => {
                self.record(path, SourceDependencyKind::FailedLookup);
                Err(error)
            }
        }
    }

    fn read_dir(&mut self, path: &Path) -> Result<Vec<SourceDirEntry>, SourceProviderError> {
        match self.inner.read_dir(path) {
            Ok(entries) => {
                self.record(path, SourceDependencyKind::DirectoryRead);
                Ok(entries)
            }
            Err(error) => {
                self.record(path, SourceDependencyKind::FailedLookup);
                Err(error)
            }
        }
    }

    fn is_file(&mut self, path: &Path) -> bool {
        let exists = self.inner.is_file(path);
        self.record(path, SourceDependencyKind::FileProbe { exists });
        exists
    }

    fn is_dir(&mut self, path: &Path) -> bool {
        let exists = self.inner.is_dir(path);
        self.record(path, SourceDependencyKind::DirectoryProbe { exists });
        exists
    }

    fn canonicalize(&mut self, path: &Path) -> Result<PathBuf, SourceProviderError> {
        match self.inner.canonicalize(path) {
            Ok(canonical) => {
                self.record(path, SourceDependencyKind::Canonicalize);
                Ok(canonical)
            }
            Err(error) => {
                self.record(path, SourceDependencyKind::FailedLookup);
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiskSourceProvider, OverlayDocument, OverlaySourceProvider, SourceDependencyKind,
        SourceProvider, TrackingSourceProvider,
    };
    use crate::{DocumentVersion, SourcePath, SourceText};
    use std::path::Path;

    #[test]
    fn overlay_provider_prefers_unsaved_text_over_disk() {
        let mut provider = OverlaySourceProvider::new(DiskSourceProvider::new());
        provider.insert_overlay(OverlayDocument::new(
            SourcePath::new("main.sifr"),
            Some("file:///main.sifr".to_string()),
            DocumentVersion::new(3),
            SourceText::new("value = 2\n"),
            Some("value = 1\n"),
        ));

        let source = provider
            .read_file(Path::new("main.sifr"))
            .expect("overlay source exists");
        assert_eq!(source.as_str(), "value = 2\n");
        assert!(!provider.overlays()[Path::new("main.sifr")].matches_disk);
    }

    #[test]
    fn tracking_provider_records_successes_and_failed_lookups() {
        let mut provider = TrackingSourceProvider::new(DiskSourceProvider::new());
        assert!(!provider.is_file(Path::new("definitely_missing.sifr")));
        let _ = provider.read_file(Path::new("definitely_missing.sifr"));

        assert_eq!(
            provider
                .dependencies()
                .iter()
                .map(|dependency| &dependency.kind)
                .collect::<Vec<_>>(),
            vec![
                &SourceDependencyKind::FileProbe { exists: false },
                &SourceDependencyKind::FailedLookup,
            ]
        );
    }

    #[test]
    fn overlay_provider_synthesizes_nested_directories() {
        let root = std::env::temp_dir().join(format!(
            "sifr_overlay_nested_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        ));
        let nested_file = root.join("pkg").join("mod.sifr");
        let mut provider = OverlaySourceProvider::new(DiskSourceProvider::new());
        provider.insert_overlay(OverlayDocument::new(
            SourcePath::new(&nested_file),
            Some("file:///tmp/pkg/mod.sifr".to_string()),
            DocumentVersion::new(1),
            SourceText::new("value: int = 1\n"),
            None,
        ));

        assert!(provider.is_dir(&root));
        let root_entries = provider.read_dir(&root).expect("overlay root is readable");
        assert_eq!(root_entries.len(), 1);
        assert!(root_entries[0].is_dir);
        assert_eq!(root_entries[0].path, root.join("pkg"));

        let nested_entries = provider
            .read_dir(&root.join("pkg"))
            .expect("overlay child directory is readable");
        assert_eq!(nested_entries.len(), 1);
        assert!(nested_entries[0].is_file);
        assert_eq!(nested_entries[0].path, nested_file);
    }
}
