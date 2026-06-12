//! Runtime-backed generated async TLS helpers.

use crate::RustItem;

const TLS_RUNTIME: &str = r#"
fn __sifr_tls_error(message: String) -> TlsError {
    TlsError { message }
}

fn __sifr_tls_client_config_platform(
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<TlsClientConfig, TlsError> {
    sifr_runtime::tls::client_config_platform(alpn_protocols)
        .map(TlsClientConfig::new)
        .map_err(__sifr_tls_error)
}

fn __sifr_tls_client_config_with_roots(
    root_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<TlsClientConfig, TlsError> {
    sifr_runtime::tls::client_config_with_roots(root_pem, alpn_protocols)
        .map(TlsClientConfig::new)
        .map_err(__sifr_tls_error)
}

fn __sifr_tls_client_config_with_roots_and_client_auth(
    root_pem: Vec<u8>,
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<TlsClientConfig, TlsError> {
    sifr_runtime::tls::client_config_with_roots_and_client_auth(
        root_pem,
        cert_pem,
        key_pem,
        alpn_protocols,
    )
    .map(TlsClientConfig::new)
    .map_err(__sifr_tls_error)
}

fn __sifr_tls_server_config(
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<TlsServerConfig, TlsError> {
    sifr_runtime::tls::server_config(cert_pem, key_pem, alpn_protocols)
        .map(TlsServerConfig::new)
        .map_err(__sifr_tls_error)
}

fn __sifr_tls_server_config_require_client_auth(
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    client_ca_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<TlsServerConfig, TlsError> {
    sifr_runtime::tls::server_config_require_client_auth(
        cert_pem,
        key_pem,
        client_ca_pem,
        alpn_protocols,
    )
    .map(TlsServerConfig::new)
    .map_err(__sifr_tls_error)
}

fn __sifr_tls_client_config_close(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::close_client_config(handle).map_err(__sifr_tls_error)
}

fn __sifr_tls_server_config_close(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::close_server_config(handle).map_err(__sifr_tls_error)
}

async fn __sifr_tls_connect(
    config_handle: i64,
    tcp_handle: i64,
    server_name: String,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<TlsStream, TlsError> {
    sifr_runtime::tls::connect_tls(
        config_handle,
        tcp_handle,
        server_name,
        timeout_seconds,
        has_timeout,
    )
    .await
    .map(TlsStream::new)
    .map_err(__sifr_tls_error)
}

async fn __sifr_tls_accept(
    config_handle: i64,
    tcp_handle: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<TlsStream, TlsError> {
    sifr_runtime::tls::accept_tls(config_handle, tcp_handle, timeout_seconds, has_timeout)
        .await
        .map(TlsStream::new)
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_stream_read_chunk(
    handle: i64,
    max_bytes: i64,
) -> Result<Option<Vec<u8>>, TlsError> {
    sifr_runtime::tls::tls_stream_read_chunk(handle, max_bytes)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_stream_write(handle: i64, data: Vec<u8>) -> Result<i64, TlsError> {
    sifr_runtime::tls::tls_stream_write(handle, data)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_stream_write_all(handle: i64, data: Vec<u8>) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_stream_write_all(handle, data)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_stream_flush(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_stream_flush(handle)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_stream_close_notify(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_stream_close_notify(handle)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_stream_close(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_stream_close(handle)
        .await
        .map_err(__sifr_tls_error)
}

fn __sifr_tls_stream_split(handle: i64) -> (TlsReadHalf, TlsWriteHalf) {
    let (read, write) = sifr_runtime::tls::tls_stream_split(handle);
    (TlsReadHalf::new(read), TlsWriteHalf::new(write))
}

fn __sifr_tls_stream_alpn_protocol(handle: i64) -> Result<Option<Vec<u8>>, TlsError> {
    sifr_runtime::tls::tls_stream_alpn_protocol(handle).map_err(__sifr_tls_error)
}

fn __sifr_tls_stream_protocol_version(handle: i64) -> Result<Option<String>, TlsError> {
    sifr_runtime::tls::tls_stream_protocol_version(handle).map_err(__sifr_tls_error)
}

async fn __sifr_tls_read_half_read_chunk(
    handle: i64,
    max_bytes: i64,
) -> Result<Option<Vec<u8>>, TlsError> {
    sifr_runtime::tls::tls_read_half_read_chunk(handle, max_bytes)
        .await
        .map_err(__sifr_tls_error)
}

fn __sifr_tls_read_half_close(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_read_half_close(handle).map_err(__sifr_tls_error)
}

async fn __sifr_tls_write_half_write(handle: i64, data: Vec<u8>) -> Result<i64, TlsError> {
    sifr_runtime::tls::tls_write_half_write(handle, data)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_write_half_write_all(handle: i64, data: Vec<u8>) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_write_half_write_all(handle, data)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_write_half_flush(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_write_half_flush(handle)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_write_half_close_notify(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_write_half_close_notify(handle)
        .await
        .map_err(__sifr_tls_error)
}

async fn __sifr_tls_write_half_close(handle: i64) -> Result<(), TlsError> {
    sifr_runtime::tls::tls_write_half_close(handle)
        .await
        .map_err(__sifr_tls_error)
}
"#;

pub(crate) fn build_tls_runtime_items() -> Vec<RustItem> {
    vec![RustItem::Attr(TLS_RUNTIME.to_string())]
}
