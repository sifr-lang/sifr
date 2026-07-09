use std::{
    collections::HashMap,
    future::Future,
    io,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, AtomicI64, Ordering},
        Arc, LazyLock, Mutex, MutexGuard,
    },
};

use sifr_runtime::interop::SifrIntBridge;
use tokio::{
    io::{AsyncRead, AsyncReadExt as _, AsyncWrite, AsyncWriteExt as _},
    process::{Child, Command},
};

use super::status_tuple;

type ProcessFuture<T> = Pin<Box<dyn Future<Output = Result<T, io::Error>> + Send>>;
type AsyncPipeReader = Box<dyn AsyncRead + Unpin + Send>;
type AsyncPipeWriter = Box<dyn AsyncWrite + Unpin + Send>;

static PROCESS_ASYNC_CHILDREN: LazyLock<Mutex<HashMap<i64, Child>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PROCESS_ASYNC_CHILD_OBSERVED: LazyLock<Mutex<HashMap<i64, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PROCESS_ASYNC_PIPE_READERS: LazyLock<Mutex<HashMap<i64, AsyncPipeReader>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PROCESS_ASYNC_PIPE_WRITERS: LazyLock<Mutex<HashMap<i64, AsyncPipeWriter>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_PROCESS_ASYNC_CHILD_ID: AtomicI64 = AtomicI64::new(1);

pub fn process_async_spawn(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin_mode: &str,
    stdout_mode: &str,
    stderr_mode: &str,
    has_stdin: bool,
) -> ProcessFuture<SifrIntBridge> {
    let program = program.to_string();
    let args = args.to_vec();
    let env = env.to_vec();
    let cwd = cwd.to_string();
    let stdin_mode = stdin_mode.to_string();
    let stdout_mode = stdout_mode.to_string();
    let stderr_mode = stderr_mode.to_string();
    Box::pin(async move {
        if has_stdin {
            return Err(io::Error::other(
                "async process spawn does not consume Command.stdin_bytes",
            ));
        }
        let mut command = async_command(&program, &args, &env, &cwd, has_cwd);
        command.stdin(stdio_from_mode(&stdin_mode)?);
        command.stdout(stdio_from_mode(&stdout_mode)?);
        command.stderr(stdio_from_mode(&stderr_mode)?);
        let child = command.spawn()?;
        Ok(SifrIntBridge::from(insert_async_child(child)))
    })
}

pub fn process_async_wait(handle: SifrIntBridge) -> ProcessFuture<Vec<SifrIntBridge>> {
    let handle = handle_value(handle);
    Box::pin(async move { wait_for_async_child(handle).await })
}

pub fn process_handle_wait(handle: SifrIntBridge) -> ProcessFuture<Vec<SifrIntBridge>> {
    let handle = handle_value(handle);
    Box::pin(async move {
        if let Some(observed) = process_async_child_observed().get(&handle).cloned() {
            observed.store(true, Ordering::SeqCst);
        }
        let result = wait_for_async_child(handle).await;
        process_async_child_observed().remove(&handle);
        result
    })
}

pub fn process_async_kill(handle: SifrIntBridge) -> ProcessFuture<()> {
    let handle = handle_value(handle);
    Box::pin(async move {
        let mut children = process_async_children();
        let child = children
            .get_mut(&handle)
            .ok_or_else(|| missing_async_child_error(handle))?;
        child.start_kill()
    })
}

pub fn process_async_terminate(handle: SifrIntBridge) -> ProcessFuture<()> {
    let handle = handle_value(handle);
    Box::pin(async move {
        #[cfg(unix)]
        {
            let pid = {
                let mut children = process_async_children();
                let child = children
                    .get_mut(&handle)
                    .ok_or_else(|| missing_async_child_error(handle))?;
                child.id().ok_or_else(|| {
                    io::Error::other(format!(
                        "async process child handle {handle} has no running process id"
                    ))
                })?
            };
            let status = Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .await?;
            if status.success() {
                Ok(())
            } else {
                Err(io::Error::other(format!(
                    "async process terminate failed with status: {status}"
                )))
            }
        }
        #[cfg(not(unix))]
        {
            let _ = handle;
            Err(io::Error::other(
                "async process terminate is unsupported on this host; use async_kill for forceful termination",
            ))
        }
    })
}

pub fn process_async_child_stdin(handle: SifrIntBridge) -> Result<SifrIntBridge, io::Error> {
    let handle = handle_value(handle);
    let mut children = process_async_children();
    let child = children
        .get_mut(&handle)
        .ok_or_else(|| missing_async_child_error(handle))?;
    let pipe = child.stdin.take().ok_or_else(|| {
        io::Error::other(format!(
            "async process stdin pipe is not available or already taken for child handle: {handle}"
        ))
    })?;
    let pipe_handle = next_async_child_id();
    process_async_pipe_writers().insert(pipe_handle, Box::new(pipe));
    Ok(SifrIntBridge::from(pipe_handle))
}

