//! HTTP transport runtime support for generated Sifr programs.

use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};

use bytes::Bytes;
use http::header::CONNECTION;
use http::{HeaderMap, HeaderName, HeaderValue, Method, Request, Response, StatusCode, Version};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use tokio::sync::oneshot;

use crate::timeouts::timeout_duration;

#[derive(Clone, Debug)]
pub struct HttpRequestParts {
    pub method: String,
    pub target: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct HttpResponseParts {
    pub status: i64,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Clone)]
struct ResponseSpec {
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
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
    match tokio::time::timeout(timeout_duration(seconds, "HTTP")?, operation).await {
        Ok(result) => result,
        Err(_) => Err(format!("{name} timed out")),
    }
}

fn body_limit(max_body_bytes: i64) -> Result<usize, String> {
    if max_body_bytes <= 0 {
        return Err("HTTP body limit must be positive".to_string());
    }
    usize::try_from(max_body_bytes).map_err(|error| format!("invalid HTTP body limit: {error}"))
}

fn checked_body(body: Vec<u8>, max_body_bytes: i64) -> Result<Vec<u8>, String> {
    let limit = body_limit(max_body_bytes)?;
    if body.len() > limit {
        return Err("HTTP body exceeds configured limit".to_string());
    }
    Ok(body)
}

fn validate_header_pair(name: &str, value: &str) -> Result<(HeaderName, HeaderValue), String> {
    let name = HeaderName::from_bytes(name.as_bytes())
        .map_err(|error| format!("invalid HTTP header name: {error}"))?;
    let value = HeaderValue::from_str(value)
        .map_err(|error| format!("invalid HTTP header value: {error}"))?;
    Ok((name, value))
}

fn append_headers(headers: &mut HeaderMap, pairs: &[(String, String)]) -> Result<(), String> {
    for (name, value) in pairs {
        let (name, value) = validate_header_pair(name, value)?;
        headers.append(name, value);
    }
    Ok(())
}

fn headers_to_pairs(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            let value = value.to_str().map_or_else(
                |_| "<non-text-header-value>".to_string(),
                std::string::ToString::to_string,
            );
            (name.as_str().to_string(), value)
        })
        .collect()
}

fn version_label(version: Version) -> String {
    match version {
        Version::HTTP_09 => "HTTP/0.9",
        Version::HTTP_10 => "HTTP/1.0",
        Version::HTTP_11 => "HTTP/1.1",
        Version::HTTP_2 => "HTTP/2",
        Version::HTTP_3 => "HTTP/3",
        _ => "HTTP/unknown",
    }
    .to_string()
}

fn build_request(
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    version: Version,
) -> Result<Request<Full<Bytes>>, String> {
    let method = method
        .parse::<Method>()
        .map_err(|error| format!("invalid HTTP method: {error}"))?;
    let mut request = Request::builder()
        .method(method)
        .uri(target)
        .version(version)
        .body(Full::new(Bytes::from(body)))
        .map_err(|error| format!("failed to build HTTP request: {error}"))?;
    append_headers(request.headers_mut(), &headers)?;
    Ok(request)
}

fn build_response(spec: ResponseSpec, version: Version) -> Result<Response<Full<Bytes>>, String> {
    let status = u16::try_from(spec.status)
        .ok()
        .and_then(|value| StatusCode::from_u16(value).ok())
        .ok_or_else(|| "invalid HTTP response status".to_string())?;
    let mut response = Response::builder()
        .status(status)
        .version(version)
        .body(Full::new(Bytes::from(spec.body)))
        .map_err(|error| format!("failed to build HTTP response: {error}"))?;
    append_headers(response.headers_mut(), &spec.headers)?;
    if version == Version::HTTP_11 && !response.headers().contains_key(CONNECTION) {
        response
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
    }
    Ok(response)
}

async fn collect_limited(mut body: Incoming, max_body_bytes: i64) -> Result<Vec<u8>, String> {
    let limit = body_limit(max_body_bytes)?;
    let mut collected = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|error| format!("failed to read HTTP body frame: {error}"))?;
        if let Some(data) = frame.data_ref() {
            if collected.len().saturating_add(data.len()) > limit {
                return Err("HTTP body exceeds configured limit".to_string());
            }
            collected.extend_from_slice(data);
        }
    }
    Ok(collected)
}

fn request_parts_from_request(request: Request<()>, body: Vec<u8>) -> HttpRequestParts {
    let (parts, ()) = request.into_parts();
    HttpRequestParts {
        method: parts.method.to_string(),
        target: parts.uri.to_string(),
        version: version_label(parts.version),
        headers: headers_to_pairs(&parts.headers),
        body,
    }
}

