use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::path::{Path, PathBuf};

pub const PYTHON_CERTIFICATIONS_FILE: &str = "sifr.python-certifications.json";
pub const PYTHON_CERTIFICATION_SCHEMA_VERSION: u32 = 3;
pub const ARROW_CERTIFICATION_SCHEMA_VERSION: u32 = PYTHON_CERTIFICATION_SCHEMA_VERSION;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PythonCertificationArtifact {
    pub schema_version: u32,
    pub environment_digest: String,
    pub arrow: Vec<ArrowCertification>,
    pub dlpack: Vec<DlpackCertification>,
}

#[derive(Deserialize)]
struct PythonCertificationHeader {
    schema_version: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArrowCertification {
    pub target: String,
    pub kind: ArrowCertifiedKind,
    pub fixture: String,
    pub fixture_digest: String,
    pub producer_module: String,
    pub producer_type: String,
    pub distributions: Vec<ArrowCertifiedDistribution>,
    pub schema_mode: ArrowCertifiedSchemaMode,
    pub identity_method: ArrowCertifiedIdentityMethod,
    pub pointer_identity_verified: bool,
    pub exact_release_count: u64,
    pub copy_performed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DlpackCertification {
    pub target: String,
    pub fixture: String,
    pub fixture_digest: String,
    pub producer_module: String,
    pub producer_type: String,
    pub distributions: Vec<ArrowCertifiedDistribution>,
    pub device: DlpackCertifiedDevice,
    pub stream_policy: DlpackCertifiedStreamPolicy,
    pub pointer_identity_verified: bool,
    pub exact_deleter_count: u64,
    pub copy_performed: bool,
    pub within_run_assertions: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DlpackCertifiedDevice {
    Cpu,
    Cuda,
    Any,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DlpackCertifiedStreamPolicy {
    None,
    Parameter,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArrowCertifiedKind {
    Array,
    Schema,
    Stream,
    DeviceArray,
    DeviceStream,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct ArrowCertifiedDistribution {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArrowCertifiedSchemaMode {
    Omitted,
    Parameter,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArrowCertifiedIdentityMethod {
    BufferAddress,
    SchemaFormat,
}

pub fn load_python_certifications(
    package_root: &Path,
    environment_digest: &str,
) -> Result<PythonCertificationArtifact, String> {
    let path = package_root.join(PYTHON_CERTIFICATIONS_FILE);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("could not read '{}': {error}", path.display()))?;
    let artifact = parse_python_certification_artifact(&path, &bytes)?;
    validate_python_certifications(package_root, environment_digest, &artifact)?;
    Ok(artifact)
}

pub fn load_python_certifications_for_update(
    package_root: &Path,
    environment_digest: &str,
    replaced_target: &str,
) -> Result<PythonCertificationArtifact, String> {
    let path = package_root.join(PYTHON_CERTIFICATIONS_FILE);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("could not read '{}': {error}", path.display()))?;
    let mut artifact = parse_python_certification_artifact(&path, &bytes)?;
    artifact
        .arrow
        .retain(|certification| certification.target != replaced_target);
    validate_python_certifications(package_root, environment_digest, &artifact)?;
    Ok(artifact)
}

pub fn load_python_certifications_for_dlpack_update(
    package_root: &Path,
    environment_digest: &str,
    replaced_target: &str,
) -> Result<PythonCertificationArtifact, String> {
    let path = package_root.join(PYTHON_CERTIFICATIONS_FILE);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("could not read '{}': {error}", path.display()))?;
    let mut artifact = parse_python_certification_artifact(&path, &bytes)?;
    artifact
        .dlpack
        .retain(|certification| certification.target != replaced_target);
    validate_python_certifications(package_root, environment_digest, &artifact)?;
    Ok(artifact)
}

pub fn write_python_certifications(
    package_root: &Path,
    artifact: &PythonCertificationArtifact,
) -> Result<PathBuf, String> {
    validate_python_certifications(package_root, &artifact.environment_digest, artifact)?;
    let path = package_root.join(PYTHON_CERTIFICATIONS_FILE);
    let mut bytes = serde_json::to_vec_pretty(artifact)
        .map_err(|error| format!("could not serialize Arrow certifications: {error}"))?;
    bytes.push(b'\n');
    std::fs::write(&path, bytes)
        .map_err(|error| format!("could not write '{}': {error}", path.display()))?;
    Ok(path)
}

fn parse_python_certification_artifact(
    path: &Path,
    bytes: &[u8],
) -> Result<PythonCertificationArtifact, String> {
    let header = serde_json::from_slice::<PythonCertificationHeader>(bytes)
        .map_err(|error| format!("invalid '{}': {error}", path.display()))?;
    if header.schema_version != PYTHON_CERTIFICATION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported Python certification schema version {}; expected {}",
            header.schema_version, PYTHON_CERTIFICATION_SCHEMA_VERSION
        ));
    }
    serde_json::from_slice::<PythonCertificationArtifact>(bytes)
        .map_err(|error| format!("invalid '{}': {error}", path.display()))
}

pub fn validate_python_certifications(
    package_root: &Path,
    environment_digest: &str,
    artifact: &PythonCertificationArtifact,
) -> Result<(), String> {
    if artifact.schema_version != ARROW_CERTIFICATION_SCHEMA_VERSION {
        return Err(format!(
            "unsupported Python certification schema version {}; expected {}",
            artifact.schema_version, ARROW_CERTIFICATION_SCHEMA_VERSION
        ));
    }
    if artifact.environment_digest != environment_digest {
        return Err(
            "Python certification environment digest does not match the selected environment"
                .to_string(),
        );
    }
    let mut targets = std::collections::BTreeSet::new();
    let mut previous_target: Option<&str> = None;
    for certification in &artifact.arrow {
        if !is_dotted_target(&certification.target)
            || certification.producer_module.trim().is_empty()
            || certification.producer_type.trim().is_empty()
        {
            return Err("Arrow certification identities must be valid and non-empty".to_string());
        }
        if previous_target.is_some_and(|previous| previous >= certification.target.as_str()) {
            return Err("Arrow certifications must be sorted by unique target".to_string());
        }
        previous_target = Some(&certification.target);
        if !targets.insert(certification.target.as_str()) {
            return Err(format!(
                "duplicate Arrow certification target '{}'",
                certification.target
            ));
        }
        if certification.copy_performed || !certification.pointer_identity_verified {
            return Err(format!(
                "Arrow certification '{}' does not prove a no-copy transfer",
                certification.target
            ));
        }
        if matches!(certification.kind, ArrowCertifiedKind::Schema)
            != matches!(
                certification.identity_method,
                ArrowCertifiedIdentityMethod::SchemaFormat
            )
        {
            return Err(format!(
                "Arrow certification '{}' identity method does not match its kind",
                certification.target
            ));
        }
        if certification.exact_release_count != 1 {
            return Err(format!(
                "Arrow certification '{}' must prove exactly one release",
                certification.target
            ));
        }
        if certification.distributions.is_empty()
            || certification
                .distributions
                .iter()
                .any(|distribution| distribution.name.is_empty() || distribution.version.is_empty())
        {
            return Err(format!(
                "Arrow certification '{}' must bind at least one exact distribution name/version",
                certification.target
            ));
        }
        if !certification
            .distributions
            .windows(2)
            .all(|pair| pair[0] < pair[1])
        {
            return Err(format!(
                "Arrow certification '{}' distributions must be sorted and unique",
                certification.target
            ));
        }
        let fixture = safe_fixture_path(package_root, &certification.fixture)?;
        let metadata = std::fs::symlink_metadata(&fixture).map_err(|error| {
            format!("could not inspect fixture '{}': {error}", fixture.display())
        })?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "Arrow certification fixture '{}' must be a regular package file",
                fixture.display()
            ));
        }
        let canonical_root = package_root.canonicalize().map_err(|error| {
            format!(
                "could not resolve package root '{}': {error}",
                package_root.display()
            )
        })?;
        let canonical_fixture = fixture.canonicalize().map_err(|error| {
            format!("could not resolve fixture '{}': {error}", fixture.display())
        })?;
        if !canonical_fixture.starts_with(&canonical_root) {
            return Err(format!(
                "Arrow certification fixture '{}' escapes the package",
                certification.fixture
            ));
        }
        let bytes = std::fs::read(&fixture)
            .map_err(|error| format!("could not read fixture '{}': {error}", fixture.display()))?;
        let digest = format!("{:x}", Sha256::digest(bytes));
        if digest != certification.fixture_digest {
            return Err(format!(
                "Arrow certification fixture digest is stale for '{}'",
                certification.target
            ));
        }
    }
    super::dlpack_certification::validate_dlpack_certifications(package_root, &artifact.dlpack)?;
    Ok(())
}

