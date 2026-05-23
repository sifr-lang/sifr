use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::package_sections::{
    parse_dependencies, parse_scripts, SifrDependency, SifrScript,
};
use crate::manifest::production::{
    parse_source_config, reject_production_manifest_bins, reject_production_manifest_exports,
};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SifrPackageName(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SifrEdition(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CompilerRequirement(pub String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageSourceRoot(pub PathBuf);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportRoot(pub String);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustPolicy {
    pub native: Vec<String>,
    pub build_scripts: Vec<String>,
    pub proc_macros: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrManifest {
    pub package_name: SifrPackageName,
    pub edition: SifrEdition,
    pub compiler_requirement: CompilerRequirement,
    pub default_run: Option<String>,
    pub source_roots: Vec<PackageSourceRoot>,
    pub exports: Vec<ImportRoot>,
    pub source_features: BTreeMap<String, String>,
    pub scripts: BTreeMap<String, SifrScript>,
    pub dependencies: BTreeMap<String, SifrDependency>,
    pub dev_dependencies: BTreeMap<String, SifrDependency>,
    pub trust: TrustPolicy,
    pub production_schema: bool,
}

impl SifrManifest {
    pub fn load(
        cargo_package_id: &CargoPackageId,
        manifest_path: &Path,
    ) -> Result<Self, PackageDiagnostic> {
        let source = std::fs::read_to_string(manifest_path).map_err(|error| {
            PackageDiagnostic::missing_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                error.to_string(),
            )
        })?;
        Self::parse(cargo_package_id, manifest_path, &source)
    }

    pub fn parse(
        cargo_package_id: &CargoPackageId,
        manifest_path: &Path,
        source: &str,
    ) -> Result<Self, PackageDiagnostic> {
        let value = source.parse::<toml::Table>().map_err(|error| {
            PackageDiagnostic::missing_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                error.to_string(),
            )
        })?;

        let package = table(&value, "package").ok_or_else(|| {
            PackageDiagnostic::invalid_sifr_manifest(
                cargo_package_id,
                manifest_path.to_path_buf(),
                "package",
                "missing [package] table",
            )
        })?;
        let package_name =
            required_string(cargo_package_id, manifest_path, package, "package.name")
                .map(SifrPackageName)?;
        let edition = required_string(cargo_package_id, manifest_path, package, "package.edition")
            .map(SifrEdition)?;
        validate_edition(cargo_package_id, manifest_path, &edition)?;
        let compiler_requirement = required_string(
            cargo_package_id,
            manifest_path,
            package,
            "package.sifr-version",
        )
        .map(CompilerRequirement)?;
        validate_compiler_requirement(cargo_package_id, manifest_path, &compiler_requirement)?;
        let default_run = package
            .get("default-run")
            .and_then(toml::Value::as_str)
            .map(str::to_string);

        let source_table = table(&value, "source");
        let source_roots = parse_source_config(cargo_package_id, manifest_path, source_table)?;
        let production_schema = source_table
            .map(|source| !source.contains_key("roots"))
            .unwrap_or(true);
        if production_schema {
            reject_production_manifest_exports(cargo_package_id, manifest_path, &value)?;
            reject_production_manifest_bins(cargo_package_id, manifest_path, &value)?;
        }

        let exports = table(&value, "exports")
            .and_then(|exports| exports.get("modules"))
            .map(|modules| parse_exports(cargo_package_id, manifest_path, modules))
            .transpose()?
            .unwrap_or_else(|| vec![ImportRoot(package_name.0.clone())]);

        let source_features = table(&value, "features")
            .map(|features| {
                features
                    .iter()
                    .map(|(name, value)| (name.clone(), value.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let scripts = table(&value, "scripts")
            .map(|scripts| parse_scripts(cargo_package_id, manifest_path, scripts))
            .transpose()?
            .unwrap_or_default();
        let dependencies = table(&value, "dependencies")
            .map(|dependencies| parse_dependencies(cargo_package_id, manifest_path, dependencies))
            .transpose()?
            .unwrap_or_default();
        let dev_dependencies = table(&value, "dev-dependencies")
            .map(|dependencies| parse_dependencies(cargo_package_id, manifest_path, dependencies))
            .transpose()?
            .unwrap_or_default();
        let trust = table(&value, "trust")
            .map(|trust| parse_trust(cargo_package_id, manifest_path, trust))
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            package_name,
            edition,
            compiler_requirement,
            default_run,
            source_roots,
            exports,
            source_features,
            scripts,
            dependencies,
            dev_dependencies,
            trust,
            production_schema,
        })
    }

    #[must_use]
    pub fn declares_rust_backend(&self) -> bool {
        !self.trust.native.is_empty()
            || !self.trust.build_scripts.is_empty()
            || !self.trust.proc_macros.is_empty()
    }
}

fn table<'a>(value: &'a toml::Table, key: &str) -> Option<&'a toml::Table> {
    value.get(key).and_then(toml::Value::as_table)
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

pub(super) fn parse_source_roots(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Value,
) -> Result<Vec<PackageSourceRoot>, PackageDiagnostic> {
    let Some(entries) = value.as_array() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "source.roots",
            "expected a list of relative paths",
        ));
    };

    entries
        .iter()
        .map(|entry| {
            let Some(source_root) = entry.as_str() else {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    "source.roots",
                    "expected every source root to be a string",
                ));
            };
            validate_relative_path(cargo_package_id, manifest_path, "source.roots", source_root)
                .map(PackageSourceRoot)
        })
        .collect()
}

