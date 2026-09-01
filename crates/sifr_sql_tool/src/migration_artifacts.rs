use crate::lifecycle::error;
use crate::{SchemaArtifactRecord, SchemaLifecycleError, SchemaLifecycleErrorKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    CompiledMigrationGraph, CompiledMigrationStep, CompiledStepKind,
    MIGRATION_GRAPH_FORMAT_VERSION, ReplayPolicy, SchemaIr, TransactionBoundary,
    TransactionRequirement, schema_fingerprint,
};
use sifr_sql_runtime::{
    MIGRATION_EXECUTION_PLAN_FORMAT_VERSION, MigrationExecutionNode, MigrationExecutionPath,
    MigrationExecutionPlan, MigrationExecutionStep, MigrationExecutionStepKind, MigrationId,
    MigrationReplayPolicy, MigrationRuntimeConstraint, MigrationStateId,
    MigrationTransactionBoundary, MigrationTransactionRequirement,
};
use std::collections::BTreeMap;

pub const MIGRATION_GRAPH_PATH: &str = "graph.json";
pub const MIGRATION_SCHEMA_PATH: &str = "schema.json";
pub const MIGRATION_IMPACT_PATH: &str = "impact.json";
pub const MIGRATION_ARTIFACT_MANIFEST_PATH: &str = "artifact-manifest.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationArtifactManifest {
    pub format_version: u32,
    pub provider_family: String,
    pub head: String,
    pub target_fingerprint: String,
    pub artifacts: BTreeMap<String, SchemaArtifactRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationBuildArtifacts {
    files: BTreeMap<String, Vec<u8>>,
    pub manifest: MigrationArtifactManifest,
}

impl MigrationBuildArtifacts {
    #[must_use]
    pub fn files(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.files
    }
}

pub fn build_migration_artifacts(
    graph: &CompiledMigrationGraph,
    target_schema: &SchemaIr,
) -> Result<MigrationBuildArtifacts, SchemaLifecycleError> {
    let first = build_once(graph, target_schema)?;
    let second = build_once(graph, target_schema)?;
    if first != second {
        return Err(error(
            SchemaLifecycleErrorKind::NondeterministicOutput,
            "migration build produced different bytes from identical inputs",
        ));
    }
    Ok(first)
}

pub fn lower_migration_execution_plan(
    graph: &CompiledMigrationGraph,
) -> Result<MigrationExecutionPlan, SchemaLifecycleError> {
    if graph.format_version != MIGRATION_GRAPH_FORMAT_VERSION {
        return Err(error(
            SchemaLifecycleErrorKind::InvalidAuthority,
            "compiled migration graph format is not supported",
        ));
    }
    Ok(MigrationExecutionPlan {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: graph.provider_family.clone(),
        target_fingerprint: graph.target_fingerprint.clone(),
        head: runtime_id(&graph.head),
        topological_order: graph.topological_order.iter().map(runtime_id).collect(),
        baseline_fingerprints: graph
            .baseline_fingerprints
            .iter()
            .map(|(id, fingerprint)| (runtime_id(id), fingerprint.clone()))
            .collect(),
        migrations: graph
            .migrations
            .iter()
            .map(|(id, migration)| {
                (
                    runtime_id(id),
                    MigrationExecutionNode {
                        id: runtime_id(&migration.id),
                        parents: migration.parents.iter().map(runtime_id).collect(),
                        provider: MigrationRuntimeConstraint {
                            family: migration.provider.family.clone(),
                            minimum_server_version: migration
                                .provider
                                .minimum_server_version
                                .clone(),
                            required_capabilities: migration.provider.required_capabilities.clone(),
                        },
                        transaction_requirement: match migration.transaction_requirement {
                            TransactionRequirement::Required => {
                                MigrationTransactionRequirement::Required
                            }
                            TransactionRequirement::Optional => {
                                MigrationTransactionRequirement::Optional
                            }
                            TransactionRequirement::Forbidden => {
                                MigrationTransactionRequirement::Forbidden
                            }
                        },
                        checksum: migration.checksum.clone(),
                        paths: migration
                            .paths
                            .iter()
                            .map(|(parent, path)| {
                                (
                                    runtime_id(parent),
                                    MigrationExecutionPath {
                                        parent: runtime_id(&path.parent),
                                        input_fingerprint: path.input_fingerprint.clone(),
                                        output_fingerprint: path.output_fingerprint.clone(),
                                        steps: lower_steps(&path.steps),
                                        rollback: path.rollback.as_deref().map(lower_steps),
                                    },
                                )
                            })
                            .collect(),
                        author: migration.author.clone(),
                        created_at: migration.created_at.clone(),
                    },
                )
            })
            .collect(),
    })
}

