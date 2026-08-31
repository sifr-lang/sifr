use crate::error::configuration_error;
use mysql_async::{Conn, Opts};
use sifr_sql_runtime::{
    ProviderFuture, RuntimeLimits, SchemaDependencySlice, SchemaStrictness, SessionContract,
    SqlError,
};
use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MysqlTlsPolicy {
    Required,
    DisabledForLocalTest,
}

pub trait MysqlSchemaVerifier: Send + Sync + 'static {
    fn observe<'a>(
        &'a self,
        connection: &'a mut Conn,
        profile: &'a MysqlProfile,
    ) -> ProviderFuture<'a, SchemaDependencySlice>;
}

#[derive(Clone)]
pub struct MysqlProfile {
    pub(crate) opts: Opts,
    pub(crate) control_opts: Opts,
    pub(crate) profile_fingerprint: String,
    pub(crate) expected_schema: SchemaDependencySlice,
    pub(crate) strictness: SchemaStrictness,
    pub(crate) session: SessionContract,
    pub(crate) limits: RuntimeLimits,
    pub(crate) tls: MysqlTlsPolicy,
    pub(crate) verifier: Arc<dyn MysqlSchemaVerifier>,
}

impl MysqlProfile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        opts: Opts,
        control_opts: Opts,
        profile_fingerprint: impl Into<String>,
        expected_schema: SchemaDependencySlice,
        strictness: SchemaStrictness,
        session: SessionContract,
        limits: RuntimeLimits,
        tls: MysqlTlsPolicy,
        verifier: Arc<dyn MysqlSchemaVerifier>,
    ) -> Result<Self, SqlError> {
        let profile_fingerprint = profile_fingerprint.into();
        if !valid_fingerprint(&profile_fingerprint)
            || opts.stmt_cache_size() != 0
            || control_opts.stmt_cache_size() != 0
            || !tls_matches(&opts, tls)
            || !tls_matches(&control_opts, tls)
        {
            return Err(configuration_error());
        }
        Ok(Self {
            opts,
            control_opts,
            profile_fingerprint,
            expected_schema,
            strictness,
            session: session.validate()?,
            limits: limits.validate()?,
            tls,
            verifier,
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

    #[must_use]
    pub const fn tls_policy(&self) -> MysqlTlsPolicy {
        self.tls
    }
}

impl fmt::Debug for MysqlProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MysqlProfile")
            .field("connection", &"<redacted>")
            .field("control_connection", &"<redacted>")
            .field("profile_fingerprint", &self.profile_fingerprint)
            .field("schema_fingerprint", &self.expected_schema.fingerprint())
            .field("strictness", &self.strictness)
            .field("session", &self.session)
            .field("limits", &self.limits)
            .field("tls", &self.tls)
            .finish_non_exhaustive()
    }
}

fn tls_matches(opts: &Opts, policy: MysqlTlsPolicy) -> bool {
    match (policy, opts.ssl_opts()) {
        (MysqlTlsPolicy::Required, Some(ssl)) => {
            !ssl.accept_invalid_certs() && !ssl.skip_domain_validation()
        }
        (MysqlTlsPolicy::DisabledForLocalTest, None) => true,
        _ => false,
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}
