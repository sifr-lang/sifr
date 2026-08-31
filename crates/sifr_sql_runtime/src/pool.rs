use crate::{
    AsyncCleanupEvidence, CancellationCarrier, ResourceLimitKind, RuntimeLimits, SqlError,
    SqlErrorKind,
};
use std::collections::VecDeque;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

struct PoolState<T> {
    idle: VecDeque<T>,
    total: u32,
    closed: bool,
}

impl<T> Default for PoolState<T> {
    fn default() -> Self {
        Self {
            idle: VecDeque::new(),
            total: 0,
            closed: false,
        }
    }
}

struct PoolCore<T> {
    state: Mutex<PoolState<T>>,
    permits: Arc<Semaphore>,
    limits: RuntimeLimits,
}

/// Provider-neutral bounded pool coordination.
///
/// The provider owns connection creation and reset. This coordinator owns the
/// connection bound, acquisition deadline, idle queue, and discard accounting.
pub struct PoolCoordinator<T> {
    core: Arc<PoolCore<T>>,
}

impl<T> Clone for PoolCoordinator<T> {
    fn clone(&self) -> Self {
        Self {
            core: Arc::clone(&self.core),
        }
    }
}

impl<T> PoolCoordinator<T> {
    pub fn new(limits: RuntimeLimits) -> Result<Self, SqlError> {
        let limits = limits.validate()?;
        Ok(Self {
            core: Arc::new(PoolCore {
                state: Mutex::new(PoolState::default()),
                permits: Arc::new(Semaphore::new(limits.max_connections as usize)),
                limits,
            }),
        })
    }

    pub async fn acquire<Open, OpenFuture>(&self, open: Open) -> Result<PoolLease<T>, SqlError>
    where
        Open: FnOnce() -> OpenFuture,
        OpenFuture: Future<Output = Result<T, SqlError>>,
    {
        let permit = tokio::time::timeout(
            self.core.limits.acquire_timeout,
            Arc::clone(&self.core.permits).acquire_owned(),
        )
        .await
        .map_err(|_| SqlError::resource_limit(ResourceLimitKind::AcquireDeadline))?
        .map_err(|_| SqlError::new(SqlErrorKind::Connection))?;

        let open_new = {
            let mut state = lock_state(&self.core.state);
            if state.closed {
                return Err(SqlError::new(SqlErrorKind::Connection));
            }
            if let Some(resource) = state.idle.pop_front() {
                return Ok(PoolLease::new(Arc::clone(&self.core), permit, resource));
            }
            if state.total >= self.core.limits.max_connections {
                return Err(SqlError::resource_limit(ResourceLimitKind::Connections));
            }
            state.total += 1;
            true
        };

        if !open_new {
            return Err(SqlError::new(SqlErrorKind::Connection));
        }
        let resource = match tokio::time::timeout(self.core.limits.acquire_timeout, open()).await {
            Ok(Ok(resource)) => resource,
            Ok(Err(error)) => {
                decrement_total(&self.core);
                return Err(error);
            }
            Err(_) => {
                decrement_total(&self.core);
                return Err(SqlError::resource_limit(ResourceLimitKind::AcquireDeadline));
            }
        };
        Ok(PoolLease::new(Arc::clone(&self.core), permit, resource))
    }

    #[must_use]
    pub fn statistics(&self) -> PoolStatistics {
        let state = lock_state(&self.core.state);
        PoolStatistics {
            total: state.total,
            idle: u32::try_from(state.idle.len()).unwrap_or(u32::MAX),
            closed: state.closed,
        }
    }

