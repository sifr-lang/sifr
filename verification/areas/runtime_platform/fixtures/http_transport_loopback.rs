use sifr_runtime::{http, net, tls};

const CERT_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIBvzCCAWWgAwIBAgIUXg72etQ8f6Ntm5bSD9NioVQW2AIwCgYIKoZIzj0EAwIw\nFDESMBAGA1UEAwwJbG9jYWxob3N0MCAXDTI2MDYxMjAyMDYwMVoYDzIxMjYwNTE5\nMDIwNjAxWjAUMRIwEAYDVQQDDAlsb2NhbGhvc3QwWTATBgcqhkjOPQIBBggqhkjO\nPQMBBwNCAARpP8aZInRcXedS58LVHfaRvEpy7Q7q77PcXU81yOFyoFSHitSVtjiZ\nR8e8gsD/54jjAjYff7slVUuEub/M5Cp2o4GSMIGPMB0GA1UdDgQWBBRSFYkeBwSV\nwA+LGs+MsRmfi3XzDjAfBgNVHSMEGDAWgBRSFYkeBwSVwA+LGs+MsRmfi3XzDjAa\nBgNVHREEEzARgglsb2NhbGhvc3SHBH8AAAEwDAYDVR0TAQH/BAIwADAOBgNVHQ8B\nAf8EBAMCBaAwEwYDVR0lBAwwCgYIKwYBBQUHAwEwCgYIKoZIzj0EAwIDSAAwRQIg\nNj8TX4MVR0Z3gMC4Q4zRwUscL0Aw0rUcKTn37XOxlLQCIQD9zsCdWzLGaviK3dLQ\no0acvf03F3cshDbwpd25T2269Q==\n-----END CERTIFICATE-----\n";
const KEY_PEM: &str = "-----BEGIN EC PRIVATE KEY-----\nMHcCAQEEIOh7ipDPAfBIzuGhz5Uj/Rz2TXKPblKyBkOQZFPqT3dhoAoGCCqGSM49\nAwEHoUQDQgAEaT/GmSJ0XF3nUufC1R32kbxKcu0O6u+z3F1PNcjhcqBUh4rUlbY4\nmUfHvILA/+eI4wI2H3+7JVVLhLm/zOQqdg==\n-----END EC PRIVATE KEY-----\n";

#[tokio::test]
async fn http1_tcp_roundtrip() -> Result<(), String> {
    let (client, server) = tcp_pair().await?;
    let server_task = tokio::spawn(async move {
        http::http1_respond_tcp(
            server,
            201,
            vec![
                (
                    "content-type".to_string(),
                    "application/octet-stream".to_string(),
                ),
                ("x-sifr-reply".to_string(), "http1".to_string()),
            ],
            b"pong-http1".to_vec(),
            1024,
            1024,
            2.0,
            true,
        )
        .await
    });

    let response = http::http1_request_tcp(
        client,
        "POST".to_string(),
        "/http1-loopback".to_string(),
        vec![("x-sifr-request".to_string(), "http1".to_string())],
        b"ping-http1".to_vec(),
        1024,
        1024,
        2.0,
        true,
    )
    .await?;
    let request = server_task.await.map_err(|error| error.to_string())??;

    assert_eq!(response.status, 201);
    assert_eq!(response.version, "HTTP/1.1");
    assert_eq!(response.body, b"pong-http1");
    assert_eq!(request.method, "POST");
    assert_eq!(request.target, "/http1-loopback");
    assert_eq!(request.version, "HTTP/1.1");
    assert_eq!(request.body, b"ping-http1");

    Ok(())
}

