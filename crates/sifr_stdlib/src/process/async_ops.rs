use std::{future::Future, io, pin::Pin, time::Duration};

use sifr_runtime::interop::SifrIntBridge;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    process::{Child, Command},
};

type ProcessFuture<T> = Pin<Box<dyn Future<Output = Result<T, io::Error>> + Send>>;

pub fn process_async_run(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin_mode: &str,
) -> ProcessFuture<Vec<SifrIntBridge>> {
    let program = program.to_string();
    let args = args.to_vec();
    let env = env.to_vec();
    let cwd = cwd.to_string();
    let stdin_mode = stdin_mode.to_string();
    Box::pin(async move {
        validate_stdin_mode(&stdin_mode)?;
        let mut command = async_command(&program, &args, &env, &cwd, has_cwd);
        command.status().await.map(super::status_tuple)
    })
}

pub fn process_async_run_timeout(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin_mode: &str,
    timeout_seconds: f64,
) -> ProcessFuture<Vec<SifrIntBridge>> {
    let program = program.to_string();
    let args = args.to_vec();
    let env = env.to_vec();
    let cwd = cwd.to_string();
    let stdin_mode = stdin_mode.to_string();
    Box::pin(async move {
        validate_stdin_mode(&stdin_mode)?;
        let duration = timeout_duration(timeout_seconds)?;
        let mut command = async_command(&program, &args, &env, &cwd, has_cwd);
        configure_async_process_group(&mut command);
        let mut child = command.spawn()?;
        wait_with_timeout(&mut child, duration).await
    })
}

pub fn process_async_output(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin_mode: &str,
    stdin: &[u8],
    has_stdin: bool,
) -> ProcessFuture<String> {
    let program = program.to_string();
    let args = args.to_vec();
    let env = env.to_vec();
    let cwd = cwd.to_string();
    let stdin_mode = stdin_mode.to_string();
    let stdin = stdin.to_vec();
    Box::pin(async move {
        validate_stdin_mode(&stdin_mode)?;
        let mut command = async_command(&program, &args, &env, &cwd, has_cwd);
        output_command(&mut command, &stdin, has_stdin).await
    })
}

pub fn process_async_output_timeout(
    program: &str,
    args: &[String],
    env: &[String],
    cwd: &str,
    has_cwd: bool,
    stdin_mode: &str,
    stdin: &[u8],
    has_stdin: bool,
    timeout_seconds: f64,
) -> ProcessFuture<String> {
    let program = program.to_string();
    let args = args.to_vec();
    let env = env.to_vec();
    let cwd = cwd.to_string();
    let stdin_mode = stdin_mode.to_string();
    let stdin = stdin.to_vec();
    Box::pin(async move {
        validate_stdin_mode(&stdin_mode)?;
        let duration = timeout_duration(timeout_seconds)?;
        let mut command = async_command(&program, &args, &env, &cwd, has_cwd);
        configure_async_process_group(&mut command);
        output_command_timeout(&mut command, &stdin, has_stdin, duration).await
    })
}

pub fn process_async_shell_run(script: &str) -> ProcessFuture<Vec<SifrIntBridge>> {
    let script = script.to_string();
    Box::pin(async move {
        let args = shell_args(&script);
        process_async_run("sh", &args, &[], "", false, "inherit").await
    })
}

pub fn process_async_shell_output(
    script: &str,
    stdin: &[u8],
    has_stdin: bool,
) -> ProcessFuture<String> {
    let script = script.to_string();
    let stdin = stdin.to_vec();
    Box::pin(async move {
        let args = shell_args(&script);
        process_async_output("sh", &args, &[], "", false, "inherit", &stdin, has_stdin).await
    })
}

