use std::{
    collections::HashMap,
    io::{self, Write as _},
    process::{Command, ExitStatus, Output, Stdio},
    sync::{
        LazyLock, Mutex, MutexGuard,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use sifr_runtime::interop::SifrIntBridge;

mod async_child;
mod async_ops;
mod child;

pub use async_child::{
    process_async_child_stderr, process_async_child_stdin, process_async_child_stdout,
    process_async_kill, process_async_pipe_close, process_async_pipe_read,
    process_async_pipe_read_all, process_async_pipe_reader_close, process_async_pipe_write_all,
    process_async_register_scoped_child, process_async_remove_observed, process_async_spawn,
    process_async_take_child, process_async_terminate, process_async_wait, process_handle_wait,
};
pub use async_ops::{
    process_async_output, process_async_output_timeout, process_async_run,
    process_async_run_timeout, process_async_shell_output, process_async_shell_output_timeout,
    process_async_shell_run,
};
pub use child::{
    process_child_close, process_child_stderr, process_child_stdin, process_child_stdout,
    process_kill, process_pipe_close, process_pipe_read, process_pipe_read_all,
    process_pipe_reader_close, process_pipe_write_all, process_spawn, process_terminate,
    process_wait,
};

#[derive(Clone, Debug)]
struct StoredProcessOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_text: Option<String>,
    stderr_text: Option<String>,
    status: Vec<SifrIntBridge>,
    timed_out: bool,
}

static PROCESS_OUTPUTS: LazyLock<Mutex<HashMap<String, StoredProcessOutput>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_PROCESS_OUTPUT_ID: AtomicU64 = AtomicU64::new(1);

pub fn process_run(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
) -> Result<Vec<SifrIntBridge>, io::Error> {
    let mut command = normal_command(program, args, env, cwd, has_cwd);
    command.status().map(status_tuple)
}

pub fn process_output(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin: &[u8],
    has_stdin: bool,
) -> Result<String, io::Error> {
    let mut command = normal_command(program, args, env, cwd, has_cwd);
    output_command(&mut command, stdin, has_stdin).and_then(|output| store_output(output, false))
}

pub fn process_output_text(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin: &[u8],
    has_stdin: bool,
    encoding: &str,
) -> Result<String, io::Error> {
    let mut command = normal_command(program, args, env, cwd, has_cwd);
    let output = output_command(&mut command, stdin, has_stdin)?;
    store_text_output(output, false, encoding)
}

pub fn process_output_timeout(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin: &[u8],
    has_stdin: bool,
    timeout: f64,
) -> Result<String, io::Error> {
    let mut command = normal_command(program, args, env, cwd, has_cwd);
    configure_process_group(&mut command);
    output_command_timeout(&mut command, stdin, has_stdin, timeout)
}

pub fn process_shell_run(script: &str) -> Result<Vec<SifrIntBridge>, io::Error> {
    let mut command = shell_command(script);
    command.status().map(status_tuple)
}

pub fn process_shell_output(
    script: &str,
    stdin: &[u8],
    has_stdin: bool,
) -> Result<String, io::Error> {
    let mut command = shell_command(script);
    output_command(&mut command, stdin, has_stdin).and_then(|output| store_output(output, false))
}

pub fn process_shell_output_text(
    script: &str,
    stdin: &[u8],
    has_stdin: bool,
    encoding: &str,
) -> Result<String, io::Error> {
    let mut command = shell_command(script);
    let output = output_command(&mut command, stdin, has_stdin)?;
    store_text_output(output, false, encoding)
}

pub fn process_shell_output_timeout(
    script: &str,
    stdin: &[u8],
    has_stdin: bool,
    timeout: f64,
) -> Result<String, io::Error> {
    let mut command = shell_command(script);
    configure_process_group(&mut command);
    output_command_timeout(&mut command, stdin, has_stdin, timeout)
}

fn normal_command(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
) -> Command {
    let mut command = Command::new(program);
    command.args(args.iter());
    for item in env {
        if let Some((key, value)) = item.split_once('=') {
            command.env(key, value);
        }
    }
    if has_cwd {
        command.current_dir(cwd);
    }
    command
}

fn shell_command(script: &str) -> Command {
    let mut command = Command::new("sh");
    command.arg("-c");
    command.arg(script);
    command
}

fn output_command(
    command: &mut Command,
    stdin: &[u8],
    has_stdin: bool,
) -> Result<Output, io::Error> {
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn()?;
    write_child_stdin(&mut child, stdin, has_stdin)?;
    child.wait_with_output()
}

fn output_command_timeout(
    command: &mut Command,
    stdin: &[u8],
    has_stdin: bool,
    timeout: f64,
) -> Result<String, io::Error> {
    if !timeout.is_finite() || timeout < 0.0 {
        return Err(io::Error::other(format!(
            "process timeout must be finite and non-negative, got {timeout}"
        )));
    }
    let deadline = Instant::now()
        .checked_add(Duration::try_from_secs_f64(timeout).map_err(io::Error::other)?)
        .ok_or_else(|| io::Error::other("process timeout is too large for this host clock"))?;

    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn()?;
    write_child_stdin(&mut child, stdin, has_stdin)?;

    let mut timed_out = false;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            terminate_process_group_or_child(&mut child)?;
            timed_out = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let output = child.wait_with_output()?;
    store_output(output, timed_out)
}

fn write_child_stdin(
    child: &mut std::process::Child,
    stdin: &[u8],
    has_stdin: bool,
) -> Result<(), io::Error> {
    if !has_stdin {
        return Ok(());
    }
    if let Some(mut child_stdin) = child.stdin.take() {
        child_stdin.write_all(stdin)?;
    }
    Ok(())
}

pub fn process_output_stdout(handle: &str) -> Result<Vec<u8>, io::Error> {
    with_stored_output(handle, |output| Ok(output.stdout.clone()))
}

pub fn process_output_stderr(handle: &str) -> Result<Vec<u8>, io::Error> {
    with_stored_output(handle, |output| Ok(output.stderr.clone()))
}

pub fn process_output_stdout_text(handle: &str) -> Result<String, io::Error> {
    with_stored_output(handle, |output| {
        output
            .stdout_text
            .clone()
            .ok_or_else(|| io::Error::other("process output handle does not contain text stdout"))
    })
}

pub fn process_output_stderr_text(handle: &str) -> Result<String, io::Error> {
    with_stored_output(handle, |output| {
        output
            .stderr_text
            .clone()
            .ok_or_else(|| io::Error::other("process output handle does not contain text stderr"))
    })
}

pub fn process_output_status(handle: &str) -> Result<Vec<SifrIntBridge>, io::Error> {
    with_stored_output(handle, |output| Ok(output.status.clone()))
}

#[must_use]
pub fn process_output_timed_out(handle: &str) -> bool {
    process_outputs()
        .get(handle)
        .is_some_and(|output| output.timed_out)
}

pub fn process_output_close(handle: &str) {
    process_outputs().remove(handle);
}

fn store_output(output: Output, timed_out: bool) -> Result<String, io::Error> {
    store_output_parts(output, timed_out, None, None)
}

fn store_text_output(output: Output, timed_out: bool, encoding: &str) -> Result<String, io::Error> {
    let stdout_text = decode_text(&output.stdout, encoding)?;
    let stderr_text = decode_text(&output.stderr, encoding)?;
    store_output_parts(output, timed_out, Some(stdout_text), Some(stderr_text))
}

fn store_output_parts(
    output: Output,
    timed_out: bool,
    stdout_text: Option<String>,
    stderr_text: Option<String>,
) -> Result<String, io::Error> {
    store_output_components(
        output.stdout,
        output.stderr,
        output.status,
        timed_out,
        stdout_text,
        stderr_text,
    )
}

pub(super) fn store_output_components(
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: ExitStatus,
    timed_out: bool,
    stdout_text: Option<String>,
    stderr_text: Option<String>,
) -> Result<String, io::Error> {
    let id = next_output_id();
    let stored = StoredProcessOutput {
        stdout,
        stderr,
        stdout_text,
        stderr_text,
        status: status_tuple(status),
        timed_out,
    };
    process_outputs().insert(id.clone(), stored);
    Ok(id)
}

fn with_stored_output<T>(
    handle: &str,
    f: impl FnOnce(&StoredProcessOutput) -> Result<T, io::Error>,
) -> Result<T, io::Error> {
    let outputs = process_outputs();
    let output = outputs
        .get(handle)
        .ok_or_else(|| io::Error::other("unknown process output handle"))?;
    f(output)
}

fn process_outputs() -> MutexGuard<'static, HashMap<String, StoredProcessOutput>> {
    PROCESS_OUTPUTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn next_output_id() -> String {
    format!(
        "process-output-{}",
        NEXT_PROCESS_OUTPUT_ID.fetch_add(1, Ordering::Relaxed)
    )
}

fn decode_text(data: &[u8], encoding: &str) -> Result<String, io::Error> {
    sifr_runtime::encoding::decode_text(data, encoding, "strict").map_err(io::Error::other)
}

pub(super) fn status_tuple(status: ExitStatus) -> Vec<SifrIntBridge> {
    let signal = exit_signal(&status);
    vec![
        SifrIntBridge::from(i64::from(status.code().unwrap_or(-1))),
        SifrIntBridge::from(signal.unwrap_or(0)),
        SifrIntBridge::from(i64::from(signal.is_some())),
    ]
}

pub(super) fn timeout_status_tuple() -> Vec<SifrIntBridge> {
    vec![
        SifrIntBridge::from(-1),
        SifrIntBridge::from(0),
        SifrIntBridge::from(0),
        SifrIntBridge::from(1),
    ]
}

fn exit_signal(status: &ExitStatus) -> Option<i64> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt as _;

        return status.signal().map(i64::from);
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}

