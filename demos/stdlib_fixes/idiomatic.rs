use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use regex::RegexBuilder;

const WARNING: i64 = 30;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Time {
    hour: u8,
    minute: u8,
    second: u8,
}

impl Time {
    fn isoformat(&self) -> String {
        format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Timezone {
    offset_seconds: i32,
}

impl std::fmt::Display for Timezone {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.offset_seconds == 0 {
            formatter.write_str("UTC")
        } else {
            write!(formatter, "UTC({})", self.offset_seconds)
        }
    }
}

#[derive(Clone, Debug)]
struct DateTime {
    instant: SystemTime,
}

impl DateTime {
    fn isoformat(&self) -> String {
        let seconds = self
            .instant
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        format!("unix:{seconds}")
    }
}

#[derive(Clone, Debug)]
struct CompletedProcess {
    returncode: i64,
    stdout: String,
}

#[derive(Clone, Debug)]
struct Pattern {
    pattern: String,
    ignore_case: bool,
    multi_line: bool,
}

impl Pattern {
    fn search(&self, text: &str) -> Result<Option<String>, String> {
        let regex = RegexBuilder::new(&self.pattern)
            .case_insensitive(self.ignore_case)
            .multi_line(self.multi_line)
            .build()
            .map_err(|error| error.to_string())?;
        Ok(regex.find(text).map(|value| value.as_str().to_string()))
    }
}

#[derive(Clone, Debug)]
struct Logger {
    name: String,
}

#[derive(Clone, Debug)]
struct FileHandler {
    path: PathBuf,
}

static GLOBAL_LEVEL: OnceLock<Mutex<i64>> = OnceLock::new();

fn log_level() -> &'static Mutex<i64> {
    GLOBAL_LEVEL.get_or_init(|| Mutex::new(WARNING))
}

fn basic_config(level: i64) -> Logger {
    *log_level().lock().expect("logger mutex poisoned") = level;
    Logger {
        name: "root".to_string(),
    }
}

fn get_logger(name: &str) -> Logger {
    Logger {
        name: name.to_string(),
    }
}

impl Logger {
    fn info(&self, message: &str) {
        if *log_level().lock().expect("logger mutex poisoned") <= 20 {
            println!("[INFO] {}: {}", self.name, message);
        }
    }

    fn warning(&self, message: &str) {
        if *log_level().lock().expect("logger mutex poisoned") <= WARNING {
            println!("[WARNING] {}: {}", self.name, message);
        }
    }
}

impl FileHandler {
    fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path),
        }
    }

    fn emit(&self, level: &str, logger: &str, message: &str) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| error.to_string())?;
        writeln!(file, "[{level}] {logger}: {message}").map_err(|error| error.to_string())
    }
}

fn compile_flags(pattern: &str, ignore_case: bool, multi_line: bool) -> Pattern {
    Pattern {
        pattern: pattern.to_string(),
        ignore_case,
        multi_line,
    }
}

fn search_flags(pattern: &str, text: &str, ignore_case: bool) -> Result<Option<String>, String> {
    compile_flags(pattern, ignore_case, false).search(text)
}

fn getcwd() -> Result<String, String> {
    std::env::current_dir()
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

fn choice<T: Clone>(values: &[T]) -> Result<T, String> {
    values
        .first()
        .cloned()
        .ok_or_else(|| "cannot choose from an empty sequence".to_string())
}

fn run(command: &str) -> Result<CompletedProcess, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|error| error.to_string())?;
    Ok(CompletedProcess {
        returncode: output.status.code().unwrap_or(1).into(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    })
}

fn glob_tmp(prefix: &str) -> Result<Vec<String>, String> {
    let mut matches = Vec::new();
    for entry in fs::read_dir("/tmp").map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(prefix) {
            matches.push(format!("/tmp/{name}"));
        }
    }
    Ok(matches)
}

fn write_text(path: &str, text: &str) -> Result<(), String> {
    fs::write(path, text).map_err(|error| error.to_string())
}

fn read_text(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| error.to_string())
}

