use crate::{
    Cardinality, CodecContract, CodecIdentity, CodecRegistry, DatabaseType, EffectContract,
    NullCodecBehavior, Nullability, ObjectId, PanicContainment, SifrType, WireFormatIdentity,
    canonical_read_type_with_nullability_in,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
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
    /// Complete provider-owned set of schema objects reached anywhere in the
    /// statement, including predicate-only and write-target columns.
    pub accessed_objects: BTreeSet<ObjectId>,
    pub semantic_flags: BTreeSet<String>,
    /// Closed provider-owned account of every SQL capability used by the
    /// analyzed statement. Portable specialization treats this set as
    /// authoritative; callers cannot supply or narrow it.
    pub required_capabilities: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderDiagnosticSpan {
    pub kind: String,
    pub document: String,
    pub start: u32,
    pub end: u32,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderSemanticDiagnostic {
    pub code: String,
    pub message: String,
    pub primary: ProviderDiagnosticSpan,
    pub related: Vec<ProviderDiagnosticSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderAnalysisError {
    UnsupportedDatabaseType(ObjectId),
    InvalidBind { slot: u32 },
    InvalidResultField { field: String },
    Diagnostic(Box<ProviderSemanticDiagnostic>),
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
            || self.required_capabilities.is_empty()
            || self
                .required_capabilities
                .iter()
                .any(|capability| !valid_capability(capability))
        {
            return Err(ProviderAnalysisError::InvalidDialectSemantics);
        }
        if self.cardinality.validate().is_err() || self.effects.validate().is_err() {
            return Err(ProviderAnalysisError::InvalidDialectSemantics);
        }
        let accounted_objects = self
            .effects
            .referenced_objects
            .iter()
            .chain(&self.effects.affected_objects)
            .chain(
                self.result_fields
                    .iter()
                    .filter_map(|field| field.source_object.as_ref()),
            );
        if self
            .accessed_objects
            .iter()
            .any(|object| object.as_str().is_empty())
            || accounted_objects
                .into_iter()
                .any(|object| !self.accessed_objects.contains(object))
        {
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

/// Build the closed codec view needed to validate a component response. The
/// provider chooses codec identities and database types; the typed source owns
/// parameter Sifr types. Duplicate codec identities must describe one exact
/// contract.
pub fn component_codec_registry(
    analysis: &ProviderAnalysis,
    parameter_types: &[SifrType],
) -> Result<CodecRegistry, ProviderAnalysisError> {
    if parameter_types.len() != analysis.parameters.len() {
        return Err(ProviderAnalysisError::InvalidDialectSemantics);
    }
    let mut contracts = BTreeMap::new();
    for (parameter, sifr_type) in analysis.parameters.iter().zip(parameter_types) {
        insert_component_codec(
            &mut contracts,
            component_codec_contract(
                analysis,
                &parameter.codec,
                &parameter.database_type,
                sifr_type,
            )?,
        )?;
    }
    for field in &analysis.result_fields {
        insert_component_codec(
            &mut contracts,
            component_codec_contract(
                analysis,
                &field.codec,
                &field.database_type,
                &field.sifr_type,
            )?,
        )?;
    }
    CodecRegistry::for_profile(analysis.server_profile.clone(), contracts.into_values())
        .map_err(|_| ProviderAnalysisError::InvalidDialectSemantics)
}

fn insert_component_codec(
    contracts: &mut BTreeMap<CodecIdentity, CodecContract>,
    contract: CodecContract,
) -> Result<(), ProviderAnalysisError> {
    if let Some(existing) = contracts.get(&contract.identity)
        && existing != &contract
    {
        return Err(ProviderAnalysisError::InvalidDialectSemantics);
    }
    contracts.insert(contract.identity.clone(), contract);
    Ok(())
}

fn component_codec_contract(
    analysis: &ProviderAnalysis,
    identity: &CodecIdentity,
    database_type: &DatabaseType,
    sifr_type: &SifrType,
) -> Result<CodecContract, ProviderAnalysisError> {
    let encoded = serde_json::to_vec(&(identity, database_type, sifr_type))
        .map_err(|_| ProviderAnalysisError::InvalidDialectSemantics)?;
    let digest = Sha256::digest(encoded);
    let suffix = crate::fingerprint::lower_hex(&digest[..12]);
    Ok(CodecContract {
        sifr_type: sifr_type.clone(),
        database_type: database_type.clone(),
        identity: identity.clone(),
        server_profiles: BTreeSet::from([analysis.server_profile.clone()]),
        encode_error: "sifr.sql.EncodeError".to_string(),
        decode_error: "sifr.sql.DecodeError".to_string(),
        null_behavior: NullCodecBehavior::PassThrough,
        wire_format: WireFormatIdentity::new(format!("sql.component.{suffix}.v1"))
            .map_err(|_| ProviderAnalysisError::InvalidDialectSemantics)?,
        panic_containment: PanicContainment::CatchAndRedact,
    })
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

fn valid_capability(value: &str) -> bool {
    value.starts_with("sql.")
        && value.len() <= 96
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
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
            Self::Diagnostic(diagnostic) => {
                write!(formatter, "{}: {}", diagnostic.code, diagnostic.message)
            }
            Self::InvalidDialectSemantics => {
                formatter.write_str("provider returned invalid dialect semantics")
            }
        }
    }
}

impl std::error::Error for ProviderAnalysisError {}
