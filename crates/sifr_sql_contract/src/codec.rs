use crate::{DatabaseType, SchemaContractError, SchemaContractErrorKind, SifrType};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CodecIdentity(String);

impl CodecIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, SchemaContractError> {
        let value = value.into();
        validate_identity("codec", &value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WireFormatIdentity(String);

impl WireFormatIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, SchemaContractError> {
        let value = value.into();
        validate_identity("wire format", &value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NullCodecBehavior {
    Reject,
    PassThrough,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanicContainment {
    CatchAndRedact,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodecContract {
    pub identity: CodecIdentity,
    pub database_type: DatabaseType,
    pub sifr_type: SifrType,
    pub server_profiles: BTreeSet<String>,
    pub encode_error: String,
    pub decode_error: String,
    pub null_behavior: NullCodecBehavior,
    pub wire_format: WireFormatIdentity,
    pub panic_containment: PanicContainment,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodecRegistry {
    server_profile: String,
    codecs: BTreeMap<CodecIdentity, CodecContract>,
    database_types: BTreeMap<DatabaseType, CodecIdentity>,
}

impl CodecRegistry {
    pub fn for_profile(
        server_profile: impl Into<String>,
        contracts: impl IntoIterator<Item = CodecContract>,
    ) -> Result<Self, SchemaContractError> {
        let server_profile = server_profile.into();
        validate_identity("server profile", &server_profile)?;
        let mut registry = Self {
            server_profile,
            codecs: BTreeMap::new(),
            database_types: BTreeMap::new(),
        };
        for contract in contracts {
            validate_contract(&contract)?;
            if !contract.server_profiles.contains(&registry.server_profile) {
                continue;
            }
            if registry.codecs.contains_key(&contract.identity) {
                return Err(invalid(format!(
                    "duplicate codec identity '{}'",
                    contract.identity.as_str()
                )));
            }
            if registry
                .database_types
                .contains_key(&contract.database_type)
            {
                return Err(invalid("database type has more than one codec"));
            }
            registry
                .database_types
                .insert(contract.database_type.clone(), contract.identity.clone());
            registry.codecs.insert(contract.identity.clone(), contract);
        }
        Ok(registry)
    }

    #[must_use]
    pub fn server_profile(&self) -> &str {
        &self.server_profile
    }

    #[must_use]
    pub fn codec(&self, identity: &CodecIdentity) -> Option<&CodecContract> {
        self.codecs.get(identity)
    }

    #[must_use]
    pub fn codec_for_database_type(&self, database: &DatabaseType) -> Option<&CodecContract> {
        self.database_types
            .get(database)
            .and_then(|identity| self.codecs.get(identity))
    }
}

fn validate_contract(contract: &CodecContract) -> Result<(), SchemaContractError> {
    validate_identity("codec", contract.identity.as_str())?;
    validate_identity("wire format", contract.wire_format.as_str())?;
    if contract.server_profiles.is_empty()
        || contract.encode_error.trim().is_empty()
        || contract.decode_error.trim().is_empty()
    {
        return Err(invalid(
            "codec needs server profiles and closed encode and decode errors",
        ));
    }
    if contract
        .server_profiles
        .iter()
        .any(|profile| validate_identity("server profile", profile).is_err())
    {
        return Err(invalid("codec server profile identity cannot be empty"));
    }
    validate_identity("encode error type", &contract.encode_error)?;
    validate_identity("decode error type", &contract.decode_error)?;
    if let DatabaseType::Custom { codec, .. } = &contract.database_type
        && codec != &contract.identity
    {
        return Err(invalid(
            "custom database type and codec contract identities differ",
        ));
    }
    Ok(())
}

fn validate_identity(label: &str, value: &str) -> Result<(), SchemaContractError> {
    if value.is_empty()
        || value.len() > 160
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
    {
        return Err(invalid(format!("{label} identity is invalid")));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> SchemaContractError {
    SchemaContractError::new(SchemaContractErrorKind::InvalidSchema, message)
}
