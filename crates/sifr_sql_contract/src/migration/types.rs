use crate::{ObjectId, ProviderAnalysis, SchemaIr};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MigrationNodeId(String);

impl MigrationNodeId {
    pub fn new(value: impl Into<String>) -> Result<Self, MigrationCompileError> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            return Err(MigrationCompileError::new(
                MigrationCompileErrorKind::InvalidIdentity,
                "migration identities must use 1 to 128 ASCII letters, digits, dots, dashes, or underscores",
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MigrationNodeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationProviderConstraint {
    pub family: String,
    pub minimum_server_version: Option<String>,
    pub required_capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationBaseline {
    pub id: MigrationNodeId,
    pub schema: SchemaIr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionRequirement {
    Required,
    Optional,
    Forbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionBoundary {
    Begin,
    Commit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReplayPolicy {
    Never,
    Idempotent { progress_key: Vec<ObjectId> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataCallbackContract {
    pub symbol: String,
    pub referenced_objects: BTreeSet<ObjectId>,
    pub affected_objects: BTreeSet<ObjectId>,
    pub is_async: bool,
    pub returns_result: bool,
    pub nonescaping: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackfillContract {
    pub analysis: ProviderAnalysis,
    pub maximum_batch_rows: u64,
    pub replay: ReplayPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MigrationStepKind {
    Ddl {
        statement: String,
        declared_effect: Option<Box<SchemaIr>>,
    },
    SqlData {
        analysis: ProviderAnalysis,
    },
    SifrData {
        callback: DataCallbackContract,
    },
    Assertion {
        analysis: ProviderAnalysis,
    },
    Backfill {
        contract: BackfillContract,
    },
    Transaction {
        boundary: TransactionBoundary,
    },
    RecoveryPoint {
        name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationStepDefinition {
    pub id: MigrationNodeId,
    pub kind: MigrationStepKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationDefinition {
    pub id: MigrationNodeId,
    pub parents: BTreeSet<MigrationNodeId>,
    pub input_fingerprints: BTreeMap<MigrationNodeId, String>,
    pub output_fingerprint: String,
    pub provider: MigrationProviderConstraint,
    pub transaction_requirement: TransactionRequirement,
    pub steps: Vec<MigrationStepDefinition>,
    pub rollback: Option<Vec<MigrationStepDefinition>>,
    pub author: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationGraphDefinition {
    pub format_version: u32,
    pub baselines: BTreeMap<MigrationNodeId, MigrationBaseline>,
    pub migrations: BTreeMap<MigrationNodeId, MigrationDefinition>,
    pub target_schema: SchemaIr,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DdlRisk {
    pub lock_risks: BTreeSet<String>,
    pub data_rewrites: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DdlReflection {
    Reflected { schema: SchemaIr, risk: DdlRisk },
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MigrationStateIdentity(String);

impl MigrationStateIdentity {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CompiledStepKind {
    ReflectedDdl {
        statement: String,
    },
    DeclaredDdl {
        statement: String,
    },
    SqlData {
        normalized_statement: String,
    },
    SifrData {
        callback: String,
    },
    Assertion {
        normalized_statement: String,
    },
    Backfill {
        normalized_statement: String,
        maximum_batch_rows: u64,
        replay: ReplayPolicy,
    },
    Transaction {
        boundary: TransactionBoundary,
    },
    RecoveryPoint {
        name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledMigrationStep {
    pub id: MigrationNodeId,
    pub input_state: MigrationStateIdentity,
    pub output_state: MigrationStateIdentity,
    pub input_fingerprint: String,
    pub output_fingerprint: String,
    pub checksum: String,
    pub referenced_objects: BTreeSet<ObjectId>,
    pub affected_objects: BTreeSet<ObjectId>,
    pub kind: CompiledStepKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledMigrationPath {
    pub parent: MigrationNodeId,
    pub input_fingerprint: String,
    pub output_fingerprint: String,
    pub steps: Vec<CompiledMigrationStep>,
    pub rollback: Option<Vec<CompiledMigrationStep>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledMigration {
    pub id: MigrationNodeId,
    pub parents: BTreeSet<MigrationNodeId>,
    pub provider: MigrationProviderConstraint,
    pub transaction_requirement: TransactionRequirement,
    pub checksum: String,
    pub paths: BTreeMap<MigrationNodeId, CompiledMigrationPath>,
    pub author: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationImpact {
    pub migration: MigrationNodeId,
    pub step: MigrationNodeId,
    pub destructive_objects: BTreeSet<ObjectId>,
    pub lock_risks: BTreeSet<String>,
    pub data_rewrites: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompiledMigrationGraph {
    pub format_version: u32,
    pub provider_family: String,
    pub target_fingerprint: String,
    pub head: MigrationNodeId,
    pub topological_order: Vec<MigrationNodeId>,
    pub baseline_fingerprints: BTreeMap<MigrationNodeId, String>,
    pub migrations: BTreeMap<MigrationNodeId, CompiledMigration>,
    pub impacts: Vec<MigrationImpact>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MigrationCompileErrorKind {
    FormatVersion,
    InvalidIdentity,
    InvalidGraph,
    ProviderMismatch,
    CapabilityMismatch,
    FingerprintMismatch,
    InvalidStep,
    InvalidDataCallback,
    InvalidAssertion,
    InvalidBackfill,
    InvalidTransaction,
    InvalidRollback,
    UnknownSchemaObject,
    DdlReflection,
    Serialization,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationCompileError {
    pub kind: MigrationCompileErrorKind,
    pub message: String,
}

impl MigrationCompileError {
    pub fn new(kind: MigrationCompileErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for MigrationCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MigrationCompileError {}
