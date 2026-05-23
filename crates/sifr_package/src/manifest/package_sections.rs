use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrScript {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SifrDependency {
    Version(String),
    Table(BTreeMap<String, String>),
}

pub fn parse_scripts(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<BTreeMap<String, SifrScript>, PackageDiagnostic> {
    table
        .iter()
        .map(|(name, value)| {
            let script = value.as_table().ok_or_else(|| {
                PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("scripts.{name}"),
                    "expected { command = \"...\", args = [...] }",
                )
            })?;
            let command =
                required_string(cargo_package_id, manifest_path, script, "scripts.command")?;
            let args = script
                .get("args")
                .map(|value| string_array(cargo_package_id, manifest_path, "scripts.args", value))
                .transpose()?
                .unwrap_or_default();
            Ok((name.clone(), SifrScript { command, args }))
        })
        .collect()
}

pub fn parse_dependencies(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<BTreeMap<String, SifrDependency>, PackageDiagnostic> {
    table
        .iter()
        .map(|(name, value)| {
            let dependency = if let Some(version) = value.as_str() {
                SifrDependency::Version(version.to_string())
            } else if let Some(table) = value.as_table() {
                SifrDependency::Table(
                    table
                        .iter()
                        .map(|(key, value)| (key.clone(), value.to_string()))
                        .collect(),
                )
            } else {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    format!("dependencies.{name}"),
                    "expected a version string or dependency table",
                ));
            };
            Ok((name.clone(), dependency))
        })
        .collect()
}

fn required_string(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    dotted_key: &'static str,
) -> Result<String, PackageDiagnostic> {
    let local_key = dotted_key.rsplit('.').next().unwrap_or(dotted_key);
    table
        .get(local_key)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                dotted_key,
                "expected a non-empty string",
            )
        })
}

fn string_array(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    key: impl Into<String>,
    value: &toml::Value,
) -> Result<Vec<String>, PackageDiagnostic> {
    let key = key.into();
    let Some(items) = value.as_array() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            key,
            "expected an array of strings",
        ));
    };
    items
        .iter()
        .map(|item| {
            item.as_str().map(str::to_string).ok_or_else(|| {
                PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    key.clone(),
                    "expected every entry to be a string",
                )
            })
        })
        .collect()
}
