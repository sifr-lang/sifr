//! Bounded subprocess capture with Windows Job Object ownership.
use std::io::{self, Read};
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};
use windows_sys::Win32::System::Threading::{
    CREATE_NEW_PROCESS_GROUP, CREATE_SUSPENDED, OpenThread, ResumeThread, THREAD_SUSPEND_RESUME,
};

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

struct Handle(HANDLE);
impl Drop for Handle {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        // SAFETY: this wrapper exclusively owns a valid Win32 handle.
        unsafe { CloseHandle(self.0) };
    }
}

struct OwnedChild {
    child: Child,
    job: Handle,
    kill_on_close: bool,
}

impl OwnedChild {
    #[allow(unsafe_code)]
    fn spawn(command: &mut Command) -> io::Result<Self> {
        // SAFETY: a null name and attributes request a private, unnamed job.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let job = Handle(job);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: the size and pointer match the requested information class.
        if unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        // The primary thread is suspended until it is inside our job. This
        // prevents a child from creating an unowned descendant before assignment.
        command.creation_flags(CREATE_SUSPENDED | CREATE_NEW_PROCESS_GROUP);
        let mut child = command.spawn()?;
        let result = (|| {
            // SAFETY: Child retains a live process handle until it is reaped.
            if unsafe { AssignProcessToJobObject(job.0, child.as_raw_handle()) } == 0 {
                return Err(io::Error::last_os_error());
            }
            resume_primary_thread(child.id())
        })();
        if let Err(error) = result {
            // The job kills any process already assigned to it. Kill directly
            // as well when assignment failed.
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        Ok(Self {
            child,
            job,
            kill_on_close: true,
        })
    }

    #[allow(unsafe_code)]
    fn terminate(&self) {
        // SAFETY: this owned job is live and contains the child and descendants.
        unsafe { TerminateJobObject(self.job.0, 1) };
    }

    #[allow(unsafe_code)]
    fn detach_normal(&mut self) -> io::Result<()> {
        let limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        // A normally exiting user program retains background-process behavior.
        // SAFETY: the size and pointer match the requested information class.
        if unsafe {
            SetInformationJobObject(
                self.job.0,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        self.kill_on_close = false;
        Ok(())
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if self.kill_on_close {
            self.terminate();
        }
        let _ = self.child.wait();
    }
}

#[allow(unsafe_code)]
fn resume_primary_thread(pid: u32) -> io::Result<()> {
    // SAFETY: a snapshot is a new owned handle. THREADENTRY32 is initialized
    // with the required structure size before enumeration.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let snapshot = Handle(snapshot);
    let mut entry = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
        ..THREADENTRY32::default()
    };
    // SAFETY: the snapshot and entry are valid for thread enumeration.
    let mut found = unsafe { Thread32First(snapshot.0, &raw mut entry) } != 0;
    while found {
        if entry.th32OwnerProcessID == pid {
            // SAFETY: request only resume access to this process's thread.
            let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if thread.is_null() {
                return Err(io::Error::last_os_error());
            }
            let thread = Handle(thread);
            // SAFETY: this is a live thread owned by the newly spawned child.
            if unsafe { ResumeThread(thread.0) } == u32::MAX {
                return Err(io::Error::last_os_error());
            }
            return Ok(());
        }
        // SAFETY: same live snapshot and initialized entry.
        found = unsafe { Thread32Next(snapshot.0, &raw mut entry) } != 0;
    }
    Err(io::Error::other(
        "suspended subprocess primary thread disappeared",
    ))
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

/// User programs stream complete output without a correctness deadline.
pub fn run_program(command: &mut Command) -> io::Result<Output> {
    let _signals = crate::process_signals::acquire()?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut owned = OwnedChild::spawn(command)?;
    let status = loop {
        let signal = crate::process_signals::cancelled();
        if signal != 0 {
            owned.terminate();
            owned.child.wait()?;
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
        if let Some(status) = owned.child.try_wait()? {
            break status;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    owned.detach_normal()?;
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

pub fn output(command: &mut Command) -> io::Result<Output> {
    output_with_deadline(command, Duration::from_mins(40))
}

pub fn output_with_deadline(command: &mut Command, deadline: Duration) -> io::Result<Output> {
    let _signals = crate::process_signals::acquire()?;
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut owned = OwnedChild::spawn(command)?;
    let stdout = owned
        .child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("missing stdout pipe"))?;
    let stderr = owned
        .child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("missing stderr pipe"))?;
    let out = std::thread::spawn(move || capture(stdout));
    let err = std::thread::spawn(move || capture(stderr));
    let start = Instant::now();
    let mut cause = None;
    let status = loop {
        if let Some(status) = owned.child.try_wait()? {
            break status;
        }
        let signal = crate::process_signals::cancelled();
        if signal != 0 || start.elapsed() >= deadline {
            cause = Some(signal);
            owned.terminate();
            break owned.child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Descendants retaining pipes are terminated even after direct-child exit.
    owned.terminate();
    let (stdout, out_truncated) = out
        .join()
        .map_err(|_| io::Error::other("stdout capture failed"))??;
    let (mut stderr, err_truncated) = err
        .join()
        .map_err(|_| io::Error::other("stderr capture failed"))??;
    if let Some(signal) = cause {
        return Err(io::Error::new(
            if signal != 0 {
                io::ErrorKind::Interrupted
            } else {
                io::ErrorKind::TimedOut
            },
            ProcessFailure {
                cause: if signal != 0 {
                    "subprocess cancelled"
                } else {
                    "subprocess safety deadline exceeded"
                },
                signal,
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
    use std::io::Write;
    use std::path::Path;

    fn self_command(mode: &str, marker: &Path) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "process_execution::tests::windows_portability_process_child",
                "--nocapture",
            ])
            .env("SIFR_WINDOWS_PROCESS_CHILD", mode)
            .env("SIFR_WINDOWS_PROCESS_MARKER", marker);
        command
    }

    #[test]
    fn windows_portability_process_child() {
        let Ok(mode) = std::env::var("SIFR_WINDOWS_PROCESS_CHILD") else {
            return;
        };
        let marker = std::env::var("SIFR_WINDOWS_PROCESS_MARKER").unwrap();
        match mode.as_str() {
            "descendant" => {
                std::thread::sleep(Duration::from_secs(2));
                std::fs::write(marker, "escaped").unwrap();
            }
            "hold" | "exit" => {
                print!("partial");
                io::stdout().flush().unwrap();
                let mut descendant = self_command("descendant", Path::new(&marker));
                descendant.stdout(Stdio::inherit()).stderr(Stdio::inherit());
                let _descendant = descendant.spawn().unwrap();
                if mode == "hold" {
                    std::thread::sleep(Duration::from_secs(10));
                }
            }
            "large" => {
                io::stdout().write_all(&vec![0; 9_000_000]).unwrap();
            }
            "short" => {
                print!("streamed");
                io::stdout().flush().unwrap();
            }
            "cancel-run" => {
                let trigger = std::thread::spawn(|| {
                    let started = Instant::now();
                    while !crate::process_signals::test_active() {
                        assert!(started.elapsed() < Duration::from_secs(5));
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    crate::process_signals::test_cancel();
                });
                let error = output_with_deadline(
                    &mut self_command("hold", Path::new(&marker)),
                    Duration::from_secs(5),
                )
                .unwrap_err();
                trigger.join().unwrap();
                assert_eq!(error.kind(), io::ErrorKind::Interrupted);
                assert_eq!(failure_exit_code(&error), 128 + libc::SIGINT);
            }
            other => panic!("unknown process test mode: {other}"),
        }
    }

    #[test]
    fn windows_portability_timeout_partial_output_and_descendant_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("escaped");
        let error =
            output_with_deadline(&mut self_command("hold", &marker), Duration::from_secs(1))
                .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        let failure = error
            .get_ref()
            .unwrap()
            .downcast_ref::<ProcessFailure>()
            .unwrap();
        assert!(String::from_utf8_lossy(&failure.stdout).contains("partial"));
        std::thread::sleep(Duration::from_millis(2200));
        assert!(!marker.exists());
    }

    #[test]
    fn windows_portability_direct_exit_descendant_pipe_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("escaped");
        let started = Instant::now();
        let result =
            output_with_deadline(&mut self_command("exit", &marker), Duration::from_secs(5))
                .unwrap();
        assert!(result.status.success());
        assert!(started.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(2200));
        assert!(!marker.exists());
    }

    #[test]
    fn windows_portability_cancellation_and_scoped_concurrency() {
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("escaped");
        let result = self_command("cancel-run", &marker).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        std::thread::sleep(Duration::from_millis(2200));
        assert!(!marker.exists());
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let marker = marker.clone();
                std::thread::spawn(move || output(&mut self_command("short", &marker)).unwrap())
            })
            .collect();
        for worker in workers {
            assert!(worker.join().unwrap().status.success());
        }
    }

    #[test]
    fn windows_portability_bounded_capture_and_normal_streaming() {
        let root = tempfile::tempdir().unwrap();
        let marker = root.path().join("unused");
        let result = output(&mut self_command("large", &marker)).unwrap();
        assert_eq!(result.stdout.len(), STREAM_LIMIT);
        assert!(String::from_utf8_lossy(&result.stderr).contains("stdout_truncated=true"));
        assert!(
            run_program(&mut self_command("short", &marker))
                .unwrap()
                .status
                .success()
        );
    }
}
