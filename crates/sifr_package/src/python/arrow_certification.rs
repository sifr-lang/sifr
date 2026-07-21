use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::path::{Path, PathBuf};

pub const PYTHON_CERTIFICATIONS_FILE: &str = "sifr.python-certifications.json";
pub const ARROW_CERTIFICATION_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PythonCertificationArtifact {
    pub schema_version: u32,
    pub environment_digest: String,
    pub arrow: Vec<ArrowCertification>,
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
    let artifact = serde_json::from_slice::<PythonCertificationArtifact>(&bytes)
        .map_err(|error| format!("invalid '{}': {error}", path.display()))?;
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
    let mut artifact = serde_json::from_slice::<PythonCertificationArtifact>(&bytes)
        .map_err(|error| format!("invalid '{}': {error}", path.display()))?;
    artifact
        .arrow
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
    Ok(())
}

fn is_dotted_target(target: &str) -> bool {
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
            "Arrow certification fixture '{}' must stay inside the package",
            fixture
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
    entries
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn artifact_binds_environment_fixture_and_no_copy_evidence() {
        let root = temp_root("valid");
        fs::create_dir_all(root.join("fixtures")).expect("fixture directory");
        let fixture = root.join("fixtures/arrow.py");
        fs::write(&fixture, "print('evidence')\n").expect("fixture");
        let artifact = artifact(&fixture);

        let path = write_python_certifications(&root, &artifact).expect("artifact should write");
        assert_eq!(path, root.join(PYTHON_CERTIFICATIONS_FILE));
        assert_eq!(
            load_python_certifications(&root, "environment-a").expect("artifact should load"),
            artifact
        );
        assert_eq!(
            required_python_certification_archive_entries(&root),
            std::collections::BTreeSet::from([
                PathBuf::from(PYTHON_CERTIFICATIONS_FILE),
                PathBuf::from("fixtures/arrow.py"),
            ])
        );
        assert!(load_python_certifications(&root, "environment-b")
            .expect_err("environment mismatch must fail")
            .contains("environment digest"));

        fs::write(&fixture, "print('changed')\n").expect("fixture mutation");
        assert!(load_python_certifications(&root, "environment-a")
            .expect_err("stale fixture must fail")
            .contains("fixture digest is stale"));
        assert!(
            load_python_certifications_for_update(&root, "environment-a", "pkg.make_array")
                .expect("the target being recertified may have a stale fixture")
                .arrow
                .is_empty()
        );
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn artifact_rejects_uncertain_copy_and_escaping_fixture() {
        let root = temp_root("invalid");
        fs::create_dir_all(root.join("fixtures")).expect("fixture directory");
        let fixture = root.join("fixtures/arrow.py");
        fs::write(&fixture, "print('evidence')\n").expect("fixture");
        let mut artifact = artifact(&fixture);
        artifact.arrow[0].copy_performed = true;
        assert!(
            validate_python_certifications(&root, "environment-a", &artifact)
                .expect_err("copy evidence must fail")
                .contains("no-copy")
        );
        assert!(safe_fixture_path(&root, "../outside.py").is_err());
        fs::remove_dir_all(root).expect("cleanup");
    }

    fn artifact(fixture: &Path) -> PythonCertificationArtifact {
        PythonCertificationArtifact {
            schema_version: ARROW_CERTIFICATION_SCHEMA_VERSION,
            environment_digest: "environment-a".to_string(),
            arrow: vec![ArrowCertification {
                target: "pkg.make_array".to_string(),
                kind: ArrowCertifiedKind::Array,
                fixture: "fixtures/arrow.py".to_string(),
                fixture_digest: fixture_digest(fixture).expect("digest"),
                producer_module: "pyarrow.lib".to_string(),
                producer_type: "Int64Array".to_string(),
                distributions: vec![ArrowCertifiedDistribution {
                    name: "pyarrow".to_string(),
                    version: "22.0.0".to_string(),
                }],
                schema_mode: ArrowCertifiedSchemaMode::Omitted,
                identity_method: ArrowCertifiedIdentityMethod::BufferAddress,
                pointer_identity_verified: true,
                exact_release_count: 1,
                copy_performed: false,
            }],
        }
    }

    fn temp_root(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "sifr-arrow-certification-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
