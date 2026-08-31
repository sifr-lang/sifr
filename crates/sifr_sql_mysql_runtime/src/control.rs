use crate::config::MysqlProfile;
use crate::connection::ExecutionOptions;
use crate::error::map_mysql_error;
use mysql_async::{Conn, Opts, prelude::Queryable};
use sifr_runtime::cancellation::{CancellationClaimError, CancellationClaimLease};
use sifr_sql_runtime::{AsyncCleanupEvidence, SqlError, SqlErrorKind};
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub(crate) struct ControlHandle {
    pub(crate) poison: Arc<AtomicBool>,
    target_connection_id: u32,
    control_opts: Opts,
    cleanup_timeout: Duration,
}

impl ControlHandle {
    pub(crate) fn new(
        poison: Arc<AtomicBool>,
        target_connection_id: u32,
        profile: &MysqlProfile,
    ) -> Self {
        Self {
            poison,
            target_connection_id,
            control_opts: profile.control_opts.clone(),
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
    let target = control.target_connection_id;
    let opts = control.control_opts.clone();
    carrier
        .claim(Arc::new(move || {
            poison.store(true, Ordering::Release);
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let spawned_opts = opts.clone();
                handle.spawn(async move {
                    let _result = kill_query(spawned_opts, target).await;
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
    profile: &Arc<MysqlProfile>,
    options: &ExecutionOptions,
    operation: impl Future<Output = Result<T, SqlError>>,
) -> Result<T, SqlError> {
    let _claim = arm_cancellation(control, options)?;
    let deadline = options.deadline(profile)?;
    match tokio::time::timeout(deadline, operation).await {
        Ok(result) if !control.poison.load(Ordering::Acquire) => result,
        Ok(_) => Err(SqlError::new(SqlErrorKind::Cancelled)),
        Err(_) => Err(cancel_after_timeout(control, options).await),
    }
}

async fn cancel_after_timeout(control: &ControlHandle, options: &ExecutionOptions) -> SqlError {
    control.poison.store(true, Ordering::Release);
    let budget = control.cleanup_timeout;
    let result = tokio::time::timeout(
        budget,
        kill_query(control.control_opts.clone(), control.target_connection_id),
    )
    .await;
    let mut primary = SqlError::new(SqlErrorKind::Timeout);
    let evidence = match result {
        Ok(Ok(())) => None,
        Ok(Err(error)) => Some(AsyncCleanupEvidence::cleanup_failed(
            error.to_string(),
            "mysql-execution".to_string(),
            "mysql-target-connection".to_string(),
            "kill-query".to_string(),
            budget,
        )),
        Err(_) => Some(AsyncCleanupEvidence::cleanup_timed_out(
            "mysql-execution".to_string(),
            "mysql-target-connection".to_string(),
            "kill-query".to_string(),
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

pub(crate) async fn kill_query(opts: Opts, target_connection_id: u32) -> Result<(), SqlError> {
    let mut control = Conn::new(opts)
        .await
        .map_err(|error| map_mysql_error(&error))?;
    let statement = kill_query_statement(target_connection_id);
    let result = control
        .query_drop(statement)
        .await
        .map_err(|error| map_mysql_error(&error));
    let disconnect = control
        .disconnect()
        .await
        .map_err(|error| map_mysql_error(&error));
    result.and(disconnect)
}

#[must_use]
pub(crate) fn kill_query_statement(target_connection_id: u32) -> String {
    format!("KILL QUERY {target_connection_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_query_uses_only_the_numeric_server_identity() {
        assert_eq!(kill_query_statement(42), "KILL QUERY 42");
    }
}
