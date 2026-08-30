use crate::{
    Cardinality, CodecIdentity, CodecRegistry, DatabaseType, EffectContract, NullCodecBehavior,
    Nullability, ObjectId, SifrType, canonical_read_type_with_nullability_in,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderParameter {
    pub slot: u32,
    pub database_type: DatabaseType,
    pub nullability: Nullability,
    pub codec: CodecIdentity,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderResultField {
    pub name: String,
    pub sifr_type: SifrType,
    pub database_type: DatabaseType,
    pub nullability: Nullability,
    pub codec: CodecIdentity,
    pub source_object: Option<ObjectId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderAnalysis {
    pub server_profile: String,
    pub normalized_statement: String,
    pub parameters: Vec<ProviderParameter>,
    pub result_fields: Vec<ProviderResultField>,
    pub cardinality: Cardinality,
    pub effects: EffectContract,
    pub semantic_flags: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderAnalysisError {
    UnsupportedDatabaseType(ObjectId),
    InvalidBind { slot: u32 },
    InvalidResultField { field: String },
    InvalidDialectSemantics,
}

/// Dialect components own parsing and semantics. The common compiler consumes
/// only the closed provider-neutral analysis returned by this interface.
pub trait DialectSemantics {
    fn family(&self) -> &str;

    fn analyze(
        &self,
        schema_fingerprint: &str,
        source: &str,
    ) -> Result<ProviderAnalysis, ProviderAnalysisError>;
}

impl ProviderAnalysis {
    pub fn validate(&self, codecs: &CodecRegistry) -> Result<(), ProviderAnalysisError> {
        if self.server_profile != codecs.server_profile()
            || self.normalized_statement.trim().is_empty()
            || self
                .semantic_flags
                .iter()
                .any(|flag| !valid_semantic_flag(flag))
        {
            return Err(ProviderAnalysisError::InvalidDialectSemantics);
        }
        if self.cardinality.validate().is_err() || self.effects.validate().is_err() {
            return Err(ProviderAnalysisError::InvalidDialectSemantics);
        }
        for (expected_slot, parameter) in self.parameters.iter().enumerate() {
            if usize::try_from(parameter.slot) != Ok(expected_slot)
                || !codec_matches(codecs, &parameter.codec, &parameter.database_type)
            {
                return Err(ProviderAnalysisError::InvalidBind {
                    slot: parameter.slot,
                });
            }
        }
        let mut names = BTreeSet::new();
        for field in &self.result_fields {
            if field.name.is_empty()
                || !names.insert(field.name.as_str())
                || !codec_matches(codecs, &field.codec, &field.database_type)
            {
                return Err(ProviderAnalysisError::InvalidResultField {
                    field: field.name.clone(),
                });
            }
            let null_rejected = field.nullability == Nullability::Nullable
                && codecs
                    .codec(&field.codec)
                    .is_some_and(|codec| codec.null_behavior == NullCodecBehavior::Reject);
            if null_rejected
                || canonical_read_type_with_nullability_in(
                    &field.database_type,
                    field.nullability,
                    codecs,
                )
                .as_ref()
                    != Ok(&field.sifr_type)
            {
                return Err(ProviderAnalysisError::InvalidResultField {
                    field: field.name.clone(),
                });
            }
        }
        Ok(())
    }
}

fn codec_matches(
    codecs: &CodecRegistry,
    identity: &CodecIdentity,
    database: &DatabaseType,
) -> bool {
    let Some(contract) = codecs.codec(identity) else {
        return false;
    };
    contract.database_type == *database
}

fn valid_semantic_flag(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

impl fmt::Display for ProviderAnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedDatabaseType(identity) => {
                write!(
                    formatter,
                    "database type '{identity}' has no common SQL contract"
                )
            }
            Self::InvalidBind { slot } => {
                write!(
                    formatter,
                    "parameter slot {slot} has an invalid bind contract"
                )
            }
            Self::InvalidResultField { field } => {
                write!(
                    formatter,
                    "result field '{field}' has an invalid type contract"
                )
            }
            Self::InvalidDialectSemantics => {
                formatter.write_str("provider returned invalid dialect semantics")
            }
        }
    }
}

impl std::error::Error for ProviderAnalysisError {}
