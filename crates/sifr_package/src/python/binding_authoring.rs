use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

pub const PYTHON_BINDINGS_FILE: &str = "sifr.python-bindings.json";
pub const PYTHON_BINDING_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PythonBindingArtifact {
    pub schema_version: u32,
    pub environment_digest: String,
    pub bindings: Vec<PythonBinding>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PythonBinding {
    pub module: String,
    pub symbols: Vec<String>,
    pub output: String,
    pub soabi: String,
    pub distribution: Option<PythonBindingDistribution>,
    pub overrides: Vec<String>,
    pub stub_packages: Vec<String>,
    pub external_stubs: Vec<String>,
    pub sources: Vec<PythonBindingSource>,
    pub source_fingerprint: String,
    pub generated_digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PythonBindingDistribution {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PythonBindingSource {
    pub symbol: String,
    pub kind: PythonBindingSourceKind,
    pub identity: String,
    pub digest: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PythonBindingSourceKind {
    Override,
    StubPackage,
    PyTyped,
    ExternalStub,
    Introspection,
}

impl PythonBindingSourceKind {
    #[must_use]
    pub const fn precedence(self) -> u8 {
        match self {
            Self::Override => 0,
            Self::StubPackage => 1,
            Self::PyTyped => 2,
            Self::ExternalStub => 3,
            Self::Introspection => 4,
        }
    }
}

pub fn python_binding_source_fingerprint(
    module: &str,
    symbols: &[String],
    soabi: &str,
    distribution: Option<&PythonBindingDistribution>,
    sources: &[PythonBindingSource],
) -> String {
    let mut digest = Sha256::new();
    digest.update(b"sifr-python-binding-v1\0");
    digest.update(module.as_bytes());
    for symbol in symbols {
        digest.update([0]);
        digest.update(symbol.as_bytes());
    }
    digest.update([0]);
    digest.update(soabi.as_bytes());
    if let Some(distribution) = distribution {
        digest.update([0]);
        digest.update(distribution.name.as_bytes());
        digest.update([0]);
        digest.update(distribution.version.as_bytes());
    }
    for source in sources {
        digest.update([source.kind.precedence()]);
        digest.update(source.symbol.as_bytes());
        digest.update([0]);
        digest.update(source.identity.as_bytes());
        digest.update([0]);
        digest.update(source.digest.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

#[must_use]
pub fn python_binding_generated_digest(source: &[u8]) -> String {
    format!("{:x}", Sha256::digest(source))
}

pub fn load_python_bindings(
    package_root: &Path,
    environment_digest: &str,
) -> Result<PythonBindingArtifact, String> {
    let path = package_root.join(PYTHON_BINDINGS_FILE);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("could not read '{}': {error}", path.display()))?;
    let artifact = serde_json::from_slice::<PythonBindingArtifact>(&bytes)
        .map_err(|error| format!("invalid '{}': {error}", path.display()))?;
    super::binding_validation::validate_python_bindings(
        package_root,
        environment_digest,
        &artifact,
    )?;
    Ok(artifact)
}

/// Load an existing artifact for an authoring update, returning the binding
/// being replaced separately from the validated retained bindings.
pub fn load_python_bindings_for_update(
    package_root: &Path,
    module: &str,
) -> Result<(PythonBindingArtifact, Option<PythonBinding>), String> {
    let path = package_root.join(PYTHON_BINDINGS_FILE);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("could not read '{}': {error}", path.display()))?;
    let mut artifact = serde_json::from_slice::<PythonBindingArtifact>(&bytes)
        .map_err(|error| format!("invalid '{}': {error}", path.display()))?;
    if artifact.schema_version != PYTHON_BINDING_SCHEMA_VERSION
        || artifact.environment_digest.trim().is_empty()
    {
        return Err("invalid Python binding artifact header".to_string());
    }
    let replaced = artifact
        .bindings
        .iter()
        .position(|binding| binding.module == module)
        .map(|index| artifact.bindings.remove(index));
    if !artifact.bindings.is_empty() {
        super::binding_validation::validate_python_bindings(
            package_root,
            &artifact.environment_digest,
            &artifact,
        )?;
    }
    Ok((artifact, replaced))
}

pub fn write_python_bindings(
    package_root: &Path,
    artifact: &PythonBindingArtifact,
) -> Result<PathBuf, String> {
    super::binding_validation::validate_python_bindings(
        package_root,
        &artifact.environment_digest,
        artifact,
    )?;
    let path = package_root.join(PYTHON_BINDINGS_FILE);
    let mut bytes = serde_json::to_vec_pretty(artifact)
        .map_err(|error| format!("could not serialize Python bindings: {error}"))?;
    bytes.push(b'\n');
    std::fs::write(&path, bytes)
        .map_err(|error| format!("could not write '{}': {error}", path.display()))?;
    Ok(path)
}

pub fn required_python_binding_archive_entries(package_root: &Path) -> BTreeSet<PathBuf> {
    let path = package_root.join(PYTHON_BINDINGS_FILE);
    let Ok(bytes) = std::fs::read(path) else {
        return BTreeSet::new();
    };
    let Ok(artifact) = serde_json::from_slice::<PythonBindingArtifact>(&bytes) else {
        return BTreeSet::from([PathBuf::from(PYTHON_BINDINGS_FILE)]);
    };
    let mut entries = BTreeSet::from([PathBuf::from(PYTHON_BINDINGS_FILE)]);
    entries.extend(
        artifact
            .bindings
            .iter()
            .flat_map(|binding| {
                std::iter::once(binding.output.as_str())
                    .chain(binding.overrides.iter().map(String::as_str))
                    .chain(binding.external_stubs.iter().map(String::as_str))
            })
            .map(PathBuf::from)
            .filter(|path| is_safe_relative(path)),
    );
    entries
}

pub fn safe_python_binding_output(package_root: &Path, output: &Path) -> Result<PathBuf, String> {
    let output = output.to_string_lossy();
    safe_package_path(package_root, &output, "binding output")
}

pub(super) fn safe_package_path(
    package_root: &Path,
    value: &str,
    label: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(value);
    if !is_safe_relative(relative) {
        return Err(format!(
            "Python {label} '{value}' must stay inside the package"
        ));
    }
    Ok(package_root.join(relative))
}

pub(super) fn is_safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