pub fn process_async_shell_output_timeout(
    script: &str,
    stdin: &[u8],
    has_stdin: bool,
    timeout_seconds: f64,
) -> ProcessFuture<String> {
    let script = script.to_string();
    let stdin = stdin.to_vec();
    Box::pin(async move {
        let args = shell_args(&script);
        process_async_output_timeout(
            "sh",
            &args,
            &[],
            "",
            false,
            "inherit",
            &stdin,
            has_stdin,
            timeout_seconds,
        )
        .await
    })
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

fn shell_args(script: &str) -> Vec<String> {
    vec!["-c".to_string(), script.to_string()]
}

fn validate_stdin_mode(stdin_mode: &str) -> Result<(), io::Error> {
    if stdin_mode == "inherit" {
        return Ok(());
    }
    Err(io::Error::other(
        "async process stdin mode requires owned pipe support",
    ))
}

fn timeout_duration(timeout_seconds: f64) -> Result<Duration, io::Error> {
    if !timeout_seconds.is_finite() || timeout_seconds < 0.0 {
        return Err(io::Error::other(format!(
            "process timeout must be finite and non-negative, got {timeout_seconds}"
        )));
    }
    Duration::try_from_secs_f64(timeout_seconds).map_err(io::Error::other)
}

async fn wait_with_timeout(
    child: &mut Child,
    duration: Duration,
) -> Result<Vec<SifrIntBridge>, io::Error> {
    tokio::select! {
        biased;
        status = child.wait() => status.map(super::status_tuple),
        () = tokio::time::sleep(duration) => {
            terminate_async_process_group_or_child(child).await?;
            let _status = child.wait().await?;
            Ok(super::timeout_status_tuple())
        }
    }
}

async fn output_command(
    command: &mut Command,
    stdin: &[u8],
    has_stdin: bool,
) -> Result<String, io::Error> {
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    let mut child = command.spawn()?;
    collect_child_output(&mut child, stdin, has_stdin, false).await
}

async fn output_command_timeout(
    command: &mut Command,
    stdin: &[u8],
    has_stdin: bool,
    duration: Duration,
) -> Result<String, io::Error> {
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    let mut child = command.spawn()?;
    tokio::select! {
        biased;
        completed = collect_child_output(&mut child, stdin, has_stdin, false) => completed,
        () = tokio::time::sleep(duration) => {
            terminate_async_process_group_or_child(&mut child).await?;
            let status = child.wait().await?;
            super::store_output_components(Vec::new(), Vec::new(), status, true, None, None)
        }
    }
}

async fn collect_child_output(
    child: &mut Child,
    stdin: &[u8],
    has_stdin: bool,
    timed_out: bool,
) -> Result<String, io::Error> {
    let mut child_stdin = child.stdin.take();
    let mut child_stdout = child.stdout.take();
    let mut child_stderr = child.stderr.take();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let stdin_write = async {
        if has_stdin {
            if let Some(mut pipe) = child_stdin.take() {
                pipe.write_all(stdin).await?;
            }
        }
        Ok::<(), io::Error>(())
    };
    let stdout_read = async {
        if let Some(pipe) = child_stdout.as_mut() {
            pipe.read_to_end(&mut stdout).await?;
        }
        Ok::<(), io::Error>(())
    };
    let stderr_read = async {
        if let Some(pipe) = child_stderr.as_mut() {
            pipe.read_to_end(&mut stderr).await?;
        }
        Ok::<(), io::Error>(())
    };

    let (status, (), (), ()) =
        tokio::try_join!(child.wait(), stdin_write, stdout_read, stderr_read)?;
    super::store_output_components(stdout, stderr, status, timed_out, None, None)
}

fn configure_async_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        command.process_group(0);
    }
    #[cfg(not(unix))]
    {
        let _ = command;
    }
}

async fn terminate_async_process_group_or_child(child: &mut Child) -> Result<(), io::Error> {
    #[cfg(unix)]
    {
        if let Some(pid) = child.id() {
            let process_group = format!("-{pid}");
            let _term_status = Command::new("kill")
                .arg("-TERM")
                .arg(&process_group)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .await?;
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _kill_status = Command::new("kill")
                .arg("-KILL")
                .arg(&process_group)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .await?;
            Ok(())
        } else {
            child.kill().await
        }
    }
    #[cfg(not(unix))]
    {
        child.kill().await
    }
}

#[cfg(test)]
mod tests {
    use super::{
        process_async_output, process_async_output_timeout, process_async_run,
        process_async_shell_output,
    };

    fn empty() -> Vec<String> {
        Vec::new()
    }

    #[tokio::test]
    async fn async_run_and_output_are_observable() {
        let args = vec!["-c".to_string(), "printf async-stdlib".to_string()];
        let status = process_async_run("sh", &args, &empty(), "", false, "inherit")
            .await
            .expect("status should run");
        assert_eq!(status[0].to_i64_saturating(), 0);

        let handle = process_async_output("sh", &args, &empty(), "", false, "inherit", b"", false)
            .await
            .expect("output should run");
        assert_eq!(
            crate::process::process_output_stdout(&handle).expect("stdout"),
            b"async-stdlib"
        );
        crate::process::process_output_close(&handle);
    }

    #[tokio::test]
    async fn async_shell_output_and_timeout_are_observable() {
        let handle = process_async_shell_output("cat", b"async-shell-stdin", true)
            .await
            .expect("shell output should run");
        assert_eq!(
            crate::process::process_output_stdout(&handle).expect("stdout"),
            b"async-shell-stdin"
        );
        crate::process::process_output_close(&handle);

        let timed_out = process_async_output_timeout(
            "sh",
            &["-c".to_string(), "sleep 1; printf late".to_string()],
            &empty(),
            "",
            false,
            "inherit",
            b"",
            false,
            0.01,
        )
        .await
        .expect("timeout should return handle");
        assert!(crate::process::process_output_timed_out(&timed_out));
        crate::process::process_output_close(&timed_out);
    }
}