fn response_parts_from_response(response: Response<()>, body: Vec<u8>) -> HttpResponseParts {
    let (parts, ()) = response.into_parts();
    HttpResponseParts {
        status: i64::from(parts.status.as_u16()),
        version: version_label(parts.version),
        headers: headers_to_pairs(&parts.headers),
        body,
    }
}

async fn http1_client_request<S>(
    stream: S,
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_response_bytes: i64,
) -> Result<HttpResponseParts, String>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let request = build_request(method, target, headers, body, Version::HTTP_11)?;
    let io = TokioIo::new(stream);
    let (mut sender, connection) = hyper::client::conn::http1::handshake(io)
        .await
        .map_err(|error| format!("HTTP/1.1 client handshake failed: {error}"))?;
    let connection_task = tokio::spawn(async move {
        connection
            .await
            .map_err(|error| format!("HTTP/1.1 client connection failed: {error}"))
    });
    let response = sender
        .send_request(request)
        .await
        .map_err(|error| format!("HTTP/1.1 request failed: {error}"))?;
    drop(sender);
    let (parts, body) = response.into_parts();
    let body = match collect_limited(body, max_response_bytes).await {
        Ok(body) => body,
        Err(error) => {
            connection_task.abort();
            return Err(error);
        }
    };
    connection_task
        .await
        .map_err(|error| format!("HTTP/1.1 client task failed: {error}"))??;
    Ok(response_parts_from_response(
        Response::from_parts(parts, ()),
        body,
    ))
}

async fn http2_client_request<S>(
    stream: S,
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_response_bytes: i64,
) -> Result<HttpResponseParts, String>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let request = build_request(method, target, headers, body, Version::HTTP_2)?;
    let io = TokioIo::new(stream);
    let (mut sender, connection) = hyper::client::conn::http2::Builder::new(TokioExecutor::new())
        .handshake(io)
        .await
        .map_err(|error| format!("HTTP/2 client handshake failed: {error}"))?;
    let connection_task = tokio::spawn(async move {
        connection
            .await
            .map_err(|error| format!("HTTP/2 client connection failed: {error}"))
    });
    let response = sender
        .send_request(request)
        .await
        .map_err(|error| format!("HTTP/2 request failed: {error}"))?;
    drop(sender);
    let (parts, body) = response.into_parts();
    let body = match collect_limited(body, max_response_bytes).await {
        Ok(body) => body,
        Err(error) => {
            connection_task.abort();
            return Err(error);
        }
    };
    connection_task
        .await
        .map_err(|error| format!("HTTP/2 client task failed: {error}"))??;
    Ok(response_parts_from_response(
        Response::from_parts(parts, ()),
        body,
    ))
}

async fn http_server_respond<S>(
    stream: S,
    response: ResponseSpec,
    max_request_bytes: i64,
    version: Version,
) -> Result<HttpRequestParts, String>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    let request_sender = Arc::new(Mutex::new(Some(tx)));
    let response_version = version;
    let service = service_fn(move |request: Request<Incoming>| {
        let request_sender = Arc::clone(&request_sender);
        let response = response.clone();
        async move {
            let (parts, body) = request.into_parts();
            let collected = collect_limited(body, max_request_bytes).await;
            let request = collected
                .map(|body| request_parts_from_request(Request::from_parts(parts, ()), body));
            let response_result = build_response(response, response_version);
            let observed_request = match (request, &response_result) {
                (Ok(request), Ok(_)) => Ok(request),
                (Ok(_), Err(error)) => Err(format!("HTTP server response build failed: {error}")),
                (Err(error), _) => Err(error),
            };
            if let Some(tx) = lock(&request_sender).take() {
                let _ = tx.send(observed_request);
            }
            Ok::<_, std::convert::Infallible>(response_result.unwrap_or_else(|error| {
                let mut fallback = Response::new(Full::new(Bytes::from(error)));
                *fallback.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                fallback
            }))
        }
    });

    let io = TokioIo::new(stream);
    if version == Version::HTTP_2 {
        hyper::server::conn::http2::Builder::new(TokioExecutor::new())
            .serve_connection(io, service)
            .await
            .map_err(|error| format!("HTTP/2 server connection failed: {error}"))?;
    } else {
        hyper::server::conn::http1::Builder::new()
            .serve_connection(io, service)
            .await
            .map_err(|error| format!("HTTP/1.1 server connection failed: {error}"))?;
    }
    rx.await
        .map_err(|error| format!("HTTP server did not observe a request: {error}"))?
}

pub async fn http1_request_tcp(
    tcp_handle: i64,
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpResponseParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::net::consume_stream_for_http(tcp_handle)
                .map_err(|error| format!("HTTP/1.1 transport setup failed: {error}"))?;
            http1_client_request(
                stream,
                method,
                target,
                headers,
                checked_body(body, max_request_bytes)?,
                max_response_bytes,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/1.1 request",
    )
    .await
}

