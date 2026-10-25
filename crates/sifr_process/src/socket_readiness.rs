//! Non-consuming readiness for a caller-owned Unix socket.
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

#[derive(Debug, PartialEq, Eq)]
pub enum SocketReadiness {
    Pending,
    Data,
    Closed,
}

#[allow(unsafe_code)]
pub fn socket_readiness(stream: &UnixStream) -> io::Result<SocketReadiness> {
    let mut byte = 0u8;
    // SAFETY: the borrowed stream owns a live socket descriptor; byte is writable
    // for exactly one byte. PEEK preserves the framed bytes for the worker and
    // DONTWAIT keeps the owning process's cancellation loop responsive.
    let read = unsafe {
        libc::recv(
            stream.as_raw_fd(),
            (&raw mut byte).cast(),
            1,
            libc::MSG_PEEK | libc::MSG_DONTWAIT,
        )
    };
    match read {
        0 => Ok(SocketReadiness::Closed),
        1 => Ok(SocketReadiness::Data),
        _ => {
            let error = io::Error::last_os_error();
            if matches!(
                error.kind(),
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
            ) {
                Ok(SocketReadiness::Pending)
            } else {
                Err(error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    #[test]
    fn readiness_preserves_data_and_distinguishes_unused_close() {
        let (mut left, mut right) = UnixStream::pair().unwrap();
        assert_eq!(socket_readiness(&left).unwrap(), SocketReadiness::Pending);
        right.write_all(b"frame").unwrap();
        assert_eq!(socket_readiness(&left).unwrap(), SocketReadiness::Data);
        let mut bytes = [0; 5];
        left.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"frame");
        drop(right);
        assert_eq!(socket_readiness(&left).unwrap(), SocketReadiness::Closed);
    }
}
