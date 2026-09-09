//! Parse concrete schema sources, evidence, pooling and session contracts.

use super::{
    SchemaSourceKind, SqlProfileConfig, invalid, optional_string, optional_table, reject_unknown,
    required_string, string_list, string_set, valid_sql_mode, validate_provider_family,
};
use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::validate_relative_path;
use sifr_sql_contract::{PoolingMode, SchemaEvidence, SchemaStrictness, SessionContract};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) fn parse_profile(
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
            "family",
            "source",
            "source-kind",
            "server-version",
            "search-path",
            "extensions",
            "pooling",
            "schema-evidence",
            "schema-strictness",
            "sql-modes",
            "compile-flags",
            "required-features",
            "session",
            "accepted-signers",
        ],
    )?;
    let provider = required_string(cargo_package_id, manifest_path, table, &prefix, "provider")?;
    let family = required_string(cargo_package_id, manifest_path, table, &prefix, "family")?;
    validate_provider_family(cargo_package_id, manifest_path, &prefix, &family)?;
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
    let mut extensions = string_set(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "extensions",
    )?;
    extensions.extend(string_set(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "required-features",
    )?);
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
    let mut sql_modes = string_set(cargo_package_id, manifest_path, table, &prefix, "sql-modes")?;
    sql_modes.extend(string_set(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "compile-flags",
    )?);
    if sql_modes.iter().any(|mode| !valid_sql_mode(mode)) {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.sql-modes"),
            "expected short SQL mode identifiers containing only letters, digits, '_' or '-'",
        ));
    }
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
        family,
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
    sql_modes: BTreeSet<String>,
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
