mod profile;
mod requirements;

use profile::parse_profile;
use requirements::parse_requirements;

use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use sifr_sql_contract::{PoolingMode, SchemaEvidence, SchemaStrictness, SessionContract};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaSourceKind {
    SqlDdl,
    ProviderMetadata,
    GeneratedDefinitions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlProfileConfig {
    pub provider: String,
    pub family: String,
    pub sources: Vec<PathBuf>,
    pub source_kind: SchemaSourceKind,
    pub server_version: String,
    pub extensions: BTreeSet<String>,
    pub evidence: SchemaEvidence,
    pub strictness: SchemaStrictness,
    pub pooling: PoolingMode,
    pub session: SessionContract,
    pub accepted_signers: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlRequirementProviderConfig {
    pub provider: String,
    pub source: PathBuf,
    pub server_version: String,
    pub extensions: BTreeSet<String>,
    pub sql_modes: BTreeSet<String>,
    pub collation: Option<String>,
    pub character_set: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SqlRequirementConfig {
    pub capabilities: BTreeSet<String>,
    pub providers: BTreeMap<String, SqlRequirementProviderConfig>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SqlConfig {
    pub profiles: BTreeMap<String, SqlProfileConfig>,
    pub requirements: BTreeMap<String, SqlRequirementConfig>,
}

pub(super) fn parse_sql_config(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: Option<&toml::Table>,
) -> Result<SqlConfig, PackageDiagnostic> {
    let Some(table) = table else {
        return Ok(SqlConfig::default());
    };
    reject_unknown(
        cargo_package_id,
        manifest_path,
        "sql",
        table,
        &["profiles", "requirements"],
    )?;
    let profiles = optional_table(
        cargo_package_id,
        manifest_path,
        table,
        "sql.profiles",
        "profiles",
    )?
    .map(|profiles| {
        profiles
            .iter()
            .map(|(name, value)| {
                validate_profile_name(cargo_package_id, manifest_path, name)?;
                let profile = value.as_table().ok_or_else(|| {
                    invalid(
                        cargo_package_id,
                        manifest_path,
                        format!("sql.profiles.{name}"),
                        "expected a table",
                    )
                })?;
                parse_profile(cargo_package_id, manifest_path, name, profile)
                    .map(|profile| (name.clone(), profile))
            })
            .collect::<Result<BTreeMap<_, _>, PackageDiagnostic>>()
    })
    .transpose()?
    .unwrap_or_default();
    let requirements = parse_requirements(cargo_package_id, manifest_path, table)?;
    if profiles.is_empty() && requirements.is_empty() {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            "sql",
            "expected at least one named profile or schema requirement",
        ));
    }
    Ok(SqlConfig {
        profiles,
        requirements,
    })
}

fn validate_provider_family(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    prefix: &str,
    family: &str,
) -> Result<(), PackageDiagnostic> {
    if !family.is_empty()
        && family.len() <= 64
        && family
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_lowercase())
    {
        return Ok(());
    }
    Err(invalid(
        cargo_package_id,
        manifest_path,
        format!("{prefix}.providers.{family}"),
        "provider family must contain lowercase letters or underscores",
    ))
}

fn string_list(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
    key: &str,
) -> Result<Vec<String>, PackageDiagnostic> {
    let Some(value) = table.get(key) else {
        return Ok(Vec::new());
    };
    let Some(values) = value.as_array() else {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.{key}"),
            "expected an array of strings",
        ));
    };
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or_else(|| {
                    invalid(
                        cargo_package_id,
                        manifest_path,
                        format!("{prefix}.{key}"),
                        "expected every entry to be a non-empty string",
                    )
                })
        })
        .collect()
}

fn string_set(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
    key: &str,
) -> Result<BTreeSet<String>, PackageDiagnostic> {
    Ok(
        string_list(cargo_package_id, manifest_path, table, prefix, key)?
            .into_iter()
            .collect(),
    )
}

fn valid_sql_mode(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn optional_table<'a>(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &'a toml::Table,
    dotted_key: &str,
    key: &str,
) -> Result<Option<&'a toml::Table>, PackageDiagnostic> {
    table
        .get(key)
        .map(|value| {
            value.as_table().ok_or_else(|| {
                invalid(
                    cargo_package_id,
                    manifest_path,
                    dotted_key,
                    "expected a table",
                )
            })
        })
        .transpose()
}

fn required_string(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
    key: &str,
) -> Result<String, PackageDiagnostic> {
    table
        .get(key)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.{key}"),
                "expected a non-empty string",
            )
        })
}

fn optional_string(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
    key: &str,
) -> Result<Option<String>, PackageDiagnostic> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };
    value
        .as_str()
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .map(Some)
        .ok_or_else(|| {
            invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.{key}"),
                "expected a non-empty string",
            )
        })
}

fn reject_unknown(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    prefix: &str,
    table: &toml::Table,
    known: &[&str],
) -> Result<(), PackageDiagnostic> {
    if let Some(field) = table.keys().find(|field| !known.contains(&field.as_str())) {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.{field}"),
            "unsupported field; connection URLs, credentials, and environment lookups are not compile-time profile inputs",
        ));
    }
    Ok(())
}

fn validate_profile_name(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    name: &str,
) -> Result<(), PackageDiagnostic> {
    let mut chars = name.chars();
    if chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && chars.all(|character| character == '_' || character.is_alphanumeric())
    {
        Ok(())
    } else {
        Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("sql.profiles.{name}"),
            "profile names must be canonical Sifr identifiers",
        ))
    }
}

fn invalid(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    key: impl Into<String>,
    reason: impl Into<String>,
) -> PackageDiagnostic {
    PackageDiagnostic::invalid_sifr_manifest(
        cargo_package_id,
        manifest_path.to_path_buf(),
        key,
        reason,
    )
}
