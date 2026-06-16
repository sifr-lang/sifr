//! Async TLS runtime support for generated Sifr programs.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::io::Cursor;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc, LazyLock, Mutex, MutexGuard,
};

use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::server::WebPkiClientVerifier;
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use rustls_platform_verifier::ConfigVerifierExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio_rustls::{TlsAcceptor, TlsConnector, TlsStream};

use crate::timeouts::timeout_duration;

const MAX_READ_BYTES: i64 = 1_048_576;

type RuntimeTlsStream = TlsStream<TcpStream>;
type SharedReadHalf = Arc<tokio::sync::Mutex<ReadHalf<RuntimeTlsStream>>>;
type SharedWriteHalf = Arc<tokio::sync::Mutex<WriteHalf<RuntimeTlsStream>>>;

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);
static CLIENT_CONFIGS: LazyLock<Mutex<HashMap<i64, Arc<ClientConfig>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static SERVER_CONFIGS: LazyLock<Mutex<HashMap<i64, Arc<ServerConfig>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static STREAMS: LazyLock<Mutex<HashMap<i64, RuntimeTlsStream>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static READ_HALVES: LazyLock<Mutex<HashMap<i64, SharedReadHalf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static WRITE_HALVES: LazyLock<Mutex<HashMap<i64, SharedWriteHalf>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static CLOSE_NOTIFIED: LazyLock<Mutex<HashSet<i64>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

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
            return Err("TLS handle table is exhausted".to_string());
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
    match tokio::time::timeout(timeout_duration(seconds, "TLS")?, operation).await {
        Ok(result) => result,
        Err(_) => Err(format!("{name} timed out")),
    }
}

fn parse_certificates(pem: &[u8], label: &str) -> Result<Vec<CertificateDer<'static>>, String> {
    let mut reader = Cursor::new(pem);
    let certs = rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to parse {label} certificates: {error}"))?;
    if certs.is_empty() {
        return Err(format!("{label} certificate PEM contains no certificates"));
    }
    Ok(certs)
}

fn parse_private_key(pem: &[u8], label: &str) -> Result<PrivateKeyDer<'static>, String> {
    let mut reader = Cursor::new(pem);
    rustls_pemfile::private_key(&mut reader)
        .map_err(|error| format!("failed to parse {label} private key: {error}"))?
        .ok_or_else(|| format!("{label} private key PEM contains no supported private key"))
}

fn root_store_from_pem(pem: &[u8], label: &str) -> Result<RootCertStore, String> {
    let mut store = RootCertStore::empty();
    for cert in parse_certificates(pem, label)? {
        store
            .add(cert)
            .map_err(|error| format!("failed to add {label} certificate to root store: {error}"))?;
    }
    Ok(store)
}

fn apply_alpn_to_client(config: &mut ClientConfig, alpn_protocols: Vec<Vec<u8>>) {
    config.alpn_protocols = alpn_protocols;
}

fn apply_alpn_to_server(config: &mut ServerConfig, alpn_protocols: Vec<Vec<u8>>) {
    config.alpn_protocols = alpn_protocols;
}

fn insert_client_config(config: ClientConfig) -> Result<i64, String> {
    let handle = next_handle()?;
    lock(&CLIENT_CONFIGS).insert(handle, Arc::new(config));
    Ok(handle)
}

fn insert_server_config(config: ServerConfig) -> Result<i64, String> {
    let handle = next_handle()?;
    lock(&SERVER_CONFIGS).insert(handle, Arc::new(config));
    Ok(handle)
}

fn client_config(handle: i64) -> Result<Arc<ClientConfig>, String> {
    lock(&CLIENT_CONFIGS)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TLS client config handle is closed or unknown: {handle}"))
}

fn get_server_config(handle: i64) -> Result<Arc<ServerConfig>, String> {
    lock(&SERVER_CONFIGS)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TLS server config handle is closed or unknown: {handle}"))
}

fn insert_stream(stream: RuntimeTlsStream) -> Result<i64, String> {
    let handle = next_handle()?;
    lock(&STREAMS).insert(handle, stream);
    Ok(handle)
}

