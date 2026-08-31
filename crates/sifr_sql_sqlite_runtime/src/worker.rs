use crate::config::{SqliteEvidence, SqliteProfile};
use rusqlite::types::{Value, ValueRef};
use rusqlite::{Connection, InterruptHandle, OpenFlags, params_from_iter};
use sifr_runtime::cancellation::CancellationClaimLease;
use sifr_sql_runtime::{
    CancellationCarrier, OwnedSqlValue, ResourceUsage, RuntimeLimits, SchemaDependencySlice,
    SchemaProperty, SqlError, SqlErrorKind,
};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::sync::oneshot;

pub(crate) struct WorkerHandle {
    sender: mpsc::SyncSender<Command>,
    interrupt: Arc<InterruptHandle>,
    poisoned: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

pub(crate) struct WorkerRowStream {
    rows: tokio_mpsc::Receiver<Result<SqliteRow, SqlError>>,
    completion: oneshot::Receiver<Result<(), SqlError>>,
    claim: Option<CancellationClaimLease>,
    cancelled: Arc<AtomicBool>,
    interrupt: Arc<InterruptHandle>,
    poisoned: Arc<AtomicBool>,
    deadline: tokio::time::Instant,
    finished: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SqliteRow {
    values: Vec<OwnedSqlValue>,
    decoded_bytes: u64,
}

impl SqliteRow {
    #[must_use]
    pub fn values(&self) -> &[OwnedSqlValue] {
        &self.values
    }

