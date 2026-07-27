use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

const OPERATION_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_REQUEST_BYTES: usize = 16 * 1024;

static ACTIVE_REQUESTS: AtomicUsize = AtomicUsize::new(0);
static ACTIVE_SERVERS: AtomicUsize = AtomicUsize::new(0);
static COMPLETED_REQUESTS: AtomicUsize = AtomicUsize::new(0);
static CANCELLED_REQUESTS: AtomicUsize = AtomicUsize::new(0);
static RUNTIME_CALLS: AtomicUsize = AtomicUsize::new(0);
static RUNTIME_REUSED: AtomicBool = AtomicBool::new(true);
static RUNTIME_ID: OnceLock<String> = OnceLock::new();
static RUNTIME_THREAD: OnceLock<String> = OnceLock::new();

#[derive(Debug)]
pub struct HttpBridgeError {
    message: String,
}

impl HttpBridgeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn context(context: &str, error: impl fmt::Display) -> Self {
        Self::new(format!("{context}: {error}"))
    }
}

impl fmt::Display for HttpBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for HttpBridgeError {}

struct RequestGuard {
    completed: bool,
}

impl RequestGuard {
    fn enter() -> Self {
        ACTIVE_REQUESTS.fetch_add(1, Ordering::SeqCst);
        Self { completed: false }
    }

    fn complete(&mut self) {
        self.completed = true;
        COMPLETED_REQUESTS.fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        ACTIVE_REQUESTS.fetch_sub(1, Ordering::SeqCst);
        if !self.completed {
            CANCELLED_REQUESTS.fetch_add(1, Ordering::SeqCst);
        }
    }
}

struct ServerGuard {
    handle: Option<JoinHandle<Result<(), HttpBridgeError>>>,
}

struct ServerActivity;

impl ServerActivity {
    fn enter() -> Self {
        ACTIVE_SERVERS.fetch_add(1, Ordering::SeqCst);
        Self
    }
}

impl Drop for ServerActivity {
    fn drop(&mut self) {
        ACTIVE_SERVERS.fetch_sub(1, Ordering::SeqCst);
    }
}

impl ServerGuard {
    fn new(handle: JoinHandle<Result<(), HttpBridgeError>>) -> Self {
        Self {
            handle: Some(handle),
        }
    }

