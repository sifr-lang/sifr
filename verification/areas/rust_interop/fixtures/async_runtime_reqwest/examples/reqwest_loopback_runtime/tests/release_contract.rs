//! Local behavior required by the Reqwest 0.13.5 release and bounded Base64 approval.
use std::error::Error;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[test]
fn basic_auth_uses_the_expected_scalar_base64_wire_value() -> Result<(), Box<dyn Error>> {
    let request = reqwest::Client::builder()
        .no_proxy()
        .build()?
        .get("http://127.0.0.1/auth")
        .basic_auth("user", Some("pass"))
        .build()?;
    let authorization = request
        .headers()
        .get(reqwest::header::AUTHORIZATION)
        .ok_or("missing Basic Auth")?;
    assert_eq!(authorization.to_str()?, "Basic dXNlcjpwYXNz");
    assert!(authorization.is_sensitive());
    Ok(())
}

#[test]
fn body_read_timeout_survives_a_gap_between_chunk_polls() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(5), chunk_timeout_contract()).await?
    })
}

async fn chunk_timeout_contract() -> Result<(), Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let (continue_tx, continue_rx) = tokio::sync::oneshot::channel();
    let (written_tx, written_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await?;
        let mut headers = Vec::new();
        let mut buffer = [0; 512];
        while !headers.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let count = socket.read(&mut buffer).await?;
            if count == 0 || headers.len() + count > 16 * 1024 {
                return Err(std::io::Error::other("incomplete or oversized request"));
            }
            headers.extend_from_slice(&buffer[..count]);
        }
        socket.write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n1\r\na\r\n").await?;
        continue_rx.await.map_err(std::io::Error::other)?;
        socket.write_all(b"1\r\nb\r\n0\r\n\r\n").await?;
        written_tx
            .send(())
            .map_err(|()| std::io::Error::other("client stopped"))?;
        Ok::<_, std::io::Error>(())
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .read_timeout(Duration::from_millis(250))
        .build()?;
    let mut response = client
        .get(format!("http://{address}/chunks"))
        .send()
        .await?;
    assert_eq!(response.chunk().await?.as_deref(), Some(&b"a"[..]));
    // The next read's deadline must remain active even while the caller is idle.
    tokio::time::sleep(Duration::from_millis(500)).await;
    continue_tx.send(()).map_err(|()| "server stopped")?;
    written_rx.await?;
    let error = response
        .chunk()
        .await
        .err()
        .ok_or("expired read deadline was reset on polling")?;
    assert!(error.is_timeout());
    server.await??;
    Ok(())
}