    #[must_use]
    pub const fn decoded_bytes(&self) -> u64 {
        self.decoded_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SqliteExecutionMetadata {
    pub statement_cache_hit: bool,
    pub last_insert_rowid: i64,
    pub changes: u64,
    pub sqlite_version: (u16, u16, u16),
}

enum Command {
    Execute {
        statement: String,
        parameters: Vec<OwnedSqlValue>,
        response: oneshot::Sender<Result<SqliteExecutionMetadata, SqlError>>,
    },
    Fetch {
        statement: String,
        parameters: Vec<OwnedSqlValue>,
        limits: RuntimeLimits,
        response: oneshot::Sender<Result<Vec<SqliteRow>, SqlError>>,
    },
    Control {
        statement: String,
        response: oneshot::Sender<Result<(), SqlError>>,
    },
    Reset {
        response: oneshot::Sender<Result<(), SqlError>>,
    },
    Observe {
        evidence: SqliteEvidence,
        response: oneshot::Sender<Result<SchemaDependencySlice, SqlError>>,
    },
    Stream {
        statement: String,
        parameters: Vec<OwnedSqlValue>,
        limits: RuntimeLimits,
        rows: tokio_mpsc::Sender<Result<SqliteRow, SqlError>>,
        ready: oneshot::Sender<Result<(), SqlError>>,
        completion: oneshot::Sender<Result<(), SqlError>>,
    },
    Shutdown,
}

impl WorkerHandle {
    pub(crate) fn open(profile: &SqliteProfile) -> Result<Self, SqlError> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let path = profile.path().to_path_buf();
        let limits = profile.limits();
        let busy_timeout = Duration::from_millis(u64::from(profile.busy_timeout_ms()));
        let required_features = profile.required_features().to_vec();
        let poisoned = Arc::new(AtomicBool::new(false));
        let worker_poisoned = Arc::clone(&poisoned);
        let join = thread::Builder::new()
            .name("sifr-sqlite-worker".to_string())
            .spawn(move || {
                let opened = open_connection(&path, limits, busy_timeout, &required_features);
                let Ok(connection) = opened else {
                    let _ = ready_sender.send(Err(opened.err().unwrap_or_else(provider_error)));
                    return;
                };
                let interrupt = Arc::new(connection.get_interrupt_handle());
                if ready_sender.send(Ok(Arc::clone(&interrupt))).is_err() {
                    return;
                }
                run_worker(
                    &connection,
                    &receiver,
                    &worker_poisoned,
                    limits.statement_cache_capacity as usize,
                );
            })
            .map_err(|_| SqlError::new(SqlErrorKind::Connection))?;
        let interrupt = ready_receiver
            .recv_timeout(profile.limits().acquire_timeout)
            .map_err(|_| SqlError::new(SqlErrorKind::Connection))??;
        Ok(Self {
            sender,
            interrupt,
            poisoned,
            join: Some(join),
        })
    }

    pub(crate) async fn execute(
        &self,
        statement: String,
        parameters: Vec<OwnedSqlValue>,
        timeout: Duration,
        cancellation: Option<&CancellationCarrier>,
    ) -> Result<SqliteExecutionMetadata, SqlError> {
        let (response, receiver) = oneshot::channel();
        self.send(Command::Execute {
            statement,
            parameters,
            response,
        })?;
        self.wait(receiver, timeout, cancellation).await
    }

    pub(crate) async fn fetch(
        &self,
        statement: String,
        parameters: Vec<OwnedSqlValue>,
        limits: RuntimeLimits,
        timeout: Duration,
        cancellation: Option<&CancellationCarrier>,
    ) -> Result<Vec<SqliteRow>, SqlError> {
        let (response, receiver) = oneshot::channel();
        self.send(Command::Fetch {
            statement,
            parameters,
            limits,
            response,
        })?;
        self.wait(receiver, timeout, cancellation).await
    }

    pub(crate) async fn control(
        &self,
        statement: impl Into<String>,
        timeout: Duration,
    ) -> Result<(), SqlError> {
        let (response, receiver) = oneshot::channel();
        self.send(Command::Control {
            statement: statement.into(),
            response,
        })?;
        self.wait(receiver, timeout, None).await
    }

    pub(crate) async fn reset(&self, timeout: Duration) -> Result<(), SqlError> {
        let (response, receiver) = oneshot::channel();
        self.send(Command::Reset { response })?;
        self.wait(receiver, timeout, None).await
    }

    pub(crate) async fn observe(
        &self,
        evidence: SqliteEvidence,
        timeout: Duration,
    ) -> Result<SchemaDependencySlice, SqlError> {
        let (response, receiver) = oneshot::channel();
        self.send(Command::Observe { evidence, response })?;
        self.wait(receiver, timeout, None).await
    }

    pub(crate) async fn stream(
        &self,
        statement: String,
        parameters: Vec<OwnedSqlValue>,
        limits: RuntimeLimits,
        timeout: Duration,
        cancellation: Option<&CancellationCarrier>,
    ) -> Result<WorkerRowStream, SqlError> {
        let deadline = tokio::time::Instant::now() + timeout;
        let (rows_sender, rows) = tokio_mpsc::channel(1);
        let (ready_sender, ready) = oneshot::channel();
        let (completion_sender, completion) = oneshot::channel();
        let cancelled = Arc::new(AtomicBool::new(false));
        let claim = if let Some(carrier) = cancellation {
            let interrupt = Arc::clone(&self.interrupt);
            let cancelled_hook = Arc::clone(&cancelled);
            let poisoned = Arc::clone(&self.poisoned);
            Some(
                carrier
                    .claim(Arc::new(move || {
                        cancelled_hook.store(true, Ordering::Release);
                        poisoned.store(true, Ordering::Release);
                        interrupt.interrupt();
                    }))
                    .map_err(|_| SqlError::new(SqlErrorKind::Cancelled))?,
            )
        } else {
            None
        };
        self.send(Command::Stream {
            statement,
            parameters,
            limits,
            rows: rows_sender,
            ready: ready_sender,
            completion: completion_sender,
        })?;
        match tokio::time::timeout(timeout, ready).await {
            Ok(Ok(Ok(()))) => Ok(WorkerRowStream {
                rows,
                completion,
                claim,
                cancelled,
                interrupt: Arc::clone(&self.interrupt),
                poisoned: Arc::clone(&self.poisoned),
                deadline,
                finished: false,
            }),
            Ok(Ok(Err(error))) => Err(error),
            Ok(Err(_)) => {
                self.poisoned.store(true, Ordering::Release);
                Err(SqlError::new(SqlErrorKind::Connection))
            }
            Err(_) => {
                self.interrupt.interrupt();
                self.poisoned.store(true, Ordering::Release);
                Err(SqlError::new(SqlErrorKind::Timeout))
            }
        }
    }

    #[must_use]
    pub(crate) fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }

    fn send(&self, command: Command) -> Result<(), SqlError> {
        if self.is_poisoned() {
            return Err(SqlError::new(SqlErrorKind::Connection));
        }
        self.sender
            .try_send(command)
            .map_err(|_| SqlError::new(SqlErrorKind::ResourceLimit))
    }

