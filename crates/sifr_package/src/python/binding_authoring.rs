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

pub fn validate_python_bindings(
    package_root: &Path,
    environment_digest: &str,
    artifact: &PythonBindingArtifact,
) -> Result<(), String> {
    if artifact.schema_version != PYTHON_BINDING_SCHEMA_VERSION {
        return Err(format!(
            "unsupported Python binding schema version {}; expected {}",
            artifact.schema_version, PYTHON_BINDING_SCHEMA_VERSION
        ));
    }
    if artifact.environment_digest != environment_digest {
        return Err(
            "Python binding environment digest does not match the selected environment".to_string(),
        );
    }
    if artifact.bindings.is_empty() {
        return Err("Python binding artifact must contain at least one binding".to_string());
    }
    let mut previous_module: Option<&str> = None;
    let mut outputs = BTreeSet::new();
    for binding in &artifact.bindings {
        validate_binding(package_root, binding)?;
        if previous_module.is_some_and(|previous| previous >= binding.module.as_str()) {
            return Err("Python bindings must be sorted by unique module".to_string());
        }
        previous_module = Some(&binding.module);
        if !outputs.insert(binding.output.as_str()) {
            return Err(format!(
                "duplicate Python binding output '{}'",
                binding.output
            ));
        }
    }
    Ok(())
}

fn validate_binding(package_root: &Path, binding: &PythonBinding) -> Result<(), String> {
    if !is_dotted_name(&binding.module) || binding.soabi.trim().is_empty() {
        return Err("Python binding module and SOABI must be valid and non-empty".to_string());
    }
    if binding.symbols.is_empty()
        || !binding.symbols.iter().all(|symbol| is_identifier(symbol))
        || !binding.symbols.windows(2).all(|pair| pair[0] < pair[1])
    {
        return Err(format!(
            "Python binding '{}' symbols must be sorted, unique identifiers",
            binding.module
        ));
    }
    let output = safe_package_path(package_root, &binding.output, "binding output")?;
    let bytes = std::fs::read(&output).map_err(|error| {
        format!(
            "could not read Python binding '{}': {error}",
            output.display()
        )
    })?;
    if python_binding_generated_digest(&bytes) != binding.generated_digest {
        return Err(format!(
            "generated Python binding '{}' has drifted",
            binding.output
        ));
    }
    if let Some(distribution) = &binding.distribution {
        if distribution.name.trim().is_empty() || distribution.version.trim().is_empty() {
            return Err(format!(
                "Python binding '{}' has an incomplete distribution identity",
                binding.module
            ));
        }
    }
    if binding.sources.len() != binding.symbols.len() {
        return Err(format!(
            "Python binding '{}' must record one source for every symbol",
            binding.module
        ));
    }
    if !binding
        .stub_packages
        .windows(2)
        .all(|pair| pair[0] < pair[1])
        || binding
            .stub_packages
            .iter()
            .any(|package| package.trim().is_empty())
    {
        return Err(format!(
            "Python binding '{}' stub packages must be sorted, unique names",
            binding.module
        ));
    }
    for configured in binding.overrides.iter().chain(&binding.external_stubs) {
        let path = safe_package_path(package_root, configured, "typing source")?;
        if !path.is_file() {
            return Err(format!(
                "Python binding typing source '{configured}' must be a package file"
            ));
        }
    }
    validate_configured_source_digests(package_root, binding)?;
    for (source, symbol) in binding.sources.iter().zip(&binding.symbols) {
        if source.symbol != *symbol
            || source.identity.trim().is_empty()
            || source.digest.len() != 64
            || !source.digest.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            return Err(format!(
                "Python binding '{}' has invalid source evidence for '{symbol}'",
                binding.module
            ));
        }
    }
    let fingerprint = python_binding_source_fingerprint(
        &binding.module,
        &binding.symbols,
        &binding.soabi,
        binding.distribution.as_ref(),
        &binding.sources,
    );
    if fingerprint != binding.source_fingerprint {
        return Err(format!(
            "Python binding '{}' source fingerprint is invalid",
            binding.module
        ));
    }
    Ok(())
}

fn validate_configured_source_digests(
    package_root: &Path,
    binding: &PythonBinding,
) -> Result<(), String> {
    for (kind, configured) in [
        (PythonBindingSourceKind::Override, &binding.overrides),
        (
            PythonBindingSourceKind::ExternalStub,
            &binding.external_stubs,
        ),
    ] {
        for (index, relative) in configured.iter().enumerate() {
            let prefix = format!("{}:{index}:", source_kind_prefix(kind));
            let consumed = binding
                .sources
                .iter()
                .any(|source| source.kind == kind && source.identity.starts_with(&prefix));
            if !consumed {
                continue;
            }
            let path = safe_package_path(package_root, relative, "typing source")?;
            let bytes = std::fs::read(&path).map_err(|error| {
                format!(
                    "could not read Python typing source '{}': {error}",
                    path.display()
                )
            })?;
            let digest = python_binding_generated_digest(&bytes);
            if !binding.sources.iter().any(|source| {
                source.kind == kind
                    && source.identity.starts_with(&prefix)
                    && source.digest == digest
            }) {
                return Err(format!(
                    "Python binding typing source '{relative}' has drifted"
                ));
            }
        }
    }
    Ok(())
}

const fn source_kind_prefix(kind: PythonBindingSourceKind) -> &'static str {
    match kind {
        PythonBindingSourceKind::Override => "override",
        PythonBindingSourceKind::ExternalStub => "external",
        PythonBindingSourceKind::StubPackage
        | PythonBindingSourceKind::PyTyped
        | PythonBindingSourceKind::Introspection => "",
    }
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
    validate_python_bindings(package_root, environment_digest, &artifact)?;
    Ok(artifact)
}

/// Load an existing artifact for an authoring update while preserving all
/// validated checked-in bindings. A newly generated declaration can add an
/// inferred import root, so the caller replaces the recorded environment
/// digest with the freshly resolved digest before writing.
pub fn load_python_bindings_for_update(
    package_root: &Path,
    module: &str,
) -> Result<PythonBindingArtifact, String> {
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
    artifact.bindings.retain(|binding| binding.module != module);
    if !artifact.bindings.is_empty() {
        validate_python_bindings(package_root, &artifact.environment_digest, &artifact)?;
    }
    Ok(artifact)
}

pub fn write_python_bindings(
    package_root: &Path,
    artifact: &PythonBindingArtifact,
) -> Result<PathBuf, String> {
    validate_python_bindings(package_root, &artifact.environment_digest, artifact)?;
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

fn safe_package_path(package_root: &Path, value: &str, label: &str) -> Result<PathBuf, String> {
    let relative = Path::new(value);
    if !is_safe_relative(relative) {
        return Err(format!(
            "Python {label} '{value}' must stay inside the package"
        ));
    }
    Ok(package_root.join(relative))
}

fn is_safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn is_dotted_name(value: &str) -> bool {
    !value.is_empty() && value.split('.').all(is_identifier)
}

fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|first| first == '_' || first.is_alphabetic())
        && chars.all(|character| character == '_' || character.is_alphanumeric())
}
