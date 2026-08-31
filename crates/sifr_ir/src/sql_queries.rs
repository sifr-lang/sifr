use crate::HirExpr;
use sifr_type_system::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirSqlQueryAdapter {
    ExpectAtMostOne,
    First,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirSqlEffectKind {
    Read,
    Write,
    ReadWrite,
    SchemaChange,
    SessionChange,
    TransactionControl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HirSqlCardinality {
    pub empty: bool,
    pub minimum: u64,
    pub maximum: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HirSqlEffectContract {
    pub effect: HirSqlEffectKind,
    pub referenced_objects: Vec<String>,
    pub affected_objects: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HirSqlParameterSlot {
    pub slot: u32,
    pub ty: Type,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HirSqlQueryTemplate {
    pub identity: String,
    pub module: String,
    pub symbol: String,
    pub profile_identity: String,
    pub profile_fingerprint: String,
    pub schema_fingerprint: String,
    pub normalized_statement: String,
    pub parameters: Vec<HirSqlParameterSlot>,
    pub row_type: Type,
    pub cardinality: HirSqlCardinality,
    pub effects: HirSqlEffectContract,
    pub deterministic_order: bool,
    pub fragment_identities: Vec<String>,
    pub adapters: Vec<HirSqlQueryAdapter>,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct HirSqlBoundQuery {
    pub template_identity: String,
    pub profile_identity: String,
    pub profile_fingerprint: String,
    pub schema_fingerprint: String,
    /// Captures remain normal typed HIR expressions in source evaluation order.
    pub captures: Vec<HirExpr>,
    pub cardinality: HirSqlCardinality,
    pub effects: HirSqlEffectContract,
    pub ty: Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirSqlExecutionMethod {
    Execute,
    FetchOne,
    FetchOptional,
    FetchAll { maximum_rows: u64 },
    Stream,
}

#[derive(Clone, Debug)]
pub struct HirSqlExecution {
    pub query: HirSqlBoundQuery,
    pub method: HirSqlExecutionMethod,
    /// These exact records are lowered to the runtime request. They are kept
    /// separately from result-container selection.
    pub runtime_cardinality: HirSqlCardinality,
    pub runtime_effects: HirSqlEffectContract,
}
