//! Bounded subprocess capture with process-group ownership and safety deadlines.
use std::io::{self, Read};
use std::os::unix::process::CommandExt;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

static CANCELLED: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

extern "C" fn cancel(signal: libc::c_int) {
    CANCELLED.store(signal, std::sync::atomic::Ordering::Relaxed);
}

#[allow(unsafe_code)]
fn install_cancellation() {
    static INSTALL: std::sync::Once = std::sync::Once::new();
    INSTALL.call_once(|| {
        // SAFETY: the handler only stores to a lock-free atomic and outlives the process.
        unsafe {
            libc::signal(libc::SIGINT, cancel as *const () as libc::sighandler_t);
            libc::signal(libc::SIGTERM, cancel as *const () as libc::sighandler_t);
        }
    });
}

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

struct OwnedChild(std::process::Child);
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
        terminate_group(self.0.id());
        let _ = self.0.wait();
    }
}

/// Failed deadlines preserve bounded partial output in the original error.
/// Callers cannot interpret Cargo build-finished as an execution success.
pub fn output(command: &mut Command) -> io::Result<Output> {
    output_with_deadline(command, Duration::from_mins(40))
}

pub fn output_with_deadline(command: &mut Command, deadline: Duration) -> io::Result<Output> {
    install_cancellation();
    command
        .process_group(0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = OwnedChild(command.spawn()?);
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
        if CANCELLED.load(std::sync::atomic::Ordering::Relaxed) != 0 || start.elapsed() >= deadline
        {
            cancelled = CANCELLED.load(std::sync::atomic::Ordering::Relaxed) != 0;
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
            format!(
                "{}; stdout_truncated={out_truncated} stderr_truncated={err_truncated}\nstdout:\n{}\nstderr:\n{}",
                if cancelled {
                    "subprocess cancelled"
                } else {
                    "subprocess safety deadline exceeded"
                },
                String::from_utf8_lossy(&stdout),
                String::from_utf8_lossy(&stderr)
            ),
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
