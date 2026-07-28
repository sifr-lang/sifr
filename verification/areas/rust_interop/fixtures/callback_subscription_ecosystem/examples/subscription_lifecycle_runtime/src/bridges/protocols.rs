use std::net::SocketAddr;

use futures::{SinkExt, StreamExt};
use redis::IntoConnectionInfo;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::{Message, Role};
use tokio_tungstenite::WebSocketStream;

use super::support::{EventCallback, Observations, SubscriptionError, OPERATION_TIMEOUT};

const MAX_REDIS_FRAME_BYTES: usize = 64 * 1024;

pub async fn run_websocket(
    listener: TcpListener,
    address: SocketAddr,
    callback: EventCallback,
    observations: Observations,
) -> Result<(), SubscriptionError> {
    timed(
        async move {
            let server = websocket_server(listener, callback, observations);
            let client = websocket_client(address);
            futures::future::try_join(server, client).await?;
            Ok(())
        },
        "WebSocket scenario",
    )
    .await
}

async fn websocket_server(
    listener: TcpListener,
    callback: EventCallback,
    observations: Observations,
) -> Result<(), SubscriptionError> {
    let (stream, _) = listener
        .accept()
        .await
        .map_err(|error| SubscriptionError::context("WebSocket accept", error))?;
    let mut socket = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;
    let message = socket
        .next()
        .await
        .ok_or_else(|| SubscriptionError::new("WebSocket peer closed before event"))?
        .map_err(|error| SubscriptionError::context("WebSocket receive", error))?;
    let event = message
        .into_text()
        .map_err(|error| SubscriptionError::context("WebSocket text event", error))?
        .to_string();
    invoke_callback(&callback, event.clone(), "WebSocket callback")?;
    observations.record_websocket(event);
    socket
        .send(Message::Text("ack".into()))
        .await
        .map_err(|error| SubscriptionError::context("WebSocket acknowledgement", error))?;
    socket
        .close(None)
        .await
        .map_err(|error| SubscriptionError::context("WebSocket close", error))
}

async fn websocket_client(address: SocketAddr) -> Result<(), SubscriptionError> {
    let stream = TcpStream::connect(address)
        .await
        .map_err(|error| SubscriptionError::context("WebSocket connect", error))?;
    let mut socket = WebSocketStream::from_raw_socket(stream, Role::Client, None).await;
    socket
        .send(Message::Text("ws:hello".into()))
        .await
        .map_err(|error| SubscriptionError::context("WebSocket send", error))?;
    let acknowledgement = socket
        .next()
        .await
        .ok_or_else(|| SubscriptionError::new("WebSocket server closed before acknowledgement"))?
        .map_err(|error| SubscriptionError::context("WebSocket acknowledgement", error))?;
    if !acknowledgement.into_text().is_ok_and(|text| text == "ack") {
        return Err(SubscriptionError::new(
            "WebSocket server returned an unexpected acknowledgement",
        ));
    }
    Ok(())
}

pub async fn run_redis(
    listener: TcpListener,
    address: SocketAddr,
    callback: EventCallback,
    observations: Observations,
) -> Result<(), SubscriptionError> {
    timed(
        async move {
            let server = redis_server(listener);
            let client = redis_client(address, callback, observations);
            futures::future::try_join(server, client).await?;
            Ok(())
        },
        "Redis Pub/Sub scenario",
    )
    .await
}

async fn redis_server(listener: TcpListener) -> Result<(), SubscriptionError> {
    let (mut stream, _) = listener
        .accept()
        .await
        .map_err(|error| SubscriptionError::context("Redis accept", error))?;
    let command = read_resp_command(&mut stream).await?;
    let command_name = command
        .first()
        .map(|part| part.to_ascii_uppercase())
        .unwrap_or_default();
    if command_name != b"SUBSCRIBE" || command.get(1).map(Vec::as_slice) != Some(b"events") {
        return Err(SubscriptionError::new(
            "Redis harness expected SUBSCRIBE events",
        ));
    }
    stream
        .write_all(
            b"*3\r\n$9\r\nsubscribe\r\n$6\r\nevents\r\n:1\r\n\
              *3\r\n$7\r\nmessage\r\n$6\r\nevents\r\n$11\r\nredis:hello\r\n",
        )
        .await
        .map_err(|error| SubscriptionError::context("Redis Pub/Sub response", error))?;
    stream
        .shutdown()
        .await
        .map_err(|error| SubscriptionError::context("Redis shutdown", error))
}

