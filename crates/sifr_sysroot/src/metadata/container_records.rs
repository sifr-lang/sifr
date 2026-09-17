//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{
    CompilerIntrinsicId, FunctionType, HirClass, HirExpr, HirFunction, PythonInteropDeclaration,
    Ref, RustInteropDeclaration, SemanticExports, Type,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub value: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    pub name: Ref<Text>,
    pub version: Ref<Text>,
    pub source_identity: [u8; 32],
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub package: Ref<Package>,
    pub name: Ref<Text>,
    pub private: bool,
    pub declarations: std::collections::BTreeMap<Ref<Text>, Ref<Declaration>>,
    pub exports: std::collections::BTreeMap<Ref<Text>, Ref<Declaration>>,
    pub hir_inventory: Ref<super::HirModule>,
    pub source: Ref<SourceFile>,
    pub dependencies: std::collections::BTreeSet<Ref<Module>>,
    pub semantic: Ref<SemanticExports>,
    pub rust: Option<Ref<RustPayload>>,
    pub templates: Vec<Ref<TemplatePayload>>,
    pub interop: Ref<InteropSummary>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub module: Ref<Module>,
    pub symbol: Ref<Text>,
    pub kind: Ref<DeclarationKind>,
    pub binder: Option<Ref<Binder>>,
    pub full_view: Option<Ref<NominalView>>,
    pub location: Ref<SourceLocation>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DeclarationKind {
    Function,
    Class,
    Protocol,
    Enum,
    Newtype,
    Alias,
    Constant,
    Metadata,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binder {
    pub declaration: Ref<Declaration>,
    pub nested_path: Vec<u32>,
    pub parameters: Vec<(Ref<Text>, Vec<Ref<Type>>)>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NominalView {
    pub name: Ref<Text>,
    pub identity: Option<Ref<Text>>,
    pub fields: Vec<(Ref<Text>, Ref<Type>)>,
    pub methods: Vec<(Ref<Text>, Ref<FunctionType>)>,
    pub parent_class: Option<Ref<Text>>,
    pub enum_variants: Vec<(Ref<Text>, Option<i64>)>,
    pub newtype_inner: Option<Ref<Type>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub relative_path: String,
    pub content_digest: [u8; 32],
    pub byte_length: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLocation {
    pub source: Ref<SourceFile>,
    pub start: u32,
    pub end: u32,
    pub documentation: Option<Ref<Text>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextRange {
    pub source: Ref<SourceFile>,
    pub start: u32,
    pub end: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticBody {
    pub owner: Ref<Declaration>,
    pub role: Ref<BodyRole>,
    pub expressions: Vec<Ref<HirExpr>>,
    pub functions: Vec<Ref<HirFunction>>,
    pub field_indices: Vec<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum BodyRole {
    Constant,
    ConstFunction,
    ArgumentDefaults,
    FieldDefaults,
    Descriptor,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RustPayload {
    pub module: Ref<Module>,
    pub source: Ref<Text>,
    pub names: std::collections::BTreeMap<Ref<Declaration>, Ref<Text>>,
    pub mappings: Vec<(u32, u32, Ref<SourceLocation>)>,
    pub validation: Ref<FragmentValidation>,
    pub generators: std::collections::BTreeSet<Ref<Text>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FragmentValidation {
    pub bytes_digest: [u8; 32],
    pub compiler_identity: [u8; 32],
    pub target_identity: [u8; 32],
    pub grammar_identity: [u8; 32],
    pub boundary: Ref<FragmentBoundary>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum FragmentBoundary {
    StdlibModule,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplatePayload {
    pub owner: Ref<Declaration>,
    pub role: Ref<TemplateRole>,
    pub function: Option<Ref<HirFunction>>,
    pub class: Option<Ref<HirClass>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TemplateRole {
    GenericFunction,
    GenericClass,
    ProjectPolicyClass,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InteropSummary {
    pub module: Ref<Module>,
    pub semantic_dependencies: std::collections::BTreeSet<Ref<Module>>,
    pub body_dependencies: std::collections::BTreeSet<Ref<Declaration>>,
    pub codegen_dependencies: std::collections::BTreeSet<Ref<Module>>,
    pub intrinsics: std::collections::BTreeSet<Ref<CompilerIntrinsicId>>,
    pub rust_declarations: Vec<Ref<RustInteropDeclaration>>,
    pub python_declarations: Vec<Ref<PythonInteropDeclaration>>,
    pub required_features: std::collections::BTreeSet<Ref<Text>>,
    pub required_support: std::collections::BTreeSet<Ref<Text>>,
}
