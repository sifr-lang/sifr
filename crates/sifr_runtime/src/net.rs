//! Async network runtime support for generated Sifr programs.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc, LazyLock, Mutex, MutexGuard,
};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{lookup_host, TcpListener, TcpSocket, TcpStream};

use crate::timeouts::timeout_duration;

const DEFAULT_BACKLOG: u32 = 1024;
const MAX_READ_BYTES: i64 = 1_048_576;

type SharedListener = Arc<TcpListener>;
type SharedReadHalf = Arc<tokio::sync::Mutex<OwnedReadHalf>>;
type SharedWriteHalf = Arc<tokio::sync::Mutex<OwnedWriteHalf>>;

#[derive(Clone)]
struct AddressPair {
    local: String,
    remote: String,
}

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);
static STREAMS: LazyLock<Mutex<HashMap<i64, TcpStream>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static LISTENERS: LazyLock<Mutex<HashMap<i64, SharedListener>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static READ_HALVES: LazyLock<Mutex<HashMap<i64, SharedReadHalf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static WRITE_HALVES: LazyLock<Mutex<HashMap<i64, SharedWriteHalf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static STREAM_ADDRS: LazyLock<Mutex<HashMap<i64, AddressPair>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static LISTENER_LOCAL_ADDRS: LazyLock<Mutex<HashMap<i64, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static WRITE_SHUTDOWN: LazyLock<Mutex<HashSet<i64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn next_handle() -> Result<i64, String> {
    loop {
        let current = NEXT_HANDLE.load(Ordering::Relaxed);
        if current == i64::MAX {
            return Err("network handle table is exhausted".to_string());
        }
        if NEXT_HANDLE
            .compare_exchange(current, current + 1, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            return Ok(current);
        }
    }
}

fn next_handle_infallible() -> i64 {
    loop {
        let current = NEXT_HANDLE.load(Ordering::Relaxed);
        let next = if current == i64::MAX { 1 } else { current + 1 };
        if NEXT_HANDLE
            .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
            && current > 0
        {
            return current;
        }
    }
}

async fn with_optional_timeout<T, F>(
    operation: F,
    seconds: f64,
    has_timeout: bool,
    name: &str,
) -> Result<T, String>
where
    F: Future<Output = Result<T, String>>,
{
    if !has_timeout {
        return operation.await;
    }
    match tokio::time::timeout(timeout_duration(seconds, "network")?, operation).await {
        Ok(result) => result,
        Err(_) => Err(format!("{name} timed out")),
    }
}

async fn resolve_socket_addrs(address: &str) -> Result<Vec<SocketAddr>, String> {
    let addrs: Vec<SocketAddr> = lookup_host(address)
        .await
        .map_err(|error| format!("failed to resolve address {address}: {error}"))?
        .collect();
    if addrs.is_empty() {
        return Err(format!("address {address} resolved to no socket addresses"));
    }
    Ok(addrs)
}

fn insert_stream(stream: TcpStream) -> Result<i64, String> {
    let local = stream
        .local_addr()
        .map_err(|error| format!("failed to inspect local address: {error}"))?
        .to_string();
    let remote = stream
        .peer_addr()
        .map_err(|error| format!("failed to inspect remote address: {error}"))?
        .to_string();
    let handle = next_handle()?;
    lock(&STREAMS).insert(handle, stream);
    lock(&STREAM_ADDRS).insert(handle, AddressPair { local, remote });
    Ok(handle)
}

async fn connect_with_local_addr(address: String, local_addr: String) -> Result<TcpStream, String> {
    let remote_addrs = resolve_socket_addrs(&address).await?;
    let local: SocketAddr = local_addr
        .parse()
        .map_err(|error| format!("invalid local address {local_addr}: {error}"))?;
    let mut last_error = None;
    for remote in remote_addrs {
        let socket = if remote.is_ipv4() {
            TcpSocket::new_v4()
        } else {
            TcpSocket::new_v6()
        }
        .map_err(|error| format!("failed to create TCP socket: {error}"))?;
        if local.is_ipv4() != remote.is_ipv4() {
            last_error = Some(format!(
                "local address {local} and remote address {remote} use different address families"
            ));
            continue;
        }
        if let Err(error) = socket.bind(local) {
            last_error = Some(format!("failed to bind local address {local}: {error}"));
            continue;
        }
        match socket.connect(remote).await {
            Ok(stream) => return Ok(stream),
            Err(error) => last_error = Some(format!("failed to connect to {remote}: {error}")),
        }
    }
    Err(last_error.unwrap_or_else(|| format!("failed to connect to {address}")))
}

