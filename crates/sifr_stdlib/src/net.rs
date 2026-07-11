use std::{future::Future, pin::Pin};

use sifr_runtime::interop::SifrIntBridge;

type NetFuture<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;

fn bridge_i64(value: &SifrIntBridge, name: &str) -> Result<i64, String> {
    value
        .try_to_i64()
        .map_err(|error| format!("{name} must fit in i64: {error}"))
}

pub fn net_connect_tcp(
    address: &str,
    timeout_seconds: f64,
    has_timeout: bool,
    local_addr: &str,
    has_local_addr: bool,
) -> NetFuture<SifrIntBridge> {
    let address = address.to_string();
    let local_addr = local_addr.to_string();
    Box::pin(async move {
        sifr_runtime::net::connect_tcp(
            address,
            timeout_seconds,
            has_timeout,
            local_addr,
            has_local_addr,
        )
        .await
        .map(Into::into)
    })
}

pub fn net_listen_tcp(
    address: &str,
    backlog: SifrIntBridge,
    has_backlog: bool,
    reuse_addr: bool,
) -> NetFuture<SifrIntBridge> {
    let address = address.to_string();
    Box::pin(async move {
        let backlog = bridge_i64(&backlog, "TCP listener backlog")?;
        sifr_runtime::net::listen_tcp(address, backlog, has_backlog, reuse_addr)
            .await
            .map(Into::into)
    })
}

pub fn net_lookup_host(
    address: &str,
    timeout_seconds: f64,
    has_timeout: bool,
) -> NetFuture<Vec<String>> {
    let address = address.to_string();
    Box::pin(
        async move { sifr_runtime::net::resolve_host(address, timeout_seconds, has_timeout).await },
    )
}

pub fn net_listener_accept(handle: SifrIntBridge) -> NetFuture<SifrIntBridge> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP listener handle")?;
        let (stream, _remote) = sifr_runtime::net::accept_tcp(handle).await?;
        Ok(stream.into())
    })
}

pub fn net_listener_local_addr(handle: SifrIntBridge) -> Result<String, String> {
    let handle = bridge_i64(&handle, "TCP listener handle")?;
    sifr_runtime::net::tcp_listener_local_addr(handle)
}

pub fn net_listener_close(handle: SifrIntBridge) -> Result<(), String> {
    let handle = bridge_i64(&handle, "TCP listener handle")?;
    sifr_runtime::net::close_tcp_listener(handle)
}

pub fn net_tcp_stream_read_chunk(
    handle: SifrIntBridge,
    max_bytes: SifrIntBridge,
) -> NetFuture<Option<Vec<u8>>> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP stream handle")?;
        let max_bytes = bridge_i64(&max_bytes, "TCP read size")?;
        sifr_runtime::net::tcp_stream_read_chunk(handle, max_bytes).await
    })
}

pub fn net_tcp_stream_write(handle: SifrIntBridge, data: &[u8]) -> NetFuture<SifrIntBridge> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP stream handle")?;
        sifr_runtime::net::tcp_stream_write(handle, data)
            .await
            .map(Into::into)
    })
}

pub fn net_tcp_stream_write_all(handle: SifrIntBridge, data: &[u8]) -> NetFuture<()> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP stream handle")?;
        sifr_runtime::net::tcp_stream_write_all(handle, data).await
    })
}

pub fn net_tcp_stream_shutdown_write(handle: SifrIntBridge) -> NetFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP stream handle")?;
        sifr_runtime::net::tcp_stream_shutdown_write(handle).await
    })
}

pub fn net_tcp_stream_close(handle: SifrIntBridge) -> NetFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP stream handle")?;
        sifr_runtime::net::tcp_stream_close(handle).await
    })
}

#[must_use]
pub fn net_tcp_stream_split(handle: SifrIntBridge) -> Result<Vec<SifrIntBridge>, String> {
    let handle = bridge_i64(&handle, "TCP stream handle")?;
    let (read, write) = sifr_runtime::net::tcp_stream_split(handle)?;
    Ok(vec![read.into(), write.into()])
}

pub fn net_tcp_stream_local_addr(handle: SifrIntBridge) -> Result<String, String> {
    let handle = bridge_i64(&handle, "TCP stream handle")?;
    sifr_runtime::net::tcp_stream_local_addr(handle)
}

pub fn net_tcp_stream_peer_addr(handle: SifrIntBridge) -> Result<String, String> {
    let handle = bridge_i64(&handle, "TCP stream handle")?;
    sifr_runtime::net::tcp_stream_remote_addr(handle)
}

pub fn net_tcp_read_half_read_chunk(
    handle: SifrIntBridge,
    max_bytes: SifrIntBridge,
) -> NetFuture<Option<Vec<u8>>> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP read half handle")?;
        let max_bytes = bridge_i64(&max_bytes, "TCP read size")?;
        sifr_runtime::net::tcp_read_half_read_chunk(handle, max_bytes).await
    })
}

pub fn net_tcp_read_half_close(handle: SifrIntBridge) -> Result<(), String> {
    let handle = bridge_i64(&handle, "TCP read half handle")?;
    sifr_runtime::net::tcp_read_half_close(handle)
}

pub fn net_tcp_write_half_write(handle: SifrIntBridge, data: &[u8]) -> NetFuture<SifrIntBridge> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP write half handle")?;
        sifr_runtime::net::tcp_write_half_write(handle, data)
            .await
            .map(Into::into)
    })
}

pub fn net_tcp_write_half_write_all(handle: SifrIntBridge, data: &[u8]) -> NetFuture<()> {
    let data = data.to_vec();
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP write half handle")?;
        sifr_runtime::net::tcp_write_half_write_all(handle, data).await
    })
}

pub fn net_tcp_write_half_shutdown_write(handle: SifrIntBridge) -> NetFuture<()> {
    Box::pin(async move {
        let handle = bridge_i64(&handle, "TCP write half handle")?;
        sifr_runtime::net::tcp_write_half_shutdown_write(handle).await
    })
}

pub fn net_tcp_write_half_close(handle: SifrIntBridge) -> Result<(), String> {
    let handle = bridge_i64(&handle, "TCP write half handle")?;
    sifr_runtime::net::tcp_write_half_close(handle)
}
