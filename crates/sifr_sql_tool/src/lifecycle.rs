use serde::{Deserialize, Serialize};
use sifr_sql_contract::{
    ObjectId, ProfileAuthority, QuerySignatureArtifact, SchemaDiff, SchemaIr, semantic_diff,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaLifecycleErrorKind {
    MissingAuthority,
    ConflictingAuthority,
    DuplicateAuthority,
    InvalidAuthority,
    CredentialDisclosure,
    NondeterministicOutput,
    Serialization,
    FileSystem,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchemaLifecycleError {
    pub kind: SchemaLifecycleErrorKind,
    pub message: String,
}

impl SchemaLifecycleError {
    pub(crate) fn new(kind: SchemaLifecycleErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for SchemaLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SchemaLifecycleError {}

#[derive(Clone, Debug)]
pub struct NamedProfileAuthority {
    pub name: String,
    pub authority: ProfileAuthority,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthorityMergeRule {
    RequireSingle,
    IdenticalSchemas,
}

pub fn resolve_build_authority(
    mut candidates: Vec<NamedProfileAuthority>,
    merge_rule: AuthorityMergeRule,
) -> Result<ProfileAuthority, SchemaLifecycleError> {
    if candidates.is_empty() {
        return Err(error(
            SchemaLifecycleErrorKind::MissingAuthority,
            "schema build requires one canonical authority",
        ));
    }
    candidates.sort_by(|left, right| left.name.cmp(&right.name));
    if candidates
        .windows(2)
        .any(|pair| pair[0].name == pair[1].name)
    {
        return Err(error(
            SchemaLifecycleErrorKind::DuplicateAuthority,
            "schema authority names must be unique",
        ));
    }
    if candidates.len() > 1 {
        if merge_rule == AuthorityMergeRule::RequireSingle {
            return Err(error(
                SchemaLifecycleErrorKind::ConflictingAuthority,
                "multiple schema inputs claim canonical authority without a merge rule",
            ));
        }
        let expected = &candidates[0].authority.profile.schema;
        if candidates.iter().skip(1).any(|candidate| {
            !semantic_diff(expected, &candidate.authority.profile.schema).is_empty()
        }) {
            return Err(error(
                SchemaLifecycleErrorKind::ConflictingAuthority,
                "the identical-schema merge rule received different schema semantics",
            ));
        }
    }
    Ok(candidates.remove(0).authority)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaPullPlan {
    pub diff: SchemaDiff,
    pub requires_acceptance: bool,
    pub replacement: Option<SchemaIr>,
}

#[must_use]
pub fn plan_pull(
    checked_snapshot: &SchemaIr,
    live_schema: SchemaIr,
    accept_non_interactive: bool,
) -> SchemaPullPlan {
    let diff = semantic_diff(checked_snapshot, &live_schema);
    let requires_acceptance = !diff.is_empty() && !accept_non_interactive;
    SchemaPullPlan {
        replacement: (!requires_acceptance && !diff.is_empty()).then_some(live_schema),
        diff,
        requires_acceptance,
    }
}

#[derive(Clone, Debug)]
pub struct NamedSchema {
    pub authority: String,
    pub schema: SchemaIr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaAuthorityDiff {
    pub authority: String,
    pub diff: SchemaDiff,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaValidationReport {
    pub valid: bool,
    pub comparisons: Vec<SchemaAuthorityDiff>,
    pub affected_queries: BTreeSet<String>,
}

pub fn validate_schema_authorities(
    canonical: &SchemaIr,
    comparisons: impl IntoIterator<Item = NamedSchema>,
    signatures: Option<&QuerySignatureArtifact>,
) -> Result<SchemaValidationReport, SchemaLifecycleError> {
    let mut seen = BTreeSet::new();
    let mut reports = Vec::new();
    let mut changed = BTreeSet::new();
    let mut global_change = false;
    for comparison in comparisons {
        if comparison.authority.trim().is_empty() || !seen.insert(comparison.authority.clone()) {
            return Err(error(
                SchemaLifecycleErrorKind::DuplicateAuthority,
                "schema validation authority names must be unique and non-empty",
            ));
        }
        let diff = semantic_diff(canonical, &comparison.schema);
        global_change |= diff.provider_changed || diff.dialect_changed;
        changed.extend(diff.objects.iter().map(|change| change.identity.clone()));
        reports.push(SchemaAuthorityDiff {
            authority: comparison.authority,
            diff,
        });
    }
    reports.sort_by(|left, right| left.authority.cmp(&right.authority));
    let affected_queries = signatures.map_or_else(BTreeSet::new, |artifact| {
        if global_change {
            artifact.entries.keys().cloned().collect()
        } else {
            affected_queries(artifact, &changed)
        }
    });
    Ok(SchemaValidationReport {
        valid: reports.iter().all(|report| report.diff.is_empty()),
        comparisons: reports,
        affected_queries,
    })
}

#[must_use]
pub fn affected_queries(
    signatures: &QuerySignatureArtifact,
    changed_objects: &BTreeSet<ObjectId>,
) -> BTreeSet<String> {
    signatures
        .entries
        .iter()
        .filter(|(_, entry)| !entry.schema_dependencies.is_disjoint(changed_objects))
        .map(|(symbol, _)| symbol.clone())
        .collect()
}

pub(crate) fn reject_credentials(schema: &SchemaIr) -> Result<(), SchemaLifecycleError> {
    reject_secret_text("provider package source", &schema.provider.package_source)?;
    for object in schema.objects.values() {
        for (key, value) in &object.semantic {
            if credential_key(key) {
                return Err(error(
                    SchemaLifecycleErrorKind::CredentialDisclosure,
                    format!(
                        "schema object '{}' contains credential-shaped property '{key}'",
                        object.identity
                    ),
                ));
            }
            reject_semantic_value(&object.identity, value)?;
        }
    }
    Ok(())
}

fn reject_semantic_value(
    identity: &ObjectId,
    value: &sifr_sql_contract::SemanticValue,
) -> Result<(), SchemaLifecycleError> {
    use sifr_sql_contract::SemanticValue;
    match value {
        SemanticValue::Text(value) => reject_secret_text(identity.as_str(), value),
        SemanticValue::List(values) => {
            for value in values {
                reject_semantic_value(identity, value)?;
            }
            Ok(())
        }
        SemanticValue::Set(values) => {
            for value in values {
                reject_semantic_value(identity, value)?;
            }
            Ok(())
        }
        SemanticValue::Map(values) => {
            for (key, value) in values {
                if credential_key(key) {
                    return Err(error(
                        SchemaLifecycleErrorKind::CredentialDisclosure,
                        format!("schema object '{identity}' contains credential-shaped metadata"),
                    ));
                }
                reject_semantic_value(identity, value)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn credential_key(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase().replace(['-', '_'], "");
    ["credential", "password", "privatekey", "secret", "token"]
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn reject_secret_text(label: &str, value: &str) -> Result<(), SchemaLifecycleError> {
    let has_url_userinfo = value
        .split_once("://")
        .and_then(|(_, rest)| rest.split_once('@').map(|(userinfo, _)| userinfo))
        .is_some_and(|userinfo| userinfo.contains(':'));
    if has_url_userinfo || value.contains("PRIVATE KEY-----") {
        return Err(error(
            SchemaLifecycleErrorKind::CredentialDisclosure,
            format!("{label} contains credential material"),
        ));
    }
    Ok(())
}

pub(crate) fn error(
    kind: SchemaLifecycleErrorKind,
    message: impl Into<String>,
) -> SchemaLifecycleError {
    SchemaLifecycleError::new(kind, message)
}

#[must_use]
pub(crate) fn reverse_dependencies(schema: &SchemaIr) -> BTreeMap<ObjectId, BTreeSet<ObjectId>> {
    let mut reverse = schema
        .objects
        .keys()
        .cloned()
        .map(|identity| (identity, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for object in schema.objects.values() {
        for dependency in &object.dependencies {
            reverse
                .entry(dependency.clone())
                .or_default()
                .insert(object.identity.clone());
        }
    }
    reverse
}
