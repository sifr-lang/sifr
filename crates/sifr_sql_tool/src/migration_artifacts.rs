use crate::lifecycle::error;
use crate::{SchemaArtifactRecord, SchemaLifecycleError, SchemaLifecycleErrorKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{CompiledMigrationGraph, SchemaIr, schema_fingerprint};
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
    let mut files = BTreeMap::from([
        (MIGRATION_GRAPH_PATH.to_string(), canonical_json(graph)?),
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
        format_version: graph.format_version,
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