pub async fn http2_request_tcp(
    tcp_handle: i64,
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpResponseParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::net::consume_stream_for_http(tcp_handle)
                .map_err(|error| format!("HTTP/2 transport setup failed: {error}"))?;
            http2_client_request(
                stream,
                method,
                target,
                headers,
                checked_body(body, max_request_bytes)?,
                max_response_bytes,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/2 request",
    )
    .await
}

pub async fn http1_respond_tcp(
    tcp_handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpRequestParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::net::consume_stream_for_http(tcp_handle)
                .map_err(|error| format!("HTTP/1.1 transport setup failed: {error}"))?;
            http_server_respond(
                stream,
                ResponseSpec {
                    status,
                    headers,
                    body: checked_body(body, max_response_bytes)?,
                },
                max_request_bytes,
                Version::HTTP_11,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/1.1 server response",
    )
    .await
}

pub async fn http2_respond_tcp(
    tcp_handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpRequestParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::net::consume_stream_for_http(tcp_handle)
                .map_err(|error| format!("HTTP/2 transport setup failed: {error}"))?;
            http_server_respond(
                stream,
                ResponseSpec {
                    status,
                    headers,
                    body: checked_body(body, max_response_bytes)?,
                },
                max_request_bytes,
                Version::HTTP_2,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/2 server response",
    )
    .await
}

pub async fn http1_request_tls(
    tls_handle: i64,
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpResponseParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::tls::consume_stream_for_http(tls_handle)
                .map_err(|error| format!("HTTP/1.1 TLS transport setup failed: {error}"))?;
            http1_client_request(
                stream,
                method,
                target,
                headers,
                checked_body(body, max_request_bytes)?,
                max_response_bytes,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/1.1 TLS request",
    )
    .await
}

pub async fn http1_respond_tls(
    tls_handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpRequestParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::tls::consume_stream_for_http(tls_handle)
                .map_err(|error| format!("HTTP/1.1 TLS transport setup failed: {error}"))?;
            http_server_respond(
                stream,
                ResponseSpec {
                    status,
                    headers,
                    body: checked_body(body, max_response_bytes)?,
                },
                max_request_bytes,
                Version::HTTP_11,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/1.1 TLS server response",
    )
    .await
}

pub async fn http2_request_tls(
    tls_handle: i64,
    method: String,
    target: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpResponseParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::tls::consume_stream_for_http(tls_handle)
                .map_err(|error| format!("HTTP/2 TLS transport setup failed: {error}"))?;
            http2_client_request(
                stream,
                method,
                target,
                headers,
                checked_body(body, max_request_bytes)?,
                max_response_bytes,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/2 request",
    )
    .await
}