async fn connect_operation(
    address: String,
    local_addr: String,
    has_local_addr: bool,
) -> Result<i64, String> {
    let stream = if has_local_addr {
        connect_with_local_addr(address, local_addr).await?
    } else {
        TcpStream::connect(address.as_str())
            .await
            .map_err(|error| format!("failed to connect to {address}: {error}"))?
    };
    stream
        .set_nodelay(true)
        .map_err(|error| format!("failed to configure TCP stream: {error}"))?;
    insert_stream(stream)
}

pub async fn connect_tcp(
    address: String,
    timeout_seconds: f64,
    has_timeout: bool,
    local_addr: String,
    has_local_addr: bool,
) -> Result<i64, String> {
    with_optional_timeout(
        connect_operation(address, local_addr, has_local_addr),
        timeout_seconds,
        has_timeout,
        "TCP connect",
    )
    .await
}

async fn listen_operation(
    address: String,
    backlog: i64,
    has_backlog: bool,
    reuse_addr: bool,
) -> Result<i64, String> {
    let addrs = resolve_socket_addrs(&address).await?;
    let bind_addr = addrs[0];
    let socket = if bind_addr.is_ipv4() {
        TcpSocket::new_v4()
    } else {
        TcpSocket::new_v6()
    }
    .map_err(|error| format!("failed to create TCP listener socket: {error}"))?;
    if reuse_addr {
        socket
            .set_reuseaddr(true)
            .map_err(|error| format!("failed to set SO_REUSEADDR: {error}"))?;
    }
    socket
        .bind(bind_addr)
        .map_err(|error| format!("failed to bind TCP listener {bind_addr}: {error}"))?;
    let backlog = if has_backlog {
        if backlog <= 0 || backlog > i64::from(u32::MAX) {
            return Err("TCP listener backlog must be positive and fit in u32".to_string());
        }
        u32::try_from(backlog).map_err(|error| format!("invalid TCP backlog: {error}"))?
    } else {
        DEFAULT_BACKLOG
    };
    let listener = socket
        .listen(backlog)
        .map_err(|error| format!("failed to listen on {bind_addr}: {error}"))?;
    let local = listener
        .local_addr()
        .map_err(|error| format!("failed to inspect listener address: {error}"))?
        .to_string();
    let handle = next_handle()?;
    lock(&LISTENERS).insert(handle, Arc::new(listener));
    lock(&LISTENER_LOCAL_ADDRS).insert(handle, local);
    Ok(handle)
}

pub async fn listen_tcp(
    address: String,
    backlog: i64,
    has_backlog: bool,
    reuse_addr: bool,
) -> Result<i64, String> {
    listen_operation(address, backlog, has_backlog, reuse_addr).await
}

pub async fn accept_tcp(handle: i64) -> Result<(i64, String), String> {
    let listener = lock(&LISTENERS)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TCP listener handle is closed or unknown: {handle}"))?;
    let (stream, remote) = listener
        .accept()
        .await
        .map_err(|error| format!("failed to accept TCP connection: {error}"))?;
    let stream_handle = insert_stream(stream)?;
    Ok((stream_handle, remote.to_string()))
}

pub fn tcp_listener_local_addr(handle: i64) -> Result<String, String> {
    lock(&LISTENER_LOCAL_ADDRS)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TCP listener handle is closed or unknown: {handle}"))
}

pub fn close_tcp_listener(handle: i64) -> Result<(), String> {
    let removed = lock(&LISTENERS).remove(&handle);
    lock(&LISTENER_LOCAL_ADDRS).remove(&handle);
    removed
        .map(|_| ())
        .ok_or_else(|| format!("TCP listener handle is closed or unknown: {handle}"))
}