    async fn wait<T>(
        &self,
        receiver: oneshot::Receiver<Result<T, SqlError>>,
        timeout: Duration,
        cancellation: Option<&CancellationCarrier>,
    ) -> Result<T, SqlError> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let claim = if let Some(carrier) = cancellation {
            let interrupt = Arc::clone(&self.interrupt);
            let cancelled_hook = Arc::clone(&cancelled);
            Some(
                carrier
                    .claim(Arc::new(move || {
                        cancelled_hook.store(true, Ordering::Release);
                        interrupt.interrupt();
                    }))
                    .map_err(|_| SqlError::new(SqlErrorKind::Cancelled))?,
            )
        } else {
            None
        };
        let outcome = tokio::time::timeout(timeout, receiver).await;
        drop(claim);
        if cancelled.load(Ordering::Acquire) {
            self.poisoned.store(true, Ordering::Release);
            return Err(SqlError::new(SqlErrorKind::Cancelled));
        }
        match outcome {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => {
                self.poisoned.store(true, Ordering::Release);
                Err(SqlError::new(SqlErrorKind::Connection))
            }
            Err(_) => {
                self.interrupt.interrupt();
                self.poisoned.store(true, Ordering::Release);
                Err(SqlError::new(SqlErrorKind::Timeout))
            }
        }
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.interrupt.interrupt();
        let _ = self.sender.try_send(Command::Shutdown);
        self.join.take();
    }
}

fn open_connection(
    path: &std::path::Path,
    limits: RuntimeLimits,
    busy_timeout: Duration,
    required_features: &[String],
) -> Result<Connection, SqlError> {
    if rusqlite::version_number() != 3_053_002 {
        return Err(SqlError::new(SqlErrorKind::Configuration));
    }
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| SqlError::new(SqlErrorKind::Connection))?;
    connection
        .busy_timeout(busy_timeout)
        .map_err(map_sqlite_error)?;
    connection.set_prepared_statement_cache_capacity(limits.statement_cache_capacity as usize);
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(map_sqlite_error)?;
    connection
        .pragma_update(None, "trusted_schema", false)
        .map_err(map_sqlite_error)?;
    connection
        .pragma_update(None, "recursive_triggers", true)
        .map_err(map_sqlite_error)?;
    verify_features(&connection, required_features)?;
    Ok(connection)
}

fn verify_features(connection: &Connection, required: &[String]) -> Result<(), SqlError> {
    for feature in required {
        let available = match feature.as_str() {
            "json" | "json1" => connection
                .query_row("SELECT json_valid('[]')", [], |_| Ok(()))
                .is_ok(),
            "fts5" => compile_option_used(connection, "ENABLE_FTS5"),
            "rtree" => compile_option_used(connection, "ENABLE_RTREE"),
            "math" => compile_option_used(connection, "ENABLE_MATH_FUNCTIONS"),
            _ => false,
        };
        if !available {
            return Err(SqlError::new(SqlErrorKind::Configuration));
        }
    }
    Ok(())
}

fn compile_option_used(connection: &Connection, option: &str) -> bool {
    connection
        .query_row("SELECT sqlite_compileoption_used(?1)", [option], |row| {
            row.get::<_, i64>(0)
        })
        .is_ok_and(|value| value == 1)
}

fn run_worker(
    connection: &Connection,
    receiver: &mpsc::Receiver<Command>,
    poisoned: &AtomicBool,
    statement_cache_capacity: usize,
) {
    let mut statement_cache = StatementCacheTracker::new(statement_cache_capacity);
    while let Ok(command) = receiver.recv() {
        match command {
            Command::Execute {
                statement,
                parameters,
                response,
            } => {
                let result = execute(connection, &mut statement_cache, &statement, parameters);
                let _ = response.send(result);
            }
            Command::Fetch {
                statement,
                parameters,
                limits,
                response,
            } => {
                let result = fetch(
                    connection,
                    &mut statement_cache,
                    &statement,
                    parameters,
                    limits,
                );
                let _ = response.send(result);
            }
            Command::Control {
                statement,
                response,
            } => {
                let result = connection
                    .execute_batch(&statement)
                    .map_err(map_sqlite_error);
                let _ = response.send(result);
            }
            Command::Reset { response } => {
                let result = reset(connection, &mut statement_cache);
                let _ = response.send(result);
            }
            Command::Observe { evidence, response } => {
                let result = observe_schema(connection, &evidence);
                let _ = response.send(result);
            }
            Command::Stream {
                statement,
                parameters,
                limits,
                rows,
                ready,
                completion,
            } => {
                let result = stream_rows(
                    connection,
                    &mut statement_cache,
                    &statement,
                    parameters,
                    limits,
                    &rows,
                    ready,
                );
                let _ = completion.send(result);
            }
            Command::Shutdown => break,
        }
        if poisoned.load(Ordering::Acquire) {
            break;
        }
    }
}

