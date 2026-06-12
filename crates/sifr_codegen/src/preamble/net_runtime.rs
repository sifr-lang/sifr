//! Runtime-backed generated async network helpers.

use crate::RustItem;

const NET_RUNTIME: &str = r#"
fn __sifr_net_error(message: String) -> NetError {
    NetError { message }
}

async fn __sifr_net_connect_tcp(
    address: String,
    timeout_seconds: f64,
    has_timeout: bool,
    local_addr: String,
    has_local_addr: bool,
) -> Result<TcpStream, NetError> {
    sifr_runtime::net::connect_tcp(
        address,
        timeout_seconds,
        has_timeout,
        local_addr,
        has_local_addr,
    )
    .await
    .map(TcpStream::new)
    .map_err(__sifr_net_error)
}

async fn __sifr_net_listen_tcp(
    address: String,
    backlog: i64,
    has_backlog: bool,
    reuse_addr: bool,
) -> Result<TcpListener, NetError> {
    sifr_runtime::net::listen_tcp(address, backlog, has_backlog, reuse_addr)
        .await
        .map(TcpListener::new)
        .map_err(__sifr_net_error)
}

async fn __sifr_net_lookup_host(
    address: String,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<Vec<SocketAddr>, NetError> {
    sifr_runtime::net::resolve_host(address, timeout_seconds, has_timeout)
        .await
        .map(|addrs| addrs.into_iter().map(SocketAddr::new).collect())
        .map_err(__sifr_net_error)
}

async fn __sifr_net_listener_accept(handle: i64) -> Result<(TcpStream, SocketAddr), NetError> {
    sifr_runtime::net::accept_tcp(handle)
        .await
        .map(|(stream, addr)| (TcpStream::new(stream), SocketAddr::new(addr)))
        .map_err(__sifr_net_error)
}

fn __sifr_net_listener_local_addr(handle: i64) -> Result<String, NetError> {
    sifr_runtime::net::tcp_listener_local_addr(handle).map_err(__sifr_net_error)
}

fn __sifr_net_listener_close(handle: i64) -> Result<(), NetError> {
    sifr_runtime::net::close_tcp_listener(handle).map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_stream_read_chunk(
    handle: i64,
    max_bytes: i64,
) -> Result<Option<Vec<u8>>, NetError> {
    sifr_runtime::net::tcp_stream_read_chunk(handle, max_bytes)
        .await
        .map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_stream_write(handle: i64, data: Vec<u8>) -> Result<i64, NetError> {
    sifr_runtime::net::tcp_stream_write(handle, data)
        .await
        .map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_stream_write_all(handle: i64, data: Vec<u8>) -> Result<(), NetError> {
    sifr_runtime::net::tcp_stream_write_all(handle, data)
        .await
        .map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_stream_shutdown_write(handle: i64) -> Result<(), NetError> {
    sifr_runtime::net::tcp_stream_shutdown_write(handle)
        .await
        .map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_stream_close(handle: i64) -> Result<(), NetError> {
    sifr_runtime::net::tcp_stream_close(handle)
        .await
        .map_err(__sifr_net_error)
}

fn __sifr_net_tcp_stream_split(handle: i64) -> (TcpReadHalf, TcpWriteHalf) {
    let (read, write) = sifr_runtime::net::tcp_stream_split(handle);
    (TcpReadHalf::new(read), TcpWriteHalf::new(write))
}

fn __sifr_net_tcp_stream_local_addr(handle: i64) -> Result<String, NetError> {
    sifr_runtime::net::tcp_stream_local_addr(handle).map_err(__sifr_net_error)
}

fn __sifr_net_tcp_stream_peer_addr(handle: i64) -> Result<String, NetError> {
    sifr_runtime::net::tcp_stream_remote_addr(handle).map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_read_half_read_chunk(
    handle: i64,
    max_bytes: i64,
) -> Result<Option<Vec<u8>>, NetError> {
    sifr_runtime::net::tcp_read_half_read_chunk(handle, max_bytes)
        .await
        .map_err(__sifr_net_error)
}

fn __sifr_net_tcp_read_half_close(handle: i64) -> Result<(), NetError> {
    sifr_runtime::net::tcp_read_half_close(handle).map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_write_half_write(handle: i64, data: Vec<u8>) -> Result<i64, NetError> {
    sifr_runtime::net::tcp_write_half_write(handle, data)
        .await
        .map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_write_half_write_all(handle: i64, data: Vec<u8>) -> Result<(), NetError> {
    sifr_runtime::net::tcp_write_half_write_all(handle, data)
        .await
        .map_err(__sifr_net_error)
}

async fn __sifr_net_tcp_write_half_shutdown_write(handle: i64) -> Result<(), NetError> {
    sifr_runtime::net::tcp_write_half_shutdown_write(handle)
        .await
        .map_err(__sifr_net_error)
}

fn __sifr_net_tcp_write_half_close(handle: i64) -> Result<(), NetError> {
    sifr_runtime::net::tcp_write_half_close(handle).map_err(__sifr_net_error)
}
"#;

pub(crate) fn build_net_runtime_items() -> Vec<RustItem> {
    vec![RustItem::Attr(NET_RUNTIME.to_string())]
}
