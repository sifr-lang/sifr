use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::validate_relative_path;
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SqlConfig {
    pub profiles: BTreeMap<String, SqlProfileConfig>,
}

pub(super) fn parse_sql_config(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: Option<&toml::Table>,
) -> Result<SqlConfig, PackageDiagnostic> {
    let Some(table) = table else {
        return Ok(SqlConfig::default());
    };
    reject_unknown(cargo_package_id, manifest_path, "sql", table, &["profiles"])?;
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
    if profiles.is_empty() {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            "sql.profiles",
            "expected at least one named profile",
        ));
    }
    Ok(SqlConfig { profiles })
}

fn parse_profile(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    name: &str,
    table: &toml::Table,
) -> Result<SqlProfileConfig, PackageDiagnostic> {
    let prefix = format!("sql.profiles.{name}");
    reject_unknown(
        cargo_package_id,
        manifest_path,
        &prefix,
        table,
        &[
            "provider",
            "source",
            "source-kind",
            "server-version",
            "search-path",
            "extensions",
            "pooling",
            "schema-evidence",
            "schema-strictness",
            "sql-modes",
            "session",
            "accepted-signers",
        ],
    )?;
    let provider = required_string(cargo_package_id, manifest_path, table, &prefix, "provider")?;
    let sources = parse_sources(cargo_package_id, manifest_path, table, &prefix)?;
    let source_kind_value = optional_string(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "source-kind",
    )?;
    let source_kind = match source_kind_value.as_deref().unwrap_or("sql-ddl") {
        "sql-ddl" => SchemaSourceKind::SqlDdl,
        "provider-metadata" => SchemaSourceKind::ProviderMetadata,
        "generated-definitions" => SchemaSourceKind::GeneratedDefinitions,
        _ => {
            return Err(invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.source-kind"),
                "expected 'sql-ddl', 'provider-metadata', or 'generated-definitions'",
            ));
        }
    };
    let server_version = required_string(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "server-version",
    )?;
    let extensions = string_set(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "extensions",
    )?;
    let search_path = string_list(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "search-path",
    )?;
    let pooling = match required_string(cargo_package_id, manifest_path, table, &prefix, "pooling")?
        .as_str()
    {
        "session" => PoolingMode::Session,
        "transaction" => PoolingMode::Transaction,
        _ => {
            return Err(invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.pooling"),
                "expected 'session' or 'transaction'",
            ));
        }
    };
    let evidence = match required_string(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "schema-evidence",
    )?
    .as_str()
    {
        "introspection" => SchemaEvidence::Introspection,
        "migration-head" => SchemaEvidence::MigrationHead,
        "signed-manifest" => SchemaEvidence::SignedManifest,
        _ => {
            return Err(invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.schema-evidence"),
                "expected 'introspection', 'migration-head', or 'signed-manifest'",
            ));
        }
    };
    let strictness = match required_string(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "schema-strictness",
    )?
    .as_str()
    {
        "exact" => SchemaStrictness::Exact,
        "compatible" => SchemaStrictness::Compatible,
        _ => {
            return Err(invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.schema-strictness"),
                "expected 'exact' or 'compatible'",
            ));
        }
    };
    let sql_modes = optional_table(
        cargo_package_id,
        manifest_path,
        table,
        &format!("{prefix}.sql-modes"),
        "sql-modes",
    )?
    .map(|modes| {
        string_map(
            cargo_package_id,
            manifest_path,
            &format!("{prefix}.sql-modes"),
            modes,
        )
    })
    .transpose()?
    .unwrap_or_default();
    let session_table = optional_table(
        cargo_package_id,
        manifest_path,
        table,
        &format!("{prefix}.session"),
        "session",
    )?;
    let session = parse_session(
        cargo_package_id,
        manifest_path,
        &prefix,
        search_path,
        sql_modes,
        session_table,
    )?;
    let accepted_signers = string_set(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "accepted-signers",
    )?;
    if evidence == SchemaEvidence::SignedManifest && accepted_signers.is_empty() {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.accepted-signers"),
            "signed-manifest evidence requires at least one signer identity",
        ));
    }
    if pooling == PoolingMode::Transaction && session.role.is_some() {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.session.role"),
            "transaction pooling cannot carry a persistent role",
        ));
    }
    Ok(SqlProfileConfig {
        provider,
        sources,
        source_kind,
        server_version,
        extensions,
        evidence,
        strictness,
        pooling,
        session,
        accepted_signers,
    })
}

fn parse_sources(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
) -> Result<Vec<PathBuf>, PackageDiagnostic> {
    let Some(value) = table.get("source") else {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.source"),
            "expected a checked-in relative path or a non-empty list of paths",
        ));
    };
    let values = if let Some(value) = value.as_str() {
        vec![value]
    } else if let Some(values) = value.as_array() {
        values
            .iter()
            .map(|value| {
                value.as_str().ok_or_else(|| {
                    invalid(
                        cargo_package_id,
                        manifest_path,
                        format!("{prefix}.source"),
                        "expected every source entry to be a string",
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.source"),
            "expected a relative path or a list of relative paths",
        ));
    };
    if values.is_empty() {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.source"),
            "expected at least one schema source",
        ));
    }
    let mut paths = values
        .into_iter()
        .map(|value| {
            validate_relative_path(
                cargo_package_id,
                manifest_path,
                "sql.profiles.source",
                value,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn parse_session(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    prefix: &str,
    search_path: Vec<String>,
    sql_modes: BTreeMap<String, String>,
    table: Option<&toml::Table>,
) -> Result<SessionContract, PackageDiagnostic> {
    let Some(table) = table else {
        return Ok(SessionContract {
            search_path,
            sql_modes,
            ..SessionContract::default()
        });
    };
    let session_prefix = format!("{prefix}.session");
    reject_unknown(
        cargo_package_id,
        manifest_path,
        &session_prefix,
        table,
        &[
            "collation",
            "character-set",
            "time-zone",
            "role",
            "isolation",
        ],
    )?;
    Ok(SessionContract {
        search_path,
        sql_modes,
        collation: optional_string(
            cargo_package_id,
            manifest_path,
            table,
            &session_prefix,
            "collation",
        )?,
        character_set: optional_string(
            cargo_package_id,
            manifest_path,
            table,
            &session_prefix,
            "character-set",
        )?,
        time_zone: optional_string(
            cargo_package_id,
            manifest_path,
            table,
            &session_prefix,
            "time-zone",
        )?,
        role: optional_string(
            cargo_package_id,
            manifest_path,
            table,
            &session_prefix,
            "role",
        )?,
        isolation: optional_string(
            cargo_package_id,
            manifest_path,
            table,
            &session_prefix,
            "isolation",
        )?,
    })
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

fn string_map(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    key: &str,
    table: &toml::Table,
) -> Result<BTreeMap<String, String>, PackageDiagnostic> {
    table
        .iter()
        .map(|(name, value)| {
            value
                .as_str()
                .filter(|value| !value.is_empty())
                .map(|value| (name.clone(), value.to_string()))
                .ok_or_else(|| {
                    invalid(
                        cargo_package_id,
                        manifest_path,
                        format!("{key}.{name}"),
                        "expected a non-empty string",
                    )
                })
        })
        .collect()
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
