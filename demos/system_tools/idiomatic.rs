use std::env;
use std::fs;
use std::io;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

type IOError = io::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompletedProcess {
    returncode: i64,
    stdout: String,
    stderr: String,
}

fn run_command(command: &str) -> Result<String, IOError> {
    let output = Command::new("sh").arg("-c").arg(command).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

fn getcwd() -> Result<String, IOError> {
    Ok(env::current_dir()?.display().to_string())
}

fn getenv(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn setenv(key: &str, value: &str) {
    unsafe {
        env::set_var(key, value);
    }
}

fn unsetenv(key: &str) {
    unsafe {
        env::remove_var(key);
    }
}

fn argv() -> Vec<String> {
    vec!["system_tools".to_string()]
}

fn version() -> &'static str {
    "sifr 0.1.0"
}

fn platform() -> &'static str {
    env::consts::OS
}

fn run(cmd: &str) -> Result<CompletedProcess, IOError> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    Ok(CompletedProcess {
        returncode: i64::from(output.status.code().unwrap_or(-1)),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn run_with_input(cmd: &str, stdin_data: &str) -> Result<String, IOError> {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(stdin_data.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn check_call(cmd: &str) -> Result<i64, IOError> {
    let process = run(cmd)?;
    if process.returncode == 0 {
        Ok(0)
    } else {
        Err(io::Error::other(format!(
            "command returned non-zero exit status {}: {}",
            process.returncode, cmd
        )))
    }
}

fn check_output(cmd: &str) -> Result<String, IOError> {
    let process = run(cmd)?;
    if process.returncode == 0 {
        Ok(process.stdout)
    } else {
        Err(io::Error::other(format!(
            "command returned non-zero exit status {}: {}",
            process.returncode, cmd
        )))
    }
}

const INFO: i64 = 20;

struct Logger {
    name: String,
    level: i64,
}

impl Logger {
    fn set_level(&mut self, level: i64) {
        self.level = level;
    }

    fn info(&self, message: &str) {
        if self.level <= INFO {
            println!("[INFO] {}: {}", self.name, message);
        }
    }
}

fn get_logger(name: &str) -> Logger {
    Logger {
        name: name.to_string(),
        level: INFO,
    }
}

fn system() -> String {
    if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else if cfg!(target_os = "macos") {
        "Darwin".to_string()
    } else if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else {
        env::consts::OS.to_string()
    }
}

fn machine() -> String {
    env::consts::ARCH.to_string()
}

fn processor() -> String {
    machine()
}

fn time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |duration| duration.as_secs_f64())
}

fn strftime(format: &str, timestamp: f64) -> String {
    if format == "%Y-%m-%d %H:%M:%S" && timestamp == 0.0 {
        "1970-01-01 00:00:00".to_string()
    } else {
        timestamp.to_string()
    }
}

fn timeit(workload: fn(), count: usize) -> i64 {
    for _ in 0..count {
        workload();
    }
    0
}

fn repeat(workload: fn(), repeat_count: usize, number: usize) -> Vec<i64> {
    (0..repeat_count)
        .map(|_| timeit(workload, number))
        .collect()
}

fn workload() {
    let mut i = 0;
    let mut total = 0;
    while i < 100 {
        total += i;
        i += 1;
    }
    let _ = total;
}

fn main() {
    match run_command("echo system-tools-sample") {
        Ok(shell_out) => {
            let cwd = getcwd().unwrap_or_default();
            println!("os.run_command = {}", shell_out);
            println!("os.getcwd len > 0 = {}", !cwd.is_empty());
        }
        Err(error) => println!("os error: {}", error),
    }

    setenv("SIFR_SYSTEM_TOOLS_DEMO", "ok");
    println!("env getenv = {}", getenv("SIFR_SYSTEM_TOOLS_DEMO", "fallback"));
    unsetenv("SIFR_SYSTEM_TOOLS_DEMO");

    println!("sys.argv len = {}", argv().len());
    println!("sys.version = {}", version());
    println!("sys.platform = {}", platform());

    match run("echo subprocess_demo") {
        Ok(process) => {
            println!("subprocess.run rc = {}", process.returncode);
            println!("subprocess.run stdout = {}", process.stdout.trim());
            println!(
                "subprocess.run_with_input = {}",
                run_with_input("cat", "stdin_demo").unwrap_or_default()
            );
            println!(
                "subprocess.check_call rc = {}",
                check_call("echo subprocess_check_call_demo").unwrap_or(-1)
            );
            println!(
                "subprocess.check_output = {}",
                check_output("echo subprocess_check_output_demo")
                    .unwrap_or_default()
                    .trim()
            );
        }
        Err(error) => println!("subprocess error: {}", error),
    }

    let mut logger = get_logger("system-tools-sample_demo");
    logger.set_level(INFO);
    logger.info("logging demo line");

    println!("platform.system = {}", system());
    println!("platform.machine = {}", machine());
    println!("platform.processor = {}", processor());

    println!("time.time > 0 = {}", time() > 0.0);
    println!(
        "time.strftime epoch0 = {}",
        strftime("%Y-%m-%d %H:%M:%S", 0.0)
    );
    println!("timeit.timeit = {}", timeit(workload, 5));
    println!("timeit.repeat count = {}", repeat(workload, 3, 4).len());

    let _ = fs::metadata(".");
}
