use std::{future::Future, pin::Pin};

use sifr_runtime::interop::SifrIntBridge;

type TlsFuture<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;

fn bridge_i64(value: &SifrIntBridge, name: &str) -> Result<i64, String> {
    value
        .try_to_i64()
        .map_err(|error| format!("{name} must fit in i64: {error}"))
}

pub fn tls_client_config_platform(alpn_protocols: &[Vec<u8>]) -> Result<SifrIntBridge, String> {
    sifr_runtime::tls::client_config_platform(alpn_protocols.to_vec()).map(Into::into)
}

pub fn tls_client_config_with_roots(
    root_pem: &[u8],
    alpn_protocols: &[Vec<u8>],
) -> Result<SifrIntBridge, String> {
    sifr_runtime::tls::client_config_with_roots(root_pem.to_vec(), alpn_protocols.to_vec())
        .map(Into::into)
}

pub fn tls_client_config_with_roots_and_client_auth(
    root_pem: &[u8],
    cert_pem: &[u8],
    key_pem: &[u8],
    alpn_protocols: &[Vec<u8>],
) -> Result<SifrIntBridge, String> {
    sifr_runtime::tls::client_config_with_roots_and_client_auth(
        root_pem.to_vec(),
        cert_pem.to_vec(),
        key_pem.to_vec(),
        alpn_protocols.to_vec(),
    )
    .map(Into::into)
}

pub fn tls_server_config(
    cert_pem: &[u8],
    key_pem: &[u8],
    alpn_protocols: &[Vec<u8>],
) -> Result<SifrIntBridge, String> {
    sifr_runtime::tls::server_config(cert_pem.to_vec(), key_pem.to_vec(), alpn_protocols.to_vec())
        .map(Into::into)
}

pub fn tls_server_config_require_client_auth(
    cert_pem: &[u8],
    key_pem: &[u8],
    client_ca_pem: &[u8],
    alpn_protocols: &[Vec<u8>],
) -> Result<SifrIntBridge, String> {
    sifr_runtime::tls::server_config_require_client_auth(
        cert_pem.to_vec(),
        key_pem.to_vec(),
        client_ca_pem.to_vec(),
        alpn_protocols.to_vec(),
    )
    .map(Into::into)
}

pub fn tls_client_config_close(handle: SifrIntBridge) -> Result<(), String> {
    let handle = bridge_i64(&handle, "TLS client config handle")?;
    sifr_runtime::tls::close_client_config(handle)
}

pub fn tls_server_config_close(handle: SifrIntBridge) -> Result<(), String> {
    let handle = bridge_i64(&handle, "TLS server config handle")?;
    sifr_runtime::tls::close_server_config(handle)
}

pub fn tls_connect(
    config_handle: SifrIntBridge,
    tcp_handle: SifrIntBridge,
    server_name: &str,
    timeout_seconds: f64,
    has_timeout: bool,
) -> TlsFuture<SifrIntBridge> {
    let server_name = server_name.to_string();
    Box::pin(async move {
        let config_handle = bridge_i64(&config_handle, "TLS client config handle")?;
        let tcp_handle = bridge_i64(&tcp_handle, "TCP stream handle")?;
        sifr_runtime::tls::connect_tls(
            config_handle,
            tcp_handle,
            server_name,
            timeout_seconds,
            has_timeout,
        )
        .await
        .map(Into::into)
    })
}

pub fn tls_accept(
    config_handle: SifrIntBridge,
    tcp_handle: SifrIntBridge,
    timeout_seconds: f64,
    has_timeout: bool,
) -> TlsFuture<SifrIntBridge> {
    Box::pin(async move {
        let config_handle = bridge_i64(&config_handle, "TLS server config handle")?;
        let tcp_handle = bridge_i64(&tcp_handle, "TCP stream handle")?;
        sifr_runtime::tls::accept_tls(config_handle, tcp_handle, timeout_seconds, has_timeout)
            .await
            .map(Into::into)
    })
}

