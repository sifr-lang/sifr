//! Process-owned SQL compiler sessions. Executable and project admission belong
//! to the caller; this module never searches PATH or selects a compiler.
use crate::compiler_protocol::CompilerClient;
use sifr_process::{OwnedChild, exchange_with_deadline, process_signals};
use sifr_sql_contract::{MigrationSourceDeclaration, ProfileAuthority};
use std::io::{self, Read, Write};
use std::process::{ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

struct WorkerPipes {
    input: ChildStdin,
    output: ChildStdout,
}
impl Read for WorkerPipes {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.output.read(bytes)
    }
}
impl Write for WorkerPipes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.input.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.input.flush()
    }
}

/// An explicitly admitted worker. All requests, handshake and shutdown have
/// deadlines; time spent in unrelated database operations does not consume one.
pub struct CompilerSession {
    child: OwnedChild,
    client: Option<CompilerClient<WorkerPipes>>,
    deadline: Duration,
}

impl CompilerSession {
    /// The launcher must already have verified the executable, fixed project and
    /// selected generation/source relationship. This is a lifecycle primitive,
    /// not a helper-selection or permission policy.
    pub fn spawn_admitted(
        command: &mut Command,
        source_identity: &str,
        deadline: Duration,
    ) -> Result<Self, String> {
        if deadline.is_zero() {
            return Err("SQL compiler deadline must be positive".into());
        }
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        let mut child = OwnedChild::spawn(command)
            .map_err(|error| format!("cannot start SQL compiler worker: {error}"))?;
        let pipes = WorkerPipes {
            input: child
                .stdin
                .take()
                .ok_or("SQL compiler worker has no input pipe")?,
            output: child
                .stdout
                .take()
                .ok_or("SQL compiler worker has no output pipe")?,
        };
        let client = exchange_with_deadline(&mut child, deadline, || {
            CompilerClient::connect(pipes, source_identity)
        })
        .map_err(|error| error.to_string())??;
        Ok(Self {
            child,
            client: Some(client),
            deadline,
        })
    }

    pub fn profile(&mut self, name: &str) -> Result<ProfileAuthority, String> {
        self.exchange(|client| client.profile(name))
    }

    pub fn migration(&mut self, source: &str) -> Result<Vec<MigrationSourceDeclaration>, String> {
        self.exchange(|client| client.migration(source))
    }

    fn exchange<T: Send>(
        &mut self,
        operation: impl FnOnce(&mut CompilerClient<WorkerPipes>) -> Result<T, String> + Send,
    ) -> Result<T, String> {
        let client = self
            .client
            .as_mut()
            .ok_or("SQL compiler session is closed")?;
        let result = exchange_with_deadline(&mut self.child, self.deadline, || operation(client))
            .map_err(|error| error.to_string())
            .and_then(|value| value);
        if result.is_err() {
            // A semantic failure also terminates this invocation's worker.
            // Do not leave a failed process live until an unrelated DB operation.
            self.client = None;
            self.child.terminate();
            let _ = self.child.wait();
        }
        result
    }

    pub fn finish(mut self) -> Result<(), String> {
        let client = self.client.take().ok_or("SQL compiler session is closed")?;
        exchange_with_deadline(&mut self.child, self.deadline, || client.finish())
            .map_err(|error| error.to_string())??;
        let started = Instant::now();
        loop {
            if process_signals::cancelled() != 0 {
                return Err("SQL compiler shutdown cancelled".into());
            }
            if started.elapsed() >= self.deadline {
                return Err("SQL compiler worker did not exit after shutdown".into());
            }
            match self.child.try_wait().map_err(|error| error.to_string())? {
                Some(status) if status.success() => return Ok(()),
                Some(status) => return Err(format!("SQL compiler worker exited with {status}")),
                None => std::thread::sleep(Duration::from_millis(10)),
            }
        }
    }
}
