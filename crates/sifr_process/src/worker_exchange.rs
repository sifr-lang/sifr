//! Deadlines for blocking exchanges with an owned worker on every supported host.
use crate::{OwnedChild, process_signals};
use std::io;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

/// Bound an exchange that performs IO exclusively with this worker's pipes.
/// The child and its descendants are terminated before joining a blocked IO
/// operation. Do not use this for unrelated IO or arbitrary computation.
pub fn exchange_with_deadline<T: Send>(
    child: &mut OwnedChild,
    deadline: Duration,
    exchange: impl FnOnce() -> T + Send,
) -> io::Result<T> {
    if deadline.is_zero() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "worker deadline must be positive",
        ));
    }
    let started = Instant::now();
    std::thread::scope(|scope| {
        let (sender, receiver) = mpsc::sync_channel(1);
        let operation = scope.spawn(move || {
            // A cancelled owner may no longer need the response.
            let _ = sender.send(exchange());
        });
        let result = loop {
            if process_signals::cancelled() != 0 {
                break Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "worker exchange cancelled",
                ));
            }
            let remaining = deadline.saturating_sub(started.elapsed());
            if remaining.is_zero() {
                break Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "worker exchange deadline exceeded",
                ));
            }
            match receiver.recv_timeout(remaining.min(Duration::from_millis(10))) {
                Ok(value) => break Ok(value),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    break Err(io::Error::other("worker IO operation failed"));
                }
            }
        };
        if result.is_err() {
            child.terminate();
        }
        if operation.join().is_err() {
            return Err(io::Error::other("worker IO operation panicked"));
        }
        result
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::process::{Command, Stdio};

    #[test]
    fn exchange_reuses_owned_worker_for_multiple_requests() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "while read line; do printf '%s\\n' \"$line\"; done"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut output = child.stdout.take().unwrap();
        for message in [b"first\n".as_slice(), b"second\n".as_slice()] {
            let received = exchange_with_deadline(&mut child, Duration::from_secs(2), || {
                input.write_all(message)?;
                input.flush()?;
                let mut response = vec![0; message.len()];
                output.read_exact(&mut response)?;
                Ok::<_, io::Error>(response)
            })
            .unwrap()
            .unwrap();
            assert_eq!(received, message);
        }
    }

    #[test]
    fn deadline_interrupts_silent_worker_and_descendant_pipe() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "sleep 30 & wait"])
            .stdout(Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let mut output = child.stdout.take().unwrap();
        let start = Instant::now();
        let error = exchange_with_deadline(&mut child, Duration::from_millis(100), || {
            output.read_to_end(&mut Vec::new())
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() < Duration::from_secs(2));
        assert!(!child.wait().unwrap().success());
    }

    #[test]
    fn premature_worker_eof_is_preserved_for_protocol_validation() {
        let mut command = Command::new("sh");
        command
            .args(["-c", "printf partial; exit 7"])
            .stdout(Stdio::piped());
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let mut output = child.stdout.take().unwrap();
        let result = exchange_with_deadline(&mut child, Duration::from_secs(2), || {
            let mut frame = [0; 128];
            output.read_exact(&mut frame)
        })
        .unwrap();
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(child.wait().unwrap().code(), Some(7));
    }

    #[test]
    fn io_panic_stops_worker_and_becomes_an_explicit_error() {
        let mut command = Command::new("sh");
        command.args(["-c", "sleep 30"]);
        let mut child = OwnedChild::spawn(&mut command).unwrap();
        let error = exchange_with_deadline(&mut child, Duration::from_secs(2), || {
            panic!("controlled worker IO failure");
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert!(!child.wait().unwrap().success());
    }
}
