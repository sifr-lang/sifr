use crate::error::configuration_error;
use rustls::ClientConfig;
use rustls_platform_verifier::ConfigVerifierExt;
use sifr_sql_runtime::{
    RuntimeLimits, SchemaDependencySlice, SchemaEvidenceMode, SchemaStrictness, SessionContract,
    SqlError,
};
use std::collections::BTreeMap;
use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

#[derive(Clone)]
pub enum PostgresTls {
    Disabled,
    Rustls(Arc<ClientConfig>),
}

impl PostgresTls {
    pub fn platform() -> Result<Self, SqlError> {
        let config = ClientConfig::with_platform_verifier().map_err(|_| configuration_error())?;
        Ok(Self::Rustls(Arc::new(config)))
    }
}

impl fmt::Debug for PostgresTls {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Disabled => "PostgresTls::Disabled",
            Self::Rustls(_) => "PostgresTls::Rustls(<redacted-config>)",
        })
    }
}

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
            || probe.statement.trim().is_empty()
            || probe.statement.len() > 64 * 1024
            || probe.statement.contains(';')
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
pub enum PostgresEvidence {
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

impl PostgresEvidence {
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

impl fmt::Debug for PostgresEvidence {
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
pub struct PostgresProfile {
    pub(crate) config: tokio_postgres::Config,
    pub(crate) profile_fingerprint: String,
    pub(crate) expected_schema: SchemaDependencySlice,
    pub(crate) evidence: PostgresEvidence,
    pub(crate) strictness: SchemaStrictness,
    pub(crate) session: SessionContract,
    pub(crate) limits: RuntimeLimits,
    pub(crate) tls: PostgresTls,
}

impl PostgresProfile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        database_url: &str,
        profile_fingerprint: impl Into<String>,
        expected_schema: SchemaDependencySlice,
        evidence: PostgresEvidence,
        strictness: SchemaStrictness,
        session: SessionContract,
        limits: RuntimeLimits,
        tls: PostgresTls,
    ) -> Result<Self, SqlError> {
        let config = database_url
            .parse::<tokio_postgres::Config>()
            .map_err(|_| configuration_error())?;
        let profile_fingerprint = profile_fingerprint.into();
        if !valid_fingerprint(&profile_fingerprint) {
            return Err(configuration_error());
        }
        validate_evidence(&evidence)?;
        Ok(Self {
            config,
            profile_fingerprint,
            expected_schema,
            evidence,
            strictness,
            session: session.validate()?,
            limits: limits.validate()?,
            tls,
        })
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
    pub const fn limits(&self) -> RuntimeLimits {
        self.limits
    }
}

impl fmt::Debug for PostgresProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PostgresProfile")
            .field("database", &"<redacted>")
            .field("profile_fingerprint", &self.profile_fingerprint)
            .field("schema_fingerprint", &self.expected_schema.fingerprint())
            .field("evidence", &self.evidence)
            .field("strictness", &self.strictness)
            .field("session", &self.session)
            .field("limits", &self.limits)
            .field("tls", &self.tls)
            .finish_non_exhaustive()
    }
}

fn validate_evidence(evidence: &PostgresEvidence) -> Result<(), SqlError> {
    match evidence {
        PostgresEvidence::Introspection {
            fingerprint_statement,
            probes,
        } => {
            if fingerprint_statement.trim().is_empty()
                || fingerprint_statement.contains(';')
                || probes.is_empty()
            {
                return Err(configuration_error());
            }
        }
        PostgresEvidence::MigrationHead {
            head_statement,
            accepted_states,
        } => {
            if head_statement.trim().is_empty()
                || head_statement.contains(';')
                || accepted_states.is_empty()
            {
                return Err(configuration_error());
            }
        }
        PostgresEvidence::SignedManifest { manifest, .. } => {
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

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;

    struct PanickingVerifier;

    impl ManifestVerifier for PanickingVerifier {
        fn verify(
            &self,
            _manifest: &SignedSchemaManifest,
        ) -> Result<SchemaDependencySlice, SqlError> {
            panic!("untrusted verifier panic")
        }
    }

    #[test]
    fn signed_manifest_verifier_panics_become_configuration_errors() {
        let verifier: Arc<dyn ManifestVerifier> = Arc::new(PanickingVerifier);
        let manifest = SignedSchemaManifest {
            signer: "deployment".to_string(),
            payload: Arc::from([1_u8]),
            signature: Arc::from([2_u8]),
        };
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            PostgresEvidence::verify_manifest(&verifier, &manifest)
        }));
        let error = result
            .expect("provider boundary must catch the panic")
            .expect_err("panic must become an error");
        assert_eq!(error.kind(), sifr_sql_runtime::SqlErrorKind::Configuration);
    }
}