async fn redis_client(
    address: SocketAddr,
    callback: EventCallback,
    observations: Observations,
) -> Result<(), SubscriptionError> {
    let raw_info = format!("redis://{address}/")
        .into_connection_info()
        .map_err(|error| SubscriptionError::context("Redis connection info", error))?;
    let settings = raw_info.redis_settings().clone().set_skip_set_lib_name();
    let client = redis::Client::open(raw_info.set_redis_settings(settings))
        .map_err(|error| SubscriptionError::context("Redis client", error))?;
    let mut pubsub = client
        .get_async_pubsub()
        .await
        .map_err(|error| SubscriptionError::context("Redis Pub/Sub connect", error))?;
    pubsub
        .subscribe("events")
        .await
        .map_err(|error| SubscriptionError::context("Redis subscribe", error))?;
    let message = pubsub
        .on_message()
        .next()
        .await
        .ok_or_else(|| SubscriptionError::new("Redis Pub/Sub stream ended before event"))?;
    let event = message
        .get_payload::<String>()
        .map_err(|error| SubscriptionError::context("Redis message payload", error))?;
    invoke_callback(&callback, event.clone(), "Redis callback")?;
    observations.record_redis(event);
    Ok(())
}

fn invoke_callback(
    callback: &EventCallback,
    event: String,
    context: &str,
) -> Result<(), SubscriptionError> {
    match callback.call((event,)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(SubscriptionError::new(format!("{context}: {error}"))),
        Err(error) => Err(SubscriptionError::context(context, error)),
    }
}

async fn read_resp_command(stream: &mut TcpStream) -> Result<Vec<Vec<u8>>, SubscriptionError> {
    let line = read_line(stream).await?;
    let count = line
        .strip_prefix(b"*")
        .ok_or_else(|| SubscriptionError::new("Redis harness expected array frame"))?;
    let count = parse_usize(count, "Redis array length")?;
    let mut parts = Vec::with_capacity(count);
    for _ in 0..count {
        let marker = read_line(stream).await?;
        let length = marker
            .strip_prefix(b"$")
            .ok_or_else(|| SubscriptionError::new("Redis harness expected bulk string"))?;
        let length = parse_usize(length, "Redis bulk length")?;
        if length > MAX_REDIS_FRAME_BYTES {
            return Err(SubscriptionError::new(
                "Redis bulk frame exceeded deterministic limit",
            ));
        }
        let mut value = vec![0_u8; length + 2];
        stream
            .read_exact(&mut value)
            .await
            .map_err(|error| SubscriptionError::context("Redis bulk read", error))?;
        if !value.ends_with(b"\r\n") {
            return Err(SubscriptionError::new("Redis bulk frame lacked terminator"));
        }
        value.truncate(length);
        parts.push(value);
    }
    Ok(parts)
}

async fn read_line(stream: &mut TcpStream) -> Result<Vec<u8>, SubscriptionError> {
    let mut line = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        let read = stream
            .read(&mut byte)
            .await
            .map_err(|error| SubscriptionError::context("Redis line read", error))?;
        if read == 0 {
            return Err(SubscriptionError::new(
                "Redis peer closed during command frame",
            ));
        }
        line.push(byte[0]);
        if line.ends_with(b"\r\n") {
            line.truncate(line.len() - 2);
            return Ok(line);
        }
        if line.len() > MAX_REDIS_FRAME_BYTES {
            return Err(SubscriptionError::new(
                "Redis command frame exceeded deterministic limit",
            ));
        }
    }
}

fn parse_usize(raw: &[u8], context: &str) -> Result<usize, SubscriptionError> {
    let text =
        std::str::from_utf8(raw).map_err(|error| SubscriptionError::context(context, error))?;
    text.parse::<usize>()
        .map_err(|error| SubscriptionError::context(context, error))
}

async fn timed<T>(
    future: impl std::future::Future<Output = Result<T, SubscriptionError>>,
    context: &str,
) -> Result<T, SubscriptionError> {
    tokio::time::timeout(OPERATION_TIMEOUT, future)
        .await
        .map_err(|_| SubscriptionError::new(format!("{context} timed out")))?
}