fn configure_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;

        command.process_group(0);
    }
    #[cfg(not(unix))]
    {
        let _ = command;
    }
}

fn terminate_process_group_or_child(child: &mut std::process::Child) -> Result<(), io::Error> {
    #[cfg(unix)]
    {
        let process_group = format!("-{}", child.id());
        let _term_status = Command::new("kill")
            .arg("-TERM")
            .arg(&process_group)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        std::thread::sleep(Duration::from_millis(50));
        let _kill_status = Command::new("kill")
            .arg("-KILL")
            .arg(&process_group)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        child.kill()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        process_output_close, process_output_status, process_output_stdout_text,
        process_output_text, process_output_timeout, process_run, process_shell_output_text,
    };

    fn empty() -> Vec<String> {
        Vec::new()
    }

    #[test]
    fn sync_process_status_and_output_are_observable() {
        let args = vec!["-c".to_string(), "printf ok".to_string()];
        let status = process_run("sh", &args, &empty(), "", false).expect("status should run");
        assert_eq!(status[0].to_i64_saturating(), 0);
        assert_eq!(status[2].to_i64_saturating(), 0);

        let handle = process_output_text("sh", &args, &empty(), "", false, b"", false, "utf-8")
            .expect("text output should decode");
        assert_eq!(
            process_output_stdout_text(&handle).expect("stdout text"),
            "ok"
        );
        assert_eq!(
            process_output_status(&handle).expect("status")[0].to_i64_saturating(),
            0
        );
        process_output_close(&handle);
    }

    #[test]
    fn sync_shell_output_and_timeout_policy_are_observable() {
        let output =
            process_shell_output_text("printf shell", b"", false, "utf-8").expect("shell output");
        assert_eq!(
            process_output_stdout_text(&output).expect("stdout text"),
            "shell"
        );
        process_output_close(&output);

        let err = process_output_timeout("sh", &empty(), &empty(), "", false, b"", false, f64::NAN)
            .expect_err("invalid timeout should be rejected");
        assert!(
            err.to_string()
                .contains("process timeout must be finite and non-negative")
        );
    }
}
