use crate::{
    ObjectId, ProfileAuthority, ProviderIdentity, SchemaDependencyRequest, SchemaIr, SchemaSlice,
    minimum_schema_slice, schema_fingerprint, verify_compatible_slice,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const SCHEMA_REQUIREMENT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaRequirementIdentity {
    pub package_id: String,
    pub name: String,
}

impl SchemaRequirementIdentity {
    pub fn new(
        package_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<Self, SchemaRequirementError> {
        let identity = Self {
            package_id: package_id.into(),
            name: name.into(),
        };
        identity.validate()?;
        Ok(identity)
    }

    pub fn validate(&self) -> Result<(), SchemaRequirementError> {
        if self.package_id.trim().is_empty() || !valid_identifier(&self.name) {
            return Err(invalid(
                "a schema requirement needs one package identity and identifier name",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn canonical_name(&self) -> String {
        format!("{}::{}", self.package_id, self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderSchemaRequirement {
    pub format_version: u32,
    pub identity: SchemaRequirementIdentity,
    pub provider: ProviderIdentity,
    pub provider_family: String,
    pub minimum_server_version: String,
    pub source_document: String,
    pub source_fingerprint: String,
    pub required_capabilities: BTreeSet<String>,
    pub schema: SchemaSlice,
    pub normalized_schema_fingerprint: String,
    pub artifact_fingerprint: String,
}

impl ProviderSchemaRequirement {
    pub fn validate(&self) -> Result<(), SchemaRequirementError> {
        self.identity.validate()?;
        if self.format_version != SCHEMA_REQUIREMENT_FORMAT_VERSION
            || !valid_family(&self.provider_family)
            || !valid_server_version(&self.minimum_server_version)
            || !valid_relative_document(&self.source_document)
            || !valid_sha256(&self.source_fingerprint)
            || !valid_sha256(&self.normalized_schema_fingerprint)
            || !valid_sha256(&self.artifact_fingerprint)
            || self.required_capabilities.is_empty()
            || self
                .required_capabilities
                .iter()
                .any(|capability| !valid_capability(capability))
            || self.schema.objects.is_empty()
            || !self.schema.absence_facts.is_empty()
        {
            return Err(invalid("schema requirement artifact is not canonical"));
        }
        let expected = artifact_fingerprint(
            &self.identity,
            &self.provider,
            &self.provider_family,
            &self.minimum_server_version,
            &self.source_document,
            &self.source_fingerprint,
            &self.required_capabilities,
            &self.schema,
            &self.normalized_schema_fingerprint,
        )?;
        if expected != self.artifact_fingerprint {
            return Err(invalid("schema requirement artifact fingerprint changed"));
        }
        Ok(())
    }

    #[must_use]
    pub fn declared_objects(&self) -> BTreeSet<ObjectId> {
        self.schema.objects.keys().cloned().collect()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn build_provider_schema_requirement(
    identity: SchemaRequirementIdentity,
    source_document: impl Into<String>,
    source_fingerprint: impl Into<String>,
    normalized: &SchemaIr,
    required_capabilities: BTreeSet<String>,
    provider_capabilities: &BTreeSet<String>,
) -> Result<ProviderSchemaRequirement, SchemaRequirementError> {
    identity.validate()?;
    let source_document = source_document.into();
    let source_fingerprint = source_fingerprint.into();
    if !valid_relative_document(&source_document) || !valid_sha256(&source_fingerprint) {
        return Err(invalid(
            "schema requirement source needs a normalized relative path and SHA-256 fingerprint",
        ));
    }
    if required_capabilities.is_empty()
        || required_capabilities
            .iter()
            .any(|capability| !valid_capability(capability))
    {
        return Err(invalid(
            "schema requirement must declare canonical provider capabilities",
        ));
    }
    let missing = required_capabilities
        .difference(provider_capabilities)
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(SchemaRequirementError::new(
            SchemaRequirementErrorKind::MissingCapability,
            format!(
                "provider '{}' does not support required SQL capabilities: {}",
                normalized.dialect.family,
                missing.join(", ")
            ),
        ));
    }
    if normalized.objects.is_empty()
        || normalized.objects.values().any(|object| {
            object
                .source
                .as_ref()
                .is_none_or(|source| source.document != source_document)
        })
    {
        return Err(invalid(
            "normalized requirement objects must come from the declared DDL artifact",
        ));
    }
    let schema = minimum_schema_slice(
        normalized,
        normalized
            .objects
            .iter()
            .map(|(identity, object)| SchemaDependencyRequest {
                identity: identity.clone(),
                properties: object.semantic.keys().cloned().collect(),
            }),
        [],
    )
    .map_err(|error| invalid(error.message))?;
    let normalized_schema_fingerprint = schema_fingerprint(normalized)
        .map_err(|error| invalid(error.message))?
        .as_str()
        .to_string();
    let provider_family = normalized.dialect.family.clone();
    let minimum_server_version = normalized.dialect.server_version.clone();
    let artifact_fingerprint = artifact_fingerprint(
        &identity,
        &normalized.provider,
        &provider_family,
        &minimum_server_version,
        &source_document,
        &source_fingerprint,
        &required_capabilities,
        &schema,
        &normalized_schema_fingerprint,
    )?;
    let artifact = ProviderSchemaRequirement {
        format_version: SCHEMA_REQUIREMENT_FORMAT_VERSION,
        identity,
        provider: normalized.provider.clone(),
        provider_family,
        minimum_server_version,
        source_document,
        source_fingerprint,
        required_capabilities,
        schema,
        normalized_schema_fingerprint,
        artifact_fingerprint,
    };
    artifact.validate()?;
    Ok(artifact)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaRequirement {
    pub identity: SchemaRequirementIdentity,
    pub providers: BTreeMap<String, ProviderSchemaRequirement>,
}

impl SchemaRequirement {
    pub fn new(
        identity: SchemaRequirementIdentity,
        artifacts: impl IntoIterator<Item = ProviderSchemaRequirement>,
    ) -> Result<Self, SchemaRequirementError> {
        identity.validate()?;
        let mut providers = BTreeMap::new();
        for artifact in artifacts {
            artifact.validate()?;
            if artifact.identity != identity
                || providers
                    .insert(artifact.provider_family.clone(), artifact)
                    .is_some()
            {
                return Err(invalid(
                    "portable requirement provider artifacts need one identity and unique families",
                ));
            }
        }
        if providers.is_empty() {
            return Err(invalid(
                "schema requirement needs at least one provider artifact",
            ));
        }
        Ok(Self {
            identity,
            providers,
        })
    }

    pub fn prove(
        &self,
        profile: &ProfileAuthority,
    ) -> Result<SchemaRequirementProof, SchemaRequirementError> {
        let family = &profile.profile.schema.dialect.family;
        let artifact = self.providers.get(family).ok_or_else(|| {
            SchemaRequirementError::new(
                SchemaRequirementErrorKind::MissingProvider,
                format!(
                    "requirement '{}' has no '{family}' provider artifact",
                    self.identity.canonical_name()
                ),
            )
        })?;
        if artifact.provider != profile.profile.schema.provider {
            return Err(SchemaRequirementError::new(
                SchemaRequirementErrorKind::ProviderMismatch,
                "requirement and concrete profile use different provider package identities",
            ));
        }
        if compare_versions(
            &profile.profile.schema.dialect.server_version,
            &artifact.minimum_server_version,
        )? == std::cmp::Ordering::Less
        {
            return Err(SchemaRequirementError::new(
                SchemaRequirementErrorKind::ProviderMismatch,
                format!(
                    "profile server version {} is older than requirement minimum {}",
                    profile.profile.schema.dialect.server_version, artifact.minimum_server_version
                ),
            ));
        }
        let missing = artifact
            .required_capabilities
            .difference(&profile.profile.capabilities)
            .cloned()
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(SchemaRequirementError::new(
                SchemaRequirementErrorKind::MissingCapability,
                format!(
                    "profile is missing SQL capabilities: {}",
                    missing.join(", ")
                ),
            ));
        }
        verify_compatible_slice(&profile.profile.schema, &artifact.schema).map_err(|error| {
            SchemaRequirementError::new(
                SchemaRequirementErrorKind::IncompatibleSchema,
                error.message,
            )
        })?;
        Ok(SchemaRequirementProof {
            requirement: self.identity.clone(),
            profile_identity: profile.nominal_identity.clone(),
            profile_fingerprint: profile.profile_fingerprint.as_str().to_string(),
            schema_fingerprint: profile.schema_fingerprint.as_str().to_string(),
            provider_family: family.clone(),
            declared_objects: artifact.declared_objects(),
            required_capabilities: artifact.required_capabilities.clone(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaRequirementProof {
    pub requirement: SchemaRequirementIdentity,
    pub profile_identity: String,
    pub profile_fingerprint: String,
    pub schema_fingerprint: String,
    pub provider_family: String,
    pub declared_objects: BTreeSet<ObjectId>,
    pub required_capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SchemaRequirementRegistry {
    requirements: BTreeMap<String, SchemaRequirement>,
}

impl SchemaRequirementRegistry {
    pub fn register(
        &mut self,
        requirement: SchemaRequirement,
    ) -> Result<(), SchemaRequirementError> {
        let key = requirement.identity.canonical_name();
        if self.requirements.insert(key.clone(), requirement).is_some() {
            return Err(invalid(format!(
                "schema requirement '{key}' is registered more than once"
            )));
        }
        Ok(())
    }

    pub fn requirement(
        &self,
        canonical_name: &str,
    ) -> Result<&SchemaRequirement, SchemaRequirementError> {
        self.requirements.get(canonical_name).ok_or_else(|| {
            SchemaRequirementError::new(
                SchemaRequirementErrorKind::UnknownRequirement,
                format!("schema requirement '{canonical_name}' is not registered"),
            )
        })
    }

    pub fn entries(&self) -> impl ExactSizeIterator<Item = (&str, &SchemaRequirement)> + '_ {
        self.requirements
            .iter()
            .map(|(name, requirement)| (name.as_str(), requirement))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.requirements.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.requirements.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaRequirementErrorKind {
    InvalidArtifact,
    UnknownRequirement,
    MissingProvider,
    ProviderMismatch,
    MissingCapability,
    IncompatibleSchema,
    UndeclaredObject,
    UndeclaredBehavior,
    InvalidWitnessUse,
    ExecutionProfileMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaRequirementError {
    pub kind: SchemaRequirementErrorKind,
    pub message: String,
}

impl SchemaRequirementError {
    #[must_use]
    pub fn new(kind: SchemaRequirementErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for SchemaRequirementError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SchemaRequirementError {}

#[allow(clippy::too_many_arguments)]
fn artifact_fingerprint(
    identity: &SchemaRequirementIdentity,
    provider: &ProviderIdentity,
    provider_family: &str,
    minimum_server_version: &str,
    source_document: &str,
    source_fingerprint: &str,
    required_capabilities: &BTreeSet<String>,
    schema: &SchemaSlice,
    normalized_schema_fingerprint: &str,
) -> Result<String, SchemaRequirementError> {
    let bytes = serde_json::to_vec(&(
        SCHEMA_REQUIREMENT_FORMAT_VERSION,
        identity,
        provider,
        provider_family,
        minimum_server_version,
        source_document,
        source_fingerprint,
        required_capabilities,
        schema,
        normalized_schema_fingerprint,
    ))
    .map_err(|error| invalid(format!("cannot serialize schema requirement: {error}")))?;
    Ok(lower_hex(&Sha256::digest(bytes)))
}

fn compare_versions(left: &str, right: &str) -> Result<std::cmp::Ordering, SchemaRequirementError> {
    let mut left = version_parts(left)?;
    let mut right = version_parts(right)?;
    let width = left.len().max(right.len());
    left.resize(width, 0);
    right.resize(width, 0);
    Ok(left.cmp(&right))
}

fn version_parts(value: &str) -> Result<Vec<u64>, SchemaRequirementError> {
    if !valid_server_version(value) {
        return Err(invalid(format!("invalid SQL server version '{value}'")));
    }
    value
        .split('.')
        .map(|part| {
            part.parse::<u64>()
                .map_err(|_| invalid("SQL server version overflows"))
        })
        .collect()
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value.bytes().enumerate().all(|(index, byte)| {
            byte == b'_' || byte.is_ascii_lowercase() || (index > 0 && byte.is_ascii_digit())
        })
}

fn valid_family(value: &str) -> bool {
    valid_identifier(value)
}

fn valid_server_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value.split('.').all(|part| {
            !part.is_empty() && part.len() <= 8 && part.bytes().all(|byte| byte.is_ascii_digit())
        })
}

fn valid_relative_document(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('/')
        && !value.contains('\\')
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "..")
}

fn valid_capability(value: &str) -> bool {
    value.starts_with("sql.")
        && value.len() <= 96
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
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

fn invalid(message: impl Into<String>) -> SchemaRequirementError {
    SchemaRequirementError::new(SchemaRequirementErrorKind::InvalidArtifact, message)
}