pub fn tls_stream_read_chunk(
    handle: SifrIntBridge,
    max_bytes: SifrIntBridge,
) -> TlsFuture<Option<Vec<u8>>> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS stream handle")?;
        let max_bytes = bridge_i64(&max_bytes, "TLS read size")?;
        sifr_runtime::tls::tls_stream_read_chunk(handle, max_bytes).await
    })
}

pub fn tls_stream_write(handle: SifrIntBridge, data: &[u8]) -> TlsFuture<SifrIntBridge> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS stream handle")?;
        sifr_runtime::tls::tls_stream_write(handle, data)
            .await
            .map(Into::into)
    })
}

pub fn tls_stream_write_all(handle: SifrIntBridge, data: &[u8]) -> TlsFuture<()> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS stream handle")?;
        sifr_runtime::tls::tls_stream_write_all(handle, data).await
    })
}

pub fn tls_stream_flush(handle: SifrIntBridge) -> TlsFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS stream handle")?;
        sifr_runtime::tls::tls_stream_flush(handle).await
    })
}

pub fn tls_stream_close_notify(handle: SifrIntBridge) -> TlsFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS stream handle")?;
        sifr_runtime::tls::tls_stream_close_notify(handle).await
    })
}

pub fn tls_stream_close(handle: SifrIntBridge) -> TlsFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS stream handle")?;
        sifr_runtime::tls::tls_stream_close(handle).await
    })
}

#[must_use]
pub fn tls_stream_split(handle: SifrIntBridge) -> Result<Vec<SifrIntBridge>, String> {
    let handle = bridge_i64(&handle, "TLS stream handle")?;
    let (read, write) = sifr_runtime::tls::tls_stream_split(handle)?;
    Ok(vec![read.into(), write.into()])
}

pub fn tls_stream_alpn_protocol(handle: SifrIntBridge) -> Result<Option<Vec<u8>>, String> {
    let handle = bridge_i64(&handle, "TLS stream handle")?;
    sifr_runtime::tls::tls_stream_alpn_protocol(handle)
}

pub fn tls_stream_protocol_version(handle: SifrIntBridge) -> Result<Option<String>, String> {
    let handle = bridge_i64(&handle, "TLS stream handle")?;
    sifr_runtime::tls::tls_stream_protocol_version(handle)
}

pub fn tls_read_half_read_chunk(
    handle: SifrIntBridge,
    max_bytes: SifrIntBridge,
) -> TlsFuture<Option<Vec<u8>>> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS read half handle")?;
        let max_bytes = bridge_i64(&max_bytes, "TLS read size")?;
        sifr_runtime::tls::tls_read_half_read_chunk(handle, max_bytes).await
    })
}

pub fn tls_read_half_close(handle: SifrIntBridge) -> Result<(), String> {
    let handle = bridge_i64(&handle, "TLS read half handle")?;
    sifr_runtime::tls::tls_read_half_close(handle)
}

pub fn tls_write_half_write(handle: SifrIntBridge, data: &[u8]) -> TlsFuture<SifrIntBridge> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS write half handle")?;
        sifr_runtime::tls::tls_write_half_write(handle, data)
            .await
            .map(Into::into)
    })
}

pub fn tls_write_half_write_all(handle: SifrIntBridge, data: &[u8]) -> TlsFuture<()> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS write half handle")?;
        sifr_runtime::tls::tls_write_half_write_all(handle, data).await
    })
}

pub fn tls_write_half_flush(handle: SifrIntBridge) -> TlsFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS write half handle")?;
        sifr_runtime::tls::tls_write_half_flush(handle).await
    })
}

pub fn tls_write_half_close_notify(handle: SifrIntBridge) -> TlsFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS write half handle")?;
        sifr_runtime::tls::tls_write_half_close_notify(handle).await
    })
}

pub fn tls_write_half_close(handle: SifrIntBridge) -> TlsFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TLS write half handle")?;
        sifr_runtime::tls::tls_write_half_close(handle).await
    })
}
