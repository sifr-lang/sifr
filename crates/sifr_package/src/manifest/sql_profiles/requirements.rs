//! Parse portable schema requirements and exact provider artifact declarations.

use super::{
    SqlRequirementConfig, SqlRequirementProviderConfig, invalid, optional_string, optional_table,
    reject_unknown, required_string, string_set, valid_sql_mode, validate_profile_name,
    validate_provider_family,
};
use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::validate_relative_path;
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn parse_requirements(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    sql: &toml::Table,
) -> Result<BTreeMap<String, SqlRequirementConfig>, PackageDiagnostic> {
    optional_table(
        cargo_package_id,
        manifest_path,
        sql,
        "sql.requirements",
        "requirements",
    )?
    .map(|requirements| {
        requirements
            .iter()
            .map(|(name, value)| {
                validate_profile_name(cargo_package_id, manifest_path, name)?;
                let table = value.as_table().ok_or_else(|| {
                    invalid(
                        cargo_package_id,
                        manifest_path,
                        format!("sql.requirements.{name}"),
                        "expected a table",
                    )
                })?;
                parse_requirement(cargo_package_id, manifest_path, name, table)
                    .map(|requirement| (name.clone(), requirement))
            })
            .collect()
    })
    .transpose()
    .map(Option::unwrap_or_default)
}

fn parse_requirement(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    name: &str,
    table: &toml::Table,
) -> Result<SqlRequirementConfig, PackageDiagnostic> {
    let prefix = format!("sql.requirements.{name}");
    reject_unknown(
        cargo_package_id,
        manifest_path,
        &prefix,
        table,
        &["capabilities", "providers"],
    )?;
    let capabilities = string_set(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "capabilities",
    )?;
    if capabilities.is_empty()
        || capabilities.iter().any(|capability| {
            !capability.starts_with("sql.")
                || capability.len() > 96
                || capability.bytes().any(|byte| {
                    !(byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'-'))
                })
        })
    {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.capabilities"),
            "expected a non-empty set of canonical 'sql.*' capabilities",
        ));
    }
    let providers = optional_table(
        cargo_package_id,
        manifest_path,
        table,
        &format!("{prefix}.providers"),
        "providers",
    )?
    .ok_or_else(|| {
        invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.providers"),
            "expected at least one provider artifact",
        )
    })?
    .iter()
    .map(|(family, value)| {
        validate_provider_family(cargo_package_id, manifest_path, &prefix, family)?;
        let provider = value.as_table().ok_or_else(|| {
            invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.providers.{family}"),
                "expected a table",
            )
        })?;
        parse_requirement_provider(cargo_package_id, manifest_path, &prefix, family, provider)
            .map(|provider| (family.clone(), provider))
    })
    .collect::<Result<BTreeMap<_, _>, _>>()?;
    if providers.is_empty() {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.providers"),
            "expected at least one provider artifact",
        ));
    }
    Ok(SqlRequirementConfig {
        capabilities,
        providers,
    })
}

fn parse_requirement_provider(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    requirement_prefix: &str,
    family: &str,
    table: &toml::Table,
) -> Result<SqlRequirementProviderConfig, PackageDiagnostic> {
    let prefix = format!("{requirement_prefix}.providers.{family}");
    reject_unknown(
        cargo_package_id,
        manifest_path,
        &prefix,
        table,
        &[
            "provider",
            "source",
            "server-version",
            "extensions",
            "sql-modes",
            "compile-flags",
            "required-features",
            "collation",
            "character-set",
        ],
    )?;
    let provider = required_string(cargo_package_id, manifest_path, table, &prefix, "provider")?;
    let source = required_string(cargo_package_id, manifest_path, table, &prefix, "source")?;
    let source = validate_relative_path(
        cargo_package_id,
        manifest_path,
        &format!("{prefix}.source"),
        &source,
    )?;
    let server_version = required_string(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "server-version",
    )?;
    if server_version.split('.').any(|part| {
        part.is_empty() || part.len() > 8 || !part.bytes().all(|byte| byte.is_ascii_digit())
    }) {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.server-version"),
            "expected a numeric dotted server version",
        ));
    }
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
            "expected canonical SQL mode identifiers",
        ));
    }
    let collation = optional_string(cargo_package_id, manifest_path, table, &prefix, "collation")?;
    let character_set = optional_string(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "character-set",
    )?;
    if family == "mysql" && (collation.is_none() || character_set.is_none()) {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            prefix.clone(),
            "MySQL requirement providers need exact collation and character-set settings",
        ));
    }
    Ok(SqlRequirementProviderConfig {
        provider,
        source,
        server_version,
        extensions,
        sql_modes,
        collation,
        character_set,
    })
}
