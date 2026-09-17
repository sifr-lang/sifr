//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{HirExpr, Ref, Text, TextRange, Type};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirTemplateSegment {
    pub value: Ref<Text>,
    pub mappings: Vec<Ref<HirTemplateStaticMapping>>,
    pub virtual_range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirTemplateStaticMapping {
    pub source_range: Ref<TextRange>,
    pub virtual_range: Ref<TextRange>,
    pub offsets: Vec<Ref<HirTemplateOffsetMapping>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirTemplateOffsetMapping {
    pub source_range: Ref<TextRange>,
    pub virtual_range: Ref<TextRange>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirTemplateFormatSpec {
    pub range: Ref<TextRange>,
    pub parts: Vec<Ref<HirTemplateFormatSpecPart>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirTemplateFormatSpecPart {
    Literal(Ref<Text>),
    Interpolation {
        value: Ref<HirExpr>,
        clone_from_borrow: bool,
        source_range: Ref<TextRange>,
        conversion: Option<char>,
        format_spec: Option<Ref<HirTemplateFormatSpec>>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirTemplateInterpolation {
    pub value: Ref<HirExpr>,
    pub clone_from_borrow: bool,
    pub value_type: Ref<Type>,
    pub source_range: Ref<TextRange>,
    pub expression_range: Ref<TextRange>,
    pub expression_source: Ref<Text>,
    pub virtual_range: Ref<TextRange>,
    pub conversion: Option<char>,
    pub format_spec: Option<Ref<HirTemplateFormatSpec>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirTemplateString {
    pub source_range: Ref<TextRange>,
    pub virtual_source: Ref<Text>,
    pub segments: Vec<Ref<HirTemplateSegment>>,
    pub interpolations: Vec<Ref<HirTemplateInterpolation>>,
    pub ty: Ref<Type>,
}