fn parse_exports(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &toml::Value,
) -> Result<Vec<ImportRoot>, PackageDiagnostic> {
    let Some(entries) = value.as_array() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "exports.modules",
            "expected a list of import roots",
        ));
    };

    entries
        .iter()
        .map(|entry| {
            let Some(export) = entry.as_str().filter(|value| !value.is_empty()) else {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    "exports.modules",
                    "expected every export to be a non-empty string",
                ));
            };
            if !export.split('.').all(valid_identifier) {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    "exports.modules",
                    format!("`{export}` is not a valid dotted import root"),
                ));
            }
            Ok(ImportRoot(export.to_string()))
        })
        .collect()
}

fn parse_trust(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<TrustPolicy, PackageDiagnostic> {
    Ok(TrustPolicy {
        native: optional_string_list(cargo_package_id, manifest_path, table, "trust.native")?,
        build_scripts: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.build-scripts",
        )?,
        proc_macros: optional_string_list(
            cargo_package_id,
            manifest_path,
            table,
            "trust.proc-macros",
        )?,
    })
}

fn optional_string_list(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    dotted_key: &'static str,
) -> Result<Vec<String>, PackageDiagnostic> {
    let local_key = dotted_key.rsplit('.').next().unwrap_or(dotted_key);
    let Some(value) = table.get(local_key) else {
        return Ok(Vec::new());
    };
    let Some(entries) = value.as_array() else {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            dotted_key,
            "expected a list of strings",
        ));
    };

    entries
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .ok_or_else(|| {
                    PackageDiagnostic::invalid_sifr_manifest(
                        cargo_package_id,
                        manifest_path.to_path_buf(),
                        dotted_key,
                        "expected every entry to be a non-empty string",
                    )
                })
        })
        .collect()
}

pub(super) fn validate_relative_path(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    key: &'static str,
    path: &str,
) -> Result<PathBuf, PackageDiagnostic> {
    if path.is_empty() {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            key,
            "path must not be empty",
        ));
    }
    let raw = Path::new(path);
    if raw.is_absolute() {
        return Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            key,
            "path must be relative",
        ));
    }

    let mut normalized = PathBuf::new();
    for component in raw.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(PackageDiagnostic::invalid_sifr_manifest(
                    cargo_package_id,
                    manifest_path.to_path_buf(),
                    key,
                    "path must not escape the package root",
                ));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        Ok(PathBuf::from("."))
    } else {
        Ok(normalized)
    }
}

fn validate_edition(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    edition: &SifrEdition,
) -> Result<(), PackageDiagnostic> {
    if edition.0 == "2026" {
        Ok(())
    } else {
        Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "package.edition",
            format!("unsupported Sifr edition `{}`", edition.0),
        ))
    }
}

fn validate_compiler_requirement(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    requirement: &CompilerRequirement,
) -> Result<(), PackageDiagnostic> {
    if requirement.0.contains("0.3") || requirement.0 == "*" {
        Ok(())
    } else {
        Err(PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            "package.sifr-version",
            format!(
                "compiler requirement `{}` does not match this milestone compiler compatibility window",
                requirement.0
            ),
        ))
    }
}

fn valid_identifier(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}
