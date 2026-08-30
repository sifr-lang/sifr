use crate::fingerprint::lower_hex;
use crate::{
    SchemaContractError, SchemaContractErrorKind, SchemaFingerprint, SchemaIr, SchemaSlice,
    schema_fingerprint,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_compiler_component::ContextArtifact;
use sifr_structural_identity::profile_identity;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaEvidence {
    Introspection,
    MigrationHead,
    SignedManifest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SchemaStrictness {
    Exact,
    Compatible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PoolingMode {
    Session,
    Transaction,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionContract {
    pub search_path: Vec<String>,
    pub sql_modes: BTreeSet<String>,
    pub collation: Option<String>,
    pub character_set: Option<String>,
    pub time_zone: Option<String>,
    pub role: Option<String>,
    pub isolation: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaProfile {
    pub package_id: String,
    pub name: String,
    pub source_files: BTreeSet<String>,
    pub source_fingerprints: BTreeMap<String, String>,
    pub evidence: SchemaEvidence,
    pub strictness: SchemaStrictness,
    pub pooling: PoolingMode,
    pub session: SessionContract,
    pub accepted_signers: BTreeSet<String>,
    pub schema: SchemaIr,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileFingerprint(String);

impl ProfileFingerprint {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileAuthority {
    pub nominal_identity: String,
    pub profile_fingerprint: ProfileFingerprint,
    pub schema_fingerprint: SchemaFingerprint,
    pub profile: SchemaProfile,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSchemaManifest {
    pub format_version: u32,
    pub profile_identity: String,
    pub profile_fingerprint: ProfileFingerprint,
    pub schema_fingerprint: SchemaFingerprint,
    pub provider_package_id: String,
    pub evidence: SchemaEvidence,
    pub strictness: SchemaStrictness,
    pub session: SessionContract,
    pub dependency_slice: SchemaSlice,
    pub accepted_signers: BTreeSet<String>,
}

impl ProfileAuthority {
    #[must_use]
    pub fn runtime_manifest(&self, dependency_slice: SchemaSlice) -> RuntimeSchemaManifest {
        RuntimeSchemaManifest {
            format_version: 1,
            profile_identity: self.nominal_identity.clone(),
            profile_fingerprint: self.profile_fingerprint.clone(),
            schema_fingerprint: self.schema_fingerprint.clone(),
            provider_package_id: self.profile.schema.provider.package_id.clone(),
            evidence: self.profile.evidence,
            strictness: self.profile.strictness,
            session: self.profile.session.clone(),
            dependency_slice,
            accepted_signers: self.profile.accepted_signers.clone(),
        }
    }
}

pub fn build_profile_authority(
    profile: SchemaProfile,
) -> Result<ProfileAuthority, SchemaContractError> {
    validate_profile(&profile)?;
    crate::normalization::validate_normalized_schema(&profile.schema)?;
    let schema_fingerprint = schema_fingerprint(&profile.schema)?;
    let nominal = profile_identity(&profile.package_id, &profile.name);
    let nominal_identity = lower_hex(nominal.as_bytes());
    let canonical = serde_json::to_vec(&(
        &nominal_identity,
        &schema_fingerprint,
        &profile.source_files,
        &profile.source_fingerprints,
        profile.evidence,
        profile.strictness,
        profile.pooling,
        &profile.session,
        &profile.accepted_signers,
    ))
    .map_err(|error| {
        SchemaContractError::new(
            SchemaContractErrorKind::Serialization,
            format!("cannot serialize canonical schema profile: {error}"),
        )
    })?;
    let profile_fingerprint = ProfileFingerprint(lower_hex(&Sha256::digest(canonical)));
    Ok(ProfileAuthority {
        nominal_identity,
        profile_fingerprint,
        schema_fingerprint,
        profile,
    })
}

pub fn schema_context_artifact(
    authority: &ProfileAuthority,
) -> Result<ContextArtifact, SchemaContractError> {
    let payload = serde_json::to_vec(&authority.profile.schema).map_err(|error| {
        SchemaContractError::new(
            SchemaContractErrorKind::Serialization,
            format!("cannot serialize schema context artifact: {error}"),
        )
    })?;
    Ok(ContextArtifact {
        kind: "sifr.sql.schema-ir".to_string(),
        identity: authority.nominal_identity.clone(),
        format_version: authority.profile.schema.format_version,
        fingerprint: authority.schema_fingerprint.as_str().to_string(),
        payload,
    })
}

fn validate_profile(profile: &SchemaProfile) -> Result<(), SchemaContractError> {
    if profile.package_id.is_empty() || !valid_profile_name(&profile.name) {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "profile requires a package identity and a canonical identifier name",
        ));
    }
    if profile.source_files.is_empty()
        || profile.source_files.iter().any(|path| {
            path.is_empty()
                || path.starts_with('/')
                || path.split('/').any(|part| part == ".." || part.is_empty())
        })
    {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "profile schema sources must be non-empty normalized relative paths",
        ));
    }
    if profile.source_fingerprints.keys().collect::<BTreeSet<_>>()
        != profile.source_files.iter().collect::<BTreeSet<_>>()
        || profile
            .source_fingerprints
            .values()
            .any(|fingerprint| !valid_sha256(fingerprint))
    {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "profile schema source fingerprints must cover every source with exact SHA-256 values",
        ));
    }
    if profile.evidence == SchemaEvidence::SignedManifest && profile.accepted_signers.is_empty() {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "signed-manifest evidence requires at least one accepted signer",
        ));
    }
    if profile.pooling == PoolingMode::Transaction && profile.session.role.is_some() {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "transaction pooling cannot carry a persistent session role",
        ));
    }
    if profile.session.sql_modes.iter().any(|mode| {
        mode.is_empty()
            || mode.len() > 64
            || !mode
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    }) {
        return Err(SchemaContractError::new(
            SchemaContractErrorKind::InvalidProfile,
            "SQL modes must be short identifiers, not arbitrary values",
        ));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_profile_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && characters.all(|character| character == '_' || character.is_alphanumeric())
}
