use sifr_sql_runtime::{
    RuntimeLimits, SchemaDependencySlice, SchemaEvidenceMode, SchemaStrictness, SqlError,
    SqlErrorKind,
};
use std::collections::BTreeMap;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationProbe {
    pub property_identity: String,
    pub statement: String,
}

impl VerificationProbe {
    pub fn new(
        property_identity: impl Into<String>,
        statement: impl Into<String>,
    ) -> Result<Self, SqlError> {
        let probe = Self {
            property_identity: property_identity.into(),
            statement: statement.into(),
        };
        if probe.property_identity.is_empty()
            || probe.property_identity.len() > 512
            || probe.property_identity.chars().any(char::is_control)
            || !valid_observation_statement(&probe.statement)
        {
            return Err(configuration_error());
        }
        Ok(probe)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedSchemaManifest {
    pub signer: String,
    pub payload: Arc<[u8]>,
    pub signature: Arc<[u8]>,
}

pub trait ManifestVerifier: Send + Sync + 'static {
    fn verify(&self, manifest: &SignedSchemaManifest) -> Result<SchemaDependencySlice, SqlError>;
}

#[derive(Clone)]
pub enum SqliteEvidence {
    Introspection {
        fingerprint_statement: String,
        probes: Vec<VerificationProbe>,
    },
    MigrationHead {
        head_statement: String,
        accepted_states: BTreeMap<String, SchemaDependencySlice>,
    },
    SignedManifest {
        manifest: SignedSchemaManifest,
        verifier: Arc<dyn ManifestVerifier>,
    },
}

impl SqliteEvidence {
    #[must_use]
    pub const fn mode(&self) -> SchemaEvidenceMode {
        match self {
            Self::Introspection { .. } => SchemaEvidenceMode::Introspection,
            Self::MigrationHead { .. } => SchemaEvidenceMode::MigrationHead,
            Self::SignedManifest { .. } => SchemaEvidenceMode::SignedManifest,
        }
    }

    pub(crate) fn verify_manifest(
        verifier: &Arc<dyn ManifestVerifier>,
        manifest: &SignedSchemaManifest,
    ) -> Result<SchemaDependencySlice, SqlError> {
        catch_unwind(AssertUnwindSafe(|| verifier.verify(manifest)))
            .unwrap_or_else(|_| Err(configuration_error()))
    }
}

impl fmt::Debug for SqliteEvidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Introspection { probes, .. } => formatter
                .debug_struct("Introspection")
                .field("probe_count", &probes.len())
                .finish(),
            Self::MigrationHead {
                accepted_states, ..
            } => formatter
                .debug_struct("MigrationHead")
                .field("state_count", &accepted_states.len())
                .finish(),
            Self::SignedManifest { manifest, .. } => formatter
                .debug_struct("SignedManifest")
                .field("signer", &manifest.signer)
                .field("payload_len", &manifest.payload.len())
                .field("signature_len", &manifest.signature.len())
                .finish(),
        }
    }
}

#[derive(Clone)]
pub struct SqliteProfile {
    path: PathBuf,
    pub(crate) profile_fingerprint: String,
    pub(crate) expected_schema: SchemaDependencySlice,
    pub(crate) evidence: SqliteEvidence,
    pub(crate) strictness: SchemaStrictness,
    attached_files: BTreeMap<String, PathBuf>,
    required_features: Vec<String>,
    minimum_version: (u16, u16, u16),
    limits: RuntimeLimits,
    busy_timeout_ms: u32,
}

impl SqliteProfile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: impl Into<PathBuf>,
        profile_fingerprint: impl Into<String>,
        expected_schema: SchemaDependencySlice,
        evidence: SqliteEvidence,
        strictness: SchemaStrictness,
        attached_files: BTreeMap<String, PathBuf>,
        required_features: Vec<String>,
        minimum_version: (u16, u16, u16),
        limits: RuntimeLimits,
        busy_timeout_ms: u32,
    ) -> Result<Self, SqlError> {
        let path = path.into();
        let profile_fingerprint = profile_fingerprint.into();
        if path.as_os_str().is_empty()
            || !valid_fingerprint(&profile_fingerprint)
            || minimum_version != (3, 53, 2)
            || busy_timeout_ms == 0
            || required_features
                .iter()
                .any(|feature| !valid_feature(feature))
            || attached_files.iter().any(|(name, path)| {
                !valid_schema_name(name)
                    || matches!(name.as_str(), "main" | "temp")
                    || path.as_os_str().is_empty()
            })
        {
            return Err(configuration_error());
        }
        validate_evidence(&evidence)?;
        Ok(Self {
            path,
            profile_fingerprint,
            expected_schema,
            evidence,
            strictness,
            attached_files,
            required_features,
            minimum_version,
            limits: limits.validate()?,
            busy_timeout_ms,
        })
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn profile_fingerprint(&self) -> &str {
        &self.profile_fingerprint
    }

    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        self.expected_schema.fingerprint()
    }

    #[must_use]
    pub fn required_features(&self) -> &[String] {
        &self.required_features
    }

    #[must_use]
    pub fn attached_files(&self) -> &BTreeMap<String, PathBuf> {
        &self.attached_files
    }

    #[must_use]
    pub const fn minimum_version(&self) -> (u16, u16, u16) {
        self.minimum_version
    }

    #[must_use]
    pub const fn limits(&self) -> RuntimeLimits {
        self.limits
    }

    #[must_use]
    pub const fn busy_timeout_ms(&self) -> u32 {
        self.busy_timeout_ms
    }
}

impl fmt::Debug for SqliteProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SqliteProfile")
            .field("path", &"<redacted>")
            .field("profile_fingerprint", &self.profile_fingerprint)
            .field("schema_fingerprint", &self.expected_schema.fingerprint())
            .field("evidence", &self.evidence)
            .field("strictness", &self.strictness)
            .field(
                "attached_schemas",
                &self.attached_files.keys().collect::<Vec<_>>(),
            )
            .field("required_features", &self.required_features)
            .field("minimum_version", &self.minimum_version)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

fn validate_evidence(evidence: &SqliteEvidence) -> Result<(), SqlError> {
    match evidence {
        SqliteEvidence::Introspection {
            fingerprint_statement,
            probes,
        } => {
            if !valid_observation_statement(fingerprint_statement) || probes.is_empty() {
                return Err(configuration_error());
            }
        }
        SqliteEvidence::MigrationHead {
            head_statement,
            accepted_states,
        } => {
            if !valid_observation_statement(head_statement) || accepted_states.is_empty() {
                return Err(configuration_error());
            }
        }
        SqliteEvidence::SignedManifest { manifest, .. } => {
            if manifest.signer.is_empty()
                || manifest.signer.len() > 256
                || manifest.signer.chars().any(char::is_control)
                || manifest.payload.is_empty()
                || manifest.signature.is_empty()
            {
                return Err(configuration_error());
            }
        }
    }
    Ok(())
}

fn valid_observation_statement(statement: &str) -> bool {
    !statement.trim().is_empty()
        && statement.len() <= 64 * 1024
        && !statement.contains(';')
        && !statement.chars().any(char::is_control)
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn valid_feature(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_schema_name(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn configuration_error() -> SqlError {
    SqlError::new(SqlErrorKind::Configuration)
}