fn take_stream(handle: i64) -> Result<TcpStream, String> {
    lock(&STREAMS)
        .remove(&handle)
        .ok_or_else(|| format!("TCP stream handle is closed or unknown: {handle}"))
}

fn restore_stream(handle: i64, stream: TcpStream) {
    lock(&STREAMS).insert(handle, stream);
}

pub(crate) fn consume_stream_for_tls(handle: i64) -> Result<TcpStream, String> {
    let stream = take_stream(handle)?;
    lock(&STREAM_ADDRS).remove(&handle);
    lock(&WRITE_SHUTDOWN).remove(&handle);
    Ok(stream)
}

#[cfg(feature = "http")]
pub(crate) fn consume_stream_for_http(handle: i64) -> Result<TcpStream, String> {
    consume_stream_for_tls(handle)
}

fn ensure_write_open(handle: i64) -> Result<(), String> {
    if lock(&WRITE_SHUTDOWN).contains(&handle) {
        return Err("TCP write side is already shut down".to_string());
    }
    Ok(())
}

fn validate_read_size(max_bytes: i64) -> Result<usize, String> {
    if max_bytes <= 0 {
        return Err("TCP read size must be positive".to_string());
    }
    if max_bytes > MAX_READ_BYTES {
        return Err(format!("TCP read size exceeds {MAX_READ_BYTES} bytes"));
    }
    usize::try_from(max_bytes).map_err(|error| format!("invalid TCP read size: {error}"))
}

pub async fn tcp_stream_read_chunk(handle: i64, max_bytes: i64) -> Result<Option<Vec<u8>>, String> {
    let mut buf = vec![0_u8; validate_read_size(max_bytes)?];
    let mut stream = take_stream(handle)?;
    let read = match stream.read(&mut buf).await {
        Ok(read) => read,
        Err(error) => {
            restore_stream(handle, stream);
            return Err(format!("failed to read TCP stream: {error}"));
        }
    };
    restore_stream(handle, stream);
    if read == 0 {
        return Ok(None);
    }
    buf.truncate(read);
    Ok(Some(buf))
}

pub async fn tcp_stream_write(handle: i64, data: Vec<u8>) -> Result<i64, String> {
    ensure_write_open(handle)?;
    let mut stream = take_stream(handle)?;
    let written = match stream.write(&data).await {
        Ok(written) => written,
        Err(error) => {
            restore_stream(handle, stream);
            return Err(format!("failed to write TCP stream: {error}"));
        }
    };
    restore_stream(handle, stream);
    i64::try_from(written).map_err(|error| format!("invalid TCP write count: {error}"))
}

pub async fn tcp_stream_write_all(handle: i64, data: Vec<u8>) -> Result<(), String> {
    ensure_write_open(handle)?;
    let mut stream = take_stream(handle)?;
    let result = stream
        .write_all(&data)
        .await
        .map_err(|error| format!("failed to write TCP stream: {error}"));
    restore_stream(handle, stream);
    result
}

pub async fn tcp_stream_shutdown_write(handle: i64) -> Result<(), String> {
    if lock(&WRITE_SHUTDOWN).contains(&handle) {
        return Ok(());
    }
    let mut stream = take_stream(handle)?;
    let result = stream
        .shutdown()
        .await
        .map_err(|error| format!("failed to shut down TCP write side: {error}"));
    restore_stream(handle, stream);
    result?;
    lock(&WRITE_SHUTDOWN).insert(handle);
    Ok(())
}

pub async fn tcp_stream_close(handle: i64) -> Result<(), String> {
    let removed = lock(&STREAMS).remove(&handle);
    lock(&STREAM_ADDRS).remove(&handle);
    lock(&WRITE_SHUTDOWN).remove(&handle);
    removed
        .map(|_| ())
        .ok_or_else(|| format!("TCP stream handle is closed or unknown: {handle}"))
}

