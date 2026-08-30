use crate::{
    Cardinality, EffectContract, ObjectId, QueryParameterSlot, QueryTemplateContract, SifrType,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;

pub const QUERY_SIGNATURE_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuerySignatureFormat {
    CanonicalJson,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionPolicy {
    Private,
    Exported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionStability {
    pub emitted_columns: Vec<String>,
    pub used_star: bool,
    pub unstable_expressions: Vec<usize>,
    pub duplicate_names: BTreeSet<String>,
    pub schema_sensitive_types: BTreeSet<String>,
}

impl ProjectionStability {
    pub fn validate(self, policy: ProjectionPolicy) -> Result<Vec<String>, QuerySignatureError> {
        if !self.duplicate_names.is_empty() {
            return Err(QuerySignatureError::new(
                "result names are duplicated; add explicit unique aliases",
                Some("alias every duplicated projection"),
            ));
        }
        if !self.unstable_expressions.is_empty() {
            return Err(QuerySignatureError::new(
                "result expressions have unstable names; add explicit aliases",
                Some("insert an AS alias for each expression"),
            ));
        }
        if policy == ProjectionPolicy::Exported && self.used_star {
            return Err(QuerySignatureError::new(
                "an exported query cannot use SELECT *",
                Some("replace * with the explicit projected columns"),
            ));
        }
        if policy == ProjectionPolicy::Exported && !self.schema_sensitive_types.is_empty() {
            return Err(QuerySignatureError::new(
                "an exported query exposes schema-sensitive anonymous types",
                Some("cast or map each field to a stable public SQL type"),
            ));
        }
        if self.emitted_columns.is_empty() {
            return Err(QuerySignatureError::new(
                "a row-returning projection must emit at least one column",
                None,
            ));
        }
        Ok(self.emitted_columns)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuerySignatureEntry {
    pub module: String,
    pub symbol: String,
    pub template_identity: String,
    pub profile_identity: String,
    pub schema_fingerprint: String,
    pub parameters: Vec<QueryParameterSlot>,
    pub result: Vec<(String, SifrType)>,
    pub cardinality: Cardinality,
    pub effects: EffectContract,
    pub schema_dependencies: BTreeSet<ObjectId>,
}

impl QuerySignatureEntry {
    #[must_use]
    pub fn from_contract(contract: &QueryTemplateContract) -> Self {
        Self {
            module: contract.origin.module.clone(),
            symbol: contract.origin.symbol.clone(),
            template_identity: contract.identity.as_str().to_string(),
            profile_identity: contract.profile_identity.clone(),
            schema_fingerprint: contract.schema_fingerprint.clone(),
            parameters: contract.parameters.clone(),
            result: contract
                .result_fields
                .iter()
                .map(|field| (field.name.clone(), field.sifr_type.clone()))
                .collect(),
            cardinality: contract.cardinality,
            effects: contract.effects.clone(),
            schema_dependencies: contract
                .effects
                .referenced_objects
                .union(&contract.effects.affected_objects)
                .cloned()
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuerySignatureArtifact {
    pub format_version: u32,
    pub package_identity: String,
    pub entries: BTreeMap<String, QuerySignatureEntry>,
    pub fingerprint: String,
}

impl QuerySignatureArtifact {
    pub fn build(
        package_identity: impl Into<String>,
        entries: impl IntoIterator<Item = QuerySignatureEntry>,
    ) -> Result<Self, QuerySignatureError> {
        let package_identity = package_identity.into();
        if package_identity.trim().is_empty() {
            return Err(QuerySignatureError::new(
                "query signature artifact needs a package identity",
                None,
            ));
        }
        let mut by_symbol = BTreeMap::new();
        for entry in entries {
            let key = format!("{}::{}", entry.module, entry.symbol);
            if by_symbol.insert(key, entry).is_some() {
                return Err(QuerySignatureError::new(
                    "query signature artifact has a duplicate exported symbol",
                    None,
                ));
            }
        }
        let canonical = serde_json::to_vec(&(
            QUERY_SIGNATURE_FORMAT_VERSION,
            &package_identity,
            &by_symbol,
        ))
        .map_err(|error| {
            QuerySignatureError::new(format!("cannot serialize query signatures: {error}"), None)
        })?;
        let digest = Sha256::digest(canonical);
        let fingerprint = digest
            .iter()
            .fold(String::with_capacity(64), |mut value, byte| {
                let _ = write!(value, "{byte:02x}");
                value
            });
        Ok(Self {
            format_version: QUERY_SIGNATURE_FORMAT_VERSION,
            package_identity,
            entries: by_symbol,
            fingerprint,
        })
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, QuerySignatureError> {
        let rebuilt = Self::build(
            self.package_identity.clone(),
            self.entries.values().cloned(),
        )?;
        if &rebuilt != self {
            return Err(QuerySignatureError::new(
                "query signature artifact is not canonical",
                None,
            ));
        }
        serde_json::to_vec_pretty(self).map_err(|error| {
            QuerySignatureError::new(format!("cannot encode query signatures: {error}"), None)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PublicQueryChangeKind {
    Removed,
    Parameters,
    Result,
    Cardinality,
    Effects,
    SchemaDependencies,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicQueryChange {
    pub symbol: String,
    pub kind: PublicQueryChangeKind,
}

#[must_use]
pub fn compare_query_signatures(
    baseline: &QuerySignatureArtifact,
    candidate: &QuerySignatureArtifact,
) -> Vec<PublicQueryChange> {
    let mut changes = Vec::new();
    for (symbol, old) in &baseline.entries {
        let Some(new) = candidate.entries.get(symbol) else {
            changes.push(PublicQueryChange {
                symbol: symbol.clone(),
                kind: PublicQueryChangeKind::Removed,
            });
            continue;
        };
        for (changed, kind) in [
            (
                old.parameters != new.parameters,
                PublicQueryChangeKind::Parameters,
            ),
            (old.result != new.result, PublicQueryChangeKind::Result),
            (
                old.cardinality != new.cardinality,
                PublicQueryChangeKind::Cardinality,
            ),
            (old.effects != new.effects, PublicQueryChangeKind::Effects),
            (
                old.schema_dependencies != new.schema_dependencies,
                PublicQueryChangeKind::SchemaDependencies,
            ),
        ] {
            if changed {
                changes.push(PublicQueryChange {
                    symbol: symbol.clone(),
                    kind,
                });
            }
        }
    }
    changes
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuerySignatureError {
    pub message: String,
    pub machine_fix: Option<String>,
}

impl QuerySignatureError {
    fn new(message: impl Into<String>, machine_fix: Option<&str>) -> Self {
        Self {
            message: message.into(),
            machine_fix: machine_fix.map(str::to_string),
        }
    }
}

impl fmt::Display for QuerySignatureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for QuerySignatureError {}