    pub fn close(&self) {
        let idle = {
            let mut state = lock_state(&self.core.state);
            state.closed = true;
            state.total = state
                .total
                .saturating_sub(u32::try_from(state.idle.len()).unwrap_or(u32::MAX));
            std::mem::take(&mut state.idle)
        };
        drop(idle);
        self.core.permits.close();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PoolStatistics {
    pub total: u32,
    pub idle: u32,
    pub closed: bool,
}

/// One task-local checked-out resource.
pub struct PoolLease<T> {
    core: Arc<PoolCore<T>>,
    permit: Option<OwnedSemaphorePermit>,
    resource: Option<T>,
    not_send_or_sync: PhantomData<Rc<()>>,
}

/// Pool accounting for a resource that a provider must consume and discard.
///
/// This guard lets a provider hand a native connection to an owning driver
/// stream without making checked-out connections share-safe. The pool slot
/// remains occupied until the guard is dropped.
#[doc(hidden)]
pub struct PoolDiscardGuard<T> {
    core: Arc<PoolCore<T>>,
    permit: Option<OwnedSemaphorePermit>,
    not_send_or_sync: PhantomData<Rc<()>>,
}

impl<T> PoolLease<T> {
    fn new(core: Arc<PoolCore<T>>, permit: OwnedSemaphorePermit, resource: T) -> Self {
        Self {
            core,
            permit: Some(permit),
            resource: Some(resource),
            not_send_or_sync: PhantomData,
        }
    }

    pub fn resource(&self) -> Result<&T, SqlError> {
        self.resource
            .as_ref()
            .ok_or_else(|| SqlError::new(SqlErrorKind::Connection))
    }

    pub fn resource_mut(&mut self) -> Result<&mut T, SqlError> {
        self.resource
            .as_mut()
            .ok_or_else(|| SqlError::new(SqlErrorKind::Connection))
    }

    pub async fn release<Reset>(
        mut self,
        reset: Reset,
        cancellation: Option<&CancellationCarrier>,
        resource_name: &str,
    ) -> Result<(), SqlError>
    where
        Reset:
            for<'a> FnOnce(&'a mut T) -> Pin<Box<dyn Future<Output = Result<(), SqlError>> + 'a>>,
    {
        let Some(mut resource) = self.resource.take() else {
            return Err(SqlError::new(SqlErrorKind::Connection));
        };
        let budget = self.core.limits.cleanup_timeout;
        let reset_result = tokio::time::timeout(budget, reset(&mut resource)).await;
        match reset_result {
            Ok(Ok(())) => {
                let mut state = lock_state(&self.core.state);
                if state.closed {
                    state.total = state.total.saturating_sub(1);
                    drop(state);
                    drop(resource);
                } else {
                    state.idle.push_back(resource);
                }
                self.permit.take();
                Ok(())
            }
            Ok(Err(mut error)) => {
                let evidence = AsyncCleanupEvidence::cleanup_failed(
                    error.to_string(),
                    "sql-pool-release".to_string(),
                    resource_name.to_string(),
                    "session-reset".to_string(),
                    budget,
                );
                if let Some(carrier) = cancellation {
                    carrier.record_async_cleanup_evidence(evidence.clone());
                }
                error.extend_secondary([evidence]);
                decrement_total(&self.core);
                self.permit.take();
                drop(resource);
                Err(error)
            }
            Err(_) => {
                let evidence = AsyncCleanupEvidence::cleanup_timed_out(
                    "sql-pool-release".to_string(),
                    resource_name.to_string(),
                    "session-reset".to_string(),
                    budget,
                );
                if let Some(carrier) = cancellation {
                    carrier.record_async_cleanup_evidence(evidence.clone());
                }
                decrement_total(&self.core);
                self.permit.take();
                drop(resource);
                Err(SqlError::resource_limit(ResourceLimitKind::CleanupDeadline)
                    .with_secondary(evidence))
            }
        }
    }

    pub fn discard(mut self) {
        if self.resource.take().is_some() {
            decrement_total(&self.core);
        }
        self.permit.take();
    }

    /// Detach the resource for an operation that cannot return it to the pool.
    ///
    /// The returned guard preserves the connection bound until both the
    /// operation and its native resource have ended.
    #[doc(hidden)]
    pub fn detach_for_discard(mut self) -> Result<(T, PoolDiscardGuard<T>), SqlError> {
        let resource = self
            .resource
            .take()
            .ok_or_else(|| SqlError::new(SqlErrorKind::Connection))?;
        let permit = self.permit.take();
        Ok((
            resource,
            PoolDiscardGuard {
                core: Arc::clone(&self.core),
                permit,
                not_send_or_sync: PhantomData,
            },
        ))
    }
}

impl<T> Drop for PoolLease<T> {
    fn drop(&mut self) {
        if self.resource.take().is_some() {
            decrement_total(&self.core);
        }
    }
}

impl<T> Drop for PoolDiscardGuard<T> {
    fn drop(&mut self) {
        decrement_total(&self.core);
        self.permit.take();
    }
}

fn decrement_total<T>(core: &PoolCore<T>) {
    let mut state = lock_state(&core.state);
    state.total = state.total.saturating_sub(1);
}

fn lock_state<T>(mutex: &Mutex<PoolState<T>>) -> MutexGuard<'_, PoolState<T>> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