pub fn process_async_child_stdout(handle: SifrIntBridge) -> Result<SifrIntBridge, io::Error> {
    take_async_child_reader(handle, "stdout")
}

pub fn process_async_child_stderr(handle: SifrIntBridge) -> Result<SifrIntBridge, io::Error> {
    take_async_child_reader(handle, "stderr")
}

pub fn process_async_pipe_read_all(handle: SifrIntBridge) -> ProcessFuture<Vec<u8>> {
    let handle = handle_value(handle);
    Box::pin(async move {
        let mut pipe = process_async_pipe_readers()
            .remove(&handle)
            .ok_or_else(|| missing_async_pipe_reader_error(handle))?;
        let mut buffer = Vec::new();
        pipe.read_to_end(&mut buffer).await?;
        Ok(buffer)
    })
}

pub fn process_async_pipe_read(
    handle: SifrIntBridge,
    max_bytes: SifrIntBridge,
) -> ProcessFuture<Vec<u8>> {
    let handle = handle_value(handle);
    let max_bytes = handle_value(max_bytes);
    Box::pin(async move {
        if max_bytes <= 0 {
            return Err(io::Error::other(
                "async process pipe read size must be positive",
            ));
        }
        if max_bytes > 1_048_576 {
            return Err(io::Error::other(
                "async process pipe read size exceeds 1048576 bytes",
            ));
        }
        let pipe = process_async_pipe_readers()
            .remove(&handle)
            .ok_or_else(|| missing_async_pipe_reader_error(handle))?;
        let mut guard = AsyncPipeReaderGuard {
            handle,
            pipe: Some(pipe),
        };
        let mut buffer = vec![0_u8; usize::try_from(max_bytes).map_err(io::Error::other)?];
        let read_result = guard
            .pipe
            .as_mut()
            .ok_or_else(|| missing_async_pipe_reader_error(handle))?
            .read(buffer.as_mut_slice())
            .await;
        let read = match read_result {
            Ok(read) => read,
            Err(error) => {
                let _pipe = guard.pipe.take();
                return Err(error);
            }
        };
        buffer.truncate(read);
        if read == 0 {
            let _pipe = guard.pipe.take();
        }
        Ok(buffer)
    })
}

pub fn process_async_pipe_reader_close(handle: SifrIntBridge) -> Result<(), io::Error> {
    let handle = handle_value(handle);
    process_async_pipe_readers()
        .remove(&handle)
        .ok_or_else(|| missing_async_pipe_reader_error(handle))?;
    Ok(())
}

pub fn process_async_pipe_write_all(handle: SifrIntBridge, data: &[u8]) -> ProcessFuture<()> {
    let handle = handle_value(handle);
    let data = data.to_vec();
    Box::pin(async move {
        let pipe = process_async_pipe_writers()
            .remove(&handle)
            .ok_or_else(|| missing_async_pipe_writer_error(handle))?;
        let mut guard = AsyncPipeWriterGuard {
            handle,
            pipe: Some(pipe),
        };
        guard
            .pipe
            .as_mut()
            .ok_or_else(|| missing_async_pipe_writer_error(handle))?
            .write_all(data.as_slice())
            .await?;
        Ok(())
    })
}

pub fn process_async_pipe_close(handle: SifrIntBridge) -> Result<(), io::Error> {
    let handle = handle_value(handle);
    process_async_pipe_writers()
        .remove(&handle)
        .ok_or_else(|| missing_async_pipe_writer_error(handle))?;
    Ok(())
}

pub fn process_async_register_scoped_child(child: Child) -> (i64, Arc<AtomicBool>) {
    let handle = insert_async_child(child);
    let observed = Arc::new(AtomicBool::new(false));
    process_async_child_observed().insert(handle, Arc::clone(&observed));
    (handle, observed)
}

pub fn process_async_take_child(handle: i64) -> Option<Child> {
    process_async_children().remove(&handle)
}

pub fn process_async_remove_observed(handle: i64) {
    process_async_child_observed().remove(&handle);
}

