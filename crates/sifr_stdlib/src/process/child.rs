use std::{
    collections::HashMap,
    io::{self, Read as _, Write as _},
    process::{Child, Stdio},
    sync::{
        atomic::{AtomicI64, Ordering},
        LazyLock, Mutex, MutexGuard,
    },
};

use sifr_runtime::interop::SifrIntBridge;

use super::{normal_command, status_tuple};

type PipeReader = Box<dyn std::io::Read + Send>;
type PipeWriter = Box<dyn std::io::Write + Send>;

static PROCESS_CHILDREN: LazyLock<Mutex<HashMap<i64, Child>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PROCESS_PIPE_READERS: LazyLock<Mutex<HashMap<i64, PipeReader>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PROCESS_PIPE_WRITERS: LazyLock<Mutex<HashMap<i64, PipeWriter>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_PROCESS_CHILD_ID: AtomicI64 = AtomicI64::new(1);

pub fn process_spawn(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin_mode: &str,
    stdout_mode: &str,
    stderr_mode: &str,
) -> Result<SifrIntBridge, io::Error> {
    let mut command = normal_command(program, args, env, cwd, has_cwd);
    command.stdin(stdio_from_mode(stdin_mode)?);
    command.stdout(stdio_from_mode(stdout_mode)?);
    command.stderr(stdio_from_mode(stderr_mode)?);

    let child = command.spawn()?;
    let handle = next_child_id();
    process_children().insert(handle, child);
    Ok(SifrIntBridge::from(handle))
}

pub fn process_wait(handle: SifrIntBridge) -> Result<Vec<SifrIntBridge>, io::Error> {
    let handle = handle_value(handle);
    let mut child = process_children()
        .remove(&handle)
        .ok_or_else(|| missing_child_error(handle))?;
    child.wait().map(status_tuple)
}

pub fn process_kill(handle: SifrIntBridge) -> Result<(), io::Error> {
    let handle = handle_value(handle);
    let mut children = process_children();
    let child = children
        .get_mut(&handle)
        .ok_or_else(|| missing_child_error(handle))?;
    child.kill()
}

pub fn process_child_close(handle: SifrIntBridge) {
    process_children().remove(&handle_value(handle));
}

pub fn process_terminate(handle: SifrIntBridge) -> Result<(), io::Error> {
    let handle = handle_value(handle);
    let mut children = process_children();
    let child = children
        .get_mut(&handle)
        .ok_or_else(|| missing_child_error(handle))?;
    terminate_child(child)
}

pub fn process_child_stdin(handle: SifrIntBridge) -> Result<SifrIntBridge, io::Error> {
    let handle = handle_value(handle);
    let mut children = process_children();
    let child = children
        .get_mut(&handle)
        .ok_or_else(|| missing_child_error(handle))?;
    let pipe = child.stdin.take().ok_or_else(|| {
        io::Error::other(format!(
            "process stdin pipe is not available or already taken for child handle: {handle}"
        ))
    })?;
    let pipe_handle = next_child_id();
    process_pipe_writers().insert(pipe_handle, Box::new(pipe));
    Ok(SifrIntBridge::from(pipe_handle))
}

pub fn process_child_stdout(handle: SifrIntBridge) -> Result<SifrIntBridge, io::Error> {
    let handle = handle_value(handle);
    let mut children = process_children();
    let child = children
        .get_mut(&handle)
        .ok_or_else(|| missing_child_error(handle))?;
    let pipe = child.stdout.take().ok_or_else(|| {
        io::Error::other(format!(
            "process stdout pipe is not available or already taken for child handle: {handle}"
        ))
    })?;
    let pipe_handle = next_child_id();
    process_pipe_readers().insert(pipe_handle, Box::new(pipe));
    Ok(SifrIntBridge::from(pipe_handle))
}

pub fn process_child_stderr(handle: SifrIntBridge) -> Result<SifrIntBridge, io::Error> {
    let handle = handle_value(handle);
    let mut children = process_children();
    let child = children
        .get_mut(&handle)
        .ok_or_else(|| missing_child_error(handle))?;
    let pipe = child.stderr.take().ok_or_else(|| {
        io::Error::other(format!(
            "process stderr pipe is not available or already taken for child handle: {handle}"
        ))
    })?;
    let pipe_handle = next_child_id();
    process_pipe_readers().insert(pipe_handle, Box::new(pipe));
    Ok(SifrIntBridge::from(pipe_handle))
}

