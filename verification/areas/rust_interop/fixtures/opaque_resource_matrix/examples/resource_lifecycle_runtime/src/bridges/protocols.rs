use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use super::resources::ResourceError;

const IO_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_FRAME_BYTES: usize = 64 * 1024;

pub async fn serve_http(listener: TcpListener) -> Result<(), ResourceError> {
    let (mut stream, _) = timed(listener.accept(), "HTTP accept").await?;
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    while !request.windows(4).any(|window| window == b"\r\n\r\n") {
        let read = timed(stream.read(&mut buffer), "HTTP read").await?;
        if read == 0 {
            return Err(ResourceError::new("HTTP peer closed before request"));
        }
        request.extend_from_slice(&buffer[..read]);
        if request.len() > MAX_FRAME_BYTES {
            return Err(ResourceError::new("HTTP request exceeded frame limit"));
        }
    }
    let body = "echo:reqwest";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    timed(stream.write_all(response.as_bytes()), "HTTP write").await?;
    timed(stream.shutdown(), "HTTP shutdown").await?;
    Ok(())
}

pub async fn serve_redis(listener: TcpListener) -> Result<(), ResourceError> {
    let (mut stream, _) = timed(listener.accept(), "Redis accept").await?;
    loop {
        let Some(command) = read_resp_command(&mut stream).await? else {
            return Ok(());
        };
        let name = command
            .first()
            .map(|part| part.to_ascii_uppercase())
            .unwrap_or_default();
        let response: &[u8] = match name.as_slice() {
            b"PING" => b"+PONG\r\n",
            b"QUIT" => b"+OK\r\n",
            _ => b"-ERR unsupported deterministic harness command\r\n",
        };
        timed(stream.write_all(response), "Redis write").await?;
        if name == b"QUIT" {
            return Ok(());
        }
    }
}

pub async fn serve_redis_malformed(listener: TcpListener) -> Result<(), ResourceError> {
    let (mut stream, _) = timed(listener.accept(), "Redis malformed accept").await?;
    timed(
        stream.write_all(b"!not-a-resp-frame\r\n"),
        "Redis malformed write",
    )
    .await?;
    timed(stream.shutdown(), "Redis malformed shutdown").await
}

async fn read_resp_command(
    stream: &mut tokio::net::TcpStream,
) -> Result<Option<Vec<Vec<u8>>>, ResourceError> {
    let Some(line) = read_line(stream).await? else {
        return Ok(None);
    };
    let count = line
        .strip_prefix(b"*")
        .ok_or_else(|| ResourceError::new("Redis harness expected array frame"))?;
    let count = parse_usize(count, "Redis array length")?;
    let mut parts = Vec::with_capacity(count);
    for _ in 0..count {
        let marker = read_line(stream)
            .await?
            .ok_or_else(|| ResourceError::new("Redis peer closed during bulk length"))?;
        let length = marker
            .strip_prefix(b"$")
            .ok_or_else(|| ResourceError::new("Redis harness expected bulk string"))?;
        let length = parse_usize(length, "Redis bulk length")?;
        if length > MAX_FRAME_BYTES {
            return Err(ResourceError::new("Redis bulk frame exceeded limit"));
        }
        let mut value = vec![0_u8; length + 2];
        timed(stream.read_exact(&mut value), "Redis bulk read").await?;
        if !value.ends_with(b"\r\n") {
            return Err(ResourceError::new("Redis bulk frame lacked terminator"));
        }
        value.truncate(length);
        parts.push(value);
    }
    Ok(Some(parts))
}

async fn read_line(stream: &mut tokio::net::TcpStream) -> Result<Option<Vec<u8>>, ResourceError> {
    let mut line = Vec::new();
    loop {
        let mut byte = [0_u8; 1];
        let read = timed(stream.read(&mut byte), "protocol line read").await?;
        if read == 0 {
            return if line.is_empty() {
                Ok(None)
            } else {
                Err(ResourceError::new("protocol peer closed during line"))
            };
        }
        line.push(byte[0]);
        if line.ends_with(b"\r\n") {
            line.truncate(line.len() - 2);
            return Ok(Some(line));
        }
        if line.len() > MAX_FRAME_BYTES {
            return Err(ResourceError::new("protocol line exceeded limit"));
        }
    }
}

fn parse_usize(raw: &[u8], context: &str) -> Result<usize, ResourceError> {
    let text = std::str::from_utf8(raw).map_err(|error| ResourceError::context(context, error))?;
    text.parse::<usize>()
        .map_err(|error| ResourceError::context(context, error))
}

