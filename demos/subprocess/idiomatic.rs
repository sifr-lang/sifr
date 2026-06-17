use std::io;
use std::process::Command;

const PIPE: i64 = -1;
const STDOUT: i64 = -2;
const DEVNULL: i64 = -3;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedProcess {
    returncode: i64,
    stdout: String,
    stderr: String,
}

fn run(cmd: &str) -> io::Result<CompletedProcess> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    Ok(CompletedProcess {
        returncode: i64::from(output.status.code().unwrap_or(-1)),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn nonzero_exit_error(cmd: &str, returncode: i64) -> io::Error {
    io::Error::other(format!(
        "command returned non-zero exit status {returncode}: {cmd}"
    ))
}

fn check_call(cmd: &str) -> io::Result<i64> {
    let result = run(cmd)?;
    if result.returncode == 0 {
        Ok(0)
    } else {
        Err(nonzero_exit_error(cmd, result.returncode))
    }
}

fn check_output(cmd: &str) -> io::Result<String> {
    let result = run(cmd)?;
    if result.returncode == 0 {
        Ok(result.stdout)
    } else {
        Err(nonzero_exit_error(cmd, result.returncode))
    }
}

fn main() {
    let demo_ok = match run("echo runtime_subprocess").and_then(|cp| {
        Ok((
            cp.clone(),
            check_call("echo runtime_subprocess_call")?,
            check_output("echo runtime_subprocess_output")?,
        ))
    }) {
        Ok((cp, rc, out)) => {
            let constants_ok = PIPE == -1 && STDOUT == -2 && DEVNULL == -3;
            cp.returncode == 0
                && cp.stdout.trim() == "runtime_subprocess"
                && rc == 0
                && out.trim() == "runtime_subprocess_output"
                && constants_ok
        }
        Err(_) => false,
    };

    assert!(demo_ok);
    println!("runtime_subprocess_sync_boundary_governance_demo: ok");
}
