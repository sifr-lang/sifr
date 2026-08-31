use crate::lifecycle::{error, reject_credentials, reverse_dependencies};
use crate::{SchemaLifecycleError, SchemaLifecycleErrorKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_sql_contract::{
    ObjectId, ProfileAuthority, SchemaDependencyRequest, SchemaSlice, build_profile_authority,
    generate_profile_module, minimum_schema_slice,
};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_ARTIFACT_FORMAT_VERSION: u32 = 1;

pub const SNAPSHOT_PATH: &str = "schema.json";
pub const FINGERPRINT_PATH: &str = "schema.sha256";
pub const RUNTIME_MANIFEST_PATH: &str = "runtime-manifest.json";
pub const GENERATED_MODULE_PATH: &str = "schema.sifr";
pub const GENERATED_METADATA_PATH: &str = "schema-module.json";
pub const DEPENDENCY_INDEX_PATH: &str = "dependency-index.json";
pub const ARTIFACT_MANIFEST_PATH: &str = "artifact-manifest.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaDependencyIndex {
    pub format_version: u32,
    pub schema_fingerprint: String,
    pub dependencies: BTreeMap<ObjectId, BTreeSet<ObjectId>>,
    pub dependents: BTreeMap<ObjectId, BTreeSet<ObjectId>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaArtifactRecord {
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaArtifactManifest {
    pub format_version: u32,
    pub profile_identity: String,
    pub profile_fingerprint: String,
    pub schema_fingerprint: String,
    pub artifacts: BTreeMap<String, SchemaArtifactRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaBuildArtifacts {
    files: BTreeMap<String, Vec<u8>>,
    pub manifest: SchemaArtifactManifest,
}

impl SchemaBuildArtifacts {
    #[must_use]
    pub fn files(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.files
    }
}

pub fn build_schema_artifacts(
    authority: &ProfileAuthority,
) -> Result<SchemaBuildArtifacts, SchemaLifecycleError> {
    let first = build_once(authority)?;
    let second = build_once(authority)?;
    if first != second {
        return Err(error(
            SchemaLifecycleErrorKind::NondeterministicOutput,
            "schema build produced different bytes from identical inputs",
        ));
    }
    Ok(first)
}

fn build_once(authority: &ProfileAuthority) -> Result<SchemaBuildArtifacts, SchemaLifecycleError> {
    reject_credentials(&authority.profile.schema)?;
    let rebuilt = build_profile_authority(authority.profile.clone()).map_err(|failure| {
        error(
            SchemaLifecycleErrorKind::InvalidAuthority,
            format!("schema authority is incomplete or inconsistent: {failure}"),
        )
    })?;
    if rebuilt != *authority {
        return Err(error(
            SchemaLifecycleErrorKind::InvalidAuthority,
            "schema authority fingerprints do not match its canonical profile",
        ));
    }

    let generated =
        generate_profile_module(authority).map_err(|failure| contract_error(&failure))?;
    let dependency_slice = complete_dependency_slice(authority)?;
    let runtime_manifest = authority.runtime_manifest(dependency_slice);
    let dependency_index = dependency_index(authority);

    let mut files = BTreeMap::new();
    files.insert(
        SNAPSHOT_PATH.to_string(),
        canonical_json(&authority.profile.schema)?,
    );
    files.insert(
        FINGERPRINT_PATH.to_string(),
        format!("{}\n", authority.schema_fingerprint.as_str()).into_bytes(),
    );
    files.insert(
        RUNTIME_MANIFEST_PATH.to_string(),
        canonical_json(&runtime_manifest)?,
    );
    files.insert(
        GENERATED_MODULE_PATH.to_string(),
        generated.source.into_bytes(),
    );
    files.insert(
        GENERATED_METADATA_PATH.to_string(),
        canonical_json(&generated.metadata)?,
    );
    files.insert(
        DEPENDENCY_INDEX_PATH.to_string(),
        canonical_json(&dependency_index)?,
    );

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
    let manifest = SchemaArtifactManifest {
        format_version: SCHEMA_ARTIFACT_FORMAT_VERSION,
        profile_identity: authority.nominal_identity.clone(),
        profile_fingerprint: authority.profile_fingerprint.as_str().to_string(),
        schema_fingerprint: authority.schema_fingerprint.as_str().to_string(),
        artifacts,
    };
    files.insert(
        ARTIFACT_MANIFEST_PATH.to_string(),
        canonical_json(&manifest)?,
    );
    Ok(SchemaBuildArtifacts { files, manifest })
}

fn complete_dependency_slice(
    authority: &ProfileAuthority,
) -> Result<SchemaSlice, SchemaLifecycleError> {
    let requests =
        authority
            .profile
            .schema
            .objects
            .values()
            .map(|object| SchemaDependencyRequest {
                identity: object.identity.clone(),
                properties: object.semantic.keys().cloned().collect(),
            });
    minimum_schema_slice(&authority.profile.schema, requests, [])
        .map_err(|failure| contract_error(&failure))
}

fn dependency_index(authority: &ProfileAuthority) -> SchemaDependencyIndex {
    let dependencies = authority
        .profile
        .schema
        .objects
        .iter()
        .map(|(identity, object)| (identity.clone(), object.dependencies.clone()))
        .collect::<BTreeMap<_, _>>();
    let dependents = reverse_dependencies(&authority.profile.schema);
    SchemaDependencyIndex {
        format_version: SCHEMA_ARTIFACT_FORMAT_VERSION,
        schema_fingerprint: authority.schema_fingerprint.as_str().to_string(),
        dependencies,
        dependents,
    }
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, SchemaLifecycleError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|failure| {
        error(
            SchemaLifecycleErrorKind::Serialization,
            format!("cannot serialize schema artifact: {failure}"),
        )
    })?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn contract_error(failure: &sifr_sql_contract::SchemaContractError) -> SchemaLifecycleError {
    error(
        SchemaLifecycleErrorKind::InvalidAuthority,
        format!("cannot build schema artifacts: {failure}"),
    )
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
