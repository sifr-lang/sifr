use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const MIGRATION_EXECUTION_PLAN_FORMAT_VERSION: u32 = 3;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MigrationId(String);

impl MigrationId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MigrationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MigrationStateId(String);

impl MigrationStateId {
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
#[serde(rename_all = "snake_case")]
pub enum MigrationTransactionBoundary {
    Begin,
    Commit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationTransactionRequirement {
    Required,
    Optional,
    Forbidden,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationRuntimeConstraint {
    pub family: String,
    pub minimum_server_version: Option<String>,
    pub required_capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MigrationReplayPolicy {
    Never,
    Idempotent { progress_keys: Vec<String> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MigrationExecutionStepKind {
    Ddl {
        statement: String,
    },
    SqlData {
        statement: String,
        normalized_statement: String,
    },
    SifrData {
        callback: String,
    },
    Assertion {
        statement: String,
        normalized_statement: String,
    },
    Backfill {
        statement: String,
        normalized_statement: String,
        maximum_batch_rows: u64,
        replay: MigrationReplayPolicy,
    },
    Transaction {
        boundary: MigrationTransactionBoundary,
    },
    RecoveryPoint {
        name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationExecutionStep {
    pub id: MigrationId,
    pub input_state: MigrationStateId,
    pub output_state: MigrationStateId,
    pub input_fingerprint: String,
    pub output_fingerprint: String,
    pub checksum: String,
    pub kind: MigrationExecutionStepKind,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationExecutionPath {
    pub parent: MigrationId,
    pub input_fingerprint: String,
    pub output_fingerprint: String,
    pub steps: Vec<MigrationExecutionStep>,
    pub rollback: Option<Vec<MigrationExecutionStep>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationExecutionNode {
    pub id: MigrationId,
    pub parents: BTreeSet<MigrationId>,
    pub provider: MigrationRuntimeConstraint,
    pub transaction_requirement: MigrationTransactionRequirement,
    pub checksum: String,
    pub paths: BTreeMap<MigrationId, MigrationExecutionPath>,
    pub author: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationExecutionPlan {
    pub format_version: u32,
    pub provider_family: String,
    pub target_fingerprint: String,
    pub head: MigrationId,
    pub topological_order: Vec<MigrationId>,
    pub baseline_fingerprints: BTreeMap<MigrationId, String>,
    pub migrations: BTreeMap<MigrationId, MigrationExecutionNode>,
}
