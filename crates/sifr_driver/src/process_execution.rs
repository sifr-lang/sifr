//! Bounded subprocess capture with process-group ownership and safety deadlines.
use std::io::{self, Read};
use std::os::unix::process::CommandExt;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const STREAM_LIMIT: usize = 8 * 1024 * 1024;

fn capture(mut stream: impl Read) -> io::Result<(Vec<u8>, bool)> {
    let mut output = Vec::new();
    let mut buffer = [0; 8192];
    let mut truncated = false;
    loop {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let keep = count.min(STREAM_LIMIT.saturating_sub(output.len()));
        output.extend_from_slice(&buffer[..keep]);
        truncated |= keep < count;
    }
    Ok((output, truncated))
}

#[allow(unsafe_code)]
fn terminate_group(pid: u32) {
    // SAFETY: kill takes a numeric owned process group, no memory is accessed.
    unsafe {
        if let Ok(pid) = i32::try_from(pid) {
            libc::kill(-pid, libc::SIGKILL);
        }
    }
}

struct OwnedChild(std::process::Child, bool);
impl std::ops::Deref for OwnedChild {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for OwnedChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.1 {
            terminate_group(self.0.id());
        }
        let _ = self.0.wait();
    }
}

#[derive(Debug)]
pub struct ProcessFailure {
    pub cause: &'static str,
    pub signal: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}
impl std::fmt::Display for ProcessFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}; stdout_truncated={} stderr_truncated={}\nstdout:\n{}\nstderr:\n{}",
            self.cause,
            self.stdout_truncated,
            self.stderr_truncated,
            String::from_utf8_lossy(&self.stdout),
            String::from_utf8_lossy(&self.stderr)
        )
    }
}
impl std::error::Error for ProcessFailure {}

/// User programs stream their complete output. Compiler capture limits and
/// correctness deadlines do not change the user's output or runtime duration.
pub fn run_program(command: &mut Command) -> io::Result<Output> {
    let _signals = crate::process_signals::acquire()?;
    command
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut child = OwnedChild(command.spawn()?, true);
    let status = loop {
        let signal = crate::process_signals::cancelled();
        if signal != 0 {
            terminate_group(child.id());
            child.wait()?;
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                ProcessFailure {
                    cause: "subprocess cancelled",
                    signal,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    stdout_truncated: false,
                    stderr_truncated: false,
                },
            ));
        }
        if let Some(status) = child.try_wait()? {
            break status;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // A normally exiting user program retains its own background-process
    // semantics. Cancellation/error paths still tear down the owned group.
    child.1 = false;
    Ok(Output {
        status,
        stdout: Vec::new(),
        stderr: Vec::new(),
    })
}

pub fn failure_exit_code(error: &io::Error) -> i32 {
    error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ProcessFailure>())
        .filter(|failure| failure.signal != 0)
        .map_or(2, |failure| 128 + failure.signal)
}

/// Failed deadlines preserve bounded partial output in the original error.
/// Callers cannot interpret Cargo build-finished as an execution success.
pub fn output(command: &mut Command) -> io::Result<Output> {
    output_with_deadline(command, Duration::from_mins(40))
}