pub fn tcp_stream_split(handle: i64) -> (i64, i64) {
    let read_handle = next_handle_infallible();
    let write_handle = next_handle_infallible();
    let Some(stream) = lock(&STREAMS).remove(&handle) else {
        return (read_handle, write_handle);
    };
    lock(&STREAM_ADDRS).remove(&handle);
    let was_shutdown = lock(&WRITE_SHUTDOWN).remove(&handle);
    let (read_half, write_half) = stream.into_split();
    lock(&READ_HALVES).insert(read_handle, Arc::new(tokio::sync::Mutex::new(read_half)));
    lock(&WRITE_HALVES).insert(write_handle, Arc::new(tokio::sync::Mutex::new(write_half)));
    if was_shutdown {
        lock(&WRITE_SHUTDOWN).insert(write_handle);
    }
    (read_handle, write_handle)
}

pub fn tcp_stream_local_addr(handle: i64) -> Result<String, String> {
    lock(&STREAM_ADDRS)
        .get(&handle)
        .map(|addr| addr.local.clone())
        .ok_or_else(|| format!("TCP stream handle is closed or unknown: {handle}"))
}

pub fn tcp_stream_remote_addr(handle: i64) -> Result<String, String> {
    lock(&STREAM_ADDRS)
        .get(&handle)
        .map(|addr| addr.remote.clone())
        .ok_or_else(|| format!("TCP stream handle is closed or unknown: {handle}"))
}

pub async fn tcp_read_half_read_chunk(
    handle: i64,
    max_bytes: i64,
) -> Result<Option<Vec<u8>>, String> {
    let reader = lock(&READ_HALVES)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TCP read half handle is closed or unknown: {handle}"))?;
    let mut guard = reader.lock().await;
    let mut buf = vec![0_u8; validate_read_size(max_bytes)?];
    let read = guard
        .read(&mut buf)
        .await
        .map_err(|error| format!("failed to read TCP read half: {error}"))?;
    if read == 0 {
        return Ok(None);
    }
    buf.truncate(read);
    Ok(Some(buf))
}

pub fn tcp_read_half_close(handle: i64) -> Result<(), String> {
    lock(&READ_HALVES)
        .remove(&handle)
        .map(|_| ())
        .ok_or_else(|| format!("TCP read half handle is closed or unknown: {handle}"))
}

fn write_half_handle(handle: i64) -> Result<SharedWriteHalf, String> {
    lock(&WRITE_HALVES)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TCP write half handle is closed or unknown: {handle}"))
}

pub async fn tcp_write_half_write(handle: i64, data: Vec<u8>) -> Result<i64, String> {
    ensure_write_open(handle)?;
    let writer = write_half_handle(handle)?;
    let mut guard = writer.lock().await;
    let written = guard
        .write(&data)
        .await
        .map_err(|error| format!("failed to write TCP write half: {error}"))?;
    i64::try_from(written).map_err(|error| format!("invalid TCP write count: {error}"))
}

pub async fn tcp_write_half_write_all(handle: i64, data: Vec<u8>) -> Result<(), String> {
    ensure_write_open(handle)?;
    let writer = write_half_handle(handle)?;
    let mut guard = writer.lock().await;
    guard
        .write_all(&data)
        .await
        .map_err(|error| format!("failed to write TCP write half: {error}"))
}

pub async fn tcp_write_half_shutdown_write(handle: i64) -> Result<(), String> {
    if lock(&WRITE_SHUTDOWN).contains(&handle) {
        return Ok(());
    }
    let writer = write_half_handle(handle)?;
    let mut guard = writer.lock().await;
    guard
        .shutdown()
        .await
        .map_err(|error| format!("failed to shut down TCP write half: {error}"))?;
    lock(&WRITE_SHUTDOWN).insert(handle);
    Ok(())
}

pub fn tcp_write_half_close(handle: i64) -> Result<(), String> {
    let removed = lock(&WRITE_HALVES).remove(&handle);
    lock(&WRITE_SHUTDOWN).remove(&handle);
    removed
        .map(|_| ())
        .ok_or_else(|| format!("TCP write half handle is closed or unknown: {handle}"))
}

pub async fn resolve_host(
    address: String,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<Vec<String>, String> {
    with_optional_timeout(
        async move {
            resolve_socket_addrs(&address)
                .await
                .map(|addrs| addrs.into_iter().map(|addr| addr.to_string()).collect())
        },
        timeout_seconds,
        has_timeout,
        "DNS resolution",
    )
    .await
}