pub async fn http2_respond_tls(
    tls_handle: i64,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    max_request_bytes: i64,
    max_response_bytes: i64,
    timeout_seconds: f64,
    has_timeout: bool,
) -> Result<HttpRequestParts, String> {
    with_optional_timeout(
        async move {
            let stream = crate::tls::consume_stream_for_http(tls_handle)
                .map_err(|error| format!("HTTP/2 TLS transport setup failed: {error}"))?;
            http_server_respond(
                stream,
                ResponseSpec {
                    status,
                    headers,
                    body: checked_body(body, max_response_bytes)?,
                },
                max_request_bytes,
                Version::HTTP_2,
            )
            .await
        },
        timeout_seconds,
        has_timeout,
        "HTTP/2 server response",
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use h2::Reason;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn query_method_uses_the_current_http_registry_constant() {
        let request = build_request(
            Method::QUERY.as_str().to_string(),
            "/search".to_string(),
            Vec::new(),
            Vec::new(),
            Version::HTTP_11,
        )
        .unwrap();

        assert_eq!(request.method(), Method::QUERY);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn http1_malformed_response_maps_to_typed_error() {
        let (client_io, mut server_io) = tokio::io::duplex(4096);
        let server = tokio::spawn(async move {
            let mut buffer = [0_u8; 256];
            let _ = server_io.read(&mut buffer).await.unwrap();
            server_io
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: nope\r\n\r\nbad")
                .await
                .unwrap();
        });

        let error = http1_client_request(
            client_io,
            "GET".to_string(),
            "/malformed".to_string(),
            Vec::new(),
            Vec::new(),
            1024,
        )
        .await
        .expect_err("malformed response must map to an HTTP error");

        assert!(
            error.contains("HTTP/1.1 request failed")
                || error.contains("failed to read HTTP body frame"),
            "{error}"
        );
        server.await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn http1_transfer_encoding_overrides_an_earlier_content_length() {
        let (client_io, mut server_io) = tokio::io::duplex(4096);
        let server = tokio::spawn(async move {
            let mut buffer = [0_u8; 256];
            let _ = server_io.read(&mut buffer).await.unwrap();
            server_io
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 999\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n4\r\nsifr\r\n0\r\n\r\n",
                )
                .await
                .unwrap();
        });

        let response = http1_client_request(
            client_io,
            "GET".to_string(),
            "/framing".to_string(),
            Vec::new(),
            Vec::new(),
            1024,
        )
        .await
        .unwrap();

        assert_eq!(response.body, b"sifr");
        server.await.unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn client_request_and_response_limits_are_independent() {
        let (client_io, server_io) = tokio::io::duplex(4096);
        let server = tokio::spawn(async move {
            http_server_respond(
                server_io,
                ResponseSpec {
                    status: 200,
                    headers: Vec::new(),
                    body: b"response-too-large".to_vec(),
                },
                1024,
                Version::HTTP_11,
            )
            .await
            .unwrap()
        });

        let error = http1_client_request(
            client_io,
            "POST".to_string(),
            "/limits".to_string(),
            Vec::new(),
            b"request-body".to_vec(),
            4,
        )
        .await
        .expect_err("small response limit must not reject outgoing request first");

        assert!(
            error.contains("HTTP body exceeds configured limit"),
            "{error}"
        );
        let observed = server.await.unwrap();
        assert_eq!(observed.body, b"request-body");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn http2_settings_hpack_and_goaway_loopback() {
        let (client_io, server_io) = tokio::io::duplex(8192);
        let mut client_builder = h2::client::Builder::new();
        client_builder.data_frame_budget(128);
        let (mut client, client_connection) = client_builder
            .handshake::<_, Bytes>(client_io)
            .await
            .unwrap();
        let client_driver = tokio::spawn(async move { client_connection.await });
        let mut server_builder = h2::server::Builder::new();
        server_builder.data_frame_budget(128);
        let mut server = server_builder
            .handshake::<_, Bytes>(server_io)
            .await
            .unwrap();

        let server_task = tokio::spawn(async move {
            let (request, mut respond) = server
                .accept()
                .await
                .expect("server connection should stay open")
                .expect("server should receive request");
            assert_eq!(request.uri(), "https://localhost/h2/hpack");
            assert_eq!(request.headers()["x-sifr-hpack"], "dynamic-table-edge");
            let response = Response::builder().status(204).body(()).unwrap();
            respond.send_response(response, true).unwrap();
            server.graceful_shutdown();
            while let Some(next) = server.accept().await {
                if next.is_err() {
                    break;
                }
            }
        });

        client = client.ready().await.unwrap();
        let request = Request::builder()
            .uri("https://localhost/h2/hpack")
            .header("x-sifr-hpack", "dynamic-table-edge")
            .body(())
            .unwrap();
        let (response, _) = client.send_request(request, true).unwrap();
        let response = response.await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        drop(client);

        server_task.await.unwrap();
        if let Err(error) = client_driver.await.unwrap() {
            assert!(
                error.reason().is_none(),
                "GOAWAY shutdown must not map to a stream error reason: {error}"
            );
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn http2_rst_stream_maps_cancel_reason() {
        let (client_io, server_io) = tokio::io::duplex(8192);
        let (mut client, client_connection) = h2::client::handshake(client_io).await.unwrap();
        let client_driver = tokio::spawn(async move { client_connection.await });
        let mut server = h2::server::handshake(server_io).await.unwrap();

        let server_task = tokio::spawn(async move {
            let (_request, mut respond) = server
                .accept()
                .await
                .expect("server connection should stay open")
                .expect("server should receive request");
            let response = Response::builder().status(200).body(()).unwrap();
            let mut stream = respond.send_response(response, false).unwrap();
            stream.send_reset(Reason::CANCEL);
            server.graceful_shutdown();
            while let Some(next) = server.accept().await {
                if next.is_err() {
                    break;
                }
            }
        });

        client = client.ready().await.unwrap();
        let request = Request::builder()
            .uri("https://localhost/h2/reset")
            .body(())
            .unwrap();
        let (response, _) = client.send_request(request, true).unwrap();
        match response.await {
            Ok(response) => {
                let error = response
                    .into_body()
                    .data()
                    .await
                    .expect("RST_STREAM must be observed on response body")
                    .expect_err("RST_STREAM must reject response body");
                assert_eq!(error.reason(), Some(Reason::CANCEL));
            }
            Err(error) => assert_eq!(error.reason(), Some(Reason::CANCEL)),
        }
        drop(client);

        server_task.await.unwrap();
        if let Err(error) = client_driver.await.unwrap() {
            assert!(
                error.reason().is_none(),
                "RST_STREAM must not become a connection error reason: {error}"
            );
        }
    }
}
