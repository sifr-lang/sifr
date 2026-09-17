//! Explicit DX.5 wire records. IDs replace live compiler ownership and recursion.
use super::{HirExpr, Ref, Text, Type};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirSqlQueryAdapter {
    ExpectAtMostOne,
    First,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirSqlEffectKind {
    Read,
    Write,
    ReadWrite,
    SchemaChange,
    SessionChange,
    TransactionControl,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlCardinality {
    pub empty: bool,
    pub minimum: u64,
    pub maximum: Option<u64>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlEffectContract {
    pub effect: Ref<HirSqlEffectKind>,
    pub referenced_objects: Vec<Ref<Text>>,
    pub affected_objects: Vec<Ref<Text>>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlParameterSlot {
    pub slot: u32,
    pub ty: Ref<Type>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlQueryTemplate {
    pub identity: Ref<Text>,
    pub module: Ref<Text>,
    pub symbol: Ref<Text>,
    pub profile_identity: Ref<Text>,
    pub profile_fingerprint: Ref<Text>,
    pub schema_fingerprint: Ref<Text>,
    pub normalized_statement: Ref<Text>,
    pub parameters: Vec<Ref<HirSqlParameterSlot>>,
    pub row_type: Ref<Type>,
    pub cardinality: Ref<HirSqlCardinality>,
    pub effects: Ref<HirSqlEffectContract>,
    pub deterministic_order: bool,
    pub fragment_identities: Vec<Ref<Text>>,
    pub adapters: Vec<Ref<HirSqlQueryAdapter>>,
    pub ty: Ref<Type>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlBoundQuery {
    pub template_identity: Ref<Text>,
    pub profile_identity: Ref<Text>,
    pub profile_fingerprint: Ref<Text>,
    pub schema_fingerprint: Ref<Text>,
    pub captures: Vec<Ref<HirExpr>>,
    pub cardinality: Ref<HirSqlCardinality>,
    pub effects: Ref<HirSqlEffectContract>,
    pub ty: Ref<Type>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum HirSqlExecutionMethod {
    Execute,
    FetchOne,
    FetchOptional,
    FetchAll { maximum_rows: u64 },
    Stream,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HirSqlExecution {
    pub query: Ref<HirSqlBoundQuery>,
    pub method: Ref<HirSqlExecutionMethod>,
    pub runtime_cardinality: Ref<HirSqlCardinality>,
    pub runtime_effects: Ref<HirSqlEffectContract>,
}