fn take_stream(handle: i64) -> Result<RuntimeTlsStream, String> {
    lock(&STREAMS)
        .remove(&handle)
        .ok_or_else(|| format!("TLS stream handle is closed or unknown: {handle}"))
}

#[cfg(feature = "http")]
pub(crate) fn consume_stream_for_http(handle: i64) -> Result<RuntimeTlsStream, String> {
    let stream = take_stream(handle)?;
    lock(&CLOSE_NOTIFIED).remove(&handle);
    Ok(stream)
}

fn restore_stream(handle: i64, stream: RuntimeTlsStream) {
    lock(&STREAMS).insert(handle, stream);
}

fn ensure_write_open(handle: i64) -> Result<(), String> {
    if lock(&CLOSE_NOTIFIED).contains(&handle) {
        return Err("TLS write side is already close-notified".to_string());
    }
    Ok(())
}

fn validate_read_size(max_bytes: i64) -> Result<usize, String> {
    if max_bytes <= 0 {
        return Err("TLS read size must be positive".to_string());
    }
    if max_bytes > MAX_READ_BYTES {
        return Err(format!("TLS read size exceeds {MAX_READ_BYTES} bytes"));
    }
    usize::try_from(max_bytes).map_err(|error| format!("invalid TLS read size: {error}"))
}

pub fn client_config_platform(alpn_protocols: Vec<Vec<u8>>) -> Result<i64, String> {
    let mut config = ClientConfig::with_platform_verifier()
        .map_err(|error| format!("failed to configure platform TLS verifier: {error}"))?;
    apply_alpn_to_client(&mut config, alpn_protocols);
    insert_client_config(config)
}

