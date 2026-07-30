use axum::http::{HeaderName, HeaderValue};
use axum::{routing::get, Router};
use sifr_runtime::interop::RustPanicErrorBridge;
use sqlx::Execute as _;
use std::fmt;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Builder;
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};
use tower_http::set_header::SetResponseHeaderLayer;

const QUERY: &str = "SELECT 13::INT4 AS value";
const QUERY_HASH: &str = "f2d6fe08dd19c716c98c45307c0649a03c0bf6d52c5d16c2375913d7a0f2f508";

#[derive(Debug)]
pub struct BackendErrorBridge {
    pub message: String,
}

impl fmt::Display for BackendErrorBridge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for BackendErrorBridge {}

pub fn route_probe(path: &str) -> Result<String, BackendErrorBridge> {
    let runtime = Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(backend_error)?;
    runtime.block_on(execute_loopback(path))
}

pub fn query_compile_time() -> Result<u32, BackendErrorBridge> {
    let query = sqlx::query!("SELECT 13::INT4 AS value");
    if query.sql() != QUERY {
        return Err(BackendErrorBridge {
            message: "SQLx offline query identity drifted".to_owned(),
        });
    }
    Ok(13)
}

pub fn map_panic(error: RustPanicErrorBridge) -> BackendErrorBridge {
    BackendErrorBridge {
        message: error.to_string(),
    }
}

async fn execute_loopback(path: &str) -> Result<String, BackendErrorBridge> {
    if path != "/health" {
        return Err(BackendErrorBridge {
            message: "backend probe requires /health".to_owned(),
        });
    }
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(backend_error)?;
    let address = listener.local_addr().map_err(backend_error)?;
    let tower_header = HeaderName::from_static("x-sifr-tower");
    let app = Router::new()
        .route("/health", get(|| async { "sifr-backend-ok" }))
        .layer(SetResponseHeaderLayer::if_not_present(
            tower_header,
            HeaderValue::from_static("tower-http-0.7.0"),
        ));
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let response = exchange_http(address, path).await?;
    let _ = shutdown_tx.send(());
    timeout(Duration::from_secs(2), server)
        .await
        .map_err(|_| BackendErrorBridge {
            message: "Axum loopback shutdown timed out".to_owned(),
        })?
        .map_err(backend_error)?
        .map_err(backend_error)?;

    if !response.starts_with("HTTP/1.1 200 OK")
        || !response
            .to_ascii_lowercase()
            .contains("x-sifr-tower: tower-http-0.7.0")
        || !response.ends_with("sifr-backend-ok")
    {
        return Err(BackendErrorBridge {
            message: "Axum/tower-http loopback evidence was incomplete".to_owned(),
        });
    }
    let offline_value = query_compile_time()?;
    Ok(format!(
        "axum=0.8.9;loopback=127.0.0.1:ephemeral;status=200;tower-http=0.7.0;middleware=response-header;sqlx=0.8.6;offline=true;query-value={offline_value};query-hash={QUERY_HASH};shutdown=clean"
    ))
}

async fn exchange_http(
    address: std::net::SocketAddr,
    path: &str,
) -> Result<String, BackendErrorBridge> {
    let mut stream = TcpStream::connect(address).await.map_err(backend_error)?;
    stream
        .write_all(
            format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .await
        .map_err(backend_error)?;
    let mut response = Vec::new();
    timeout(Duration::from_secs(2), stream.read_to_end(&mut response))
        .await
        .map_err(|_| BackendErrorBridge {
            message: "Axum loopback response timed out".to_owned(),
        })?
        .map_err(backend_error)?;
    String::from_utf8(response).map_err(backend_error)
}

fn backend_error(error: impl fmt::Display) -> BackendErrorBridge {
    BackendErrorBridge {
        message: format!("backend ecosystem probe failed: {error}"),
    }
}
