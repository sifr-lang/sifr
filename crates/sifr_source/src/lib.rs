//! Canonical source text, line-map, and editor position conversion primitives.
//!
//! This crate intentionally sits at the bottom of the Sifr dependency graph.
//! Higher compiler and tooling crates may depend on it; it must not depend on
//! syntax, diagnostics, frontend, analysis, LSP, package, driver, or CLI crates.
#![cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]

use ruff_text_size::{TextRange, TextSize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
    Utf32,
}

impl PositionEncoding {
    pub const UTF8: Self = Self::Utf8;
    pub const UTF16: Self = Self::Utf16;
    pub const UTF32: Self = Self::Utf32;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextRangeUtf {
    pub start: TextPosition,
    pub end: TextPosition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineMap {
    line_starts: Arc<[TextSize]>,
    text_len: TextSize,
}

impl LineMap {
    #[must_use]
    pub fn new(text: &str) -> Self {
        let mut line_starts = vec![TextSize::new(0)];
        for (index, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                if let Ok(next) = u32::try_from(index + 1) {
                    line_starts.push(TextSize::new(next));
                }
            }
        }
        let text_len = TextSize::new(u32::try_from(text.len()).unwrap_or(u32::MAX));
        Self {
            line_starts: Arc::from(line_starts),
            text_len,
        }
    }

    #[must_use]
    pub fn line_starts(&self) -> &[TextSize] {
        &self.line_starts
    }

    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    #[must_use]
    pub fn eof(&self) -> TextSize {
        self.text_len
    }

    #[must_use]
    pub fn line_full_byte_range(&self, line: u32) -> Option<TextRange> {
        let line = usize::try_from(line).ok()?;
        let start = *self.line_starts.get(line)?;
        let end = self
            .line_starts
            .get(line + 1)
            .copied()
            .unwrap_or(self.text_len);
        Some(TextRange::new(start, end))
    }

    #[must_use]
    pub fn line_byte_range(&self, text: &str, line: u32) -> Option<TextRange> {
        let full = self.line_full_byte_range(line)?;
        let start = usize::try_from(full.start().to_u32()).ok()?;
        let mut end = usize::try_from(full.end().to_u32()).ok()?;
        let line_text = text.get(start..end)?;
        if line_text.ends_with("\r\n") {
            end = end.saturating_sub(2);
        } else if line_text.ends_with('\n') {
            end = end.saturating_sub(1);
        }
        Some(TextRange::new(
            full.start(),
            TextSize::new(u32::try_from(end).ok()?),
        ))
    }

    fn line_index_at_offset(&self, offset: TextSize) -> Option<usize> {
        if offset > self.text_len {
            return None;
        }
        Some(match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.checked_sub(1)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceText {
    text: Arc<str>,
    line_map: LineMap,
}

impl SourceText {
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let line_map = LineMap::new(&text);
        Self {
            text: Arc::from(text),
            line_map,
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn line_map(&self) -> &LineMap {
        &self.line_map
    }

    #[must_use]
    pub fn source_hash(&self) -> u64 {
        stable_hash(self.as_str())
    }

    #[must_use]
    pub fn byte_offset(&self, position: &TextPosition) -> Option<TextSize> {
        self.byte_offset_with_encoding(position, PositionEncoding::Utf8)
    }

    #[must_use]
    pub fn byte_offset_with_encoding(
        &self,
        position: &TextPosition,
        encoding: PositionEncoding,
    ) -> Option<TextSize> {
        let line_range = self
            .line_map
            .line_byte_range(self.as_str(), position.line)?;
        let line_start = usize::try_from(line_range.start().to_u32()).ok()?;
        let line_end = usize::try_from(line_range.end().to_u32()).ok()?;
        let line_text = self.as_str().get(line_start..line_end)?;
        let character = usize::try_from(position.character).ok()?;
        let offset =
            match encoding {
                PositionEncoding::Utf8 => {
                    let offset = line_start.checked_add(character)?;
                    (offset <= line_end && self.as_str().is_char_boundary(offset))
                        .then_some(offset)?
                }
                PositionEncoding::Utf16 => line_start.checked_add(
                    encoded_character_byte_offset(line_text, character, EncodedWidth::Utf16)?,
                )?,
                PositionEncoding::Utf32 => line_start.checked_add(
                    encoded_character_byte_offset(line_text, character, EncodedWidth::Utf32)?,
                )?,
            };
        u32::try_from(offset).ok().map(TextSize::new)
    }

    #[must_use]
    pub fn text_position(&self, offset: TextSize) -> Option<TextPosition> {
        self.position_at(offset, PositionEncoding::Utf8)
    }

    #[must_use]
    pub fn position_at(
        &self,
        offset: TextSize,
        encoding: PositionEncoding,
    ) -> Option<TextPosition> {
        let offset_usize = usize::try_from(offset.to_u32()).ok()?;
        if offset_usize > self.as_str().len() || !self.as_str().is_char_boundary(offset_usize) {
            return None;
        }
        let line_index = self.line_map.line_index_at_offset(offset)?;
        let line = u32::try_from(line_index).ok()?;
        let line_range = self.line_map.line_byte_range(self.as_str(), line)?;
        if offset > line_range.end() {
            return None;
        }
        let line_start = usize::try_from(line_range.start().to_u32()).ok()?;
        let prefix = self.as_str().get(line_start..offset_usize)?;
        let character = match encoding {
            PositionEncoding::Utf8 => offset_usize.checked_sub(line_start)?,
            PositionEncoding::Utf16 => prefix.encode_utf16().count(),
            PositionEncoding::Utf32 => prefix.chars().count(),
        };
        Some(TextPosition {
            line,
            character: u32::try_from(character).ok()?,
        })
    }

    #[must_use]
    pub fn text_range(&self, range: TextRange) -> Option<TextRangeUtf> {
        self.range_at(range, PositionEncoding::Utf8)
    }

    #[must_use]
    pub fn range_at(&self, range: TextRange, encoding: PositionEncoding) -> Option<TextRangeUtf> {
        Some(TextRangeUtf {
            start: self.position_at(range.start(), encoding)?,
            end: self.position_at(range.end(), encoding)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFile {
    pub canonical_path: PathBuf,
    pub uri: Option<String>,
    pub source_hash: u64,
    pub document_version: Option<i64>,
    pub source: SourceText,
}

impl SourceFile {
    #[must_use]
    pub fn new(
        canonical_path: impl Into<PathBuf>,
        uri: Option<String>,
        document_version: Option<i64>,
        source: SourceText,
    ) -> Self {
        let source_hash = source.source_hash();
        Self {
            canonical_path: canonical_path.into(),
            uri,
            source_hash,
            document_version,
            source,
        }
    }
}

#[derive(Clone, Copy)]
enum EncodedWidth {
    Utf16,
    Utf32,
}

fn encoded_character_byte_offset(
    text: &str,
    character: usize,
    width: EncodedWidth,
) -> Option<usize> {
    let mut consumed = 0usize;
    for (offset, ch) in text.char_indices() {
        if consumed == character {
            return Some(offset);
        }
        let char_width = match width {
            EncodedWidth::Utf16 => ch.len_utf16(),
            EncodedWidth::Utf32 => 1,
        };
        if consumed.checked_add(char_width)? > character {
            return None;
        }
        consumed = consumed.checked_add(char_width)?;
    }
    (consumed == character).then_some(text.len())
}

fn stable_hash(text: &str) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    text.as_bytes().iter().fold(FNV_OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    })
}

#[cfg(test)]
mod tests {
    use super::{PositionEncoding, SourceText, TextPosition};
    use ruff_text_size::{TextRange, TextSize};

    #[test]
    fn converts_utf8_utf16_and_utf32_positions() {
        let source = SourceText::new("a🦀b\r\nζ\n");
        assert_eq!(
            source.byte_offset_with_encoding(
                &TextPosition {
                    line: 0,
                    character: 5
                },
                PositionEncoding::Utf8,
            ),
            Some(TextSize::new(5))
        );
        assert_eq!(
            source.byte_offset_with_encoding(
                &TextPosition {
                    line: 0,
                    character: 3
                },
                PositionEncoding::Utf16,
            ),
            Some(TextSize::new(5))
        );
        assert_eq!(
            source.byte_offset_with_encoding(
                &TextPosition {
                    line: 0,
                    character: 2
                },
                PositionEncoding::Utf32,
            ),
            Some(TextSize::new(5))
        );
    }

    #[test]
    fn rejects_invalid_boundaries_and_crlf_interior() {
        let source = SourceText::new("a🦀b\r\n");
        assert_eq!(
            source.byte_offset_with_encoding(
                &TextPosition {
                    line: 0,
                    character: 2
                },
                PositionEncoding::Utf8,
            ),
            None
        );
        assert_eq!(
            source.byte_offset_with_encoding(
                &TextPosition {
                    line: 0,
                    character: 2
                },
                PositionEncoding::Utf16,
            ),
            None
        );
        assert_eq!(
            source.position_at(TextSize::new(7), PositionEncoding::Utf8),
            None
        );
    }

    #[test]
    fn supports_eof_and_range_round_trips() {
        let source = SourceText::new("alpha\nβeta\n");
        let eof = source.line_map().eof();
        assert_eq!(
            source.position_at(eof, PositionEncoding::Utf8),
            Some(TextPosition {
                line: 2,
                character: 0
            })
        );
        let range = TextRange::new(TextSize::new(6), TextSize::new(11));
        let utf16 = source.range_at(range, PositionEncoding::Utf16).unwrap();
        assert_eq!(
            source.byte_offset_with_encoding(&utf16.start, PositionEncoding::Utf16),
            Some(range.start())
        );
        assert_eq!(
            source.byte_offset_with_encoding(&utf16.end, PositionEncoding::Utf16),
            Some(range.end())
        );
    }
}
