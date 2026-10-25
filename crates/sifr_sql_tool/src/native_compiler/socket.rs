//! Anonymous inherited channel. The outer launcher owns the canonical process.
use crate::compiler_protocol::CompilerClient;
use sifr_sql_contract::{MigrationSourceDeclaration, ProfileAuthority};
use std::io::{self, Read, Write};
use std::os::fd::AsFd;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

struct DeadlineSocket {
    stream: UnixStream,
    deadline: Instant,
}
impl DeadlineSocket {
    fn new(stream: UnixStream, timeout: Duration) -> Result<Self, String> {
        if !stream
            .peer_addr()
            .map_err(|error| error.to_string())?
            .is_unnamed()
            || !stream
                .local_addr()
                .map_err(|error| error.to_string())?
                .is_unnamed()
        {
            return Err("SQL compiler channel must be an anonymous socket pair".into());
        }
        let tick = Some(Duration::from_millis(50));
        stream
            .set_read_timeout(tick)
            .map_err(|error| error.to_string())?;
        stream
            .set_write_timeout(tick)
            .map_err(|error| error.to_string())?;
        let deadline = Instant::now()
            .checked_add(timeout)
            .ok_or("SQL compiler deadline is invalid")?;
        Ok(Self { stream, deadline })
    }
    fn begin(&mut self, timeout: Duration) -> Result<(), String> {
        self.deadline = Instant::now()
            .checked_add(timeout)
            .ok_or("SQL compiler deadline is invalid")?;
        Ok(())
    }
    fn check(&self) -> io::Result<()> {
        if sifr_process::process_signals::cancelled() != 0 {
            // IpcTransport retries Interrupted. Use a terminal error after the
            // owner's scoped signal handler has recorded cancellation.
            return Err(io::Error::other("SQL compiler exchange cancelled"));
        }
        if Instant::now() >= self.deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "SQL compiler exchange deadline exceeded",
            ));
        }
        Ok(())
    }
}
impl Read for DeadlineSocket {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        loop {
            self.check()?;
            match self.stream.read(bytes) {
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                    ) => {}
                value => return value,
            }
        }
    }
}
impl Write for DeadlineSocket {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        loop {
            self.check()?;
            match self.stream.write(bytes) {
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::WouldBlock
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::Interrupted
                    ) => {}
                value => return value,
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

pub(super) struct SocketSession {
    client: CompilerClient<DeadlineSocket>,
    timeout: Duration,
}
impl SocketSession {
    pub(super) fn from_stdin(source_identity: &str, timeout: Duration) -> Result<Self, String> {
        let descriptor = io::stdin()
            .as_fd()
            .try_clone_to_owned()
            .map_err(|error| error.to_string())?;
        let stream = DeadlineSocket::new(UnixStream::from(descriptor), timeout)?;
        Ok(Self {
            client: CompilerClient::connect(stream, source_identity)?,
            timeout,
        })
    }
    pub(super) fn profile(&mut self, name: &str) -> Result<ProfileAuthority, String> {
        self.client.transport_mut().begin(self.timeout)?;
        self.client.profile(name)
    }
    pub(super) fn migration(
        &mut self,
        source: &str,
    ) -> Result<Vec<MigrationSourceDeclaration>, String> {
        self.client.transport_mut().begin(self.timeout)?;
        self.client.migration(source)
    }
    pub(super) fn finish(mut self) -> Result<(), String> {
        self.client.transport_mut().begin(self.timeout)?;
        self.client.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn silent_socket_has_one_absolute_deadline_and_peer_close_is_eof() {
        let (left, mut right) = UnixStream::pair().unwrap();
        let mut stream = DeadlineSocket::new(left, Duration::from_millis(120)).unwrap();
        let started = Instant::now();
        assert_eq!(
            stream.read(&mut [0; 1]).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert!(started.elapsed() < Duration::from_secs(2));
        stream.begin(Duration::from_secs(2)).unwrap();
        right.write_all(b"x").unwrap();
        assert_eq!(stream.read(&mut [0; 1]).unwrap(), 1);
        drop(right);
        assert_eq!(stream.read(&mut [0; 1]).unwrap(), 0);
    }
}
