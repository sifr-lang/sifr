use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::PathBuf;

const DISCOVERY_KEY: &str = "manifest";
const ALIASES_KEY: &str = "aliases";
const SUPPORTED_KEYS: &[&str] = &[DISCOVERY_KEY, ALIASES_KEY];
const MISPLACED_COMPILER_KEYS: &[&str] = &[
    "package",
    "source",
    "exports",
    "features",
    "trust",
    "sifr-version",
    "edition",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoSifrMetadata {
    pub manifest: PathBuf,
    pub aliases: BTreeMap<String, CargoSifrAliasMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoSifrAliasMetadata {
    pub dependency: String,
    pub import: String,
}

impl CargoSifrMetadata {
    pub fn from_cargo_metadata_value(
        cargo_package_id: &CargoPackageId,
        cargo_package_name: &str,
        metadata: &Value,
    ) -> Result<Option<Self>, PackageDiagnostic> {
        let Some(sifr) = metadata.get("sifr") else {
            return Ok(None);
        };
        let Some(table) = sifr.as_object() else {
            return Err(PackageDiagnostic::invalid_cargo_sifr_metadata(
                cargo_package_id,
                cargo_package_name,
                "`sifr` must be a table",
            ));
        };

        for key in table.keys() {
            if MISPLACED_COMPILER_KEYS.contains(&key.as_str()) {
                return Err(PackageDiagnostic::misplaced_sifr_metadata(
                    cargo_package_id,
                    cargo_package_name,
                    key,
                ));
            }
            if !SUPPORTED_KEYS.contains(&key.as_str()) {
                return Err(PackageDiagnostic::invalid_cargo_sifr_metadata(
                    cargo_package_id,
                    cargo_package_name,
                    format!("unsupported key `{key}`"),
                ));
            }
        }

        let manifest = table
            .get(DISCOVERY_KEY)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| {
                PackageDiagnostic::invalid_cargo_sifr_metadata(
                    cargo_package_id,
                    cargo_package_name,
                    "`manifest` must be a non-empty string",
                )
            })?;

        if manifest.is_absolute() || manifest.components().any(|part| part.as_os_str() == "..") {
            return Err(PackageDiagnostic::invalid_cargo_sifr_metadata(
                cargo_package_id,
                cargo_package_name,
                "`manifest` must be relative to the Cargo package root and must not escape it",
            ));
        }

        let aliases = table
            .get(ALIASES_KEY)
            .map(|value| parse_aliases(cargo_package_id, cargo_package_name, value))
            .transpose()?
            .unwrap_or_default();

        Ok(Some(Self { manifest, aliases }))
    }
}

fn parse_aliases(
    cargo_package_id: &CargoPackageId,
    cargo_package_name: &str,
    value: &Value,
) -> Result<BTreeMap<String, CargoSifrAliasMetadata>, PackageDiagnostic> {
    let Some(table) = value.as_object() else {
        return Err(PackageDiagnostic::invalid_cargo_sifr_metadata(
            cargo_package_id,
            cargo_package_name,
            "`aliases` must be a table",
        ));
    };

    table
        .iter()
        .map(|(alias, entry)| {
            let Some(alias_table) = entry.as_object() else {
                return Err(PackageDiagnostic::invalid_cargo_sifr_metadata(
                    cargo_package_id,
                    cargo_package_name,
                    format!("alias `{alias}` must be a table"),
                ));
            };
            let dependency = alias_table
                .get("dependency")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    PackageDiagnostic::invalid_cargo_sifr_metadata(
                        cargo_package_id,
                        cargo_package_name,
                        format!("alias `{alias}` requires non-empty `dependency`"),
                    )
                })?;
            let import = alias_table
                .get("import")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    PackageDiagnostic::invalid_cargo_sifr_metadata(
                        cargo_package_id,
                        cargo_package_name,
                        format!("alias `{alias}` requires non-empty `import`"),
                    )
                })?;

            Ok((
                alias.clone(),
                CargoSifrAliasMetadata {
                    dependency: dependency.to_string(),
                    import: import.to_string(),
                },
            ))
        })
        .collect()
}
