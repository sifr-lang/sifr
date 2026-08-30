use crate::{SqlError, SqlErrorKind};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unverified;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verified;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerificationEvidence {
    profile_fingerprint: String,
    schema_fingerprint: String,
}

impl VerificationEvidence {
    pub fn new(
        profile_fingerprint: impl Into<String>,
        schema_fingerprint: impl Into<String>,
    ) -> Result<Self, SqlError> {
        let evidence = Self {
            profile_fingerprint: profile_fingerprint.into(),
            schema_fingerprint: schema_fingerprint.into(),
        };
        if !valid_fingerprint(&evidence.profile_fingerprint)
            || !valid_fingerprint(&evidence.schema_fingerprint)
        {
            return Err(SqlError::new(SqlErrorKind::SchemaContract));
        }
        Ok(evidence)
    }

    #[must_use]
    pub fn profile_fingerprint(&self) -> &str {
        &self.profile_fingerprint
    }

    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        &self.schema_fingerprint
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderLeaseToken(String);

impl ProviderLeaseToken {
    pub fn new(value: impl Into<String>) -> Result<Self, SqlError> {
        let value = value.into();
        if value.is_empty() || value.len() > 160 || value.chars().any(char::is_control) {
            return Err(SqlError::new(SqlErrorKind::Provider));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug)]
struct PoolIdentity {
    profile_fingerprint: String,
    schema_fingerprint: Option<String>,
}

/// A pool is the only share-safe SQL handle.
pub struct Pool<P, S> {
    identity: Arc<PoolIdentity>,
    marker: PhantomData<fn() -> (P, S)>,
}

impl<P, S> Clone for Pool<P, S> {
    fn clone(&self) -> Self {
        Self {
            identity: Arc::clone(&self.identity),
            marker: PhantomData,
        }
    }
}

impl<P> Pool<P, Unverified> {
    pub fn new(profile_fingerprint: impl Into<String>) -> Result<Self, SqlError> {
        let profile_fingerprint = profile_fingerprint.into();
        if !valid_fingerprint(&profile_fingerprint) {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
        Ok(Self {
            identity: Arc::new(PoolIdentity {
                profile_fingerprint,
                schema_fingerprint: None,
            }),
            marker: PhantomData,
        })
    }

    pub fn verify(self, evidence: VerificationEvidence) -> Result<Pool<P, Verified>, SqlError> {
        if self.identity.profile_fingerprint != evidence.profile_fingerprint {
            return Err(SqlError::new(SqlErrorKind::SchemaContract));
        }
        Ok(Pool {
            identity: Arc::new(PoolIdentity {
                profile_fingerprint: evidence.profile_fingerprint,
                schema_fingerprint: Some(evidence.schema_fingerprint),
            }),
            marker: PhantomData,
        })
    }
}

fn valid_fingerprint(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

impl<P> Pool<P, Verified> {
    #[must_use]
    pub fn profile_fingerprint(&self) -> &str {
        &self.identity.profile_fingerprint
    }

    #[must_use]
    pub fn schema_fingerprint(&self) -> &str {
        self.identity
            .schema_fingerprint
            .as_ref()
            .map_or("", String::as_str)
    }

    /// Provider bridges call this after they acquire one native lease.
    #[doc(hidden)]
    #[must_use]
    pub fn attach_lease(&self, lease: ProviderLeaseToken) -> Connection<P, Verified> {
        Connection {
            lease,
            marker: PhantomData,
            not_send_or_sync: PhantomData,
        }
    }
}

/// A connection is scoped to one task and cannot be cloned or sent.
///
/// ```compile_fail
/// use sifr_sql_runtime::{Pool, ProviderLeaseToken, Unverified, VerificationEvidence};
/// struct App;
/// let profile = "a".repeat(64);
/// let schema = "b".repeat(64);
/// let pool = Pool::<App, Unverified>::new(profile.as_str()).unwrap();
/// let evidence = VerificationEvidence::new(profile, schema).unwrap();
/// let pool = pool.verify(evidence).unwrap();
/// let connection = pool.attach_lease(ProviderLeaseToken::new("lease").unwrap());
/// std::thread::spawn(move || drop(connection));
/// ```
pub struct Connection<P, S> {
    lease: ProviderLeaseToken,
    marker: PhantomData<fn() -> (P, S)>,
    not_send_or_sync: PhantomData<Rc<()>>,
}

impl<P, S> Connection<P, S> {
    #[must_use]
    pub fn lease_id(&self) -> &str {
        self.lease.as_str()
    }

    pub fn begin(&mut self) -> Transaction<'_, P, S> {
        Transaction {
            connection: self,
            active: true,
        }
    }

    pub fn stream<R>(&mut self) -> RowStream<'_, P, R> {
        RowStream {
            owner: PhantomData,
            not_send_or_sync: PhantomData,
        }
    }
}

/// A transaction exclusively borrows its connection until completion.
///
/// ```compile_fail
/// use sifr_sql_runtime::{Pool, ProviderLeaseToken, Unverified, VerificationEvidence};
/// struct App;
/// let profile = "a".repeat(64);
/// let schema = "b".repeat(64);
/// let pool = Pool::<App, Unverified>::new(profile.as_str()).unwrap();
/// let evidence = VerificationEvidence::new(profile, schema).unwrap();
/// let pool = pool.verify(evidence).unwrap();
/// let mut connection = pool.attach_lease(ProviderLeaseToken::new("lease").unwrap());
/// let transaction = connection.begin();
/// let _ = connection.lease_id();
/// let _ = transaction.is_active();
/// ```
pub struct Transaction<'connection, P, S> {
    connection: &'connection mut Connection<P, S>,
    active: bool,
}

impl<P, S> Transaction<'_, P, S> {
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    #[must_use]
    pub fn lease_id(&self) -> &str {
        self.connection.lease_id()
    }

    pub fn finish(mut self) {
        self.active = false;
    }

    pub fn stream<R>(&mut self) -> RowStream<'_, P, R> {
        RowStream {
            owner: PhantomData,
            not_send_or_sync: PhantomData,
        }
    }
}

/// A borrowed row stream keeps its connection or transaction borrowed.
pub struct RowStream<'owner, P, R> {
    owner: PhantomData<&'owner mut (P, R)>,
    not_send_or_sync: PhantomData<Rc<()>>,
}

/// A pool-created stream owns its leased connection until close.
pub struct OwnedRowStream<P, R> {
    connection: Connection<P, Verified>,
    row: PhantomData<fn() -> R>,
}

impl<P, R> OwnedRowStream<P, R> {
    #[doc(hidden)]
    #[must_use]
    pub fn new(connection: Connection<P, Verified>) -> Self {
        Self {
            connection,
            row: PhantomData,
        }
    }

    #[must_use]
    pub fn lease_id(&self) -> &str {
        self.connection.lease_id()
    }

    #[must_use]
    pub fn close(self) -> Connection<P, Verified> {
        self.connection
    }
}
