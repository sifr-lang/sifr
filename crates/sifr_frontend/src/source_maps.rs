use ruff_text_size::TextRange;
pub use sifr_source::{PositionEncoding, SourceText, TextPosition, TextRangeUtf};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub(crate) u32);

impl FileId {
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
pub struct SourceRevision(pub(crate) u64);

impl SourceRevision {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceHash(pub(crate) String);

impl SourceHash {
    #[doc(hidden)]
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn from_source_text(source: &str) -> Self {
        crate::stable_source_hash(source)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFileView {
    pub id: FileId,
    pub canonical_path: SourcePath,
    pub uri: Option<SourceUri>,
    pub source_hash: SourceHash,
    pub document_version: Option<DocumentVersion>,
    pub source: SourceText,
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
        file: FileId,
        position: &TextPosition,
        encoding: PositionEncoding,
    ) -> Option<TextRange> {
        let source = self.source_for_file(file)?;
        let offset = source.byte_offset_with_encoding(position, encoding)?;
        Some(TextRange::new(offset, offset))
    }

    #[must_use]
    pub fn span_to_text_range(
        &self,
        file: FileId,
        span: TextRange,
        encoding: PositionEncoding,
    ) -> Option<TextRangeUtf> {
        self.source_for_file(file)?.range_at(span, encoding)
    }

    #[must_use]
    pub fn source_for_file(&self, file: FileId) -> Option<&SourceText> {
        self.files
            .iter()
            .find(|source_file| source_file.id == file)
            .map(|source_file| &source_file.source)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DocumentVersion, FileId, PositionEncoding, SourceFileView, SourceHash, SourceMapView,
        SourcePath, SourceRevision, SourceText, TextPosition,
    };
    use ruff_text_size::{TextRange, TextSize};

    fn source_map(source: &str) -> SourceMapView {
        SourceMapView {
            files: vec![SourceFileView {
                id: FileId::new(0),
                canonical_path: SourcePath::new("main.sifr"),
                uri: None,
                source_hash: SourceHash("hash".to_string()),
                document_version: Some(DocumentVersion::new(7)),
                source: SourceText::new(source),
            }],
            revision: SourceRevision(0),
        }
    }

    #[test]
    fn source_map_lookup_round_trips_multibyte_positions() {
        let map = source_map("a🦀b\n");
        let position = TextPosition {
            line: 0,
            character: 5,
        };
        let span = map
            .text_position_to_span(FileId::new(0), &position, PositionEncoding::Utf8)
            .unwrap();
        assert_eq!(span, TextRange::new(TextSize::new(5), TextSize::new(5)));
        assert_eq!(
            map.span_to_text_range(FileId::new(0), span, PositionEncoding::Utf16),
            Some(sifr_source::TextRangeUtf {
                start: TextPosition {
                    line: 0,
                    character: 3,
                },
                end: TextPosition {
                    line: 0,
                    character: 3,
                },
            })
        );
    }

    #[test]
    fn source_map_lookup_rejects_unregistered_files_and_invalid_boundaries() {
        let map = source_map("a🦀b\r\n");
        assert_eq!(
            map.text_position_to_span(
                FileId::new(1),
                &TextPosition {
                    line: 0,
                    character: 0,
                },
                PositionEncoding::Utf8,
            ),
            None
        );
        assert_eq!(
            map.text_position_to_span(
                FileId::new(0),
                &TextPosition {
                    line: 0,
                    character: 2,
                },
                PositionEncoding::Utf8,
            ),
            None
        );
    }
}
