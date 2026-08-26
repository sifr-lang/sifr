use std::cell::RefCell;
use std::fmt;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use redis::IntoConnectionInfo;
use rusqlite::Connection;
use sifr_runtime::interop::{Handle, HandleStateError, PoisonOnPanic, catch_unwind_silently};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

use super::protocols;

const OPERATION_TIMEOUT: Duration = Duration::from_secs(3);
const REDIS_RECONNECT_ATTEMPTS: usize = 2;
static NEXT_DATABASE_ID: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_TASKS: AtomicUsize = AtomicUsize::new(0);
thread_local! {
    static CLOSE_OBSERVATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[derive(Debug)]
pub struct ResourceError {
    message: String,
}

impl ResourceError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(crate) fn context(context: &str, error: impl fmt::Display) -> Self {
        Self::new(format!("{context}: {error}"))
    }
}

impl fmt::Display for ResourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ResourceError {}

#[derive(Clone)]
pub struct ResourceMatrix {
    state: Rc<RefCell<ResourceState>>,
}

impl fmt::Debug for ResourceMatrix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResourceMatrix")
            .finish_non_exhaustive()
    }
}

struct ResourceState {
    closed: bool,
    http_url: Option<String>,
    http_client: Option<reqwest::Client>,
    sqlite: Option<Connection>,
    sqlite_file: Option<TemporaryDatabase>,
    redis: Option<redis::aio::ConnectionManager>,
    postgres: Option<Arc<tokio_postgres::Client>>,
    tasks: Vec<TrackedTask>,
}

impl Drop for ResourceState {
    fn drop(&mut self) {
        for task in &mut self.tasks {
            task.abort();
        }
        self.tasks.clear();
        self.sqlite.take();
        self.sqlite_file.take();
    }
}

struct TemporaryDatabase {
    path: Option<PathBuf>,
}

impl TemporaryDatabase {
    fn create() -> Result<Self, ResourceError> {
        let path = unique_database_path();
        if path.exists() {
            return Err(ResourceError::new(
                "temporary SQLite database path already exists",
            ));
        }
        Ok(Self { path: Some(path) })
    }

    fn path(&self) -> Result<&std::path::Path, ResourceError> {
        self.path
            .as_deref()
            .ok_or_else(|| ResourceError::new("temporary SQLite database path missing"))
    }

