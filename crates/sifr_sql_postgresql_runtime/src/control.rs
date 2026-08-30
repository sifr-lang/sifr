use crate::config::PostgresProfile;
use crate::config::PostgresTls;
use crate::connection::{ExecutionOptions, PostgresNativeConnection, cancel_query};
use sifr_runtime::cancellation::{CancellationClaimError, CancellationClaimLease};
use sifr_sql_runtime::{AsyncCleanupEvidence, SqlError, SqlErrorKind};
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::Ordering;

pub(crate) struct ControlHandle {
    poison: Arc<std::sync::atomic::AtomicBool>,
    token: tokio_postgres::CancelToken,
    tls: PostgresTls,
    cleanup_timeout: std::time::Duration,
}

impl ControlHandle {
    pub(crate) fn new(native: &PostgresNativeConnection, profile: &PostgresProfile) -> Self {
        Self {
            poison: Arc::clone(&native.poisoned),
            token: native.cancel_token.clone(),
            tls: profile.tls.clone(),
            cleanup_timeout: profile.limits.cleanup_timeout,
        }
    }
}

pub(crate) fn arm_cancellation(
    control: &ControlHandle,
    options: &ExecutionOptions,
) -> Result<Option<CancellationClaimLease>, SqlError> {
    let Some(carrier) = &options.cancellation else {
        return Ok(None);
    };
    let poison = Arc::clone(&control.poison);
    let token = control.token.clone();
    let tls = control.tls.clone();
    carrier
        .claim(Arc::new(move || {
            poison.store(true, Ordering::Release);
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let spawned_tls = tls.clone();
                let spawned_token = token.clone();
                handle.spawn(async move {
                    let _result = cancel_query(&spawned_tls, &spawned_token).await;
                });
            }
        }))
        .map(Some)
        .map_err(|error| match error {
            CancellationClaimError::CancelledBeforeClaim => SqlError::new(SqlErrorKind::Cancelled),
            CancellationClaimError::AlreadyClaimed | CancellationClaimError::StateUnavailable => {
                SqlError::new(SqlErrorKind::Provider)
            }
        })
}

pub(crate) async fn run_controlled<T>(
    control: &ControlHandle,
    profile: &Arc<PostgresProfile>,
    options: &ExecutionOptions,
    operation: impl Future<Output = Result<T, SqlError>>,
) -> Result<T, SqlError> {
    let _claim = arm_cancellation(control, options)?;
    let deadline = options.deadline(profile)?;
    match tokio::time::timeout(deadline, operation).await {
        Ok(result) => {
            if control.poison.load(Ordering::Acquire) {
                Err(SqlError::new(SqlErrorKind::Cancelled))
            } else {
                result
            }
        }
        Err(_) => Err(cancel_after_timeout(control, options).await),
    }
}

pub(crate) async fn cancel_after_timeout(
    control: &ControlHandle,
    options: &ExecutionOptions,
) -> SqlError {
    control.poison.store(true, Ordering::Release);
    let budget = control.cleanup_timeout;
    let cancel_result =
        tokio::time::timeout(budget, cancel_query(&control.tls, &control.token)).await;
    let mut primary = SqlError::new(SqlErrorKind::Timeout);
    let evidence = match cancel_result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(AsyncCleanupEvidence::cleanup_failed(
            error.to_string(),
            "postgresql-execution".to_string(),
            "postgresql-connection".to_string(),
            "cancel-query".to_string(),
            budget,
        )),
        Err(_) => Some(AsyncCleanupEvidence::cleanup_timed_out(
            "postgresql-execution".to_string(),
            "postgresql-connection".to_string(),
            "cancel-query".to_string(),
            budget,
        )),
    };
    if let Some(evidence) = evidence {
        if let Some(carrier) = &options.cancellation {
            carrier.record_async_cleanup_evidence(evidence.clone());
        }
        primary.extend_secondary([evidence]);
    }
    primary
}