#[tokio::test]
async fn http2_tcp_roundtrip() -> Result<(), String> {
    let (client, server) = tcp_pair().await?;
    let server_task = tokio::spawn(async move {
        http::http2_respond_tcp(
            server,
            202,
            vec![
                (
                    "content-type".to_string(),
                    "application/octet-stream".to_string(),
                ),
                ("x-sifr-reply".to_string(), "h2c".to_string()),
            ],
            b"pong-h2c".to_vec(),
            1024,
            1024,
            2.0,
            true,
        )
        .await
    });

    let response = http::http2_request_tcp(
        client,
        "PUT".to_string(),
        "http://localhost/http2-loopback".to_string(),
        vec![("x-sifr-request".to_string(), "h2c".to_string())],
        b"ping-h2c".to_vec(),
        1024,
        1024,
        2.0,
        true,
    )
    .await?;
    let request = server_task.await.map_err(|error| error.to_string())??;

    assert_eq!(response.status, 202);
    assert_eq!(response.version, "HTTP/2");
    assert_eq!(response.body, b"pong-h2c");
    assert_eq!(request.method, "PUT");
    assert_eq!(request.target, "http://localhost/http2-loopback");
    assert_eq!(request.version, "HTTP/2");
    assert_eq!(request.body, b"ping-h2c");
    Ok(())
}

#[tokio::test]
async fn http2_tls_roundtrip_with_alpn() -> Result<(), String> {
    let (client, server) = tls_pair(vec![b"h2".to_vec()]).await?;
    assert_eq!(tls::tls_stream_alpn_protocol(client)?, Some(b"h2".to_vec()));
    assert_eq!(tls::tls_stream_alpn_protocol(server)?, Some(b"h2".to_vec()));

    let server_task = tokio::spawn(async move {
        http::http2_respond_tls(
            server,
            203,
            vec![
                (
                    "content-type".to_string(),
                    "application/octet-stream".to_string(),
                ),
                ("x-sifr-reply".to_string(), "https-h2".to_string()),
            ],
            b"pong-https-h2".to_vec(),
            1024,
            1024,
            2.0,
            true,
        )
        .await
    });

    let response = http::http2_request_tls(
        client,
        "PATCH".to_string(),
        "https://localhost/https-h2-loopback".to_string(),
        vec![("x-sifr-request".to_string(), "https-h2".to_string())],
        b"ping-https-h2".to_vec(),
        1024,
        1024,
        2.0,
        true,
    )
    .await?;
    let request = server_task.await.map_err(|error| error.to_string())??;

    assert_eq!(response.status, 203);
    assert_eq!(response.version, "HTTP/2");
    assert_eq!(response.body, b"pong-https-h2");
    assert_eq!(request.method, "PATCH");
    assert_eq!(request.target, "https://localhost/https-h2-loopback");
    assert_eq!(request.version, "HTTP/2");
    assert_eq!(request.body, b"ping-https-h2");
    Ok(())
}

async fn tcp_pair() -> Result<(i64, i64), String> {
    let listener = net::listen_tcp("127.0.0.1:0".to_string(), 0, false, true).await?;
    let addr = net::tcp_listener_local_addr(listener)?;
    let client = net::connect_tcp(addr, 2.0, true, String::new(), false).await?;
    let (server, _remote) = net::accept_tcp(listener).await?;
    net::close_tcp_listener(listener)?;
    Ok((client, server))
}

async fn tls_pair(alpn: Vec<Vec<u8>>) -> Result<(i64, i64), String> {
    let server_config = tls::server_config(
        CERT_PEM.as_bytes().to_vec(),
        KEY_PEM.as_bytes().to_vec(),
        alpn.clone(),
    )?;
    let client_config = tls::client_config_with_roots(CERT_PEM.as_bytes().to_vec(), alpn)?;
    let (client_tcp, server_tcp) = tcp_pair().await?;
    let server_task =
        tokio::spawn(async move { tls::accept_tls(server_config, server_tcp, 2.0, true).await });
    let client = tls::connect_tls(
        client_config,
        client_tcp,
        "localhost".to_string(),
        2.0,
        true,
    )
    .await?;
    let server = server_task.await.map_err(|error| error.to_string())??;
    Ok((client, server))
}
