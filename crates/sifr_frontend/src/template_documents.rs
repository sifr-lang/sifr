use ruff_text_size::{TextRange, TextSize};
use sifr_lowering::HirTemplateString;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateSourceMapKind {
    Static,
    Interpolation { index: usize },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateSourceMapEntry {
    pub source_range: TextRange,
    pub virtual_range: TextRange,
    pub kind: TemplateSourceMapKind,
}

/// Frontend-owned embedded document for one typed template string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateDocumentView {
    pub source_range: TextRange,
    pub source: String,
    pub mappings: Vec<TemplateSourceMapEntry>,
}

impl TemplateDocumentView {
    #[must_use]
    pub fn from_hir(template: &HirTemplateString) -> Self {
        let mut mappings = Vec::new();
        for segment in &template.segments {
            mappings.extend(
                segment
                    .mappings
                    .iter()
                    .map(|static_mapping| TemplateSourceMapEntry {
                        source_range: static_mapping.source_range,
                        virtual_range: static_mapping.virtual_range,
                        kind: TemplateSourceMapKind::Static,
                    }),
            );
        }
        mappings.extend(template.interpolations.iter().enumerate().map(
            |(index, interpolation)| TemplateSourceMapEntry {
                source_range: interpolation.source_range,
                virtual_range: interpolation.virtual_range,
                kind: TemplateSourceMapKind::Interpolation { index },
            },
        ));
        mappings
            .sort_by_key(|mapping| (mapping.source_range.start(), mapping.virtual_range.start()));
        Self {
            source_range: template.source_range,
            source: template.virtual_source.clone(),
            mappings,
        }
    }

    #[must_use]
    pub fn virtual_range_for_source(&self, offset: TextSize) -> Option<TextRange> {
        self.mappings
            .iter()
            .find(|mapping| contains_offset(mapping.source_range, offset))
            .map(|mapping| mapping.virtual_range)
    }

    #[must_use]
    pub fn source_range_for_virtual(&self, offset: TextSize) -> Option<TextRange> {
        self.mappings
            .iter()
            .find(|mapping| contains_offset(mapping.virtual_range, offset))
            .map(|mapping| mapping.source_range)
    }

    #[must_use]
    pub fn interpolation_at_virtual_offset(&self, offset: TextSize) -> Option<usize> {
        self.mappings.iter().find_map(|mapping| {
            if contains_offset(mapping.virtual_range, offset)
                && let TemplateSourceMapKind::Interpolation { index } = mapping.kind
            {
                return Some(index);
            }
            None
        })
    }
}

fn contains_offset(range: TextRange, offset: TextSize) -> bool {
    if range.is_empty() {
        offset == range.start()
    } else {
        range.start() <= offset && offset < range.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FrontendDiagnosticStyle, FrontendSourceContext, compile_module_hir_with_source};
    use sifr_lowering::{ExternalDefs, HirExpr, HirStmt};

    fn lowered_template(source: &str) -> sifr_lowering::HirTemplateString {
        let parsed = crate::parse_source_module(source, Some("map.sifr")).expect("source parses");
        let lowered = compile_module_hir_with_source(
            "map",
            parsed.suite(),
            &ExternalDefs::default(),
            FrontendDiagnosticStyle::Bare,
            Some(FrontendSourceContext {
                display_path: "map.sifr",
                source,
            }),
        )
        .expect("source lowers");
        let function = &lowered.module.functions[0];
        let HirStmt::Return { value: Some(value) } = &function.body[0] else {
            panic!("expected return");
        };
        let HirExpr::TemplateString(template) = value else {
            panic!("expected template");
        };
        template.clone()
    }

    #[test]
    fn hole_source_and_virtual_ranges_round_trip_for_multiline_unicode() {
        let source =
            "def query(value: str) -> Template:\n    return t\"\"\"α\\nline {value}\nend\"\"\"\n";
        let template = lowered_template(source);
        let document = TemplateDocumentView::from_hir(&template);
        assert_eq!(document.source, "α\nline \u{fffc}\nend");
        let hole = document
            .mappings
            .iter()
            .find(|mapping| matches!(mapping.kind, TemplateSourceMapKind::Interpolation { .. }))
            .expect("hole mapping");
        assert_eq!(
            document.virtual_range_for_source(hole.source_range.start()),
            Some(hole.virtual_range)
        );
        assert_eq!(
            document.source_range_for_virtual(hole.virtual_range.start()),
            Some(hole.source_range)
        );
        assert_eq!(
            document.interpolation_at_virtual_offset(hole.virtual_range.start()),
            Some(0)
        );
    }

    #[test]
    fn every_mapping_preserves_bidirectional_membership() {
        let template = lowered_template(
            "def query(first: int, second: str) -> Template:\n    return t\"a\\tb{first}c{second}d\"\n",
        );
        let document = TemplateDocumentView::from_hir(&template);
        for mapping in &document.mappings {
            assert_eq!(
                document.virtual_range_for_source(mapping.source_range.start()),
                Some(mapping.virtual_range)
            );
            assert_eq!(
                document.source_range_for_virtual(mapping.virtual_range.start()),
                Some(mapping.source_range)
            );
        }
    }

    #[test]
    fn generated_template_documents_preserve_ordered_bidirectional_ranges() {
        let bodies = [
            "t\"plain {first} text {second!r:>8}\"",
            "t\"\"\"α\\nline {first}\\n{second}\nend\"\"\"",
            "tr\"raw\\t{first}{second}\"",
            "t\"left {first}\" t\" right {second}\"",
            "t\"{first}{second}\"",
        ];

        for body in bodies {
            let source =
                format!("def query(first: int, second: str) -> Template:\n    return {body}\n");
            let document = TemplateDocumentView::from_hir(&lowered_template(&source));

            for pair in document.mappings.windows(2) {
                assert!(pair[0].source_range.end() <= pair[1].source_range.start());
                assert!(pair[0].virtual_range.end() <= pair[1].virtual_range.start());
            }

            for mapping in &document.mappings {
                for source_offset in offsets(mapping.source_range) {
                    let virtual_range = document
                        .virtual_range_for_source(source_offset)
                        .expect("every source offset must resolve");
                    assert_eq!(virtual_range, mapping.virtual_range);
                    assert_eq!(
                        document.source_range_for_virtual(virtual_range.start()),
                        Some(mapping.source_range)
                    );
                }
                for virtual_offset in offsets(mapping.virtual_range) {
                    let source_range = document
                        .source_range_for_virtual(virtual_offset)
                        .expect("every virtual offset must resolve");
                    assert_eq!(source_range, mapping.source_range);
                    assert_eq!(
                        document.virtual_range_for_source(source_range.start()),
                        Some(mapping.virtual_range)
                    );
                    if let TemplateSourceMapKind::Interpolation { index } = mapping.kind {
                        assert_eq!(
                            document.interpolation_at_virtual_offset(virtual_offset),
                            Some(index)
                        );
                    }
                }
            }
        }
    }

    fn offsets(range: TextRange) -> impl Iterator<Item = TextSize> {
        (range.start().to_u32()..range.end().to_u32()).map(TextSize::new)
    }
}