fn reader_from_path(path: &str) -> Result<Vec<Vec<String>>, String> {
    let content = read_text(path)?;
    Ok(content
        .lines()
        .map(|line| line.split(',').map(str::to_string).collect::<Vec<_>>())
        .collect())
}

fn now() -> DateTime {
    DateTime {
        instant: SystemTime::now(),
    }
}

fn main() {
    let path = "/tmp/sifr_demo_remediation.txt";
    match File::create(path) {
        Ok(mut file) => {
            let _ = file.write_all(b"hello from open()\n");
            let _ = file.write_all(b"second line\n");
            match read_text(path) {
                Ok(content) => println!("open write ok = {}", !content.is_empty()),
                Err(message) => println!("open write error: {message}"),
            }
        }
        Err(error) => println!("open write error: {}", error),
    }

    let path2 = "/tmp/sifr_demo_ctx.txt";
    match File::create(path2) {
        Ok(mut file) => {
            let _ = file.write_all(b"context manager works");
            match read_text(path2) {
                Ok(content) => println!(
                    "context manager ok = {}",
                    content == "context manager works"
                ),
                Err(message) => println!("context manager error: {message}"),
            }
        }
        Err(error) => println!("context manager error: {}", error),
    }

    match File::open(path) {
        Ok(mut file) => {
            let mut content = String::new();
            let _ = file.read_to_string(&mut content);
            println!("open read ok = {}", !content.is_empty());
        }
        Err(error) => println!("open read error: {}", error),
    }

    let t = Time {
        hour: 10,
        minute: 30,
        second: 45,
    };
    println!("time isoformat = {}", t.isoformat());
    let t2 = Time {
        hour: 10,
        minute: 30,
        second: 45,
    };
    println!("time eq = {}", t == t2);

    let tz = Timezone { offset_seconds: 0 };
    println!("timezone utc = {}", tz);

    let iso = now().isoformat();
    println!("now isoformat ok = {}", !iso.is_empty());

    match run("echo hello_subprocess") {
        Ok(result) => {
            println!("subprocess returncode = {}", result.returncode);
            println!("subprocess stdout ok = {}", !result.stdout.is_empty());
        }
        Err(message) => println!("subprocess error: {message}"),
    }

    match glob_tmp("sifr_demo_") {
        Ok(matches) => println!("glob found = {}", !matches.is_empty()),
        Err(message) => println!("glob error: {message}"),
    }

    match search_flags("hello", "HELLO WORLD", true) {
        Ok(found) => println!("re ignorecase = {}", found.is_some()),
        Err(message) => println!("re error: {message}"),
    }
    match compile_flags("^line", false, true).search("line1\nline2") {
        Ok(found) => println!("re multiline = {}", found.is_some()),
        Err(message) => println!("re error: {message}"),
    }

    match getcwd() {
        Ok(cwd) => println!("os getcwd ok = {}", !cwd.is_empty()),
        Err(message) => println!("os getcwd error: {message}"),
    }

    let items = [1_i64, 2, 3, 4, 5];
    match choice(&items) {
        Ok(picked) => println!("random choice ok = {}", (1..=5).contains(&picked)),
        Err(message) => println!("random choice error: {message}"),
    }

    let root = basic_config(WARNING);
    root.info("should not print");
    root.warning("root warning visible");
    let logger2 = get_logger("myapp");
    logger2.info("should not print either");
    logger2.warning("myapp warning visible");
    println!("basicConfig global level ok");

    let handler = FileHandler::new("/tmp/sifr_demo_fh_log.txt");
    let _ = handler.emit("INFO", "demo", "file handler test");
    match read_text("/tmp/sifr_demo_fh_log.txt") {
        Ok(content) => println!("file handler wrote ok = {}", !content.is_empty()),
        Err(message) => println!("file handler error: {message}"),
    }

    let csv_path = "/tmp/sifr_demo_csv.csv";
    let _ = write_text(csv_path, "name,age\nalice,30\nbob,25");
    match reader_from_path(csv_path) {
        Ok(rows) => println!("csv reader_from_path rows = {}", rows.len()),
        Err(message) => println!("csv error: {message}"),
    }

    let _ = fs::remove_file(Path::new(path));
    let _ = fs::remove_file(Path::new(path2));
}
