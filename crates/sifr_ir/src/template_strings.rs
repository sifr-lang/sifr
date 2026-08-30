use crate::HirExpr;
use ruff_text_size::TextRange;
use sifr_type_system::Type;

/// One decoded static string in a PEP 750 template.
#[derive(Debug, Clone)]
pub struct HirTemplateSegment {
    pub value: String,
    /// A segment can span more than one source token after implicit literal
    /// concatenation. Empty synthetic boundary strings have no mapping.
    pub mappings: Vec<HirTemplateStaticMapping>,
    pub virtual_range: TextRange,
}

#[derive(Debug, Clone)]
pub struct HirTemplateStaticMapping {
    pub source_range: TextRange,
    pub virtual_range: TextRange,
    /// Exact decoded-character mappings. Escapes can consume several source
    /// bytes and produce one virtual character, so range endpoints alone are
    /// not sufficient for safe editor edits.
    pub offsets: Vec<HirTemplateOffsetMapping>,
}

#[derive(Debug, Clone)]
pub struct HirTemplateOffsetMapping {
    pub source_range: TextRange,
    pub virtual_range: TextRange,
}

/// A format specification is evaluated eagerly after its owning value.
#[derive(Debug, Clone)]
pub struct HirTemplateFormatSpec {
    pub range: TextRange,
    pub parts: Vec<HirTemplateFormatSpecPart>,
}

#[derive(Debug, Clone)]
pub enum HirTemplateFormatSpecPart {
    Literal(String),
    Interpolation {
        value: Box<HirExpr>,
        clone_from_borrow: bool,
        source_range: TextRange,
        conversion: Option<char>,
        format_spec: Option<Box<HirTemplateFormatSpec>>,
    },
}

/// One eagerly evaluated interpolation retained by a template value.
#[derive(Debug, Clone)]
pub struct HirTemplateInterpolation {
    pub value: Box<HirExpr>,
    pub clone_from_borrow: bool,
    pub value_type: Type,
    pub source_range: TextRange,
    pub expression_range: TextRange,
    pub expression_source: String,
    pub virtual_range: TextRange,
    pub conversion: Option<char>,
    pub format_spec: Option<HirTemplateFormatSpec>,
}

/// A typed PEP 750 template. `segments.len() == interpolations.len() + 1`.
#[derive(Debug, Clone)]
pub struct HirTemplateString {
    pub source_range: TextRange,
    pub virtual_source: String,
    pub segments: Vec<HirTemplateSegment>,
    pub interpolations: Vec<HirTemplateInterpolation>,
    pub ty: Type,
}

impl HirTemplateString {
    pub fn for_each_value(&self, visit: &mut impl FnMut(&HirExpr)) {
        for interpolation in &self.interpolations {
            visit(&interpolation.value);
            if let Some(spec) = &interpolation.format_spec {
                spec.for_each_value(visit);
            }
        }
    }

    pub fn any_value(&self, mut predicate: impl FnMut(&HirExpr) -> bool) -> bool {
        let mut found = false;
        self.for_each_value(&mut |value| found |= predicate(value));
        found
    }

    pub fn for_each_value_mut(&mut self, visit: &mut impl FnMut(&mut HirExpr)) {
        for interpolation in &mut self.interpolations {
            visit(&mut interpolation.value);
            if let Some(spec) = &mut interpolation.format_spec {
                spec.for_each_value_mut(visit);
            }
        }
    }
}

impl HirTemplateFormatSpec {
    fn for_each_value(&self, visit: &mut impl FnMut(&HirExpr)) {
        for part in &self.parts {
            if let HirTemplateFormatSpecPart::Interpolation {
                value, format_spec, ..
            } = part
            {
                visit(value);
                if let Some(spec) = format_spec {
                    spec.for_each_value(visit);
                }
            }
        }
    }

    fn for_each_value_mut(&mut self, visit: &mut impl FnMut(&mut HirExpr)) {
        for part in &mut self.parts {
            if let HirTemplateFormatSpecPart::Interpolation {
                value, format_spec, ..
            } = part
            {
                visit(value);
                if let Some(spec) = format_spec {
                    spec.for_each_value_mut(visit);
                }
            }
        }
    }
}
