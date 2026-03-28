// --- stdlib: sifr.subprocess ---
const PIPE: i64 = -1 as i64;
const STDOUT: i64 = -2 as i64;
const DEVNULL: i64 = -3 as i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CompletedProcess {
    returncode: i64,
    stdout: String,
    stderr: String,
}
impl CompletedProcess {
    fn new(returncode: i64, stdout: String, stderr: String) -> Self {
        return Self {
            returncode: returncode,
            stdout: stdout,
            stderr: stderr,
        };
    }
}
impl std::fmt::Display for CompletedProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "CompletedProcess(returncode={}, stdout={}, stderr={})",
            self.returncode, self.stdout, self.stderr
        );
    }
}
fn _nonzero_exit_error(cmd: &String, returncode: i64) -> String {
    return format!(
        "{}{}{}{}",
        "command returned non-zero exit status ".to_string(),
        format!("{}", returncode),
        ": ".to_string(),
        cmd
    );
}
fn run(cmd: &String) -> Result<CompletedProcess, IOError> {
    let __sifr_try_res: Result<Result<CompletedProcess, IOError>, IOError> = (|| {
        let result: Vec<String> = ({
            let __output = std::process::Command::new("sh".to_string())
                .arg("-c".to_string())
                .arg(&cmd)
                .output()
                .map_err(__io_err)?;
            let __stdout = String::from_utf8_lossy(&__output.stdout).to_string();
            let __stderr = String::from_utf8_lossy(&__output.stderr).to_string();
            let __returncode = __output.status.code().unwrap_or(-1).to_string();
            Ok(vec![__stdout, __stderr, __returncode])
        })?;
        let mut stdout: String = "".to_string();
        let mut stderr: String = "".to_string();
        let mut rc_str: String = "".to_string();
        let mut rc: i64 = 0 as i64;
        for (i, value) in Box::new(
            (result)
                .iter()
                .cloned()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
        ) {
            if i == (0 as i64) {
                stdout = format!("{}{}", value, "".to_string());
            }
            if i == (1 as i64) {
                stderr = format!("{}{}", value, "".to_string());
            }
            if i == (2 as i64) {
                rc_str = format!("{}{}", value, "".to_string());
            }
        }
        if rc_str != "".to_string() {
            let __sifr_try_res: Result<(), ParseError> = (|| {
                let parsed: i64 = (rc_str).parse::<i64>().map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
                rc = parsed;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _: String = e.message;
                rc = -(1 as i64);
            }
        }
        return Ok(Ok(CompletedProcess::new(rc, stdout, stderr)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}
fn check_call(cmd: &String) -> Result<i64, IOError> {
    let __sifr_try_res: Result<Result<i64, IOError>, IOError> = (|| {
        let result: CompletedProcess = run(cmd)?;
        if result.returncode != (0 as i64) {
            return Err(IOError::new(_nonzero_exit_error(cmd, result.returncode)));
        }
        return Ok(Ok(result.returncode));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}
fn check_output(cmd: &String) -> Result<String, IOError> {
    let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
        let result: CompletedProcess = run(cmd)?;
        if result.returncode != (0 as i64) {
            return Err(IOError::new(_nonzero_exit_error(cmd, result.returncode)));
        }
        return Ok(Ok(result.stdout));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message));
        }
    }
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            kind: "Other".to_string(),
        };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound {
        "FileNotFound".to_string()
    } else {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "PermissionDenied".to_string()
        } else {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "FileExists".to_string()
            } else {
                "Other".to_string()
            }
        }
    };
    return IOError {
        message: msg,
        kind: kind,
    };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {}

fn main() {
    let mut demo_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let cp: CompletedProcess = run(&"echo runtime_wave4_demo".to_string())?;
        let rc: i64 = check_call(&"echo runtime_wave4_demo_call".to_string())?;
        let out: String = check_output(&"echo runtime_wave4_demo_output".to_string())?;
        let constants_ok: bool =
            ((PIPE == -(1 as i64)) && (STDOUT == -(2 as i64))) && (DEVNULL == -(3 as i64));
        demo_ok = (((((cp.returncode == (0 as i64))
            && (cp.stdout.trim().to_string() == "runtime_wave4_demo".to_string()))
            && (rc == (0 as i64)))
            && (out.trim().to_string() == "runtime_wave4_demo_output".to_string()))
            && (constants_ok));
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    assert!(demo_ok);
    println!("ad_hoc_runtime_wave4_subprocess_sync_boundary_governance_demo: ok");
}