impl WorkerRowStream {
    pub(crate) async fn next(&mut self) -> Result<Option<SqliteRow>, SqlError> {
        if self.finished {
            return Ok(None);
        }
        if self.cancelled.load(Ordering::Acquire) {
            self.finished = true;
            return Err(SqlError::new(SqlErrorKind::Cancelled));
        }
        let remaining = self.remaining()?;
        match tokio::time::timeout(remaining, self.rows.recv()).await {
            Ok(Some(Ok(row))) => Ok(Some(row)),
            Ok(Some(Err(error))) => {
                self.finished = true;
                Err(error)
            }
            Ok(None) => {
                self.finished = true;
                self.await_completion().await?;
                Ok(None)
            }
            Err(_) => Err(self.timeout()),
        }
    }

    pub(crate) async fn close(&mut self) -> Result<(), SqlError> {
        if self.finished {
            return Ok(());
        }
        self.rows.close();
        let result = self.await_completion().await;
        self.finished = true;
        result
    }

    pub(crate) fn abort(&mut self) {
        self.rows.close();
        self.interrupt.interrupt();
        self.poisoned.store(true, Ordering::Release);
        self.finished = true;
    }

    fn remaining(&self) -> Result<Duration, SqlError> {
        self.deadline
            .checked_duration_since(tokio::time::Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| SqlError::new(SqlErrorKind::Timeout))
    }

    async fn await_completion(&mut self) -> Result<(), SqlError> {
        let remaining = self.remaining()?;
        match tokio::time::timeout(remaining, &mut self.completion).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(SqlError::new(SqlErrorKind::Connection)),
            Err(_) => Err(self.timeout()),
        }
    }

    fn timeout(&self) -> SqlError {
        self.interrupt.interrupt();
        self.poisoned.store(true, Ordering::Release);
        SqlError::new(SqlErrorKind::Timeout)
    }
}

impl Drop for WorkerRowStream {
    fn drop(&mut self) {
        self.rows.close();
        self.claim.take();
    }
}

#[allow(clippy::too_many_arguments)]
fn stream_rows(
    connection: &Connection,
    statement_cache: &mut StatementCacheTracker,
    statement: &str,
    parameters: Vec<OwnedSqlValue>,
    limits: RuntimeLimits,
    sender: &tokio_mpsc::Sender<Result<SqliteRow, SqlError>>,
    ready: oneshot::Sender<Result<(), SqlError>>,
) -> Result<(), SqlError> {
    let values = match encode_parameters(parameters) {
        Ok(values) => values,
        Err(error) => {
            let _ = ready.send(Err(error.clone()));
            return Err(error);
        }
    };
    let mut prepared = match connection
        .prepare_cached(statement)
        .map_err(map_sqlite_error)
    {
        Ok(prepared) => prepared,
        Err(error) => {
            let _ = ready.send(Err(error.clone()));
            return Err(error);
        }
    };
    statement_cache.record(statement);
    let column_count = prepared.column_count();
    let mut rows = match prepared
        .query(params_from_iter(values))
        .map_err(map_sqlite_error)
    {
        Ok(rows) => rows,
        Err(error) => {
            let _ = ready.send(Err(error.clone()));
            return Err(error);
        }
    };
    if ready.send(Ok(())).is_err() {
        return Ok(());
    }
    let mut usage = ResourceUsage::default();
    loop {
        let next = match rows.next().map_err(map_sqlite_error) {
            Ok(next) => next,
            Err(error) => {
                let _ = sender.blocking_send(Err(error.clone()));
                return Err(error);
            }
        };
        let Some(row) = next else {
            return Ok(());
        };
        let mut values = Vec::with_capacity(column_count);
        let mut decoded_bytes = 0_u64;
        for index in 0..column_count {
            let (value, bytes) = decode_value(row.get_ref(index).map_err(map_sqlite_error)?)?;
            decoded_bytes = decoded_bytes.saturating_add(bytes);
            values.push(value);
        }
        if let Err(error) = usage.account_row(decoded_bytes, limits) {
            let _ = sender.blocking_send(Err(error.clone()));
            return Err(error);
        }
        if sender
            .blocking_send(Ok(SqliteRow {
                values,
                decoded_bytes,
            }))
            .is_err()
        {
            return Ok(());
        }
    }
}