pub async fn serve_postgres(listener: TcpListener) -> Result<(), ResourceError> {
    let (mut stream, _) = timed(listener.accept(), "PostgreSQL accept").await?;
    let startup = read_pg_startup(&mut stream).await?;
    if startup.get(..4) != Some(&196_608_i32.to_be_bytes()) {
        return Err(ResourceError::new(
            "PostgreSQL harness received unexpected startup protocol",
        ));
    }
    write_pg_message(&mut stream, b'R', &0_i32.to_be_bytes()).await?;
    write_pg_message(&mut stream, b'K', &[0; 8]).await?;
    write_pg_message(&mut stream, b'Z', b"I").await?;

    loop {
        let Some((tag, body)) = read_pg_message(&mut stream).await? else {
            return Ok(());
        };
        match tag {
            b'Q' => {
                if !body.starts_with(b"SELECT 1") {
                    write_pg_error(&mut stream, "unsupported deterministic query").await?;
                    write_pg_message(&mut stream, b'Z', b"I").await?;
                    continue;
                }
                write_pg_row_description(&mut stream).await?;
                let mut row = Vec::new();
                row.extend_from_slice(&1_i16.to_be_bytes());
                row.extend_from_slice(&1_i32.to_be_bytes());
                row.push(b'1');
                write_pg_message(&mut stream, b'D', &row).await?;
                write_pg_message(&mut stream, b'C', b"SELECT 1\0").await?;
                write_pg_message(&mut stream, b'Z', b"I").await?;
            }
            b'X' => return Ok(()),
            _ => {
                write_pg_error(&mut stream, "unsupported deterministic frame").await?;
                write_pg_message(&mut stream, b'Z', b"I").await?;
            }
        }
    }
}

pub async fn serve_postgres_early_close(listener: TcpListener) -> Result<(), ResourceError> {
    let (mut stream, _) = timed(listener.accept(), "PostgreSQL early-close accept").await?;
    let _startup = read_pg_startup(&mut stream).await?;
    timed(stream.shutdown(), "PostgreSQL early-close shutdown").await
}

async fn read_pg_startup(stream: &mut tokio::net::TcpStream) -> Result<Vec<u8>, ResourceError> {
    let mut length = [0_u8; 4];
    timed(stream.read_exact(&mut length), "PostgreSQL startup length").await?;
    let length = i32::from_be_bytes(length);
    if !(8..=MAX_FRAME_BYTES as i32).contains(&length) {
        return Err(ResourceError::new("invalid PostgreSQL startup length"));
    }
    let mut body = vec![0_u8; length as usize - 4];
    timed(stream.read_exact(&mut body), "PostgreSQL startup body").await?;
    Ok(body)
}

async fn read_pg_message(
    stream: &mut tokio::net::TcpStream,
) -> Result<Option<(u8, Vec<u8>)>, ResourceError> {
    let mut tag = [0_u8; 1];
    let read = timed(stream.read(&mut tag), "PostgreSQL message tag").await?;
    if read == 0 {
        return Ok(None);
    }
    let mut length = [0_u8; 4];
    timed(stream.read_exact(&mut length), "PostgreSQL message length").await?;
    let length = i32::from_be_bytes(length);
    if !(4..=MAX_FRAME_BYTES as i32).contains(&length) {
        return Err(ResourceError::new("invalid PostgreSQL message length"));
    }
    let mut body = vec![0_u8; length as usize - 4];
    timed(stream.read_exact(&mut body), "PostgreSQL message body").await?;
    Ok(Some((tag[0], body)))
}

async fn write_pg_message(
    stream: &mut tokio::net::TcpStream,
    tag: u8,
    body: &[u8],
) -> Result<(), ResourceError> {
    let length = i32::try_from(body.len() + 4)
        .map_err(|error| ResourceError::context("PostgreSQL response length", error))?;
    let mut frame = Vec::with_capacity(body.len() + 5);
    frame.push(tag);
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(body);
    timed(stream.write_all(&frame), "PostgreSQL write").await
}

async fn write_pg_row_description(stream: &mut tokio::net::TcpStream) -> Result<(), ResourceError> {
    let mut body = Vec::new();
    body.extend_from_slice(&1_i16.to_be_bytes());
    body.extend_from_slice(b"value\0");
    body.extend_from_slice(&0_i32.to_be_bytes());
    body.extend_from_slice(&0_i16.to_be_bytes());
    body.extend_from_slice(&23_i32.to_be_bytes());
    body.extend_from_slice(&4_i16.to_be_bytes());
    body.extend_from_slice(&(-1_i32).to_be_bytes());
    body.extend_from_slice(&0_i16.to_be_bytes());
    write_pg_message(stream, b'T', &body).await
}

async fn write_pg_error(
    stream: &mut tokio::net::TcpStream,
    message: &str,
) -> Result<(), ResourceError> {
    let mut body = Vec::new();
    body.extend_from_slice(b"SERROR\0C0A000\0M");
    body.extend_from_slice(message.as_bytes());
    body.extend_from_slice(b"\0\0");
    write_pg_message(stream, b'E', &body).await
}

async fn timed<T, E>(
    future: impl std::future::Future<Output = Result<T, E>>,
    context: &str,
) -> Result<T, ResourceError>
where
    E: std::fmt::Display,
{
    tokio::time::timeout(IO_TIMEOUT, future)
        .await
        .map_err(|_| ResourceError::new(format!("{context} timed out")))?
        .map_err(|error| ResourceError::context(context, error))
}