fn async_command(
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

fn stdio_from_mode(mode: &str) -> Result<std::process::Stdio, io::Error> {
    match mode {
        "pipe" => Ok(std::process::Stdio::piped()),
        "inherit" => Ok(std::process::Stdio::inherit()),
        "null" => Ok(std::process::Stdio::null()),
        _ => Err(io::Error::other(format!(
            "unsupported async process stdio mode: {mode}"
        ))),
    }
}

fn take_async_child_reader(
    handle: SifrIntBridge,
    stream_name: &str,
) -> Result<SifrIntBridge, io::Error> {
    let handle = handle_value(handle);
    let mut children = process_async_children();
    let child = children
        .get_mut(&handle)
        .ok_or_else(|| missing_async_child_error(handle))?;
    let pipe: AsyncPipeReader = match stream_name {
        "stdout" => Box::new(child.stdout.take().ok_or_else(|| {
            io::Error::other(format!(
                "async process stdout pipe is not available or already taken for child handle: {handle}"
            ))
        })?),
        "stderr" => Box::new(child.stderr.take().ok_or_else(|| {
            io::Error::other(format!(
                "async process stderr pipe is not available or already taken for child handle: {handle}"
            ))
        })?),
        _ => return Err(io::Error::other("unsupported async process reader stream")),
    };
    let pipe_handle = next_async_child_id();
    process_async_pipe_readers().insert(pipe_handle, pipe);
    Ok(SifrIntBridge::from(pipe_handle))
}

async fn wait_for_async_child(handle: i64) -> Result<Vec<SifrIntBridge>, io::Error> {
    let child = process_async_children()
        .remove(&handle)
        .ok_or_else(|| missing_async_child_error(handle))?;
    let mut guard = AsyncChildWaitGuard {
        handle,
        child: Some(child),
    };
    let status = guard
        .child
        .as_mut()
        .ok_or_else(|| missing_async_child_error(handle))?
        .wait()
        .await?;
    let _child = guard.child.take();
    Ok(status_tuple(status))
}

fn insert_async_child(child: Child) -> i64 {
    let handle = next_async_child_id();
    process_async_children().insert(handle, child);
    handle
}

fn handle_value(handle: SifrIntBridge) -> i64 {
    handle.to_i64_saturating()
}

fn next_async_child_id() -> i64 {
    NEXT_PROCESS_ASYNC_CHILD_ID.fetch_add(1, Ordering::SeqCst)
}

fn process_async_children() -> MutexGuard<'static, HashMap<i64, Child>> {
    PROCESS_ASYNC_CHILDREN
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn process_async_child_observed() -> MutexGuard<'static, HashMap<i64, Arc<AtomicBool>>> {
    PROCESS_ASYNC_CHILD_OBSERVED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn process_async_pipe_readers() -> MutexGuard<'static, HashMap<i64, AsyncPipeReader>> {
    PROCESS_ASYNC_PIPE_READERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn process_async_pipe_writers() -> MutexGuard<'static, HashMap<i64, AsyncPipeWriter>> {
    PROCESS_ASYNC_PIPE_WRITERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn missing_async_child_error(handle: i64) -> io::Error {
    io::Error::other(format!(
        "async process child handle {handle} is closed or unknown"
    ))
}

fn missing_async_pipe_reader_error(handle: i64) -> io::Error {
    io::Error::other(format!(
        "async process pipe reader handle is closed or unknown: {handle}"
    ))
}

fn missing_async_pipe_writer_error(handle: i64) -> io::Error {
    io::Error::other(format!(
        "async process pipe writer handle is closed or unknown: {handle}"
    ))
}

struct AsyncChildWaitGuard {
    handle: i64,
    child: Option<Child>,
}

impl Drop for AsyncChildWaitGuard {
    fn drop(&mut self) {
        if let Some(child) = self.child.take() {
            process_async_children().insert(self.handle, child);
        }
    }
}

struct AsyncPipeReaderGuard {
    handle: i64,
    pipe: Option<AsyncPipeReader>,
}

impl Drop for AsyncPipeReaderGuard {
    fn drop(&mut self) {
        if let Some(pipe) = self.pipe.take() {
            process_async_pipe_readers().insert(self.handle, pipe);
        }
    }
}

struct AsyncPipeWriterGuard {
    handle: i64,
    pipe: Option<AsyncPipeWriter>,
}

impl Drop for AsyncPipeWriterGuard {
    fn drop(&mut self) {
        if let Some(pipe) = self.pipe.take() {
            process_async_pipe_writers().insert(self.handle, pipe);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        process_async_child_stdout, process_async_pipe_read_all, process_async_spawn,
        process_async_wait,
    };
    use sifr_runtime::interop::SifrIntBridge;

    fn empty() -> Vec<String> {
        Vec::new()
    }

    #[tokio::test]
    async fn async_spawn_stdout_and_wait_are_observable() {
        let handle = process_async_spawn(
            "sh",
            &["-c".to_string(), "printf async-child".to_string()],
            &empty(),
            "",
            false,
            "inherit",
            "pipe",
            "inherit",
            false,
        )
        .await
        .expect("spawn should succeed");
        let stdout = process_async_child_stdout(handle.clone()).expect("stdout should exist");
        let bytes = process_async_pipe_read_all(stdout)
            .await
            .expect("stdout should read");
        assert_eq!(bytes, b"async-child");
        let status = process_async_wait(handle)
            .await
            .expect("wait should succeed");
        assert_eq!(status[0].to_i64_saturating(), 0);
    }

    #[tokio::test]
    async fn missing_async_child_handles_are_errors() {
        let err = process_async_wait(SifrIntBridge::from(9_999_999))
            .await
            .expect_err("unknown child");
        assert!(err.to_string().contains("closed or unknown"));
    }
}