fn observe_schema(
    connection: &Connection,
    evidence: &SqliteEvidence,
) -> Result<SchemaDependencySlice, SqlError> {
    match evidence {
        SqliteEvidence::Introspection {
            fingerprint_statement,
            probes,
        } => {
            let fingerprint = connection
                .query_row(fingerprint_statement, [], |row| row.get::<_, String>(0))
                .map_err(map_sqlite_error)?;
            let mut properties = Vec::with_capacity(probes.len());
            for probe in probes {
                let value = connection
                    .query_row(&probe.statement, [], |row| row.get::<_, Option<String>>(0))
                    .map_err(map_sqlite_error)?;
                properties.push(SchemaProperty::new(probe.property_identity.clone(), value)?);
            }
            SchemaDependencySlice::new(fingerprint, properties)
        }
        SqliteEvidence::MigrationHead {
            head_statement,
            accepted_states,
        } => {
            let mut statement = connection
                .prepare(head_statement)
                .map_err(map_sqlite_error)?;
            let mut rows = statement.query([]).map_err(map_sqlite_error)?;
            let mut heads = Vec::new();
            while let Some(row) = rows.next().map_err(map_sqlite_error)? {
                heads.push(row.get::<_, String>(0).map_err(map_sqlite_error)?);
            }
            heads.sort();
            accepted_states
                .get(&heads.join(","))
                .cloned()
                .ok_or_else(|| SqlError::new(SqlErrorKind::SchemaContract))
        }
        SqliteEvidence::SignedManifest { .. } => Err(SqlError::new(SqlErrorKind::Configuration)),
    }
}

fn execute(
    connection: &Connection,
    statement_cache: &mut StatementCacheTracker,
    statement: &str,
    parameters: Vec<OwnedSqlValue>,
) -> Result<SqliteExecutionMetadata, SqlError> {
    let values = encode_parameters(parameters)?;
    let was_cached = statement_cache.contains(statement);
    let mut prepared = connection
        .prepare_cached(statement)
        .map_err(map_sqlite_error)?;
    statement_cache.record(statement);
    let changes = prepared
        .execute(params_from_iter(values))
        .map_err(map_sqlite_error)?;
    Ok(SqliteExecutionMetadata {
        statement_cache_hit: was_cached,
        last_insert_rowid: connection.last_insert_rowid(),
        changes: u64::try_from(changes).unwrap_or(u64::MAX),
        sqlite_version: (3, 53, 2),
    })
}

fn fetch(
    connection: &Connection,
    statement_cache: &mut StatementCacheTracker,
    statement: &str,
    parameters: Vec<OwnedSqlValue>,
    limits: RuntimeLimits,
) -> Result<Vec<SqliteRow>, SqlError> {
    let values = encode_parameters(parameters)?;
    let mut prepared = connection
        .prepare_cached(statement)
        .map_err(map_sqlite_error)?;
    statement_cache.record(statement);
    let column_count = prepared.column_count();
    let mut rows = prepared
        .query(params_from_iter(values))
        .map_err(map_sqlite_error)?;
    let mut output = Vec::new();
    let mut usage = ResourceUsage::default();
    while let Some(row) = rows.next().map_err(map_sqlite_error)? {
        let mut values = Vec::with_capacity(column_count);
        let mut decoded_bytes = 0_u64;
        for index in 0..column_count {
            let (value, bytes) = decode_value(row.get_ref(index).map_err(map_sqlite_error)?)?;
            decoded_bytes = decoded_bytes.saturating_add(bytes);
            values.push(value);
        }
        usage.account_row(decoded_bytes, limits)?;
        output.push(SqliteRow {
            values,
            decoded_bytes,
        });
    }
    Ok(output)
}

