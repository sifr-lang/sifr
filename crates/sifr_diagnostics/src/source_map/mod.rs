use ruff_text_size::TextRange;
use serde::{Deserialize, Serialize};
use sifr_source::SourceText;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceId(u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub source_id: SourceId,
    #[serde(with = "text_range_serde")]
    pub range: TextRange,
    pub lowered_from: Option<Box<SourceSpan>>,
}

impl SourceSpan {
    #[must_use]
    pub const fn new(source_id: SourceId, range: TextRange) -> Self {
        Self {
            source_id,
            range,
            lowered_from: None,
        }
    }

    pub fn new_validated(
        source_map: &SourceMap,
        source_id: SourceId,
        range: TextRange,
    ) -> Result<Self, SourceMapError> {
        let span = Self::new(source_id, range);
        source_map.validate_span(&span)?;
        Ok(span)
    }

    #[must_use]
    pub fn with_lowered_from(mut self, lowered_from: SourceSpan) -> Self {
        self.lowered_from = Some(Box::new(lowered_from));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceMapError {
    UnknownSource(SourceId),
    InvalidSpan {
        source_id: SourceId,
        byte_start: u32,
        byte_end: u32,
        source_len: u32,
    },
}

#[derive(Debug, Clone)]
struct SourceFile {
    canonical_path: Option<String>,
    display_path: String,
    module_name: Option<String>,
    source_hash: u64,
    text: SourceText,
}

#[derive(Debug, Default, Clone)]
pub struct SourceMap {
    next_id: u32,
    sources: HashMap<SourceId, SourceFile>,
}

impl SourceMap {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_source(
        &mut self,
        display_path: impl Into<String>,
        text: impl Into<String>,
    ) -> SourceId {
        self.register_source_with_metadata(display_path, None::<String>, None::<String>, text)
    }

    pub fn register_source_with_metadata(
        &mut self,
        display_path: impl Into<String>,
        canonical_path: Option<impl Into<String>>,
        module_name: Option<impl Into<String>>,
        text: impl Into<String>,
    ) -> SourceId {
        let id = SourceId(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .unwrap_or_else(|| panic!("source id allocation overflowed"));
        let text = SourceText::new(text);
        let source_hash = text.source_hash();
        self.sources.insert(
            id,
            SourceFile {
                canonical_path: canonical_path.map(Into::into),
                display_path: display_path.into(),
                module_name: module_name.map(Into::into),
                source_hash,
                text,
            },
        );
        id
    }

    #[must_use]
    pub fn display_path(&self, source_id: SourceId) -> Option<&str> {
        self.sources
            .get(&source_id)
            .map(|source| source.display_path.as_str())
    }

    #[must_use]
    pub fn canonical_path(&self, source_id: SourceId) -> Option<&str> {
        self.sources
            .get(&source_id)
            .and_then(|source| source.canonical_path.as_deref())
    }

    #[must_use]
    pub fn module_name(&self, source_id: SourceId) -> Option<&str> {
        self.sources
            .get(&source_id)
            .and_then(|source| source.module_name.as_deref())
    }

    #[must_use]
    pub fn source_hash(&self, source_id: SourceId) -> Option<u64> {
        self.sources
            .get(&source_id)
            .map(|source| source.source_hash)
    }

    pub fn validate_span(&self, span: &SourceSpan) -> Result<(), SourceMapError> {
        let source = self
            .sources
            .get(&span.source_id)
            .ok_or(SourceMapError::UnknownSource(span.source_id))?;
        let byte_start = span.range.start().to_u32();
        let byte_end = span.range.end().to_u32();
        let source_len = u32::try_from(source.text.as_str().len()).unwrap_or(u32::MAX);
        if byte_start > byte_end || byte_end > source_len {
            return Err(SourceMapError::InvalidSpan {
                source_id: span.source_id,
                byte_start,
                byte_end,
                source_len,
            });
        }
        Ok(())
    }

    pub(crate) fn source(&self, source_id: SourceId) -> Option<&SourceText> {
        self.sources.get(&source_id).map(|source| &source.text)
    }
}

mod text_range_serde {
    #![allow(clippy::trivially_copy_pass_by_ref)]

    use ruff_text_size::{TextRange, TextSize};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct TextRangeJson {
        byte_start: u32,
        byte_end: u32,
    }

    pub(super) fn serialize<S>(range: &TextRange, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        TextRangeJson {
            byte_start: range.start().to_u32(),
            byte_end: range.end().to_u32(),
        }
        .serialize(serializer)
    }

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<TextRange, D::Error>
    where
        D: Deserializer<'de>,
    {
        let range = TextRangeJson::deserialize(deserializer)?;
        Ok(TextRange::new(
            TextSize::new(range.byte_start),
            TextSize::new(range.byte_end),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{SourceMap, SourceMapError, SourceSpan};
    use ruff_text_size::{TextRange, TextSize};

    #[test]
    fn validates_zero_length_and_eof_spans() {
        let mut source_map = SourceMap::new();
        let source_id = source_map.register_source("emoji.sifr", "let x = '🦀'\n");
        let eof = TextSize::new(15);
        assert!(
            source_map
                .validate_span(&SourceSpan::new(source_id, TextRange::new(eof, eof)))
                .is_ok()
        );
    }

    #[test]
    fn new_validated_rejects_invalid_construction() {
        let mut source_map = SourceMap::new();
        let source_id = source_map.register_source("short.sifr", "x\n");
        let err = SourceSpan::new_validated(
            &source_map,
            source_id,
            TextRange::new(TextSize::new(3), TextSize::new(4)),
        )
        .unwrap_err();
        assert!(matches!(err, SourceMapError::InvalidSpan { .. }));
    }

    #[test]
    fn rejects_out_of_bounds_spans() {
        let mut source_map = SourceMap::new();
        let source_id = source_map.register_source("short.sifr", "x\n");
        let err = source_map
            .validate_span(&SourceSpan::new(
                source_id,
                TextRange::new(TextSize::new(0), TextSize::new(99)),
            ))
            .unwrap_err();
        assert!(matches!(err, SourceMapError::InvalidSpan { .. }));
    }

    #[test]
    fn source_metadata_is_recorded() {
        let mut source_map = SourceMap::new();
        let source_id = source_map.register_source_with_metadata(
            "src/main.sifr",
            Some("/repo/src/main.sifr"),
            Some("main"),
            "x = 1\n",
        );
        assert_eq!(source_map.display_path(source_id), Some("src/main.sifr"));
        assert_eq!(
            source_map.canonical_path(source_id),
            Some("/repo/src/main.sifr")
        );
        assert_eq!(source_map.module_name(source_id), Some("main"));
        assert!(source_map.source_hash(source_id).is_some());
    }
}
