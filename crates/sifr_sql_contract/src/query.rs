use crate::{
    Cardinality, CodecRegistry, DialectIdentity, EffectContract, ProfileModuleRegistry,
    ProviderAnalysis, ProviderResultField, QueryEffect, SifrType,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sifr_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QueryTemplateIdentity(String);

impl QueryTemplateIdentity {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryOrigin {
    pub module: String,
    pub symbol: String,
    pub source_start: u32,
    pub source_end: u32,
}

impl QueryOrigin {
    pub fn new(
        module: impl Into<String>,
        symbol: impl Into<String>,
        source_start: u32,
        source_end: u32,
    ) -> Result<Self, QueryContractError> {
        let origin = Self {
            module: module.into(),
            symbol: symbol.into(),
            source_start,
            source_end,
        };
        if !valid_symbol_path(&origin.module)
            || !valid_symbol_name(&origin.symbol)
            || source_start >= source_end
        {
            return Err(QueryContractError::new(
                QueryContractErrorKind::InvalidTemplate,
                "a reusable query needs one top-level symbol and a non-empty source range",
            ));
        }
        Ok(origin)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryAdapter {
    ExpectAtMostOne,
    First,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryWarning {
    NondeterministicFirst,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryParameterSlot {
    pub slot: u32,
    pub sifr_type: SifrType,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryTemplateContract {
    pub identity: QueryTemplateIdentity,
    pub origin: QueryOrigin,
    pub profile_identity: String,
    pub profile_fingerprint: String,
    pub schema_fingerprint: String,
    pub dialect: DialectIdentity,
    pub normalized_statement: String,
    pub parameters: Vec<QueryParameterSlot>,
    pub result_fields: Vec<ProviderResultField>,
    pub cardinality: Cardinality,
    pub effects: EffectContract,
    pub deterministic_order: bool,
    pub fragment_identities: Vec<String>,
    pub adapters: Vec<QueryAdapter>,
}

#[derive(Clone, Debug)]
pub struct QueryTemplateDraft {
    pub origin: QueryOrigin,
    pub analysis: ProviderAnalysis,
    pub parameter_types: Vec<SifrType>,
    pub deterministic_order: bool,
    pub fragment_identities: Vec<String>,
}

impl QueryTemplateContract {
    pub fn compile(
        registry: &ProfileModuleRegistry,
        profile_name: &str,
        draft: QueryTemplateDraft,
        codecs: &CodecRegistry,
    ) -> Result<Self, QueryContractError> {
        let QueryTemplateDraft {
            origin,
            analysis,
            parameter_types,
            deterministic_order,
            fragment_identities,
        } = draft;
        let profile = registry.profile(profile_name).map_err(|error| {
            QueryContractError::new(QueryContractErrorKind::ProfileMismatch, error.message)
        })?;
        analysis.validate(codecs).map_err(|error| {
            QueryContractError::new(QueryContractErrorKind::InvalidTemplate, error.to_string())
        })?;
        if analysis.parameters.len() != parameter_types.len()
            || fragment_identities.iter().any(|identity| {
                identity.is_empty()
                    || identity.len() > 160
                    || identity.chars().any(char::is_control)
            })
            || fragment_identities
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
        {
            return Err(QueryContractError::new(
                QueryContractErrorKind::InvalidTemplate,
                "query parameters or fragment identities are not canonical",
            ));
        }
        let parameters = analysis
            .parameters
            .iter()
            .zip(parameter_types)
            .map(|(parameter, sifr_type)| QueryParameterSlot {
                slot: parameter.slot,
                sifr_type,
            })
            .collect::<Vec<_>>();
        let identity = derive_identity(
            profile.authority().nominal_identity.as_str(),
            &origin,
            &analysis.normalized_statement,
        )?;
        let contract = Self {
            identity,
            origin,
            profile_identity: profile.authority().nominal_identity.clone(),
            profile_fingerprint: profile.authority().profile_fingerprint.as_str().to_string(),
            schema_fingerprint: profile.authority().schema_fingerprint.as_str().to_string(),
            dialect: profile.authority().profile.schema.dialect.clone(),
            normalized_statement: analysis.normalized_statement,
            parameters,
            result_fields: analysis.result_fields,
            cardinality: analysis.cardinality,
            effects: analysis.effects,
            deterministic_order,
            fragment_identities,
            adapters: Vec::new(),
        };
        contract.validate()?;
        Ok(contract)
    }

    pub fn validate(&self) -> Result<(), QueryContractError> {
        if self.normalized_statement.trim().is_empty()
            || !valid_fingerprint(&self.profile_fingerprint)
            || !valid_fingerprint(&self.schema_fingerprint)
            || self.cardinality.validate().is_err()
            || self.effects.validate().is_err()
            || self
                .parameters
                .iter()
                .enumerate()
                .any(|(slot, parameter)| usize::try_from(parameter.slot) != Ok(slot))
        {
            return Err(QueryContractError::new(
                QueryContractErrorKind::InvalidTemplate,
                "query template contract is not canonical",
            ));
        }
        Ok(())
    }

    pub fn expect_at_most_one(mut self) -> Result<Self, QueryContractError> {
        self.cardinality = match self.cardinality {
            Cardinality::Empty => Cardinality::Empty,
            Cardinality::Interval { minimum, .. } => Cardinality::Interval {
                minimum: minimum.min(1),
                maximum: Some(1),
            },
        };
        self.adapters.push(QueryAdapter::ExpectAtMostOne);
        self.validate()?;
        Ok(self)
    }

    pub fn first(
        mut self,
        provider_limited_statement: impl Into<String>,
    ) -> Result<(Self, Option<QueryWarning>), QueryContractError> {
        let statement = provider_limited_statement.into();
        if statement.trim().is_empty() {
            return Err(QueryContractError::new(
                QueryContractErrorKind::InvalidTemplate,
                "first() requires a provider-validated one-row statement",
            ));
        }
        let warning = (!self.deterministic_order).then_some(QueryWarning::NondeterministicFirst);
        self.normalized_statement = statement;
        self.cardinality = match self.cardinality {
            Cardinality::Empty => Cardinality::Empty,
            Cardinality::Interval { minimum, .. } => Cardinality::Interval {
                minimum: minimum.min(1),
                maximum: Some(1),
            },
        };
        self.adapters.push(QueryAdapter::First);
        self.validate()?;
        Ok((self, warning))
    }

    #[must_use]
    pub fn returns_rows(&self) -> bool {
        !self.result_fields.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuerySymbolKind {
    TopLevelReusable,
    LocalFunction,
    Closure,
    RuntimeValue,
}

#[derive(Clone, Debug)]
pub struct QuerySymbol {
    pub module: String,
    pub name: String,
    pub kind: QuerySymbolKind,
    pub exported: bool,
    pub template: QueryTemplateContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RowOfType {
    pub template_identity: QueryTemplateIdentity,
    pub fields: Vec<(String, SifrType)>,
}

#[derive(Clone, Debug, Default)]
pub struct QuerySignatureRegistry {
    symbols: BTreeMap<(String, String), QuerySymbol>,
    identities: BTreeSet<QueryTemplateIdentity>,
}

impl QuerySignatureRegistry {
    pub fn register(&mut self, symbol: QuerySymbol) -> Result<(), QueryContractError> {
        if symbol.kind != QuerySymbolKind::TopLevelReusable
            || symbol.module != symbol.template.origin.module
            || symbol.name != symbol.template.origin.symbol
            || self
                .symbols
                .contains_key(&(symbol.module.clone(), symbol.name.clone()))
            || !self.identities.insert(symbol.template.identity.clone())
        {
            return Err(QueryContractError::new(
                QueryContractErrorKind::InvalidRowOf,
                "only one top-level reusable query can own a query signature",
            ));
        }
        self.symbols
            .insert((symbol.module.clone(), symbol.name.clone()), symbol);
        Ok(())
    }

    pub fn row_of(
        &self,
        module: &str,
        symbol: &str,
        crossing_module_boundary: bool,
    ) -> Result<RowOfType, QueryContractError> {
        let query = self
            .symbols
            .get(&(module.to_string(), symbol.to_string()))
            .ok_or_else(|| {
                QueryContractError::new(
                    QueryContractErrorKind::InvalidRowOf,
                    "RowOf requires a top-level reusable query symbol path",
                )
            })?;
        if crossing_module_boundary && !query.exported {
            return Err(QueryContractError::new(
                QueryContractErrorKind::InvalidRowOf,
                "RowOf cannot name a private query from another module",
            ));
        }
        Ok(RowOfType {
            template_identity: query.template.identity.clone(),
            fields: query
                .template
                .result_fields
                .iter()
                .map(|field| (field.name.clone(), field.sifr_type.clone()))
                .collect(),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryContractErrorKind {
    InvalidTemplate,
    ProfileMismatch,
    BindingOrder,
    InvalidRowOf,
    FragmentCategory,
    FragmentScope,
    AliasEscape,
    UnsafeSyntax,
    Cardinality,
    Effect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryContractError {
    pub kind: QueryContractErrorKind,
    pub message: String,
}

impl QueryContractError {
    #[must_use]
    pub fn new(kind: QueryContractErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn diagnostic_code(&self) -> DiagnosticCode {
        match self.kind {
            QueryContractErrorKind::BindingOrder => DiagnosticCode::SQL_BIND_COMPATIBILITY,
            QueryContractErrorKind::Cardinality => DiagnosticCode::SQL_CARDINALITY,
            QueryContractErrorKind::Effect => DiagnosticCode::SQL_EFFECT,
            QueryContractErrorKind::AliasEscape => DiagnosticCode::SQL_OWNERSHIP,
            QueryContractErrorKind::InvalidTemplate
            | QueryContractErrorKind::ProfileMismatch
            | QueryContractErrorKind::InvalidRowOf
            | QueryContractErrorKind::FragmentCategory
            | QueryContractErrorKind::FragmentScope
            | QueryContractErrorKind::UnsafeSyntax => DiagnosticCode::SQL_PROVIDER_CONTRACT,
        }
    }
}

impl fmt::Display for QueryContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for QueryContractError {}

fn derive_identity(
    profile_identity: &str,
    origin: &QueryOrigin,
    normalized_statement: &str,
) -> Result<QueryTemplateIdentity, QueryContractError> {
    let canonical =
        serde_json::to_vec(&(profile_identity, origin, normalized_statement)).map_err(|error| {
            QueryContractError::new(
                QueryContractErrorKind::InvalidTemplate,
                format!("cannot serialize query template identity: {error}"),
            )
        })?;
    let digest = Sha256::digest(canonical);
    let encoded = digest.iter().fold(
        String::with_capacity(digest.len() * 2),
        |mut output, byte| {
            let _ = write!(output, "{byte:02x}");
            output
        },
    );
    Ok(QueryTemplateIdentity(encoded))
}

fn valid_symbol_path(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(valid_symbol_name)
}

fn valid_symbol_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[must_use]
pub const fn effect_can_unify(left: QueryEffect, right: QueryEffect) -> bool {
    matches!(
        (left, right),
        (QueryEffect::Read, QueryEffect::Read)
            | (QueryEffect::Write, QueryEffect::Write)
            | (QueryEffect::ReadWrite, QueryEffect::ReadWrite)
            | (QueryEffect::SchemaChange, QueryEffect::SchemaChange)
            | (QueryEffect::SessionChange, QueryEffect::SessionChange)
            | (
                QueryEffect::TransactionControl,
                QueryEffect::TransactionControl
            )
    )
}