fn build_once(
    graph: &CompiledMigrationGraph,
    target_schema: &SchemaIr,
) -> Result<MigrationBuildArtifacts, SchemaLifecycleError> {
    let target = schema_fingerprint(target_schema).map_err(|failure| {
        error(
            SchemaLifecycleErrorKind::Serialization,
            format!("cannot fingerprint the migration target schema: {failure}"),
        )
    })?;
    if target.as_str() != graph.target_fingerprint {
        return Err(error(
            SchemaLifecycleErrorKind::InvalidAuthority,
            "compiled migration head does not match the artifact target schema",
        ));
    }
    let execution_plan = lower_migration_execution_plan(graph)?;
    let mut files = BTreeMap::from([
        (
            MIGRATION_GRAPH_PATH.to_string(),
            canonical_json(&execution_plan)?,
        ),
        (
            MIGRATION_SCHEMA_PATH.to_string(),
            canonical_json(target_schema)?,
        ),
        (
            MIGRATION_IMPACT_PATH.to_string(),
            canonical_json(&graph.impacts)?,
        ),
    ]);
    let artifacts = files
        .iter()
        .map(|(path, bytes)| {
            (
                path.clone(),
                SchemaArtifactRecord {
                    sha256: lower_hex(&Sha256::digest(bytes)),
                    size: bytes.len() as u64,
                },
            )
        })
        .collect();
    let manifest = MigrationArtifactManifest {
        format_version: MIGRATION_EXECUTION_PLAN_FORMAT_VERSION,
        provider_family: graph.provider_family.clone(),
        head: graph.head.to_string(),
        target_fingerprint: graph.target_fingerprint.clone(),
        artifacts,
    };
    files.insert(
        MIGRATION_ARTIFACT_MANIFEST_PATH.to_string(),
        canonical_json(&manifest)?,
    );
    Ok(MigrationBuildArtifacts { files, manifest })
}

fn lower_steps(steps: &[CompiledMigrationStep]) -> Vec<MigrationExecutionStep> {
    steps
        .iter()
        .map(|step| MigrationExecutionStep {
            id: runtime_id(&step.id),
            input_state: MigrationStateId::new(step.input_state.as_str()),
            output_state: MigrationStateId::new(step.output_state.as_str()),
            input_fingerprint: step.input_fingerprint.clone(),
            output_fingerprint: step.output_fingerprint.clone(),
            checksum: step.checksum.clone(),
            kind: lower_step_kind(&step.kind),
        })
        .collect()
}

fn lower_step_kind(kind: &CompiledStepKind) -> MigrationExecutionStepKind {
    match kind {
        CompiledStepKind::ReflectedDdl { statement }
        | CompiledStepKind::DeclaredDdl { statement } => MigrationExecutionStepKind::Ddl {
            statement: statement.clone(),
        },
        CompiledStepKind::SqlData {
            statement,
            normalized_statement,
        } => MigrationExecutionStepKind::SqlData {
            statement: statement.clone(),
            normalized_statement: normalized_statement.clone(),
        },
        CompiledStepKind::SifrData { callback } => MigrationExecutionStepKind::SifrData {
            callback: callback.clone(),
        },
        CompiledStepKind::Assertion {
            statement,
            normalized_statement,
        } => MigrationExecutionStepKind::Assertion {
            statement: statement.clone(),
            normalized_statement: normalized_statement.clone(),
        },
        CompiledStepKind::Backfill {
            statement,
            normalized_statement,
            maximum_batch_rows,
            replay,
        } => MigrationExecutionStepKind::Backfill {
            statement: statement.clone(),
            normalized_statement: normalized_statement.clone(),
            maximum_batch_rows: *maximum_batch_rows,
            replay: match replay {
                ReplayPolicy::Never => MigrationReplayPolicy::Never,
                ReplayPolicy::Idempotent { progress_key } => MigrationReplayPolicy::Idempotent {
                    progress_keys: progress_key
                        .iter()
                        .map(|object| object.as_str().to_string())
                        .collect(),
                },
            },
        },
        CompiledStepKind::Transaction { boundary } => MigrationExecutionStepKind::Transaction {
            boundary: match boundary {
                TransactionBoundary::Begin => MigrationTransactionBoundary::Begin,
                TransactionBoundary::Commit => MigrationTransactionBoundary::Commit,
            },
        },
        CompiledStepKind::RecoveryPoint { name } => {
            MigrationExecutionStepKind::RecoveryPoint { name: name.clone() }
        }
    }
}

fn runtime_id(id: &sifr_sql_contract::MigrationNodeId) -> MigrationId {
    MigrationId::new(id.as_str())
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, SchemaLifecycleError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|failure| {
        error(
            SchemaLifecycleErrorKind::Serialization,
            format!("cannot serialize migration artifact: {failure}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}
