use crate::cargo::metadata::CargoPackageId;
use crate::diag::PackageDiagnostic;
use crate::manifest::package_sections::{
    SifrDependency, SifrScript, parse_dependencies, parse_scripts,
};
use crate::manifest::production::{parse_source_config, validate_manifest_shape};
use crate::manifest::sifr_fields::{
    parse_python_config, parse_rust_interop_config, parse_trust, validate_compiler_requirement,
    validate_edition,
};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

mod load;

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
    pub python: Vec<String>,
    pub python_native: Vec<String>,
    pub rust_build_scripts: Vec<String>,
    pub rust_proc_macros: Vec<String>,
    pub native_links: Vec<String>,
    pub unsafe_rust_bridges: Vec<String>,
    pub build_env: Vec<String>,
    pub rust_no_panic: Vec<String>,
    pub rust_panic_abort: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PythonConfig {
    pub venv: Option<PathBuf>,
    pub pyproject: Option<PathBuf>,
    pub lock: Option<PathBuf>,
    pub interpreter: Option<PathBuf>,
    pub requires_imports: Vec<String>,
}

impl PythonConfig {
    #[must_use]
    pub fn selects_environment(&self) -> bool {
        self.venv.is_some()
            || self.interpreter.is_some()
            || self.pyproject.is_some()
            || self.lock.is_some()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RustInteropConfig {
    pub bridges: Vec<PathBuf>,
    pub direct_crate_bindings: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SifrManifest {
    pub package_name: SifrPackageName,
    pub edition: SifrEdition,
    pub compiler_requirement: CompilerRequirement,
    pub default_run: Option<String>,
    pub source_root: PackageSourceRoot,
    pub source_features: BTreeMap<String, String>,
    pub scripts: BTreeMap<String, SifrScript>,
    pub dependencies: BTreeMap<String, SifrDependency>,
    pub dev_dependencies: BTreeMap<String, SifrDependency>,
    pub trust: TrustPolicy,
    pub python: PythonConfig,
    pub rust: RustInteropConfig,
}

impl SifrManifest {
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
        let source_root = parse_source_config(cargo_package_id, manifest_path, source_table)?;
        validate_manifest_shape(cargo_package_id, manifest_path, &value)?;

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
        let trust = optional_table(cargo_package_id, manifest_path, &value, "trust")?
            .map(|trust| parse_trust(cargo_package_id, manifest_path, trust))
            .transpose()?
            .unwrap_or_default();
        let python = optional_python_table(cargo_package_id, manifest_path, &value, "python")?
            .map(|python| parse_python_config(cargo_package_id, manifest_path, python))
            .transpose()?
            .unwrap_or_default();
        let rust = optional_table(cargo_package_id, manifest_path, &value, "rust")?
            .map(|rust| parse_rust_interop_config(cargo_package_id, manifest_path, rust))
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            package_name,
            edition,
            compiler_requirement,
            default_run,
            source_root,
            source_features,
            scripts,
            dependencies,
            dev_dependencies,
            trust,
            python,
            rust,
        })
    }

    #[must_use]
    pub fn declares_rust_backend(&self) -> bool {
        !self.trust.native.is_empty()
            || !self.trust.build_scripts.is_empty()
            || !self.trust.proc_macros.is_empty()
            || !self.rust.bridges.is_empty()
            || self.rust.direct_crate_bindings
            || !self.trust.rust_build_scripts.is_empty()
            || !self.trust.rust_proc_macros.is_empty()
            || !self.trust.native_links.is_empty()
            || !self.trust.unsafe_rust_bridges.is_empty()
            || !self.trust.build_env.is_empty()
            || !self.trust.rust_no_panic.is_empty()
            || !self.trust.rust_panic_abort.is_empty()
    }
}

fn table<'a>(value: &'a toml::Table, key: &str) -> Option<&'a toml::Table> {
    value.get(key).and_then(toml::Value::as_table)
}

fn optional_table<'a>(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &'a toml::Table,
    key: &'static str,
) -> Result<Option<&'a toml::Table>, PackageDiagnostic> {
    let Some(entry) = value.get(key) else {
        return Ok(None);
    };
    entry.as_table().map(Some).ok_or_else(|| {
        PackageDiagnostic::invalid_sifr_manifest(
            cargo_package_id,
            manifest_path.to_path_buf(),
            key,
            "expected a table",
        )
    })
}

fn optional_python_table<'a>(
    cargo_package_id: &CargoPackageId,
    manifest_path: &Path,
    value: &'a toml::Table,
    key: &'static str,
) -> Result<Option<&'a toml::Table>, PackageDiagnostic> {
    let Some(entry) = value.get(key) else {
        return Ok(None);
    };
    entry.as_table().map(Some).ok_or_else(|| {
        PackageDiagnostic::python_environment_config(
            cargo_package_id,
            manifest_path,
            key,
            "expected a table",
        )
    })
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

pub(crate) fn validate_relative_path(
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