pub fn process_pipe_read_all(handle: SifrIntBridge) -> Result<Vec<u8>, io::Error> {
    let handle = handle_value(handle);
    let mut pipe = process_pipe_readers()
        .remove(&handle)
        .ok_or_else(|| missing_pipe_reader_error(handle))?;
    let mut buffer = Vec::new();
    pipe.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn process_pipe_read(
    handle: SifrIntBridge,
    max_bytes: SifrIntBridge,
) -> Result<Vec<u8>, io::Error> {
    let handle = handle_value(handle);
    let max_bytes = handle_value(max_bytes);
    if max_bytes <= 0 {
        return Err(io::Error::other("process pipe read size must be positive"));
    }
    if max_bytes > 1_048_576 {
        return Err(io::Error::other(
            "process pipe read size exceeds 1048576 bytes",
        ));
    }

    let mut buffer = vec![0_u8; usize::try_from(max_bytes).map_err(io::Error::other)?];
    let read = {
        let mut readers = process_pipe_readers();
        let pipe = readers
            .get_mut(&handle)
            .ok_or_else(|| missing_pipe_reader_error(handle))?;
        pipe.read(buffer.as_mut_slice())?
    };
    buffer.truncate(read);
    if read == 0 {
        process_pipe_readers().remove(&handle);
    }
    Ok(buffer)
}

pub fn process_pipe_reader_close(handle: SifrIntBridge) -> Result<(), io::Error> {
    let handle = handle_value(handle);
    process_pipe_readers()
        .remove(&handle)
        .ok_or_else(|| missing_pipe_reader_error(handle))?;
    Ok(())
}

pub fn process_pipe_write_all(handle: SifrIntBridge, data: &[u8]) -> Result<(), io::Error> {
    let handle = handle_value(handle);
    let mut writers = process_pipe_writers();
    let pipe = writers
        .get_mut(&handle)
        .ok_or_else(|| missing_pipe_writer_error(handle))?;
    pipe.write_all(data)
}

pub fn process_pipe_close(handle: SifrIntBridge) -> Result<(), io::Error> {
    let handle = handle_value(handle);
    process_pipe_writers()
        .remove(&handle)
        .ok_or_else(|| missing_pipe_writer_error(handle))?;
    Ok(())
}

fn stdio_from_mode(mode: &str) -> Result<Stdio, io::Error> {
    match mode {
        "pipe" => Ok(Stdio::piped()),
        "inherit" => Ok(Stdio::inherit()),
        "null" => Ok(Stdio::null()),
        _ => Err(io::Error::other(format!(
            "unsupported process stdio mode: {mode}"
        ))),
    }
}

fn handle_value(handle: SifrIntBridge) -> i64 {
    handle.to_i64_saturating()
}

fn next_child_id() -> i64 {
    NEXT_PROCESS_CHILD_ID.fetch_add(1, Ordering::SeqCst)
}

fn process_children() -> MutexGuard<'static, HashMap<i64, Child>> {
    PROCESS_CHILDREN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn process_pipe_readers() -> MutexGuard<'static, HashMap<i64, PipeReader>> {
    PROCESS_PIPE_READERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn process_pipe_writers() -> MutexGuard<'static, HashMap<i64, PipeWriter>> {
    PROCESS_PIPE_WRITERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn missing_child_error(handle: i64) -> io::Error {
    io::Error::other(format!(
        "process child handle is closed or unknown: {handle}"
    ))
}

fn missing_pipe_reader_error(handle: i64) -> io::Error {
    io::Error::other(format!(
        "process pipe reader handle is closed or unknown: {handle}"
    ))
}

fn missing_pipe_writer_error(handle: i64) -> io::Error {
    io::Error::other(format!(
        "process pipe writer handle is closed or unknown: {handle}"
    ))
}

#[cfg(unix)]
fn terminate_child(child: &mut Child) -> Result<(), io::Error> {
    let status = std::process::Command::new("kill")
        .arg("-TERM")
        .arg(child.id().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "process terminate failed with status: {status}"
        )))
    }
}

#[cfg(not(unix))]
fn terminate_child(_child: &mut Child) -> Result<(), io::Error> {
    Err(io::Error::other(
        "process terminate is unsupported on this host; use kill for forceful termination",
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        process_child_stdout, process_kill, process_pipe_read_all, process_spawn, process_wait,
    };
    use sifr_runtime::interop::SifrIntBridge;

    fn empty() -> Vec<String> {
        Vec::new()
    }

    #[test]
    fn spawned_child_stdout_and_wait_are_observable() {
        let handle = process_spawn(
            "sh",
            &["-c".to_string(), "printf child".to_string()],
            &empty(),
            "",
            false,
            "inherit",
            "pipe",
            "inherit",
        )
        .expect("spawn should succeed");
        let stdout = process_child_stdout(handle.clone()).expect("stdout should be available");
        let bytes = process_pipe_read_all(stdout).expect("stdout should read");
        assert_eq!(bytes, b"child");
        let status = process_wait(handle).expect("wait should succeed");
        assert_eq!(status[0].to_i64_saturating(), 0);
    }

    #[test]
    fn missing_child_handles_are_errors() {
        let err = process_kill(SifrIntBridge::from(9_999_999)).expect_err("unknown child");
        assert!(err.to_string().contains("closed or unknown"));
    }
}
