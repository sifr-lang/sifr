use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{ComponentIdentity, DiagnosticRegistry};

pub const COMPONENT_PROTOCOL_MAJOR: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub document: String,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TemplatePart {
    Static { text: String, span: SourceSpan },
    Hole { index: u32, span: SourceSpan },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ClosedType {
    Bool,
    Int,
    Float,
    Str,
    Bytes,
    None,
    Optional { item: Box<Self> },
    Tuple { items: Vec<Self> },
    List { item: Box<Self> },
    Record { fields: Vec<RecordField> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordField {
    pub name: String,
    pub ty: ClosedType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoleDescriptor {
    pub index: u32,
    pub ty: ClosedType,
    pub fragment_identity: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisContext {
    pub schema_profile: Option<String>,
    pub schema_fingerprint: Option<String>,
    pub semantic_profile: BTreeMap<String, String>,
    pub imported_signatures: Vec<String>,
    pub artifacts: Vec<ContextArtifact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextArtifact {
    pub kind: String,
    pub identity: String,
    pub format_version: u32,
    pub fingerprint: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanKind {
    Expression,
    Statement,
    Fragment,
    Document,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddedAnalysisRequest {
    pub protocol_major: u16,
    pub component: ComponentIdentity,
    pub provider_diagnostics: DiagnosticRegistry,
    pub compiler_semantic_version: String,
    pub parts: Vec<TemplatePart>,
    pub holes: Vec<HoleDescriptor>,
    pub context: AnalysisContext,
    pub plan_kind: PlanKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyDescriptor {
    pub identity: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SemanticOperation {
    Literal { value: String },
    Hole { index: u32 },
    Sequence { operations: Vec<Self> },
    ProviderNode { tag: String, payload: Vec<u8> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum RuntimeLowering {
    NoRuntime,
    ProviderCall {
        declaration: String,
        payload: Vec<u8>,
        parameter_order: Vec<u32>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticLifecycle {
    Active,
    Deprecated,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddedDiagnostic {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub lifecycle: DiagnosticLifecycle,
    pub message: String,
    pub primary: SourceSpan,
    pub related: Vec<SourceSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceMapEntry {
    pub provider_start: u32,
    pub provider_end: u32,
    pub source: SourceSpan,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddedPlan {
    pub provider_identity: String,
    pub protocol_major: u16,
    pub plan_kind: PlanKind,
    pub schema_identity: Option<String>,
    pub result_type: ClosedType,
    pub operations: Vec<SemanticOperation>,
    pub runtime: RuntimeLowering,
    pub dependencies: Vec<DependencyDescriptor>,
    pub diagnostics: Vec<EmbeddedDiagnostic>,
    pub source_map: Vec<SourceMapEntry>,
    pub stable_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmbeddedAnalysisResponse {
    pub protocol_major: u16,
    pub plan: EmbeddedPlan,
}