pub(super) fn validate_distributions(
    target: &str,
    distributions: &[ArrowCertifiedDistribution],
    protocol: &str,
) -> Result<(), String> {
    if distributions.is_empty()
        || distributions
            .iter()
            .any(|distribution| distribution.name.is_empty() || distribution.version.is_empty())
    {
        return Err(format!(
            "{protocol} certification '{target}' must bind at least one exact distribution name/version"
        ));
    }
    if !distributions.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(format!(
            "{protocol} certification '{target}' distributions must be sorted and unique"
        ));
    }
    Ok(())
}

pub(super) fn validate_fixture(
    package_root: &Path,
    target: &str,
    fixture_name: &str,
    expected_digest: &str,
    protocol: &str,
) -> Result<(), String> {
    let fixture = safe_fixture_path(package_root, fixture_name)?;
    let metadata = std::fs::symlink_metadata(&fixture)
        .map_err(|error| format!("could not inspect fixture '{}': {error}", fixture.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "{protocol} certification fixture '{}' must be a regular package file",
            fixture.display()
        ));
    }
    let canonical_root = package_root.canonicalize().map_err(|error| {
        format!(
            "could not resolve package root '{}': {error}",
            package_root.display()
        )
    })?;
    let canonical_fixture = fixture
        .canonicalize()
        .map_err(|error| format!("could not resolve fixture '{}': {error}", fixture.display()))?;
    if !canonical_fixture.starts_with(&canonical_root) {
        return Err(format!(
            "{protocol} certification fixture '{fixture_name}' escapes the package"
        ));
    }
    let bytes = std::fs::read(&fixture)
        .map_err(|error| format!("could not read fixture '{}': {error}", fixture.display()))?;
    let digest = format!("{:x}", Sha256::digest(bytes));
    if digest != expected_digest {
        return Err(format!(
            "{protocol} certification fixture digest is stale for '{target}'"
        ));
    }
    Ok(())
}