    async fn finish(&mut self) -> Result<(), HttpBridgeError> {
        let Some(handle) = self.handle.take() else {
            return Ok(());
        };
        let abort = handle.abort_handle();
        let joined = tokio::time::timeout(OPERATION_TIMEOUT, handle).await;
        match joined {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => Err(HttpBridgeError::context("loopback task join failed", error)),
            Err(_) => {
                abort.abort();
                Err(HttpBridgeError::new("loopback task join timed out"))
            }
        }
    }
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

pub async fn request_roundtrip(
    payload: &str,
    mode: &str,
) -> Result<String, HttpBridgeError> {
    note_generated_runtime()?;
    let mut request = RequestGuard::enter();
    let listener = tokio::time::timeout(
        OPERATION_TIMEOUT,
        TcpListener::bind(("127.0.0.1", 0)),
    )
    .await
    .map_err(|_| HttpBridgeError::new("loopback bind timed out"))?
    .map_err(|error| HttpBridgeError::context("loopback bind failed", error))?;
    let address = listener
        .local_addr()
        .map_err(|error| HttpBridgeError::context("loopback address failed", error))?;
    let response_body = format!("echo:{payload}");
    let delay = if mode == "slow" {
        Duration::from_millis(500)
    } else {
        Duration::ZERO
    };
    let server_activity = ServerActivity::enter();
    let mut server = ServerGuard::new(tokio::spawn(serve_once(
        listener,
        response_body,
        delay,
        server_activity,
    )));
    let client = reqwest::Client::builder()
        .no_proxy()
        .timeout(OPERATION_TIMEOUT)
        .build()
        .map_err(|error| HttpBridgeError::context("reqwest client failed", error))?;
    let response = client
        .post(format!("http://{address}/echo"))
        .body(payload.to_string())
        .send()
        .await
        .map_err(|error| HttpBridgeError::context("reqwest request failed", error))?;
    if !response.status().is_success() {
        return Err(HttpBridgeError::new(format!(
            "loopback status was {}",
            response.status()
        )));
    }
    let body = response
        .text()
        .await
        .map_err(|error| HttpBridgeError::context("reqwest body failed", error))?;
    server.finish().await?;
    request.complete();
    Ok(body)
}

pub fn runtime_snapshot() -> String {
    format!(
        "active_requests={};active_servers={};completed={};cancelled={};runtime_calls={};runtime_reused={}",
        ACTIVE_REQUESTS.load(Ordering::SeqCst),
        ACTIVE_SERVERS.load(Ordering::SeqCst),
        COMPLETED_REQUESTS.load(Ordering::SeqCst),
        CANCELLED_REQUESTS.load(Ordering::SeqCst),
        RUNTIME_CALLS.load(Ordering::SeqCst),
        RUNTIME_REUSED.load(Ordering::SeqCst),
    )
}

fn note_generated_runtime() -> Result<(), HttpBridgeError> {
    let handle = tokio::runtime::Handle::try_current()
        .map_err(|error| HttpBridgeError::context("generated Tokio runtime missing", error))?;
    if handle.runtime_flavor() != tokio::runtime::RuntimeFlavor::CurrentThread {
        return Err(HttpBridgeError::new(
            "generated Tokio runtime is not current-thread",
        ));
    }
    let current_runtime = format!("{:?}", handle.id());
    let first_runtime = RUNTIME_ID.get_or_init(|| current_runtime.clone());
    if first_runtime != &current_runtime {
        RUNTIME_REUSED.store(false, Ordering::SeqCst);
    }
    let current_thread = format!("{:?}", std::thread::current().id());
    let first_thread = RUNTIME_THREAD.get_or_init(|| current_thread.clone());
    if first_thread != &current_thread {
        RUNTIME_REUSED.store(false, Ordering::SeqCst);
    }
    RUNTIME_CALLS.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

async fn serve_once(
    listener: TcpListener,
    response_body: String,
    delay: Duration,
    _activity: ServerActivity,
) -> Result<(), HttpBridgeError> {
    let (mut stream, _) = tokio::time::timeout(OPERATION_TIMEOUT, listener.accept())
        .await
        .map_err(|_| HttpBridgeError::new("loopback accept timed out"))?
        .map_err(|error| HttpBridgeError::context("loopback accept failed", error))?;
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = tokio::time::timeout(OPERATION_TIMEOUT, stream.read(&mut buffer))
            .await
            .map_err(|_| HttpBridgeError::new("loopback read timed out"))?
            .map_err(|error| HttpBridgeError::context("loopback read failed", error))?;
        if read == 0 {
            return Err(HttpBridgeError::new(
                "loopback client closed before request headers",
            ));
        }
        request.extend_from_slice(&buffer[..read]);
        if request.len() > MAX_REQUEST_BYTES {
            return Err(HttpBridgeError::new("loopback request headers too large"));
        }
    }
    tokio::time::sleep(delay).await;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response_body.len(),
        response_body
    );
    tokio::time::timeout(OPERATION_TIMEOUT, stream.write_all(response.as_bytes()))
        .await
        .map_err(|_| HttpBridgeError::new("loopback write timed out"))?
        .map_err(|error| HttpBridgeError::context("loopback write failed", error))?;
    tokio::time::timeout(OPERATION_TIMEOUT, stream.shutdown())
        .await
        .map_err(|_| HttpBridgeError::new("loopback shutdown timed out"))?
        .map_err(|error| HttpBridgeError::context("loopback shutdown failed", error))?;
    Ok(())
}