pub fn client_config_with_roots(
    root_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<i64, String> {
    let roots = root_store_from_pem(&root_pem, "client root")?;
    let mut config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    apply_alpn_to_client(&mut config, alpn_protocols);
    insert_client_config(config)
}

pub fn client_config_with_roots_and_client_auth(
    root_pem: Vec<u8>,
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<i64, String> {
    let roots = root_store_from_pem(&root_pem, "client root")?;
    let certs = parse_certificates(&cert_pem, "client identity")?;
    let key = parse_private_key(&key_pem, "client identity")?;
    let mut config = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_client_auth_cert(certs, key)
        .map_err(|error| format!("failed to configure TLS client certificate: {error}"))?;
    apply_alpn_to_client(&mut config, alpn_protocols);
    insert_client_config(config)
}

pub fn server_config(
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<i64, String> {
    let certs = parse_certificates(&cert_pem, "server identity")?;
    let key = parse_private_key(&key_pem, "server identity")?;
    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|error| format!("failed to configure TLS server certificate: {error}"))?;
    apply_alpn_to_server(&mut config, alpn_protocols);
    insert_server_config(config)
}

pub fn server_config_require_client_auth(
    cert_pem: Vec<u8>,
    key_pem: Vec<u8>,
    client_ca_pem: Vec<u8>,
    alpn_protocols: Vec<Vec<u8>>,
) -> Result<i64, String> {
    let certs = parse_certificates(&cert_pem, "server identity")?;
    let key = parse_private_key(&key_pem, "server identity")?;
    let client_roots = root_store_from_pem(&client_ca_pem, "client CA")?;
    let verifier = WebPkiClientVerifier::builder(Arc::new(client_roots))
        .build()
        .map_err(|error| format!("failed to configure client certificate verifier: {error}"))?;
    let mut config = ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(certs, key)
        .map_err(|error| format!("failed to configure TLS server certificate: {error}"))?;
    apply_alpn_to_server(&mut config, alpn_protocols);
    insert_server_config(config)
}

pub fn close_client_config(handle: i64) -> Result<(), String> {
    lock(&CLIENT_CONFIGS)
        .remove(&handle)
        .map(|_| ())
        .ok_or_else(|| format!("TLS client config handle is closed or unknown: {handle}"))
}

pub fn close_server_config(handle: i64) -> Result<(), String> {
    lock(&SERVER_CONFIGS)
        .remove(&handle)
        .map(|_| ())
        .ok_or_else(|| format!("TLS server config handle is closed or unknown: {handle}"))
}

pub async fn connect_tls(
    config_handle: i64,
    tcp_handle: i64,
    server_name: String,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<i64, String> {
    with_optional_timeout(
        async move {
            let tcp = crate::net::consume_stream_for_tls(tcp_handle)
                .map_err(|error| format!("TLS transport setup failed: {error}"))?;
            let config = client_config(config_handle)?;
            let server_name = ServerName::try_from(server_name)
                .map_err(|error| format!("invalid TLS server name: {error}"))?;
            TlsConnector::from(config)
                .connect(server_name, tcp)
                .await
                .map(TlsStream::from)
                .map_err(|error| format!("TLS client handshake failed: {error}"))
                .and_then(insert_stream)
        },
        timeout_seconds,
        has_timeout,
        "TLS client handshake",
    )
    .await
}

pub async fn accept_tls(
    config_handle: i64,
    tcp_handle: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<i64, String> {
    with_optional_timeout(
        async move {
            let tcp = crate::net::consume_stream_for_tls(tcp_handle)
                .map_err(|error| format!("TLS transport setup failed: {error}"))?;
            let config = get_server_config(config_handle)?;
            TlsAcceptor::from(config)
                .accept(tcp)
                .await
                .map(TlsStream::from)
                .map_err(|error| format!("TLS server handshake failed: {error}"))
                .and_then(insert_stream)
        },
        timeout_seconds,
        has_timeout,
        "TLS server handshake",
    )
    .await
}

pub async fn tls_stream_read_chunk(handle: i64, max_bytes: i64) -> Result<Option<Vec<u8>>, String> {
    let mut buf = vec![0_u8; validate_read_size(max_bytes)?];
    let mut stream = take_stream(handle)?;
    let read = match stream.read(&mut buf).await {
        Ok(read) => read,
        Err(error) => {
            restore_stream(handle, stream);
            return Err(format!("failed to read TLS stream: {error}"));
        }
    };
    restore_stream(handle, stream);
    if read == 0 {
        return Ok(None);
    }
    buf.truncate(read);
    Ok(Some(buf))
}

pub async fn tls_stream_write(handle: i64, data: Vec<u8>) -> Result<i64, String> {
    ensure_write_open(handle)?;
    let mut stream = take_stream(handle)?;
    let written = match stream.write(&data).await {
        Ok(written) => written,
        Err(error) => {
            restore_stream(handle, stream);
            return Err(format!("failed to write TLS stream: {error}"));
        }
    };
    restore_stream(handle, stream);
    i64::try_from(written).map_err(|error| format!("invalid TLS write count: {error}"))
}

pub async fn tls_stream_write_all(handle: i64, data: Vec<u8>) -> Result<(), String> {
    ensure_write_open(handle)?;
    let mut stream = take_stream(handle)?;
    let result = stream
        .write_all(&data)
        .await
        .map_err(|error| format!("failed to write TLS stream: {error}"));
    restore_stream(handle, stream);
    result
}

pub async fn tls_stream_flush(handle: i64) -> Result<(), String> {
    if lock(&CLOSE_NOTIFIED).contains(&handle) {
        return Ok(());
    }
    let mut stream = take_stream(handle)?;
    let result = stream
        .flush()
        .await
        .map_err(|error| format!("failed to flush TLS stream: {error}"));
    restore_stream(handle, stream);
    result
}

pub async fn tls_stream_close_notify(handle: i64) -> Result<(), String> {
    if lock(&CLOSE_NOTIFIED).contains(&handle) {
        return Ok(());
    }
    let mut stream = take_stream(handle)?;
    let result = stream
        .shutdown()
        .await
        .map_err(|error| format!("failed to send TLS close_notify: {error}"));
    restore_stream(handle, stream);
    result?;
    lock(&CLOSE_NOTIFIED).insert(handle);
    Ok(())
}

pub async fn tls_stream_close(handle: i64) -> Result<(), String> {
    let Some(mut stream) = lock(&STREAMS).remove(&handle) else {
        return Err(format!("TLS stream handle is closed or unknown: {handle}"));
    };
    let result = if lock(&CLOSE_NOTIFIED).remove(&handle) {
        Ok(())
    } else {
        stream
            .shutdown()
            .await
            .map_err(|error| format!("failed to close TLS stream: {error}"))
    };
    drop(stream);
    result
}

pub fn tls_stream_split(handle: i64) -> (i64, i64) {
    let read_handle = next_handle_infallible();
    let write_handle = next_handle_infallible();
    let Some(stream) = lock(&STREAMS).remove(&handle) else {
        return (read_handle, write_handle);
    };
    let was_close_notified = lock(&CLOSE_NOTIFIED).remove(&handle);
    let (read_half, write_half) = tokio::io::split(stream);
    lock(&READ_HALVES).insert(read_handle, Arc::new(tokio::sync::Mutex::new(read_half)));
    lock(&WRITE_HALVES).insert(write_handle, Arc::new(tokio::sync::Mutex::new(write_half)));
    if was_close_notified {
        lock(&CLOSE_NOTIFIED).insert(write_handle);
    }
    (read_handle, write_handle)
}

pub fn tls_stream_alpn_protocol(handle: i64) -> Result<Option<Vec<u8>>, String> {
    lock(&STREAMS)
        .get(&handle)
        .map(|stream| stream.get_ref().1.alpn_protocol().map(<[u8]>::to_vec))
        .ok_or_else(|| format!("TLS stream handle is closed or unknown: {handle}"))
}

pub fn tls_stream_protocol_version(handle: i64) -> Result<Option<String>, String> {
    lock(&STREAMS)
        .get(&handle)
        .map(|stream| {
            stream
                .get_ref()
                .1
                .protocol_version()
                .map(|version| format!("{version:?}"))
        })
        .ok_or_else(|| format!("TLS stream handle is closed or unknown: {handle}"))
}

pub async fn tls_read_half_read_chunk(
    handle: i64,
    max_bytes: i64,
) -> Result<Option<Vec<u8>>, String> {
    let reader = lock(&READ_HALVES)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TLS read half handle is closed or unknown: {handle}"))?;
    let mut guard = reader.lock().await;
    let mut buf = vec![0_u8; validate_read_size(max_bytes)?];
    let read = guard
        .read(&mut buf)
        .await
        .map_err(|error| format!("failed to read TLS read half: {error}"))?;
    if read == 0 {
        return Ok(None);
    }
    buf.truncate(read);
    Ok(Some(buf))
}

pub fn tls_read_half_close(handle: i64) -> Result<(), String> {
    lock(&READ_HALVES)
        .remove(&handle)
        .map(|_| ())
        .ok_or_else(|| format!("TLS read half handle is closed or unknown: {handle}"))
}

fn write_half_handle(handle: i64) -> Result<SharedWriteHalf, String> {
    lock(&WRITE_HALVES)
        .get(&handle)
        .cloned()
        .ok_or_else(|| format!("TLS write half handle is closed or unknown: {handle}"))
}

pub async fn tls_write_half_write(handle: i64, data: Vec<u8>) -> Result<i64, String> {
    ensure_write_open(handle)?;
    let writer = write_half_handle(handle)?;
    let mut guard = writer.lock().await;
    let written = guard
        .write(&data)
        .await
        .map_err(|error| format!("failed to write TLS write half: {error}"))?;
    i64::try_from(written).map_err(|error| format!("invalid TLS write count: {error}"))
}

pub async fn tls_write_half_write_all(handle: i64, data: Vec<u8>) -> Result<(), String> {
    ensure_write_open(handle)?;
    let writer = write_half_handle(handle)?;
    let mut guard = writer.lock().await;
    guard
        .write_all(&data)
        .await
        .map_err(|error| format!("failed to write TLS write half: {error}"))
}

pub async fn tls_write_half_flush(handle: i64) -> Result<(), String> {
    if lock(&CLOSE_NOTIFIED).contains(&handle) {
        return Ok(());
    }
    let writer = write_half_handle(handle)?;
    let mut guard = writer.lock().await;
    guard
        .flush()
        .await
        .map_err(|error| format!("failed to flush TLS write half: {error}"))
}

pub async fn tls_write_half_close_notify(handle: i64) -> Result<(), String> {
    if lock(&CLOSE_NOTIFIED).contains(&handle) {
        return Ok(());
    }
    let writer = write_half_handle(handle)?;
    let mut guard = writer.lock().await;
    guard
        .shutdown()
        .await
        .map_err(|error| format!("failed to send TLS close_notify: {error}"))?;
    lock(&CLOSE_NOTIFIED).insert(handle);
    Ok(())
}

pub async fn tls_write_half_close(handle: i64) -> Result<(), String> {
    let removed = lock(&WRITE_HALVES).remove(&handle);
    let was_close_notified = lock(&CLOSE_NOTIFIED).remove(&handle);
    let Some(writer) = removed else {
        return Err(format!(
            "TLS write half handle is closed or unknown: {handle}"
        ));
    };
    if was_close_notified {
        return Ok(());
    }
    let mut guard = writer.lock().await;
    guard
        .shutdown()
        .await
        .map_err(|error| format!("failed to close TLS write half: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::{
        BasicConstraints, CertificateParams, CertifiedIssuer, ExtendedKeyUsagePurpose, IsCa,
        KeyPair, KeyUsagePurpose,
    };

    struct TlsMaterials {
        server_ca_pem: Vec<u8>,
        server_cert_pem: Vec<u8>,
        server_key_pem: Vec<u8>,
        client_ca_pem: Vec<u8>,
        client_cert_pem: Vec<u8>,
        client_key_pem: Vec<u8>,
    }

    fn ca(name: &str) -> CertifiedIssuer<'static, KeyPair> {
        let mut params = CertificateParams::new(vec![name.to_string()]).unwrap();
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
        params.key_usages = vec![
            KeyUsagePurpose::KeyCertSign,
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::CrlSign,
        ];
        CertifiedIssuer::self_signed(params, KeyPair::generate().unwrap()).unwrap()
    }

    fn signed_leaf(
        names: Vec<String>,
        usage: ExtendedKeyUsagePurpose,
        issuer: &CertifiedIssuer<'static, KeyPair>,
    ) -> (Vec<u8>, Vec<u8>) {
        let mut params = CertificateParams::new(names).unwrap();
        params.is_ca = IsCa::ExplicitNoCa;
        params.key_usages = vec![
            KeyUsagePurpose::DigitalSignature,
            KeyUsagePurpose::KeyEncipherment,
        ];
        params.extended_key_usages = vec![usage];
        let key = KeyPair::generate().unwrap();
        let cert = params.signed_by(&key, issuer).unwrap();
        (cert.pem().into_bytes(), key.serialize_pem().into_bytes())
    }

    fn materials() -> TlsMaterials {
        let server_ca = ca("sifr-tls-server-ca.local");
        let client_ca = ca("sifr-tls-client-ca.local");
        let (server_cert_pem, server_key_pem) = signed_leaf(
            vec!["localhost".to_string()],
            ExtendedKeyUsagePurpose::ServerAuth,
            &server_ca,
        );
        let (client_cert_pem, client_key_pem) = signed_leaf(
            vec!["sifr-tls-client.local".to_string()],
            ExtendedKeyUsagePurpose::ClientAuth,
            &client_ca,
        );
        TlsMaterials {
            server_ca_pem: server_ca.pem().into_bytes(),
            server_cert_pem,
            server_key_pem,
            client_ca_pem: client_ca.pem().into_bytes(),
            client_cert_pem,
            client_key_pem,
        }
    }

    async fn loopback_tcp_pair() -> (i64, i64) {
        let listener = crate::net::listen_tcp("127.0.0.1:0".to_string(), 8, true, false)
            .await
            .unwrap();
        let addr = crate::net::tcp_listener_local_addr(listener).unwrap();
        let client = tokio::spawn(async move {
            crate::net::connect_tcp(addr, 5.0, true, String::new(), false)
                .await
                .unwrap()
        });
        let (server, _) = crate::net::accept_tcp(listener).await.unwrap();
        crate::net::close_tcp_listener(listener).unwrap();
        (client.await.unwrap(), server)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tls_loopback_split_close_notify_and_alpn() {
        let materials = materials();
        let client_config = client_config_with_roots_and_client_auth(
            materials.server_ca_pem,
            materials.client_cert_pem,
            materials.client_key_pem,
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
        .unwrap();
        let server_config = server_config_require_client_auth(
            materials.server_cert_pem,
            materials.server_key_pem,
            materials.client_ca_pem,
            vec![b"h2".to_vec(), b"http/1.1".to_vec()],
        )
        .unwrap();
        let (client_tcp, server_tcp) = loopback_tcp_pair().await;

        let client = tokio::spawn(async move {
            connect_tls(
                client_config,
                client_tcp,
                "localhost".to_string(),
                5.0,
                true,
            )
            .await
            .unwrap()
        });
        let server_tls = accept_tls(server_config, server_tcp, 5.0, true)
            .await
            .unwrap();
        let client_tls = client.await.unwrap();

        assert_eq!(
            tls_stream_alpn_protocol(client_tls).unwrap(),
            Some(b"h2".to_vec())
        );
        assert!(tls_stream_protocol_version(client_tls).unwrap().is_some());
        tls_stream_write_all(client_tls, b"ping".to_vec())
            .await
            .unwrap();
        tls_stream_flush(client_tls).await.unwrap();
        assert_eq!(
            tls_stream_read_chunk(server_tls, 4).await.unwrap(),
            Some(b"ping".to_vec())
        );

        let (server_read, server_write) = tls_stream_split(server_tls);
        tls_write_half_write_all(server_write, b"pong".to_vec())
            .await
            .unwrap();
        tls_write_half_close_notify(server_write).await.unwrap();
        tls_write_half_close_notify(server_write).await.unwrap();
        tls_write_half_flush(server_write).await.unwrap();
        let late_write = tls_write_half_write(server_write, b"!".to_vec()).await;
        assert!(late_write
            .unwrap_err()
            .contains("TLS write side is already close-notified"));
        assert_eq!(
            tls_stream_read_chunk(client_tls, 4).await.unwrap(),
            Some(b"pong".to_vec())
        );
        tls_read_half_close(server_read).unwrap();
        tls_write_half_close(server_write).await.unwrap();
        tls_stream_close(client_tls).await.unwrap();
        close_client_config(client_config).unwrap();
        close_server_config(server_config).unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn mtls_rejects_missing_client_certificate() {
        let materials = materials();
        let client_config =
            client_config_with_roots(materials.server_ca_pem, vec![b"http/1.1".to_vec()]).unwrap();
        let server_config = server_config_require_client_auth(
            materials.server_cert_pem,
            materials.server_key_pem,
            materials.client_ca_pem,
            vec![b"http/1.1".to_vec()],
        )
        .unwrap();
        let (client_tcp, server_tcp) = loopback_tcp_pair().await;

        let client = tokio::spawn(async move {
            connect_tls(
                client_config,
                client_tcp,
                "localhost".to_string(),
                5.0,
                true,
            )
            .await
        });
        let server = accept_tls(server_config, server_tcp, 5.0, true).await;
        let client = client.await.unwrap();
        let server_error =
            server.expect_err("mTLS server handshake without a client certificate must fail");
        assert!(server_error.contains("TLS server handshake failed"));
        if let Ok(handle) = client {
            tls_stream_close(handle).await.unwrap();
        }
        close_client_config(client_config).unwrap();
        close_server_config(server_config).unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn invalid_root_rejects_server_certificate() {
        let good_materials = materials();
        let unrelated_root_materials = materials();
        let wrong_root = unrelated_root_materials.server_ca_pem;
        let client_config =
            client_config_with_roots(wrong_root, vec![b"http/1.1".to_vec()]).unwrap();
        let server_config = server_config(
            good_materials.server_cert_pem,
            good_materials.server_key_pem,
            vec![b"http/1.1".to_vec()],
        )
        .unwrap();
        let (client_tcp, server_tcp) = loopback_tcp_pair().await;

        let client = tokio::spawn(async move {
            connect_tls(
                client_config,
                client_tcp,
                "localhost".to_string(),
                5.0,
                true,
            )
            .await
        });
        let server = accept_tls(server_config, server_tcp, 5.0, true).await;
        let client = client.await.unwrap();
        assert!(
            client.is_err(),
            "TLS client handshake with an untrusted server root must fail"
        );
        if let Err(error) = client {
            assert!(error.contains("TLS client handshake failed"));
        }
        if let Ok(handle) = server {
            tls_stream_close(handle).await.unwrap();
        }
        close_client_config(client_config).unwrap();
        close_server_config(server_config).unwrap();
    }
}
