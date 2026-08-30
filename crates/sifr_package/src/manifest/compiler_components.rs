use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::sifr::validate_relative_path;
use semver::Version;
use sifr_compiler_component::{
    ComponentIdentity, ComponentRegistration, DiagnosticCodeDeclaration, DiagnosticLifecycle,
    DiagnosticRegistry, DiagnosticRegistryOwner, ProtocolRange,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompilerComponentConfig {
    pub kind: String,
    pub artifact: PathBuf,
    pub version: Version,
    pub sha256: String,
    pub protocol: ProtocolRange,
    pub processors: Vec<String>,
    pub diagnostics: DiagnosticRegistry,
}

impl CompilerComponentConfig {
    #[must_use]
    pub fn registrations(&self, package: &str) -> Vec<ComponentRegistration> {
        self.processors
            .iter()
            .map(|processor| ComponentRegistration {
                identity: ComponentIdentity {
                    package: package.to_string(),
                    processor: processor.clone(),
                    version: self.version.clone(),
                    sha256: self.sha256.clone(),
                },
                protocol: self.protocol,
                artifact: self.artifact.to_string_lossy().into_owned(),
                diagnostics: self.diagnostics.clone(),
            })
            .collect()
    }
}

pub(super) fn parse_compiler_components(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
) -> Result<BTreeMap<String, CompilerComponentConfig>, PackageDiagnostic> {
    table
        .iter()
        .map(|(name, value)| {
            let component = value.as_table().ok_or_else(|| {
                invalid(
                    cargo_package_id,
                    manifest_path,
                    format!("compiler-components.{name}"),
                    "expected a table",
                )
            })?;
            let config = parse_component(cargo_package_id, manifest_path, name, component)?;
            Ok((name.clone(), config))
        })
        .collect()
}

fn parse_component(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    name: &str,
    table: &toml::Table,
) -> Result<CompilerComponentConfig, PackageDiagnostic> {
    let prefix = format!("compiler-components.{name}");
    let kind = string_field(cargo_package_id, manifest_path, table, &prefix, "kind")?;
    if kind != "embedded-language-provider" {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.kind"),
            "expected 'embedded-language-provider'",
        ));
    }
    let artifact_text = string_field(cargo_package_id, manifest_path, table, &prefix, "artifact")?;
    let artifact = validate_relative_path(
        cargo_package_id,
        manifest_path,
        "compiler-components.artifact",
        &artifact_text,
    )?;
    let version_text = string_field(cargo_package_id, manifest_path, table, &prefix, "version")?;
    let version = Version::parse(&version_text).map_err(|error| {
        invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.version"),
            format!("expected an exact semantic version: {error}"),
        )
    })?;
    let sha256 = string_field(cargo_package_id, manifest_path, table, &prefix, "sha256")?;
    if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.sha256"),
            "expected a SHA-256 hex value",
        ));
    }
    let minimum = integer_field(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "protocol-min",
    )?;
    let maximum = integer_field(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "protocol-max",
    )?;
    let protocol = ProtocolRange { minimum, maximum };
    protocol.validate().map_err(|error| {
        invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.protocol-min"),
            error.to_string(),
        )
    })?;
    let processors = string_list(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "processors",
    )?;
    if processors.is_empty() {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.processors"),
            "expected at least one processor identity",
        ));
    }
    let diagnostic_namespace = string_field(
        cargo_package_id,
        manifest_path,
        table,
        &prefix,
        "diagnostic-namespace",
    )?;
    let diagnostics = DiagnosticRegistry {
        owner: DiagnosticRegistryOwner::Provider {
            namespace: diagnostic_namespace,
        },
        declarations: diagnostic_declarations(cargo_package_id, manifest_path, table, &prefix)?,
    };
    DiagnosticRegistry::compiler()
        .validate_with(std::slice::from_ref(&diagnostics))
        .map_err(|error| {
            invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.diagnostics"),
                error.to_string(),
            )
        })?;
    let known = [
        "kind",
        "artifact",
        "version",
        "sha256",
        "protocol-min",
        "protocol-max",
        "processors",
        "diagnostic-namespace",
        "diagnostics",
    ];
    if let Some(unknown) = table.keys().find(|field| !known.contains(&field.as_str())) {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.{unknown}"),
            "unsupported compiler component field",
        ));
    }
    Ok(CompilerComponentConfig {
        kind,
        artifact,
        version,
        sha256: sha256.to_ascii_lowercase(),
        protocol,
        processors,
        diagnostics,
    })
}

fn diagnostic_declarations(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
) -> Result<Vec<DiagnosticCodeDeclaration>, PackageDiagnostic> {
    let Some(items) = table.get("diagnostics").and_then(toml::Value::as_array) else {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.diagnostics"),
            "expected a list of diagnostic declarations",
        ));
    };
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let key = format!("{prefix}.diagnostics[{index}]");
            let declaration = item.as_table().ok_or_else(|| {
                invalid(
                    cargo_package_id,
                    manifest_path,
                    &key,
                    "expected a table with code and lifecycle",
                )
            })?;
            if declaration.len() != 2
                || !declaration.contains_key("code")
                || !declaration.contains_key("lifecycle")
            {
                return Err(invalid(
                    cargo_package_id,
                    manifest_path,
                    &key,
                    "expected only code and lifecycle fields",
                ));
            }
            let code = string_field(cargo_package_id, manifest_path, declaration, &key, "code")?;
            let lifecycle = match string_field(
                cargo_package_id,
                manifest_path,
                declaration,
                &key,
                "lifecycle",
            )?
            .as_str()
            {
                "active" => DiagnosticLifecycle::Active,
                "deprecated" => DiagnosticLifecycle::Deprecated,
                _ => {
                    return Err(invalid(
                        cargo_package_id,
                        manifest_path,
                        format!("{key}.lifecycle"),
                        "expected 'active' or 'deprecated'",
                    ));
                }
            };
            Ok(DiagnosticCodeDeclaration { code, lifecycle })
        })
        .collect()
}

fn string_field(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
    field: &str,
) -> Result<String, PackageDiagnostic> {
    table
        .get(field)
        .and_then(toml::Value::as_str)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| {
            invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.{field}"),
                "expected a non-empty string",
            )
        })
}

fn integer_field(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
    field: &str,
) -> Result<u16, PackageDiagnostic> {
    table
        .get(field)
        .and_then(toml::Value::as_integer)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or_else(|| {
            invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.{field}"),
                "expected a positive 16-bit integer",
            )
        })
}

fn string_list(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    table: &toml::Table,
    prefix: &str,
    field: &str,
) -> Result<Vec<String>, PackageDiagnostic> {
    let Some(items) = table.get(field).and_then(toml::Value::as_array) else {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.{field}"),
            "expected a list of strings",
        ));
    };
    let values = items
        .iter()
        .map(|item| {
            item.as_str()
                .filter(|value| !value.is_empty())
                .map(str::to_string)
        })
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| {
            invalid(
                cargo_package_id,
                manifest_path,
                format!("{prefix}.{field}"),
                "expected non-empty processor identities",
            )
        })?;
    let mut canonical = values.clone();
    canonical.sort();
    canonical.dedup();
    if canonical != values {
        return Err(invalid(
            cargo_package_id,
            manifest_path,
            format!("{prefix}.{field}"),
            "processor identities must be unique and sorted",
        ));
    }
    Ok(values)
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