fn reset(
    connection: &Connection,
    statement_cache: &mut StatementCacheTracker,
) -> Result<(), SqlError> {
    if !connection.is_autocommit() {
        connection
            .execute_batch("ROLLBACK")
            .map_err(map_sqlite_error)?;
    }
    connection.flush_prepared_statement_cache();
    statement_cache.clear();
    connection
        .pragma_update(None, "foreign_keys", true)
        .map_err(map_sqlite_error)?;
    Ok(())
}

struct StatementCacheTracker {
    capacity: usize,
    statements: VecDeque<String>,
}

impl StatementCacheTracker {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            statements: VecDeque::with_capacity(capacity),
        }
    }

    fn contains(&self, statement: &str) -> bool {
        self.statements.iter().any(|cached| cached == statement)
    }

    fn record(&mut self, statement: &str) {
        if self.capacity == 0 {
            return;
        }
        if let Some(position) = self
            .statements
            .iter()
            .position(|cached| cached == statement)
        {
            self.statements.remove(position);
        } else if self.statements.len() == self.capacity {
            self.statements.pop_front();
        }
        self.statements.push_back(statement.to_string());
    }

    fn clear(&mut self) {
        self.statements.clear();
    }
}

fn encode_parameters(values: Vec<OwnedSqlValue>) -> Result<Vec<Value>, SqlError> {
    values.into_iter().map(encode_value).collect()
}

fn encode_value(value: OwnedSqlValue) -> Result<Value, SqlError> {
    match value {
        OwnedSqlValue::Null => Ok(Value::Null),
        OwnedSqlValue::Bool(value) => Ok(Value::Integer(i64::from(value))),
        OwnedSqlValue::Signed(value) => Ok(Value::Integer(value)),
        OwnedSqlValue::Unsigned(value) => i64::try_from(value)
            .map(Value::Integer)
            .map_err(|_| SqlError::new(SqlErrorKind::Encode)),
        OwnedSqlValue::Float(value) if value.is_finite() => Ok(Value::Real(value)),
        OwnedSqlValue::ExactInteger(value) | OwnedSqlValue::Text(value) => Ok(Value::Text(value)),
        OwnedSqlValue::Bytes(value) => Ok(Value::Blob(value.to_vec())),
        OwnedSqlValue::Float(_) | OwnedSqlValue::Sequence(_) | OwnedSqlValue::Encoded { .. } => {
            Err(SqlError::new(SqlErrorKind::Encode))
        }
    }
}

fn decode_value(value: ValueRef<'_>) -> Result<(OwnedSqlValue, u64), SqlError> {
    match value {
        ValueRef::Null => Ok((OwnedSqlValue::Null, 0)),
        ValueRef::Integer(value) => Ok((OwnedSqlValue::Signed(value), 8)),
        ValueRef::Real(value) if value.is_finite() => Ok((OwnedSqlValue::Float(value), 8)),
        ValueRef::Text(value) => {
            let text =
                std::str::from_utf8(value).map_err(|_| SqlError::new(SqlErrorKind::Decode))?;
            Ok((OwnedSqlValue::Text(text.to_string()), value.len() as u64))
        }
        ValueRef::Blob(value) => Ok((OwnedSqlValue::Bytes(Arc::from(value)), value.len() as u64)),
        ValueRef::Real(_) => Err(SqlError::new(SqlErrorKind::Decode)),
    }
}

#[allow(clippy::needless_pass_by_value)]
fn map_sqlite_error(error: rusqlite::Error) -> SqlError {
    match error {
        rusqlite::Error::SqliteFailure(inner, _)
            if inner.code == rusqlite::ErrorCode::OperationInterrupted =>
        {
            SqlError::new(SqlErrorKind::Cancelled)
        }
        rusqlite::Error::SqliteFailure(inner, _)
            if matches!(inner.code, rusqlite::ErrorCode::ConstraintViolation) =>
        {
            SqlError::new(SqlErrorKind::Constraint)
        }
        rusqlite::Error::SqliteFailure(inner, _)
            if matches!(
                inner.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ) =>
        {
            SqlError::new(SqlErrorKind::Timeout)
        }
        _ => SqlError::new(SqlErrorKind::Provider),
    }
}

fn provider_error() -> SqlError {
    SqlError::new(SqlErrorKind::Provider)
}