pub fn output_with_deadline(command: &mut Command, deadline: Duration) -> io::Result<Output> {
    let _signals = crate::process_signals::acquire()?;
    command
        .process_group(0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = OwnedChild(command.spawn()?, true);
    let pid = child.id();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing stdout pipe"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("missing stderr pipe"))?;
    let out = std::thread::spawn(move || capture(stdout));
    let err = std::thread::spawn(move || capture(stderr));
    let start = Instant::now();
    let mut timed_out = false;
    let mut cancelled = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if crate::process_signals::cancelled() != 0 || start.elapsed() >= deadline {
            cancelled = crate::process_signals::cancelled() != 0;
            timed_out = true;
            terminate_group(pid);
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Also close descendants retaining pipes after the direct child has exited.
    terminate_group(pid);
    let (stdout, out_truncated) = out
        .join()
        .map_err(|_| io::Error::other("stdout capture failed"))??;
    let (mut stderr, err_truncated) = err
        .join()
        .map_err(|_| io::Error::other("stderr capture failed"))??;
    if timed_out {
        return Err(io::Error::new(
            if cancelled {
                io::ErrorKind::Interrupted
            } else {
                io::ErrorKind::TimedOut
            },
            ProcessFailure {
                cause: if cancelled {
                    "subprocess cancelled"
                } else {
                    "subprocess safety deadline exceeded"
                },
                signal: if cancelled {
                    crate::process_signals::cancelled()
                } else {
                    0
                },
                stdout,
                stderr,
                stdout_truncated: out_truncated,
                stderr_truncated: err_truncated,
            },
        ));
    }
    if out_truncated || err_truncated {
        stderr.extend_from_slice(format!("\n[sifr-process] stdout_truncated={out_truncated} stderr_truncated={err_truncated} exit_status={status}\n").as_bytes());
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dx3_process_deadline_keeps_partial_output_and_kills_group() {
        let root = std::env::temp_dir().join(format!("sifr-dx3-process-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let marker = root.join("escaped");
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg("printf partial; printf error >&2; (sleep 1; touch \"$1\") & wait")
            .arg("sh")
            .arg(&marker);
        let error = output_with_deadline(&mut command, Duration::from_millis(100)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(error.to_string().contains("partial"));
        assert!(error.to_string().contains("error"));
        std::thread::sleep(Duration::from_millis(1100));
        assert!(!marker.exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn dx3_signal_child() {
        let Ok(mode) = std::env::var("SIFR_DX3_PROCESS_CHILD") else {
            return;
        };
        if mode == "stream" {
            let result =
                run_program(Command::new("sh").args(["-c", "head -c 9000000 /dev/zero; exit 7"]))
                    .unwrap();
            assert_eq!(result.status.code(), Some(7));
            return;
        }
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "printf '\\377'; (sleep .1; kill -TERM \"$PPID\") & sleep 10",
        ]);
        let failure = output(&mut command).unwrap_err();
        assert_eq!(failure.kind(), io::ErrorKind::Interrupted);
        let raw = failure
            .get_ref()
            .unwrap()
            .downcast_ref::<ProcessFailure>()
            .unwrap();
        assert_eq!(raw.stdout, vec![255]);
        assert_eq!(raw.signal, libc::SIGTERM);
        // A later independent operation must not inherit a latched cancellation.
        assert!(
            output(Command::new("sh").args(["-c", "exit 0"]))
                .unwrap()
                .status
                .success()
        );
    }

    #[test]
    fn dx3_program_streams_and_cancellation_state_are_scoped() {
        for mode in ["stream", "cancel"] {
            let result = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "process_execution::tests::dx3_signal_child",
                    "--nocapture",
                ])
                .env("SIFR_DX3_PROCESS_CHILD", mode)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            if mode == "stream" {
                assert_eq!(
                    result.stdout.iter().filter(|byte| **byte == 0).count(),
                    9_000_000
                );
            }
        }
    }
    #[test]
    fn dx3_process_truncation_preserves_exit_status() {
        let result =
            output(Command::new("sh").args(["-c", "head -c 9000000 /dev/zero; exit 7"])).unwrap();
        assert_eq!(result.status.code(), Some(7));
        assert_eq!(result.stdout.len(), STREAM_LIMIT);
        assert!(String::from_utf8_lossy(&result.stderr).contains("stdout_truncated=true"));
    }

    #[test]
    fn dx3_process_cargo_success_text_cannot_mask_failed_program() {
        let result = output(Command::new("sh").args([
            "-c",
            "echo '{\"reason\":\"build-finished\",\"success\":true}'; exit 7",
        ]))
        .unwrap();
        assert_eq!(result.status.code(), Some(7));
    }
}