pub(super) fn is_dotted_target(target: &str) -> bool {
    target.split('.').all(|segment| {
        let mut chars = segment.chars();
        chars
            .next()
            .is_some_and(|first| first == '_' || first.is_alphabetic())
            && chars.all(|character| character == '_' || character.is_alphanumeric())
    })
}

pub fn fixture_digest(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|error| format!("could not read fixture '{}': {error}", path.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn safe_fixture_path(package_root: &Path, fixture: &str) -> Result<PathBuf, String> {
    let relative = Path::new(fixture);
    if relative.is_absolute()
        || relative
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "Arrow certification fixture '{fixture}' must stay inside the package"
        ));
    }
    Ok(package_root.join(relative))
}

pub fn required_python_certification_archive_entries(
    package_root: &Path,
) -> std::collections::BTreeSet<PathBuf> {
    let path = package_root.join(PYTHON_CERTIFICATIONS_FILE);
    let Ok(bytes) = std::fs::read(&path) else {
        return std::collections::BTreeSet::new();
    };
    let Ok(artifact) = serde_json::from_slice::<PythonCertificationArtifact>(&bytes) else {
        return std::collections::BTreeSet::from([PathBuf::from(PYTHON_CERTIFICATIONS_FILE)]);
    };
    let mut entries = std::collections::BTreeSet::from([PathBuf::from(PYTHON_CERTIFICATIONS_FILE)]);
    entries.extend(artifact.arrow.into_iter().filter_map(|certification| {
        let fixture = PathBuf::from(certification.fixture);
        (!fixture.is_absolute()
            && fixture
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_))))
        .then_some(fixture)
    }));
    entries.extend(artifact.dlpack.into_iter().filter_map(|certification| {
        let fixture = PathBuf::from(certification.fixture);
        (!fixture.is_absolute()
            && fixture
                .components()
                .all(|component| matches!(component, std::path::Component::Normal(_))))
        .then_some(fixture)
    }));
    entries
}
