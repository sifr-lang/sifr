use std::fmt;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::task::JoinHandle;

pub const OPERATION_TIMEOUT: Duration = Duration::from_secs(4);
pub type EventCallback =
    sifr_runtime::interop::ThreadsafeCallbackBridge<(String,), Result<(), String>>;
static NEXT_TEMP_ID: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_WATCHERS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct SubscriptionError {
    message: String,
}

impl SubscriptionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn context(context: &str, error: impl fmt::Display) -> Self {
        Self::new(format!("{context}: {error}"))
    }
}

impl fmt::Display for SubscriptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SubscriptionError {}

#[derive(Default)]
struct ObservationState {
    websocket: Option<String>,
    redis: Option<String>,
    notify: Option<String>,
    foreign_thread: bool,
    failure: Option<String>,
}

#[derive(Clone, Default)]
pub struct Observations {
    state: Arc<Mutex<ObservationState>>,
}

impl Observations {
    pub fn record_websocket(&self, event: String) {
        observation_state(&self.state)
            .websocket
            .get_or_insert(event);
    }

    pub fn record_redis(&self, event: String) {
        observation_state(&self.state).redis.get_or_insert(event);
    }

    pub fn record_notify(&self, event: String, foreign_thread: bool) {
        let mut state = observation_state(&self.state);
        state.notify.get_or_insert(event);
        state.foreign_thread |= foreign_thread;
    }

    pub fn record_failure(&self, error: impl Into<String>) {
        observation_state(&self.state)
            .failure
            .get_or_insert_with(|| error.into());
    }

    pub fn complete(&self) -> bool {
        let state = observation_state(&self.state);
        state.failure.is_some()
            || (state.websocket.is_some() && state.redis.is_some() && state.notify.is_some())
    }

    pub fn summary(&self) -> Result<String, SubscriptionError> {
        let state = observation_state(&self.state);
        if let Some(failure) = &state.failure {
            return Err(SubscriptionError::new(failure.clone()));
        }
        Ok(format!(
            "ws={};redis={};notify={};foreign-thread={}",
            state
                .websocket
                .as_deref()
                .ok_or_else(|| SubscriptionError::new("missing WebSocket callback"))?,
            state
                .redis
                .as_deref()
                .ok_or_else(|| SubscriptionError::new("missing Redis callback"))?,
            state
                .notify
                .as_deref()
                .ok_or_else(|| SubscriptionError::new("missing notify callback"))?,
            state.foreign_thread
        ))
    }

    pub fn protocol_summary(&self) -> Result<String, SubscriptionError> {
        let summary = self.summary()?;
        Ok(summary
            .split_once(";foreign-thread=")
            .map_or(summary.clone(), |(protocols, _)| protocols.to_string()))
    }

    pub fn foreign_thread(&self) -> Result<bool, SubscriptionError> {
        let state = observation_state(&self.state);
        if let Some(failure) = &state.failure {
            return Err(SubscriptionError::new(failure.clone()));
        }
        Ok(state.foreign_thread)
    }
}

fn observation_state(state: &Mutex<ObservationState>) -> MutexGuard<'_, ObservationState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub struct TemporaryDirectory {
    path: Option<PathBuf>,
}

impl TemporaryDirectory {
    pub fn create() -> Result<Self, SubscriptionError> {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "sifr-subscription-runtime-{}-{id}",
            std::process::id()
        ));
        if path.exists() {
            std::fs::remove_dir_all(&path).map_err(|error| {
                SubscriptionError::context("remove stale temp directory", error)
            })?;
        }
        std::fs::create_dir(&path)
            .map_err(|error| SubscriptionError::context("create temp directory", error))?;
        Ok(Self { path: Some(path) })
    }

    pub fn path(&self) -> Result<&Path, SubscriptionError> {
        self.path
            .as_deref()
            .ok_or_else(|| SubscriptionError::new("temporary directory is already removed"))
    }

    pub fn remove(&mut self) -> Result<bool, SubscriptionError> {
        let Some(path) = self.path.take() else {
            return Ok(true);
        };
        std::fs::remove_dir_all(&path)
            .map_err(|error| SubscriptionError::context("remove temp directory", error))?;
        Ok(!path.exists())
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_dir_all(path);
        }
    }
}

struct ActiveTask;

impl ActiveTask {
    fn enter() -> Self {
        ACTIVE_TASKS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for ActiveTask {
    fn drop(&mut self) {
        ACTIVE_TASKS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub fn spawn_tracked<F>(future: F) -> JoinHandle<Result<(), SubscriptionError>>
where
    F: Future<Output = Result<(), SubscriptionError>> + Send + 'static,
{
    tokio::spawn(async move {
        let _active = ActiveTask::enter();
        future.await
    })
}

pub async fn join_task(
    task: JoinHandle<Result<(), SubscriptionError>>,
    context: &str,
) -> Result<(), SubscriptionError> {
    tokio::time::timeout(OPERATION_TIMEOUT, task)
        .await
        .map_err(|_| SubscriptionError::new(format!("{context} timed out")))?
        .map_err(|error| SubscriptionError::context(context, error))?
}

pub async fn cancel_task(
    task: JoinHandle<Result<(), SubscriptionError>>,
) -> Result<(), SubscriptionError> {
    task.abort();
    match tokio::time::timeout(OPERATION_TIMEOUT, task).await {
        Ok(Err(error)) if error.is_cancelled() => Ok(()),
        Ok(Err(error)) => Err(SubscriptionError::context("cancel callback task", error)),
        Ok(Ok(result)) => result,
        Err(_) => Err(SubscriptionError::new("cancel callback task timed out")),
    }
}

pub fn watcher_started() {
    ACTIVE_WATCHERS.fetch_add(1, Ordering::SeqCst);
}

pub fn watcher_stopped() {
    ACTIVE_WATCHERS.fetch_sub(1, Ordering::SeqCst);
}

pub fn active_work() -> usize {
    ACTIVE_TASKS.load(Ordering::SeqCst) + ACTIVE_WATCHERS.load(Ordering::SeqCst)
}