    fn remove(mut self) -> Result<(), ResourceError> {
        let Some(path) = self.path.take() else {
            return Err(ResourceError::new(
                "temporary SQLite database path missing during cleanup",
            ));
        };
        if !path.is_file() {
            return Err(ResourceError::new(
                "temporary SQLite database was absent before cleanup",
            ));
        }
        std::fs::remove_file(&path)
            .map_err(|error| ResourceError::context("SQLite cleanup", error))?;
        if path.exists() {
            Err(ResourceError::new(
                "temporary SQLite database remained after cleanup",
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for TemporaryDatabase {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_file(path);
        }
    }
}

struct TrackedTask {
    handle: Option<JoinHandle<Result<(), ResourceError>>>,
}

impl TrackedTask {
    fn spawn(
        future: impl std::future::Future<Output = Result<(), ResourceError>> + Send + 'static,
    ) -> Self {
        let activity = TaskActivity::new();
        let handle = tokio::spawn(async move {
            let _activity = activity;
            future.await
        });
        Self {
            handle: Some(handle),
        }
    }

    async fn finish(&mut self) -> Result<(), ResourceError> {
        let Some(mut handle) = self.handle.take() else {
            return Ok(());
        };
        match tokio::time::timeout(OPERATION_TIMEOUT, &mut handle).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) if error.is_cancelled() => Ok(()),
            Ok(Err(error)) => Err(ResourceError::context("resource task join", error)),
            Err(_) => {
                handle.abort();
                match tokio::time::timeout(OPERATION_TIMEOUT, handle).await {
                    Ok(Err(error)) if error.is_cancelled() => {}
                    Ok(Ok(_)) => {}
                    Ok(Err(error)) => {
                        return Err(ResourceError::context("resource task abort join", error));
                    }
                    Err(_) => {
                        return Err(ResourceError::new("resource task abort cleanup timed out"));
                    }
                }
                Err(ResourceError::new("resource task cleanup timed out"))
            }
        }
    }

    fn abort(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl Drop for TrackedTask {
    fn drop(&mut self) {
        self.abort();
    }
}

struct TaskActivity;

impl TaskActivity {
    fn new() -> Self {
        ACTIVE_TASKS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for TaskActivity {
    fn drop(&mut self) {
        ACTIVE_TASKS.fetch_sub(1, Ordering::SeqCst);
    }
}

pub async fn open() -> Result<Handle<ResourceMatrix>, ResourceError> {
    let http_listener = bind_loopback("HTTP").await?;
    let http_address = http_listener
        .local_addr()
        .map_err(|error| ResourceError::context("HTTP local address", error))?;
    let redis_listener = bind_loopback("Redis").await?;
    let redis_address = redis_listener
        .local_addr()
        .map_err(|error| ResourceError::context("Redis local address", error))?;
    let postgres_listener = bind_loopback("PostgreSQL").await?;
    let postgres_address = postgres_listener
        .local_addr()
        .map_err(|error| ResourceError::context("PostgreSQL local address", error))?;

    let mut tasks = vec![
        TrackedTask::spawn(protocols::serve_http(http_listener)),
        TrackedTask::spawn(protocols::serve_redis(redis_listener)),
        TrackedTask::spawn(protocols::serve_postgres(postgres_listener)),
    ];
    let result = open_clients(http_address, redis_address, postgres_address, &mut tasks).await;
    match result {
        Ok(state) => Ok(Handle::new(ResourceMatrix {
            state: Rc::new(RefCell::new(state)),
        })),
        Err(error) => {
            for task in &mut tasks {
                let _ = task.finish().await;
            }
            Err(error)
        }
    }
}

pub fn contract() -> String {
    "opaque-resource-runtime-v1".to_string()
}

pub async fn run(resource: &Handle<ResourceMatrix>) -> Result<String, ResourceError> {
    let lifecycle = verify_lifecycle(resource).await?;
    let protocol_negatives = verify_protocol_failures().await?;
    let poison = verify_poison_redaction()?;
    Ok(format!(
        "{lifecycle};protocol-negatives={protocol_negatives};poison={poison}"
    ))
}

pub async fn invalid_aliasing(
    mut resource: Handle<ResourceMatrix>,
) -> Result<String, ResourceError> {
    let lifecycle = verify_lifecycle(&resource).await?;
    if lifecycle != resource_summary("echo:reqwest", "sqlite", "PONG", "1") {
        return Err(ResourceError::new(format!(
            "unexpected pre-close resource summary: {lifecycle}"
        )));
    }
    let alias = resource.clone();
    close_handle(&mut resource).await?;
    verify_lifecycle(&alias).await
}

async fn verify_protocol_failures() -> Result<String, ResourceError> {
    let redis_listener = bind_loopback("Redis malformed").await?;
    let redis_address = redis_listener
        .local_addr()
        .map_err(|error| ResourceError::context("Redis malformed address", error))?;
    let postgres_listener = bind_loopback("PostgreSQL early-close").await?;
    let postgres_address = postgres_listener
        .local_addr()
        .map_err(|error| ResourceError::context("PostgreSQL early-close address", error))?;
    let mut redis_task = TrackedTask::spawn(protocols::serve_redis_malformed(redis_listener));
    let mut postgres_task =
        TrackedTask::spawn(protocols::serve_postgres_early_close(postgres_listener));

    let probe_result = probe_protocol_failures(redis_address, postgres_address).await;
    let redis_cleanup = redis_task.finish().await;
    let postgres_cleanup = postgres_task.finish().await;
    let (redis_rejected, postgres_rejected) = probe_result?;
    redis_cleanup?;
    postgres_cleanup?;
    if !redis_rejected || !postgres_rejected {
        return Err(ResourceError::new(
            "malformed or early-close protocol frame was accepted",
        ));
    }
    Ok("redis-malformed/postgres-early-close".to_string())
}

async fn probe_protocol_failures(
    redis_address: std::net::SocketAddr,
    postgres_address: std::net::SocketAddr,
) -> Result<(bool, bool), ResourceError> {
    let redis_rejected = probe_malformed_redis(redis_address).await?;
    let postgres_rejected = probe_postgres_early_close(postgres_address).await?;
    Ok((redis_rejected, postgres_rejected))
}

async fn probe_malformed_redis(redis_address: std::net::SocketAddr) -> Result<bool, ResourceError> {
    let redis_client = redis_client(redis_address, "Redis negative")?;
    let redis_config = redis_connection_config();
    let redis_attempt = tokio::time::timeout(
        OPERATION_TIMEOUT,
        redis_client.get_multiplexed_async_connection_with_config(&redis_config),
    )
    .await
    .map_err(|_| ResourceError::new("Redis malformed probe timed out"))?;
    let rejected = match redis_attempt {
        Err(_) => true,
        Ok(mut connection) => tokio::time::timeout(
            OPERATION_TIMEOUT,
            redis::cmd("PING").query_async::<String>(&mut connection),
        )
        .await
        .map_err(|_| ResourceError::new("Redis malformed PING timed out"))?
        .is_err(),
    };
    Ok(rejected)
}

async fn probe_postgres_early_close(
    postgres_address: std::net::SocketAddr,
) -> Result<bool, ResourceError> {
    let postgres_config = postgres_config(postgres_address);
    let postgres_attempt = tokio::time::timeout(
        OPERATION_TIMEOUT,
        postgres_config.connect(tokio_postgres::NoTls),
    )
    .await
    .map_err(|_| ResourceError::new("PostgreSQL early-close probe timed out"))?;
    match postgres_attempt {
        Err(_) => Ok(true),
        Ok((client, connection)) => {
            let mut connection_task = TrackedTask::spawn(async move {
                let _expected_early_close = connection.await;
                Ok(())
            });
            let query_result =
                tokio::time::timeout(OPERATION_TIMEOUT, client.simple_query("SELECT 1")).await;
            drop(client);
            let cleanup = connection_task.finish().await;
            let query_rejected = query_result
                .map_err(|_| ResourceError::new("PostgreSQL early-close query timed out"))?
                .is_err();
            cleanup?;
            Ok(query_rejected)
        }
    }
}

async fn open_clients(
    http_address: std::net::SocketAddr,
    redis_address: std::net::SocketAddr,
    postgres_address: std::net::SocketAddr,
    tasks: &mut Vec<TrackedTask>,
) -> Result<ResourceState, ResourceError> {
    let http_client = reqwest::Client::builder()
        .no_proxy()
        .timeout(OPERATION_TIMEOUT)
        .build()
        .map_err(|error| ResourceError::context("reqwest client", error))?;

    let sqlite_file = TemporaryDatabase::create()?;
    let mut sqlite = Connection::open(sqlite_file.path()?)
        .map_err(|error| ResourceError::context("SQLite open", error))?;
    sqlite
        .execute_batch(
            "CREATE TABLE evidence(value TEXT NOT NULL);\
             INSERT INTO evidence(value) VALUES ('sqlite');",
        )
        .map_err(|error| ResourceError::context("SQLite setup", error))?;
    sqlite
        .savepoint_with_name("sifr; DROP TABLE evidence; --")
        .and_then(rusqlite::Savepoint::commit)
        .map_err(|error| ResourceError::context("SQLite savepoint", error))?;

    let redis_client = redis_client(redis_address, "Redis")?;
    let redis_config = redis_connection_manager_config();
    let redis = tokio::time::timeout(
        OPERATION_TIMEOUT,
        redis_client.get_connection_manager_with_config(redis_config),
    )
    .await
    .map_err(|_| ResourceError::new("Redis connect timed out"))?
    .map_err(|error| ResourceError::context("Redis connect", error))?;

    let postgres_config = postgres_config(postgres_address);
    let (postgres, connection) = tokio::time::timeout(
        OPERATION_TIMEOUT,
        postgres_config.connect(tokio_postgres::NoTls),
    )
    .await
    .map_err(|_| ResourceError::new("PostgreSQL connect timed out"))?
    .map_err(|error| ResourceError::context("PostgreSQL connect", error))?;
    tasks.push(TrackedTask::spawn(async move {
        connection
            .await
            .map_err(|error| ResourceError::context("PostgreSQL connection", error))
    }));

    Ok(ResourceState {
        closed: false,
        http_url: Some(format!("http://{http_address}/resource")),
        http_client: Some(http_client),
        sqlite: Some(sqlite),
        sqlite_file: Some(sqlite_file),
        redis: Some(redis),
        postgres: Some(Arc::new(postgres)),
        tasks: std::mem::take(tasks),
    })
}

pub async fn verify_lifecycle(resource: &Handle<ResourceMatrix>) -> Result<String, ResourceError> {
    let matrix = resource.inner_ref().map_err(handle_error)?.clone();
    let (http, sqlite_value, mut redis, postgres) = {
        let state = matrix.state.borrow();
        ensure_open(&state)?;
        let sqlite = state
            .sqlite
            .as_ref()
            .ok_or_else(|| ResourceError::new("SQLite resource missing"))?;
        let sqlite_value = sqlite
            .query_row("SELECT value FROM evidence", [], |row| {
                row.get::<_, String>(0)
            })
            .map_err(|error| ResourceError::context("SQLite query", error))?;
        (
            state
                .http_client
                .as_ref()
                .ok_or_else(|| ResourceError::new("HTTP resource missing"))?
                .clone(),
            sqlite_value,
            state
                .redis
                .as_ref()
                .ok_or_else(|| ResourceError::new("Redis resource missing"))?
                .clone(),
            state
                .postgres
                .as_ref()
                .ok_or_else(|| ResourceError::new("PostgreSQL resource missing"))?
                .clone(),
        )
    };
    let http_url = {
        let state = matrix.state.borrow();
        if state.tasks.len() != 4 {
            return Err(ResourceError::new(
                "resource cleanup handles were not fully owned",
            ));
        }
        state
            .http_url
            .clone()
            .ok_or_else(|| ResourceError::new("HTTP loopback address missing"))?
    };
    let (http_value, redis_value, postgres_value) =
        tokio::time::timeout(OPERATION_TIMEOUT, async {
            let http_value = http
                .get(http_url)
                .send()
                .await
                .map_err(|error| ResourceError::context("reqwest request", error))?
                .text()
                .await
                .map_err(|error| ResourceError::context("reqwest body", error))?;
            let redis_value: String = redis::cmd("PING")
                .query_async(&mut redis)
                .await
                .map_err(|error| ResourceError::context("Redis PING", error))?;
            let rows = postgres
                .simple_query("SELECT 1")
                .await
                .map_err(|error| ResourceError::context("PostgreSQL SELECT", error))?;
            let postgres_value = rows
                .iter()
                .find_map(|message| match message {
                    tokio_postgres::SimpleQueryMessage::Row(row) => row.get(0),
                    _ => None,
                })
                .ok_or_else(|| ResourceError::new("PostgreSQL row missing"))?
                .to_string();
            Ok::<_, ResourceError>((http_value, redis_value, postgres_value))
        })
        .await
        .map_err(|_| ResourceError::new("resource operations timed out"))??;
    Ok(resource_summary(
        &http_value,
        &sqlite_value,
        &redis_value,
        &postgres_value,
    ))
}

pub fn verify_poison_redaction() -> Result<String, ResourceError> {
    let mut handle = Handle::new(ResourceMatrix::empty());
    let unwind = catch_unwind_silently(std::panic::AssertUnwindSafe(|| {
        let _guard = PoisonOnPanic::new(
            &mut handle,
            sifr_runtime::interop::__generated_glue::token(),
        );
        panic!("opaque-resource-secret-must-not-escape");
    }));
    if unwind.is_ok() {
        return Err(ResourceError::new("poison probe unexpectedly returned"));
    }
    match handle.inner_ref() {
        Err(HandleStateError::Poisoned(error)) => {
            let message = error.to_string();
            if message == "Rust bridge panicked" {
                Ok(message)
            } else {
                Err(ResourceError::new("poison payload was not redacted"))
            }
        }
        _ => Err(ResourceError::new("poisoned handle remained accessible")),
    }
}

pub async fn aclose(mut resource: Handle<ResourceMatrix>) -> Result<(), ResourceError> {
    let first = close_handle(&mut resource).await?;
    let second = close_handle(&mut resource).await?;
    CLOSE_OBSERVATION.with_borrow_mut(|observation| {
        *observation = Some(format!("{first}/{second}"));
    });
    Ok(())
}

pub fn close_observation() -> Result<String, ResourceError> {
    CLOSE_OBSERVATION
        .take()
        .ok_or_else(|| ResourceError::new("opaque async close was not observed"))
}

async fn close_handle(resource: &mut Handle<ResourceMatrix>) -> Result<String, ResourceError> {
    let matrix = match resource.inner_ref() {
        Ok(matrix) => matrix.clone(),
        Err(HandleStateError::Closed) => return Ok("already-closed".to_string()),
        Err(error) => return Err(handle_error(error)),
    };
    let (already_closed, mut tasks, sqlite_file) = {
        let mut state = matrix.state.borrow_mut();
        if state.closed {
            (true, Vec::new(), None)
        } else {
            state.closed = true;
            state.http_url.take();
            state.http_client.take();
            state.redis.take();
            state.postgres.take();
            state.sqlite.take();
            (
                false,
                std::mem::take(&mut state.tasks),
                state.sqlite_file.take(),
            )
        }
    };
    if already_closed {
        resource.mark_closed(sifr_runtime::interop::__generated_glue::token());
        return Ok("already-closed".to_string());
    }
    let mut cleanup_error = None;
    for task in &mut tasks {
        if let Err(error) = task.finish().await
            && cleanup_error.is_none()
        {
            cleanup_error = Some(error);
        }
    }
    if let Some(file) = sqlite_file
        && let Err(error) = file.remove()
        && cleanup_error.is_none()
    {
        cleanup_error = Some(error);
    }
    resource.mark_closed(sifr_runtime::interop::__generated_glue::token());
    if ACTIVE_TASKS.load(Ordering::SeqCst) != 0 && cleanup_error.is_none() {
        cleanup_error = Some(ResourceError::new(
            "resource tasks remained active after close",
        ));
    }
    if let Some(error) = cleanup_error {
        return Err(error);
    }
    Ok("closed".to_string())
}

impl ResourceMatrix {
    fn empty() -> Self {
        Self {
            state: Rc::new(RefCell::new(ResourceState {
                closed: false,
                http_url: None,
                http_client: None,
                sqlite: None,
                sqlite_file: None,
                redis: None,
                postgres: None,
                tasks: Vec::new(),
            })),
        }
    }
}

fn redis_client(
    address: std::net::SocketAddr,
    context: &str,
) -> Result<redis::Client, ResourceError> {
    let raw_info = format!("redis://{address}/")
        .into_connection_info()
        .map_err(|error| ResourceError::context(&format!("{context} connection info"), error))?;
    let settings = raw_info.redis_settings().clone().set_skip_set_lib_name();
    redis::Client::open(raw_info.set_redis_settings(settings))
        .map_err(|error| ResourceError::context(&format!("{context} client"), error))
}

fn redis_connection_config() -> redis::AsyncConnectionConfig {
    redis::AsyncConnectionConfig::new()
        .set_connection_timeout(Some(OPERATION_TIMEOUT))
        .set_response_timeout(Some(OPERATION_TIMEOUT))
}

fn redis_connection_manager_config() -> redis::aio::ConnectionManagerConfig {
    redis::aio::ConnectionManagerConfig::new()
        .set_number_of_retries(REDIS_RECONNECT_ATTEMPTS)
        .set_connection_timeout(Some(OPERATION_TIMEOUT))
        .set_response_timeout(Some(OPERATION_TIMEOUT))
}

fn resource_summary(http: &str, sqlite: &str, redis: &str, postgres: &str) -> String {
    format!(
        "http={http};sqlite={sqlite};redis={redis}/retries={REDIS_RECONNECT_ATTEMPTS};postgres={postgres}"
    )
}

fn postgres_config(address: std::net::SocketAddr) -> tokio_postgres::Config {
    let mut config = tokio_postgres::Config::new();
    config
        .host("127.0.0.1")
        .port(address.port())
        .user("sifr")
        .dbname("sifr")
        .connect_timeout(OPERATION_TIMEOUT);
    config
}

async fn bind_loopback(context: &str) -> Result<TcpListener, ResourceError> {
    tokio::time::timeout(OPERATION_TIMEOUT, TcpListener::bind(("127.0.0.1", 0)))
        .await
        .map_err(|_| ResourceError::new(format!("{context} bind timed out")))?
        .map_err(|error| ResourceError::context(&format!("{context} bind"), error))
}

fn ensure_open(state: &ResourceState) -> Result<(), ResourceError> {
    if state.closed {
        Err(ResourceError::new("resource is closed"))
    } else {
        Ok(())
    }
}

fn handle_error(error: HandleStateError) -> ResourceError {
    ResourceError::context("opaque handle", error)
}

fn unique_database_path() -> PathBuf {
    let id = NEXT_DATABASE_ID.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!(
        "sifr-resource-lifecycle-{}-{id}.sqlite3",
        std::process::id()
    ))
}
